use serde_json::Value;
use std::io::{self, Write};
use tracing_subscriber::EnvFilter;

/// Environment variable that sets the log filter, in `tracing` EnvFilter syntax.
pub const LOG_FILTER_ENV: &str = "SECUREWIPE_LOG";

/// Build the filter for [`init_tracing`]: `SECUREWIPE_LOG` when set and valid, `info` otherwise.
pub fn log_filter(spec: Option<&str>) -> EnvFilter {
    spec.and_then(|s| EnvFilter::try_new(s).ok())
        .unwrap_or_else(|| EnvFilter::new("info"))
}

/// Send `tracing` events to stderr as one JSON object per line.
///
/// stdout stays reserved for the command's JSON result, which the desktop UI parses.
/// Returns false when a global subscriber was already installed.
pub fn init_tracing() -> bool {
    let spec = std::env::var(LOG_FILTER_ENV).ok();
    tracing_subscriber::fmt()
        .json()
        .flatten_event(true)
        .with_current_span(false)
        .with_span_list(false)
        .with_writer(io::stderr)
        .with_env_filter(log_filter(spec.as_deref()))
        .try_init()
        .is_ok()
}

pub struct Logger {
    // In future, this could hold file handles, log levels, etc.
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

impl Logger {
    pub fn new() -> Self {
        Self {}
    }

    pub fn log_json(&self, data: &Value) {
        // For now, log to stderr for structured logging
        if let Ok(json_str) = serde_json::to_string(data) {
            let _ = writeln!(io::stderr(), "{}", json_str);
        }
    }

    /// Log a plain message at info level through `tracing`.
    pub fn log_info(&self, message: &str) {
        tracing::info!(target: "securewipe::cli", "{message}");
    }

    /// Log a plain message at error level through `tracing`.
    pub fn log_error(&self, message: &str) {
        tracing::error!(target: "securewipe::cli", "{message}");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn log_filter_defaults_to_info_and_accepts_overrides() {
        assert_eq!(log_filter(None).to_string(), "info");
        assert_eq!(log_filter(Some("not a filter ===")).to_string(), "info");
        assert_eq!(
            log_filter(Some("securewipe=debug")).to_string(),
            "securewipe=debug"
        );
    }

    #[test]
    fn init_tracing_installs_once() {
        // The first call in this process may or may not win against other tests,
        // but a second call must never succeed or panic.
        let _ = init_tracing();
        assert!(!init_tracing());
    }

    use super::*;
    use serde_json::json;

    #[test]
    fn test_logger_creation() {
        let _logger = Logger::new();
        // Basic compilation test
    }

    #[test]
    fn test_log_json() {
        let logger = Logger::new();
        let test_data = json!({
            "test": "value",
            "number": 42
        });

        // This should not panic
        logger.log_json(&test_data);
    }

    #[test]
    fn test_log_info() {
        let logger = Logger::new();
        logger.log_info("Test info message");
        // Should not panic
    }

    #[test]
    fn test_log_error() {
        let logger = Logger::new();
        logger.log_error("Test error message");
        // Should not panic
    }

    #[test]
    fn test_log_structured_format() {
        let logger = Logger::new();

        // Test that the structured format includes expected fields
        let test_message = "Test structured logging";
        logger.log_info(test_message);
        logger.log_error(test_message);

        // These should generate JSON with level, message, and timestamp fields
        // In a real test environment, we might capture stderr to verify format
    }
}
