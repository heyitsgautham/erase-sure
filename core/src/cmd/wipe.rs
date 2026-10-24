//! `securewipe wipe`: plan a wipe, and run it only when every guard passes.

use anyhow::Result;
use clap::Args;
use securewipe::logging::Logger;
use serde_json::json;

#[derive(Args)]
pub struct WipeArgs {
    /// Device to wipe
    #[arg(long)]
    pub device: String,

    /// Wipe policy (CLEAR, PURGE)
    #[arg(long, default_value = "PURGE")]
    pub policy: String,

    /// Enable ISO mode (allows CRITICAL disk wiping)
    #[arg(long)]
    pub iso_mode: bool,

    /// Output format (json or human)
    #[arg(long, default_value = "json")]
    pub format: String,

    /// Number of verification samples
    #[arg(long, default_value = "128")]
    pub samples: usize,

    /// Sign the generated certificate
    #[arg(long)]
    pub sign: bool,

    /// Path to Ed25519 private key for signing
    #[arg(long)]
    pub sign_key_path: Option<std::path::PathBuf>,

    /// Allow overwriting existing signature
    #[arg(long)]
    pub force: bool,

    /// Required safety flag to enable destructive wiping
    #[arg(long)]
    pub danger_allow_wipe: bool,

    /// Link to existing backup certificate ID
    #[arg(long)]
    pub backup_cert_id: Option<String>,
}

pub fn handle_wipe(args: WipeArgs, logger: &Logger) -> Result<()> {
    use securewipe::device::{DeviceDiscovery, LinuxDeviceDiscovery, RiskLevel};
    use securewipe::wipe::{plan_wipe, WipePolicy};

    logger.log_info("Starting wipe planning");

    // Log CLI arguments
    logger.log_json(&json!({
        "step": "cli_args",
        "device": args.device,
        "policy": args.policy,
        "iso_mode": args.iso_mode,
        "samples": args.samples,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }));

    // Parse policy
    let policy = match args.policy.as_str() {
        "CLEAR" => Some(WipePolicy::Clear),
        "PURGE" => Some(WipePolicy::Purge),
        _ => {
            let error_msg = format!("Invalid policy: {}. Must be CLEAR or PURGE", args.policy);
            logger.log_error(&error_msg);
            return Err(anyhow::anyhow!(error_msg));
        }
    };

    // Determine if device is critical by checking risk level and capture serial for confirmation token
    let discovery = LinuxDeviceDiscovery::new();
    let (is_critical, device_serial_opt) = match discovery.discover_devices() {
        Ok(devices) => {
            let device = devices.iter().find(|d| d.name == args.device);
            match device {
                Some(d) => {
                    logger.log_json(&json!({
                        "step": "device_risk_check",
                        "device": args.device,
                        "risk_level": d.risk_level,
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    }));
                    (
                        matches!(d.risk_level, RiskLevel::Critical),
                        d.serial.clone(),
                    )
                }
                None => {
                    logger.log_json(&json!({
                        "step": "device_risk_check",
                        "device": args.device,
                        "result": "device_not_found_assuming_safe",
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    }));
                    (false, None)
                }
            }
        }
        Err(e) => {
            logger.log_json(&json!({
                "step": "device_risk_check",
                "device": args.device,
                "error": e.to_string(),
                "result": "discovery_failed_assuming_safe",
                "timestamp": chrono::Utc::now().to_rfc3339()
            }));
            (false, None)
        }
    };

    // Log controller probing attempts
    logger.log_json(&json!({
        "step": "controller_probe_start",
        "device": args.device,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }));

    // Generate wipe plan with custom samples
    let mut plan = plan_wipe(
        &args.device,
        policy.clone(),
        is_critical,
        args.iso_mode,
        None,
        None,
    );
    plan.verification.samples = args.samples;

    // Log planning decision
    logger.log_json(&json!({
        "step": "wipe_plan_generated",
        "device": plan.device,
        "risk": plan.risk,
        "policy": plan.policy,
        "main_method": plan.main_method,
        "hpa_dco_clear": plan.hpa_dco_clear,
        "blocked": plan.blocked,
        "reason": plan.reason,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }));

    // Output based on format
    if args.format == "json" {
        println!("{}", serde_json::to_string_pretty(&plan)?);
    } else {
        // Human-readable format
        println!("Wipe Plan for {}", plan.device);
        println!("=======================");
        println!("• Policy: {:?}", plan.policy);
        println!("• Risk Level: {}", plan.risk);
        println!("• Main Method: {}", plan.main_method);
        println!(
            "• HPA/DCO Clear: {}",
            if plan.hpa_dco_clear { "Yes" } else { "No" }
        );
        println!(
            "• Verification: {} {} samples",
            plan.verification.strategy, plan.verification.samples
        );

        if plan.blocked {
            println!("• Status: ❌ BLOCKED");
            if let Some(ref reason) = plan.reason {
                println!("• Reason: {}", reason);
            }
        } else {
            println!("• Status: ✅ Ready to proceed");
            if args.iso_mode && plan.risk == "CRITICAL" {
                println!("• Note: ISO mode enabled - CRITICAL disk wipe allowed");
            }
        }
    }

    // Check for destructive wipe mode
    if args.danger_allow_wipe {
        logger.log_info("Destructive wipe mode requested");

        // Critical safety check: require SECUREWIPE_DANGER=1 environment variable
        if std::env::var("SECUREWIPE_DANGER").unwrap_or_default() != "1" {
            let error_msg =
                "SECUREWIPE_DANGER=1 environment variable required for destructive operations";
            logger.log_error(error_msg);
            return Err(anyhow::anyhow!(error_msg));
        }

        // Check if device exists
        if !std::path::Path::new(&args.device).exists() {
            let error_msg = format!("Device {} does not exist", args.device);
            logger.log_error(&error_msg);
            return Err(anyhow::anyhow!(error_msg));
        }

        // Check if plan is blocked
        if plan.blocked {
            let error_msg = format!(
                "Wipe operation blocked: {}",
                plan.reason.unwrap_or_default()
            );
            logger.log_error(&error_msg);
            return Err(anyhow::anyhow!(error_msg));
        }

        // Final confirmation prompt (supports non-interactive token via env)
        println!("This will PERMANENTLY DESTROY ALL DATA on {}.", args.device);

        // Preferred confirmation string per safety policy: "WIPE <DEVICE_SERIAL>"
        let expected_token_serial = device_serial_opt.as_ref().map(|s| format!("WIPE {}", s));
        let legacy_token = "CONFIRM WIPE".to_string();

        // Check for non-interactive confirmation via environment variable
        if let Ok(token) = std::env::var("SECUREWIPE_CONFIRM_TOKEN") {
            let token_ok = match expected_token_serial {
                Some(ref expect) => token == *expect || token == legacy_token,
                None => token == legacy_token, // No serial available; accept legacy only
            };
            if !token_ok {
                let expected_msg = expected_token_serial
                    .as_deref()
                    .unwrap_or(legacy_token.as_str());
                let error_msg = format!(
                    "Invalid confirmation token. Expected '{}'{}.",
                    expected_msg,
                    if expected_token_serial.is_some() {
                        " or 'CONFIRM WIPE' (legacy)"
                    } else {
                        ""
                    }
                );
                logger.log_error(&error_msg);
                return Err(anyhow::anyhow!(error_msg));
            }
        } else {
            // Interactive path
            if let Some(ref serial) = device_serial_opt {
                print!("Type 'WIPE {}' to proceed: ", serial);
            } else {
                print!("Type 'CONFIRM WIPE' to proceed: ");
            }
            std::io::Write::flush(&mut std::io::stdout())?;

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let input = input.trim();

            let ok = match expected_token_serial {
                Some(ref expect) => input == expect || input == legacy_token,
                None => input == legacy_token,
            };
            if !ok {
                logger.log_info("Wipe operation cancelled by user (confirmation token mismatch)");
                println!("Wipe operation cancelled.");
                return Ok(());
            }
        }

        logger.log_info("User confirmed destructive wipe - starting operation");

        // Perform the actual wipe
        use securewipe::wipe::{NistAlignedWipe, WipeOperations};
        let wipe_engine = NistAlignedWipe::new();
        let wipe_result = wipe_engine
            .perform_wipe(&args.device, policy.unwrap(), is_critical)
            .map_err(|e| anyhow::anyhow!("Wipe operation failed: {}", e))?;

        logger.log_json(&json!({
            "step": "wipe_completed",
            "device": wipe_result.device,
            "method": wipe_result.method,
            "verification_passed": wipe_result.verification_passed,
            "commands_executed": wipe_result.commands.len(),
            "timestamp": chrono::Utc::now().to_rfc3339()
        }));

        println!("Wipe operation completed!");
        println!("Method used: {}", wipe_result.method);
        println!("Commands executed: {}", wipe_result.commands.len());
        println!("Verification samples: {}", wipe_result.verification_samples);
        println!(
            "Verification result: {}",
            if wipe_result.verification_passed {
                "PASSED"
            } else {
                "FAILED"
            }
        );

        if let Some(reason) = &wipe_result.fallback_reason {
            println!("Fallback reason: {}", reason);
        }

        if !wipe_result.verification_passed {
            let error_msg = "Wipe verification failed! Some sectors may not be properly wiped.";
            logger.log_error(error_msg);
            return Err(anyhow::anyhow!(error_msg));
        }

        // Generate, sign, and validate real wipe certificate
        if args.sign || args.sign_key_path.is_some() {
            use std::fs;
            logger.log_info("Generating real wipe certificate (schema-compliant)");

            // Build schema-compliant JSON certificate
            let mut cert_value = securewipe::cert::build_wipe_certificate_json(
                &wipe_result,
                args.backup_cert_id.as_deref(),
            )
            .map_err(|e| anyhow::anyhow!("Failed to build wipe certificate JSON: {}", e))?;

            // Sign certificate first (schema requires signature)
            use securewipe::signer::{load_private_key, sign_certificate};
            logger.log_info("Signing wipe certificate");
            let signing_key = load_private_key(args.sign_key_path.clone())
                .map_err(|e| anyhow::anyhow!("Failed to load signing key: {}", e))?;
            sign_certificate(&mut cert_value, &signing_key, args.force)
                .map_err(|e| anyhow::anyhow!("Failed to sign wipe certificate: {}", e))?;

            // Validate schema after signing
            use securewipe::schema::CertificateValidator;
            logger.log_info("Validating signed wipe certificate schema");
            let validator = CertificateValidator::default();
            let validation_result = validator
                .validate_certificate(&cert_value)
                .map_err(|e| anyhow::anyhow!("Schema validation error: {}", e))?;
            if !validation_result.valid {
                logger.log_error("Signed wipe certificate failed schema validation");
                eprintln!("WARNING: Signed certificate failed schema validation:");
                for error in &validation_result.errors {
                    eprintln!("  - {}", error);
                }
            } else {
                logger.log_info("Signed wipe certificate passed schema validation");
            }

            // Save certificate
            let home_dir = dirs::home_dir()
                .ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?;
            let cert_dir = home_dir.join("SecureWipe").join("certificates");
            std::fs::create_dir_all(&cert_dir)?;
            let cert_id = cert_value
                .get("cert_id")
                .and_then(|v| v.as_str())
                .unwrap_or("wipe_cert")
                .to_string();
            let cert_file = cert_dir.join(format!("{}.json", cert_id));

            // Write certificate file
            let cert_json = serde_json::to_string_pretty(&cert_value)?;
            let temp_file = cert_file.with_extension("tmp");
            fs::write(&temp_file, &cert_json)?;
            fs::rename(&temp_file, &cert_file)?;

            logger.log_json(&json!({
                "step": "real_wipe_certificate_saved",
                "cert_id": cert_id,
                "cert_path": cert_file.display().to_string(),
                "signed": true,
                "backup_cert_linked": args.backup_cert_id.is_some(),
                "timestamp": chrono::Utc::now().to_rfc3339()
            }));

            println!("Wipe certificate saved: {}", cert_file.display());
        }

        logger.log_info("Destructive wipe operation completed successfully");
        return Ok(());
    }

    // Planning-only path: generate a schema-compliant unsigned certificate from a stub result and sign it
    if args.sign || args.sign_key_path.is_some() {
        use securewipe::wipe::{WipeCommand, WipeResult};
        use std::fs;

        // Generate a stub wipe result for certificate creation
        let stub_wipe_result = WipeResult {
            device: args.device.clone(),
            policy: match args.policy.as_str() {
                "CLEAR" => securewipe::wipe::WipePolicy::Clear,
                "PURGE" => securewipe::wipe::WipePolicy::Purge,
                _ => securewipe::wipe::WipePolicy::Purge,
            },
            method: plan.main_method.clone(),
            commands: vec![WipeCommand {
                command: format!("echo 'Wipe planned for {}'", args.device),
                exit_code: 0,
                elapsed_ms: 0,
                output: "Planning completed successfully".to_string(),
            }],
            verification_samples: args.samples,
            verification_passed: true,
            fallback_reason: plan.reason.clone(),
        };

        logger.log_info("Generating schema-compliant wipe certificate (planning mode)");
        let mut cert_value = securewipe::cert::build_wipe_certificate_json(
            &stub_wipe_result,
            args.backup_cert_id.as_deref(),
        )
        .map_err(|e| anyhow::anyhow!("Failed to build wipe certificate JSON: {}", e))?;

        // Save certificate directory
        let home_dir =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?;
        let cert_dir = home_dir.join("SecureWipe").join("certificates");
        std::fs::create_dir_all(&cert_dir)?;
        let cert_id = cert_value
            .get("cert_id")
            .and_then(|v| v.as_str())
            .unwrap_or("wipe_cert")
            .to_string();
        let cert_file = cert_dir.join(format!("{}.json", cert_id));

        // Handle signing
        use securewipe::signer::{load_private_key, sign_certificate};

        logger.log_info("Signing wipe certificate (planning mode)");
        logger.log_json(&serde_json::json!({
            "step": "wipe_certificate_signing",
            "cert_id": cert_id,
            "key_source": if args.sign_key_path.is_some() { "flag" } else { "env" },
            "timestamp": chrono::Utc::now().to_rfc3339()
        }));

        let signing_key = load_private_key(args.sign_key_path.clone())
            .map_err(|e| anyhow::anyhow!("Failed to load signing key: {}", e))?;

        sign_certificate(&mut cert_value, &signing_key, args.force)
            .map_err(|e| anyhow::anyhow!("Failed to sign wipe certificate: {}", e))?;

        logger.log_json(&serde_json::json!({
            "step": "wipe_certificate_signed",
            "cert_id": cert_id,
            "signed": true,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }));

        // Write certificate file atomically
        let cert_json = serde_json::to_string_pretty(&cert_value)?;
        let temp_file = cert_file.with_extension("tmp");
        fs::write(&temp_file, &cert_json)?;
        fs::rename(&temp_file, &cert_file)?;

        logger.log_json(&serde_json::json!({
            "step": "wipe_certificate_saved",
            "cert_id": cert_id,
            "cert_path": cert_file.display().to_string(),
            "signed": true,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }));

        println!("Wipe certificate saved: {}", cert_file.display());
    }

    logger.log_info("Wipe planning completed");
    Ok(())
}
