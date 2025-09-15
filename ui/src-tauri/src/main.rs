// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Stdio;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as TokioCommand;
use tokio::time::timeout;

#[derive(Debug, Serialize, Deserialize)]
struct LogEvent {
    line: String,
    ts: String,
    stream: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExitEvent {
    code: Option<i32>,
    ts: String,
}

fn sanitize_args(args: &[String]) -> Result<Vec<String>, String> {
    if args.is_empty() {
        return Err("No subcommand provided".to_string());
    }

    let subcommand = &args[0];
    let allowed_subcommands = ["discover", "wipe", "backup", "cert"];
    
    if !allowed_subcommands.contains(&subcommand.as_str()) {
        return Err(format!("Subcommand '{}' is not allowed", subcommand));
    }

    // Check for forbidden flags that imply destructive execution
    let forbidden_patterns = [
        "apply", "execute", "i-know", "danger", "yes", "force", "confirm",
        "--apply", "--execute", "--i-know-what-im-doing", "--danger", 
        "--yes", "--force-execute", "--confirm"
    ];

    for arg in args.iter() {
        let arg_lower = arg.to_lowercase();
        for pattern in &forbidden_patterns {
            if arg_lower.contains(pattern) {
                return Err(format!("Forbidden argument detected: {}", arg));
            }
        }
    }

    // For wipe subcommand, ensure it's planning only
    if subcommand == "wipe" {
        // Must have --format json or similar planning flags
        let has_format = args.iter().any(|arg| arg == "--format");
        if !has_format {
            return Err("Wipe command must include --format flag for planning mode".to_string());
        }
    }

    // For cert subcommand, only allow sign and verify operations
    if subcommand == "cert" && args.len() > 1 {
        let operation = &args[1];
        if !["sign", "verify"].contains(&operation.as_str()) {
            return Err(format!("Cert operation '{}' is not allowed", operation));
        }
    }

    Ok(args.to_vec())
}

fn get_current_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string()
}

fn get_executable_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "securewipe.exe"
    } else {
        "securewipe"
    }
}

#[tauri::command]
async fn run_securewipe(
    args: Vec<String>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let sanitized_args = sanitize_args(&args)?;
    let executable = get_executable_name();
    
    // Set maximum runtime based on operation
    let max_runtime = match sanitized_args.first().map(|s| s.as_str()) {
        Some("backup") => Duration::from_secs(20 * 60), // 20 minutes
        Some("discover") | Some("wipe") => Duration::from_secs(2 * 60), // 2 minutes
        Some("cert") => Duration::from_secs(1 * 60), // 1 minute
        _ => Duration::from_secs(5 * 60), // 5 minutes default
    };

    // Check if executable exists before trying to spawn
    let which_result = std::process::Command::new("which")
        .arg(executable)
        .output();
    
    let executable_exists = match which_result {
        Ok(output) => output.status.success(),
        Err(_) => false,
    };

    if !executable_exists {
        // Return mock data for development when securewipe is not available
        return handle_mock_command(&sanitized_args, &app_handle).await;
    }

    let mut cmd = TokioCommand::new(executable);
    cmd.args(&sanitized_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());

    let mut child = cmd.spawn().map_err(|e| {
        format!("Failed to spawn securewipe process: {}. Make sure 'securewipe' is in your PATH.", e)
    })?;

    // Get handles for stdout and stderr
    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

    let app_handle_clone = app_handle.clone();

    // Spawn async task to handle the process
    tokio::spawn(async move {
        let stdout_reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);

        let mut stdout_lines = stdout_reader.lines();
        let mut stderr_lines = stderr_reader.lines();

        // Read stdout and stderr concurrently
        let stdout_task = async {
            while let Ok(Some(line)) = stdout_lines.next_line().await {
                // Truncate very long lines to prevent memory issues
                let truncated_line = if line.len() > 64 * 1024 {
                    format!("{}... [TRUNCATED]", &line[..64 * 1024])
                } else {
                    line
                };

                let event = LogEvent {
                    line: truncated_line,
                    ts: get_current_timestamp(),
                    stream: "stdout".to_string(),
                };

                if let Err(e) = app_handle_clone.emit_all("securewipe://stdout", &event) {
                    eprintln!("Failed to emit stdout event: {}", e);
                }
            }
        };

        let stderr_task = async {
            while let Ok(Some(line)) = stderr_lines.next_line().await {
                let truncated_line = if line.len() > 64 * 1024 {
                    format!("{}... [TRUNCATED]", &line[..64 * 1024])
                } else {
                    line
                };

                let event = LogEvent {
                    line: truncated_line,
                    ts: get_current_timestamp(),
                    stream: "stderr".to_string(),
                };

                if let Err(e) = app_handle_clone.emit_all("securewipe://stderr", &event) {
                    eprintln!("Failed to emit stderr event: {}", e);
                }
            }
        };

        // Wait for either completion or timeout
        let result = timeout(max_runtime, async {
            tokio::join!(stdout_task, stderr_task);
            child.wait().await
        }).await;

        let exit_code = match result {
            Ok(Ok(status)) => status.code(),
            Ok(Err(e)) => {
                eprintln!("Process error: {}", e);
                None
            }
            Err(_) => {
                // Timeout occurred, kill the process
                if let Err(e) = child.kill().await {
                    eprintln!("Failed to kill timed-out process: {}", e);
                }
                None
            }
        };

        let exit_event = ExitEvent {
            code: exit_code,
            ts: get_current_timestamp(),
        };

        if let Err(e) = app_handle_clone.emit_all("securewipe://exit", &exit_event) {
            eprintln!("Failed to emit exit event: {}", e);
        }
    });

    Ok(())
}

async fn handle_mock_command(args: &[String], app_handle: &AppHandle) -> Result<(), String> {
    let subcommand = args.first().map(|s| s.as_str()).unwrap_or("");
    
    // Emit initial log
    let start_event = LogEvent {
        line: format!("[MOCK] Running: securewipe {}", args.join(" ")),
        ts: get_current_timestamp(),
        stream: "stdout".to_string(),
    };
    
    if let Err(e) = app_handle.emit_all("securewipe://stdout", &start_event) {
        eprintln!("Failed to emit mock stdout event: {}", e);
    }

    // Simulate delay
    tokio::time::sleep(Duration::from_millis(500)).await;

    let mock_output = match subcommand {
        "discover" => {
            let mock_log = LogEvent {
                line: "[MOCK] Scanning for storage devices...".to_string(),
                ts: get_current_timestamp(),
                stream: "stdout".to_string(),
            };
            if let Err(e) = app_handle.emit_all("securewipe://stdout", &mock_log) {
                eprintln!("Failed to emit mock stdout event: {}", e);
            }

            tokio::time::sleep(Duration::from_millis(800)).await;

            let mock_devices = r#"[
  {
    "path": "/dev/disk2",
    "model": "Samsung SSD 980 PRO (Mock)",
    "serial": "S5P2NG0N123456",
    "capacity": 1000204886016,
    "bus": "nvme",
    "mountpoints": [],
    "risk_level": "SAFE",
    "blocked": false
  },
  {
    "path": "/dev/disk1",
    "model": "Apple SSD (System)",
    "serial": "APPLE_SSD_123",
    "capacity": 500107862016,
    "bus": "nvme",
    "mountpoints": ["/"],
    "risk_level": "CRITICAL",
    "blocked": true,
    "block_reason": "System disk with active mount points"
  }
]"#;
            mock_devices
        },
        "wipe" => {
            let mock_log1 = LogEvent {
                line: "[MOCK] Analyzing device capabilities...".to_string(),
                ts: get_current_timestamp(),
                stream: "stdout".to_string(),
            };
            if let Err(e) = app_handle.emit_all("securewipe://stdout", &mock_log1) {
                eprintln!("Failed to emit mock stdout event: {}", e);
            }

            tokio::time::sleep(Duration::from_millis(600)).await;

            let mock_log2 = LogEvent {
                line: "[MOCK] Creating wipe plan...".to_string(),
                ts: get_current_timestamp(),
                stream: "stdout".to_string(),
            };
            if let Err(e) = app_handle.emit_all("securewipe://stdout", &mock_log2) {
                eprintln!("Failed to emit mock stdout event: {}", e);
            }

            tokio::time::sleep(Duration::from_millis(400)).await;

            r#"{
  "device_path": "/dev/disk2",
  "policy": "PURGE",
  "main_method": "NVMe Secure Erase (Mock)",
  "hpa_dco_clear": true,
  "verification": {
    "samples": 128
  },
  "blocked": false
}"#
        },
        "backup" => {
            let steps = [
                "[MOCK] Starting encrypted backup...",
                "[MOCK] Creating manifest...",
                "[MOCK] Copying files: 1.2GB / 4.8GB (25%)",
                "[MOCK] Copying files: 2.4GB / 4.8GB (50%)",
                "[MOCK] Copying files: 3.6GB / 4.8GB (75%)",
                "[MOCK] Copying files: 4.8GB / 4.8GB (100%)",
                "[MOCK] Performing integrity checks...",
                "[MOCK] Generating certificates...",
            ];

            for step in &steps {
                let log_event = LogEvent {
                    line: step.to_string(),
                    ts: get_current_timestamp(),
                    stream: "stdout".to_string(),
                };
                if let Err(e) = app_handle.emit_all("securewipe://stdout", &log_event) {
                    eprintln!("Failed to emit mock stdout event: {}", e);
                }
                tokio::time::sleep(Duration::from_millis(300)).await;
            }

            "Backup completed successfully\nCertificate JSON: ~/SecureWipe/certificates/backup_cert_mock.json\nCertificate PDF: ~/SecureWipe/certificates/backup_cert_mock.pdf"
        },
        _ => "[MOCK] Command completed"
    };

    // Emit final output
    if !mock_output.is_empty() {
        let output_event = LogEvent {
            line: mock_output.to_string(),
            ts: get_current_timestamp(),
            stream: "stdout".to_string(),
        };
        
        if let Err(e) = app_handle.emit_all("securewipe://stdout", &output_event) {
            eprintln!("Failed to emit mock stdout event: {}", e);
        }
    }

    // Emit success exit
    let exit_event = ExitEvent {
        code: Some(0),
        ts: get_current_timestamp(),
    };

    if let Err(e) = app_handle.emit_all("securewipe://exit", &exit_event) {
        eprintln!("Failed to emit mock exit event: {}", e);
    }

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![run_securewipe])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
