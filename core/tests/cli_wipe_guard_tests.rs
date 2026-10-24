//! The guard rails in front of `securewipe wipe --danger-allow-wipe`, end to end.
//!
//! Every test points at a path that is not a device and runs from an empty temp
//! directory, so no `.env` file can switch the danger flag on. None of them can
//! reach the code that writes to a disk.

use assert_cmd::Command;
use serde_json::Value;

const NOT_A_DEVICE: &str = "/nonexistent/securewipe-guard-test";

fn securewipe(cwd: &tempfile::TempDir) -> Command {
    let mut cmd = Command::cargo_bin("securewipe").unwrap();
    cmd.current_dir(cwd.path())
        .env_remove("SECUREWIPE_DANGER")
        .env_remove("SECUREWIPE_CONFIRM_TOKEN");
    cmd
}

fn stderr_of(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

#[test]
fn plan_only_prints_a_plan_and_succeeds() {
    let cwd = tempfile::TempDir::new().unwrap();
    let output = securewipe(&cwd)
        .args(["wipe", "--device", NOT_A_DEVICE, "--policy", "CLEAR"])
        .output()
        .unwrap();

    assert!(output.status.success(), "{}", stderr_of(&output));
    let plan: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(plan["device"], NOT_A_DEVICE);
    assert_eq!(plan["policy"], "CLEAR");
    assert_eq!(plan["blocked"], false);
    assert_eq!(plan["verification"]["strategy"], "random_sectors");
}

#[test]
fn unknown_policy_is_rejected() {
    let cwd = tempfile::TempDir::new().unwrap();
    let output = securewipe(&cwd)
        .args(["wipe", "--device", NOT_A_DEVICE, "--policy", "DESTROY"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(stderr_of(&output).contains("Invalid policy: DESTROY"));
}

#[test]
fn danger_flag_without_env_var_is_refused() {
    let cwd = tempfile::TempDir::new().unwrap();
    let output = securewipe(&cwd)
        .args(["wipe", "--device", NOT_A_DEVICE, "--danger-allow-wipe"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(stderr_of(&output).contains("SECUREWIPE_DANGER=1 environment variable required"));
}

#[test]
fn danger_env_var_other_than_one_is_refused() {
    let cwd = tempfile::TempDir::new().unwrap();
    let output = securewipe(&cwd)
        .env("SECUREWIPE_DANGER", "true")
        .args(["wipe", "--device", NOT_A_DEVICE, "--danger-allow-wipe"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(stderr_of(&output).contains("SECUREWIPE_DANGER=1 environment variable required"));
}

#[test]
fn missing_device_is_refused_even_with_every_flag() {
    let cwd = tempfile::TempDir::new().unwrap();
    let output = securewipe(&cwd)
        .env("SECUREWIPE_DANGER", "1")
        .env("SECUREWIPE_CONFIRM_TOKEN", "CONFIRM WIPE")
        .args(["wipe", "--device", NOT_A_DEVICE, "--danger-allow-wipe"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(stderr_of(&output).contains(&format!("Device {NOT_A_DEVICE} does not exist")));
}
