use super::*;
use std::io::Write;
use std::sync::{Arc, Mutex};

/// Stands in for the system tools. It never spawns a process.
///
/// `dd` is simulated against the target path, which in these tests is always
/// a temp file, so the verification pass reads what a real overwrite would leave.
#[derive(Default)]
struct FakeRunner {
    calls: Arc<Mutex<Vec<String>>>,
    lsblk_json: String,
    hdparm_identify: String,
    nvme_sanitize_exit: i32,
    dd_exit: i32,
    dd_output: String,
    dd_writes: bool,
}

impl FakeRunner {
    fn overwriting() -> Self {
        Self {
            lsblk_json: r#"{"blockdevices":[{"name":"loop0","mountpoint":null}]}"#.into(),
            nvme_sanitize_exit: 1,
            dd_exit: 1,
            dd_output: "dd: error writing: No space left on device".into(),
            dd_writes: true,
            ..Default::default()
        }
    }
}

impl CommandRunner for FakeRunner {
    fn run(
        &self,
        program: &str,
        args: &[&str],
        _timeout: Option<Duration>,
    ) -> Result<CommandOutput, Box<dyn std::error::Error>> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("{} {}", program, args.join(" ")));
        let reply = |exit_code: i32, output: &str| {
            Ok(CommandOutput {
                exit_code,
                output: output.to_string(),
            })
        };
        match program {
            "lsblk" => reply(0, &self.lsblk_json),
            "umount" => reply(0, ""),
            "nvme" => reply(self.nvme_sanitize_exit, ""),
            "hdparm" if args.first() == Some(&"-I") => reply(0, &self.hdparm_identify),
            "hdparm" => reply(0, ""),
            "dd" => {
                if self.dd_writes {
                    let source = args.iter().find_map(|a| a.strip_prefix("if=")).unwrap();
                    let target = args.iter().find_map(|a| a.strip_prefix("of=")).unwrap();
                    let len = std::fs::metadata(target)?.len() as usize;
                    let mut data = vec![0u8; len];
                    if source == "/dev/urandom" {
                        rand::thread_rng().fill_bytes(&mut data);
                    }
                    std::fs::write(target, data)?;
                }
                reply(self.dd_exit, &self.dd_output)
            }
            other => Err(format!("unexpected command in test: {}", other).into()),
        }
    }
}

/// A 64 KiB temp file of long byte runs, which looks neither zeroed nor
/// random to the verifier.
fn image_with_data(name: &str) -> (tempfile::TempDir, String) {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join(name);
    let mut file = std::fs::File::create(&path).unwrap();
    for _ in 0..512 {
        file.write_all(&[b'A'; 64]).unwrap();
        file.write_all(&[b'B'; 64]).unwrap();
    }
    (dir, path.to_str().unwrap().to_string())
}

fn run_wipe(
    runner: FakeRunner,
    device: &str,
    policy: WipePolicy,
) -> (Result<WipeResult, Box<dyn std::error::Error>>, Vec<String>) {
    let calls = runner.calls.clone();
    let engine = NistAlignedWipe::with_runner(Box::new(runner));
    let result = engine.perform_wipe(device, policy, false);
    let calls = calls.lock().unwrap().clone();
    (result, calls)
}

#[test]
fn clear_overwrites_with_zeros_and_verifies() {
    let (_dir, image) = image_with_data("disk.img");
    let (result, calls) = run_wipe(FakeRunner::overwriting(), &image, WipePolicy::Clear);
    let result = result.unwrap();

    assert_eq!(result.method, "overwrite");
    assert_eq!(result.verification_samples, 32);
    assert!(result.verification_passed);
    assert!(result.fallback_reason.is_some());
    assert!(std::fs::read(&image).unwrap().iter().all(|&b| b == 0));
    let dd = format!(
        "dd if=/dev/zero of={} bs=1M conv=fdatasync status=progress",
        image
    );
    assert!(calls.contains(&dd), "calls were {:?}", calls);
    assert!(!calls.iter().any(|c| c.starts_with("hdparm -N")));
}

#[test]
fn purge_clears_hpa_dco_then_writes_random_data() {
    let (_dir, image) = image_with_data("disk.img");
    let (result, calls) = run_wipe(FakeRunner::overwriting(), &image, WipePolicy::Purge);
    let result = result.unwrap();

    assert_eq!(result.method, "overwrite");
    assert_eq!(result.verification_samples, 128);
    assert!(result.verification_passed);
    let hpa = calls
        .iter()
        .position(|c| c.starts_with("hdparm -N"))
        .unwrap();
    let dd = calls
        .iter()
        .position(|c| c.starts_with("dd if=/dev/urandom"))
        .unwrap();
    assert!(hpa < dd, "HPA/DCO must be cleared before the overwrite");
    assert!(!std::fs::read(&image).unwrap().iter().all(|&b| b == 0));
}

#[test]
fn verification_fails_when_overwrite_left_data_behind() {
    let (_dir, image) = image_with_data("disk.img");
    let runner = FakeRunner {
        dd_writes: false,
        dd_exit: 0,
        ..FakeRunner::overwriting()
    };
    let (result, _) = run_wipe(runner, &image, WipePolicy::Clear);

    assert!(!result.unwrap().verification_passed);
}

#[test]
fn dd_failure_aborts_the_wipe() {
    let (_dir, image) = image_with_data("disk.img");
    let runner = FakeRunner {
        dd_writes: false,
        dd_exit: 1,
        dd_output: "dd: error writing: Input/output error".into(),
        ..FakeRunner::overwriting()
    };
    let (result, _) = run_wipe(runner, &image, WipePolicy::Clear);

    let err = result.unwrap_err().to_string();
    assert!(err.contains("Zero-fill failed"), "{}", err);
    assert!(err.contains("Input/output error"), "{}", err);
}

#[test]
fn mounted_partitions_are_unmounted_before_writing() {
    let (_dir, image) = image_with_data("disk.img");
    let runner = FakeRunner {
        lsblk_json: r#"{"blockdevices":[{"name":"loop0","mountpoint":"/mnt/a",
            "children":[{"name":"loop0p1","mountpoint":"/mnt/b"},
                        {"name":"loop0p2","mountpoint":null}]}]}"#
            .into(),
        ..FakeRunner::overwriting()
    };
    let (result, calls) = run_wipe(runner, &image, WipePolicy::Clear);
    result.unwrap();

    let first_dd = calls.iter().position(|c| c.starts_with("dd ")).unwrap();
    let umount_a = calls.iter().position(|c| c == "umount /mnt/a").unwrap();
    let umount_b = calls.iter().position(|c| c == "umount /mnt/b").unwrap();
    assert!(umount_a < first_dd && umount_b < first_dd);
    assert_eq!(calls.iter().filter(|c| c.starts_with("umount")).count(), 2);
}

#[test]
fn nvme_sanitize_success_skips_overwrite() {
    let (_dir, image) = image_with_data("nvme0n1.img");
    let runner = FakeRunner {
        nvme_sanitize_exit: 0,
        dd_writes: false,
        ..FakeRunner::overwriting()
    };
    let (result, calls) = run_wipe(runner, &image, WipePolicy::Purge);
    let result = result.unwrap();

    assert_eq!(result.method, "controller_sanitize");
    assert!(result.fallback_reason.is_none());
    assert!(calls.contains(&format!("nvme sanitize {}", image)));
    assert!(!calls.iter().any(|c| c.starts_with("dd ")));
}

#[test]
fn missing_device_is_rejected_before_any_command_runs() {
    let dir = tempfile::TempDir::new().unwrap();
    let missing = dir.path().join("absent.img");
    let (result, calls) = run_wipe(
        FakeRunner::overwriting(),
        missing.to_str().unwrap(),
        WipePolicy::Clear,
    );

    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Cannot access device"));
    assert!(calls.is_empty());
}

#[test]
fn verification_rejects_a_device_smaller_than_one_sector() {
    let dir = tempfile::TempDir::new().unwrap();
    let tiny = dir.path().join("tiny.img");
    std::fs::write(&tiny, [0u8; 100]).unwrap();
    let engine = NistAlignedWipe::with_runner(Box::new(FakeRunner::overwriting()));

    let err = engine
        .verify_wipe(tiny.to_str().unwrap(), 8)
        .unwrap_err()
        .to_string();
    assert!(err.contains("no full 512-byte sector"), "{}", err);
}

#[test]
fn dd_completed_ok_accepts_end_of_device() {
    let engine = NistAlignedWipe::with_runner(Box::new(FakeRunner::default()));
    let cmd = |exit_code: i32, output: &str| WipeCommand {
        command: "dd".into(),
        exit_code,
        elapsed_ms: 0,
        output: output.into(),
    };

    assert!(engine.dd_completed_ok(&cmd(0, "")));
    assert!(engine.dd_completed_ok(&cmd(
        1,
        "dd: error writing '/dev/sdz': No space left on device"
    )));
    assert!(!engine.dd_completed_ok(&cmd(1, "dd: Permission denied")));
    assert!(!engine.dd_completed_ok(&cmd(2, "No space left on device")));
}

#[test]
fn appears_random_separates_noise_from_patterns() {
    let engine = NistAlignedWipe::with_runner(Box::new(FakeRunner::default()));
    let mut noise = [0u8; 512];
    rand::thread_rng().fill_bytes(&mut noise);

    assert!(engine.appears_random(&noise));
    assert!(!engine.appears_random(&[0xAA; 512]));
    assert!(!engine.appears_random(&[0u8; 512]));
}

#[cfg(unix)]
#[test]
fn system_runner_captures_exit_code_and_output() {
    let out = SystemCommandRunner
        .run("sh", &["-c", "echo hello; exit 3"], Some(PROBE_TIMEOUT))
        .unwrap();
    assert_eq!(out.exit_code, 3);
    assert_eq!(out.output.trim(), "hello");

    let err_only = SystemCommandRunner
        .run("sh", &["-c", "echo oops >&2"], None)
        .unwrap();
    assert_eq!(err_only.output.trim(), "oops");
}

#[cfg(unix)]
#[test]
fn system_runner_kills_a_probe_that_overruns_its_timeout() {
    let started = Instant::now();
    let err = SystemCommandRunner
        .run("sleep", &["5"], Some(Duration::from_millis(200)))
        .unwrap_err();

    assert!(err.to_string().contains("timed out"));
    assert!(started.elapsed() < Duration::from_secs(3));
}

#[cfg(unix)]
#[test]
fn system_runner_waits_for_long_commands_without_a_timeout() {
    let out = SystemCommandRunner
        .run("sh", &["-c", "sleep 0.3; echo done"], None)
        .unwrap();
    assert_eq!(out.exit_code, 0);
    assert_eq!(out.output.trim(), "done");
}

#[test]
fn test_wipe_policy_serialization() {
    assert_eq!(
        serde_json::to_string(&WipePolicy::Purge).unwrap(),
        "\"PURGE\""
    );
    assert_eq!(
        serde_json::to_string(&WipePolicy::Clear).unwrap(),
        "\"CLEAR\""
    );
}

#[test]
fn test_wipe_policy_deserialization() {
    let purge: WipePolicy = serde_json::from_str("\"PURGE\"").unwrap();
    let clear: WipePolicy = serde_json::from_str("\"CLEAR\"").unwrap();
    assert_eq!(purge, WipePolicy::Purge);
    assert_eq!(clear, WipePolicy::Clear);
    assert!(serde_json::from_str::<WipePolicy>("\"DESTROY\"").is_err());
}

#[test]
fn test_wipe_result_serialization() {
    let result = WipeResult {
        device: "/dev/sda".to_string(),
        policy: WipePolicy::Purge,
        method: "controller_sanitize".to_string(),
        commands: vec![],
        verification_samples: 5,
        verification_passed: true,
        fallback_reason: Some("Controller sanitize not supported".to_string()),
    };

    let json = serde_json::to_string(&result).unwrap();
    let deserialized: WipeResult = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.device, "/dev/sda");
    assert!(deserialized.fallback_reason.is_some());
}

// plan_wipe tests always pass mock probe output so they never run hdparm or nvme.

#[test]
fn test_plan_wipe_safe_disk() {
    let plan = plan_wipe(
        "/dev/sda",
        Some(WipePolicy::Purge),
        false,
        false,
        Some("sanitize Security HPA DCO"),
        Some(""),
    );
    assert_eq!(plan.device, "/dev/sda");
    assert_eq!(plan.risk, "SAFE");
    assert_eq!(plan.policy, WipePolicy::Purge);
    assert_eq!(plan.main_method, "controller_sanitize");
    assert!(plan.hpa_dco_clear);
    assert!(!plan.blocked);
    assert_eq!(plan.verification.strategy, "random_sectors");
    assert_eq!(plan.verification.samples, 128);
}

#[test]
fn test_plan_wipe_critical_blocked() {
    let plan = plan_wipe(
        "/dev/sda",
        Some(WipePolicy::Clear),
        true,
        false,
        Some(""),
        Some(""),
    );
    assert!(plan.blocked);
    assert_eq!(plan.risk, "CRITICAL");
    assert!(plan.reason.unwrap().contains("CRITICAL disk wipe blocked"));
}

#[test]
fn test_plan_wipe_critical_iso_mode_allowed() {
    let plan = plan_wipe(
        "/dev/sda",
        Some(WipePolicy::Clear),
        true,
        true,
        Some(""),
        Some(""),
    );
    assert!(!plan.blocked);
    assert_eq!(plan.risk, "CRITICAL");
    assert!(plan.reason.is_none());
}

#[test]
fn test_plan_wipe_nvme_sanitize() {
    let plan = plan_wipe(
        "/dev/nvme0n1",
        None, // default to PURGE
        false,
        false,
        Some(""),
        Some("sanitize capabilities"),
    );
    assert_eq!(plan.main_method, "controller_sanitize");
    assert_eq!(plan.policy, WipePolicy::Purge);
}

#[test]
fn test_plan_wipe_fallback_overwrite() {
    let plan = plan_wipe(
        "/dev/sdb",
        Some(WipePolicy::Clear),
        false,
        false,
        Some("basic drive info"),
        Some(""),
    );
    assert_eq!(plan.main_method, "overwrite");
    assert!(!plan.hpa_dco_clear);
}

#[test]
fn test_plan_serialization() {
    let plan = plan_wipe(
        "/dev/sda",
        Some(WipePolicy::Purge),
        false,
        false,
        Some("Security sanitize HPA DCO"),
        Some(""),
    );
    let json = serde_json::to_string(&plan).unwrap();
    let deserialized: WipePlan = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.device, "/dev/sda");
    assert_eq!(deserialized.main_method, "controller_sanitize");
    assert!(deserialized.hpa_dco_clear);
}
