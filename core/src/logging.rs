use serde_json::{json, Value};
use std::io::{self, Write};

pub struct Logger {
    // In future, this could hold file handles, log levels, etc.
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
    
    #[allow(dead_code)] // Used in tests and future implementations
    pub fn log_info(&self, message: &str) {
        let entry = json!({
            "level": "info",
            "message": message,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        self.log_json(&entry);
    }
    
    #[allow(dead_code)] // Used in tests and future implementations
    pub fn log_error(&self, message: &str) {
        let entry = json!({
            "level": "error",
            "message": message,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        self.log_json(&entry);
    }
}

#[cfg(test)]
mod tests {
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