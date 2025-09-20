use anyhow::{Context, Result};
use jsonschema::{JSONSchema, ValidationError};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// JSON Schema validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub schema_id: Option<String>,
}

impl ValidationResult {
    pub fn success(schema_id: Option<String>) -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            schema_id,
        }
    }

    pub fn failure(errors: Vec<String>, schema_id: Option<String>) -> Self {
        Self {
            valid: false,
            errors,
            schema_id,
        }
    }
}

/// Certificate schema validator
pub struct CertificateValidator {
    backup_schema: Option<JSONSchema>,
    wipe_schema: Option<JSONSchema>,
}

impl CertificateValidator {
    /// Create a new validator, loading schemas from the standard location
    pub fn new() -> Result<Self> {
        Self::from_schema_dir(None)
    }

    /// Create a validator with schemas from a specific directory
    pub fn from_schema_dir(schema_dir: Option<PathBuf>) -> Result<Self> {
        let schema_dir = schema_dir.unwrap_or_else(|| {
            // Try to find schema directory relative to project root
            let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            
            // Look for certs/schemas relative to current directory or parent directories
            for _ in 0..5 {
                let candidate = path.join("certs").join("schemas");
                if candidate.exists() {
                    return candidate;
                }
                if !path.pop() {
                    break;
                }
            }
            
            // Fallback to relative path from current directory
            PathBuf::from("certs/schemas")
        });

        info!(schema_dir = %schema_dir.display(), "Loading certificate schemas");

        let backup_schema = Self::load_schema(&schema_dir, "backup_schema.json")?;
        let wipe_schema = Self::load_schema(&schema_dir, "wipe_schema.json")?;

        Ok(Self {
            backup_schema,
            wipe_schema,
        })
    }

    /// Load a schema from file
    fn load_schema(schema_dir: &Path, filename: &str) -> Result<Option<JSONSchema>> {
        let schema_path = schema_dir.join(filename);
        
        if !schema_path.exists() {
            warn!(schema_path = %schema_path.display(), "Schema file not found, validation will be skipped");
            return Ok(None);
        }

        let schema_content = fs::read_to_string(&schema_path)
            .with_context(|| format!("Failed to read schema file: {}", schema_path.display()))?;

        let schema_value: Value = serde_json::from_str(&schema_content)
            .with_context(|| format!("Failed to parse schema JSON: {}", schema_path.display()))?;

        let compiled_schema = JSONSchema::compile(&schema_value)
            .map_err(|e| anyhow::anyhow!("Failed to compile schema {}: {}", schema_path.display(), e))?;

        debug!(schema_path = %schema_path.display(), "Schema loaded successfully");
        Ok(Some(compiled_schema))
    }

    /// Validate a certificate JSON value
    pub fn validate_certificate(&self, cert_value: &Value) -> Result<ValidationResult> {
        let cert_type = cert_value.get("cert_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Certificate missing 'cert_type' field"))?;

        match cert_type {
            "backup" => self.validate_backup_certificate(cert_value),
            "wipe" => self.validate_wipe_certificate(cert_value),
            _ => Err(anyhow::anyhow!("Unsupported certificate type: {}", cert_type)),
        }
    }

    /// Validate a backup certificate
    pub fn validate_backup_certificate(&self, cert_value: &Value) -> Result<ValidationResult> {
        match &self.backup_schema {
            Some(schema) => {
                let validation_result = schema.validate(cert_value);
                match validation_result {
                    Ok(()) => {
                        debug!("Backup certificate passed schema validation");
                        Ok(ValidationResult::success(Some("backup".to_string())))
                    }
                    Err(validation_errors) => {
                        let errors: Vec<String> = validation_errors
                            .map(|error| format_validation_error(&error))
                            .collect();
                        
                        debug!(errors = ?errors, "Backup certificate failed schema validation");
                        Ok(ValidationResult::failure(errors, Some("backup".to_string())))
                    }
                }
            }
            None => {
                warn!("Backup schema not loaded, skipping validation");
                Ok(ValidationResult::success(Some("backup".to_string())))
            }
        }
    }

    /// Validate a wipe certificate
    pub fn validate_wipe_certificate(&self, cert_value: &Value) -> Result<ValidationResult> {
        match &self.wipe_schema {
            Some(schema) => {
                let validation_result = schema.validate(cert_value);
                match validation_result {
                    Ok(()) => {
                        debug!("Wipe certificate passed schema validation");
                        Ok(ValidationResult::success(Some("wipe".to_string())))
                    }
                    Err(validation_errors) => {
                        let errors: Vec<String> = validation_errors
                            .map(|error| format_validation_error(&error))
                            .collect();
                        
                        debug!(errors = ?errors, "Wipe certificate failed schema validation");
                        Ok(ValidationResult::failure(errors, Some("wipe".to_string())))
                    }
                }
            }
            None => {
                warn!("Wipe schema not loaded, skipping validation");
                Ok(ValidationResult::success(Some("wipe".to_string())))
            }
        }
    }

    /// Validate certificate from JSON string
    pub fn validate_certificate_json(&self, cert_json: &str) -> Result<ValidationResult> {
        let cert_value: Value = serde_json::from_str(cert_json)
            .context("Failed to parse certificate JSON")?;
        
        self.validate_certificate(&cert_value)
    }

    /// Validate certificate from file
    pub fn validate_certificate_file(&self, cert_path: &Path) -> Result<ValidationResult> {
        let cert_json = fs::read_to_string(cert_path)
            .with_context(|| format!("Failed to read certificate file: {}", cert_path.display()))?;
        
        self.validate_certificate_json(&cert_json)
    }
}

impl Default for CertificateValidator {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            warn!(error = %e, "Failed to load schemas, validation will be disabled");
            Self {
                backup_schema: None,
                wipe_schema: None,
            }
        })
    }
}

/// Format a validation error for human-readable output
fn format_validation_error(error: &ValidationError) -> String {
    let instance_path = error.instance_path.to_string();
    let instance_path = if instance_path.is_empty() {
        "root".to_string()
    } else {
        instance_path
    };

    format!("Validation error at {}: {}", instance_path, error)
}

/// Convenience function to validate a certificate value
pub fn validate_certificate(cert_value: &Value) -> Result<ValidationResult> {
    let validator = CertificateValidator::default();
    validator.validate_certificate(cert_value)
}

/// Convenience function to validate a certificate JSON string
pub fn validate_certificate_json(cert_json: &str) -> Result<ValidationResult> {
    let validator = CertificateValidator::default();
    validator.validate_certificate_json(cert_json)
}

/// Convenience function to validate a certificate file
pub fn validate_certificate_file(cert_path: &Path) -> Result<ValidationResult> {
    let validator = CertificateValidator::default();
    validator.validate_certificate_file(cert_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;
    use std::fs;

    fn create_test_backup_schema() -> Value {
        json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "cert_type": {
                    "const": "backup"
                },
                "cert_id": {
                    "type": "string"
                },
                "created_at": {
                    "type": "string"
                },
                "device": {
                    "type": "object",
                    "properties": {
                        "model": {"type": "string"},
                        "serial": {"type": "string"}
                    },
                    "required": ["model", "serial"]
                }
            },
            "required": ["cert_type", "cert_id", "created_at", "device"]
        })
    }

    fn create_test_wipe_schema() -> Value {
        json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "cert_type": {
                    "const": "wipe"
                },
                "cert_id": {
                    "type": "string"
                },
                "created_at": {
                    "type": "string"
                },
                "device": {
                    "type": "object",
                    "properties": {
                        "model": {"type": "string"},
                        "serial": {"type": "string"}
                    },
                    "required": ["model", "serial"]
                }
            },
            "required": ["cert_type", "cert_id", "created_at", "device"]
        })
    }

    fn setup_test_schemas() -> Result<TempDir> {
        let temp_dir = TempDir::new()?;
        let schema_dir = temp_dir.path().join("certs").join("schemas");
        fs::create_dir_all(&schema_dir)?;

        // Write test schemas
        let backup_schema = create_test_backup_schema();
        fs::write(
            schema_dir.join("backup_schema.json"),
            serde_json::to_string_pretty(&backup_schema)?
        )?;

        let wipe_schema = create_test_wipe_schema();
        fs::write(
            schema_dir.join("wipe_schema.json"),
            serde_json::to_string_pretty(&wipe_schema)?
        )?;

        Ok(temp_dir)
    }

    #[test]
    fn test_validator_creation() {
        let temp_dir = setup_test_schemas().unwrap();
        let schema_dir = temp_dir.path().join("certs").join("schemas");
        
        let validator = CertificateValidator::from_schema_dir(Some(schema_dir));
        assert!(validator.is_ok());
        
        let validator = validator.unwrap();
        assert!(validator.backup_schema.is_some());
        assert!(validator.wipe_schema.is_some());
    }

    #[test]
    fn test_valid_backup_certificate() {
        let temp_dir = setup_test_schemas().unwrap();
        let schema_dir = temp_dir.path().join("certs").join("schemas");
        let validator = CertificateValidator::from_schema_dir(Some(schema_dir)).unwrap();

        let valid_cert = json!({
            "cert_type": "backup",
            "cert_id": "backup_123",
            "created_at": "2023-12-05T14:30:22Z",
            "device": {
                "model": "Test SSD",
                "serial": "ABC123"
            }
        });

        let result = validator.validate_certificate(&valid_cert).unwrap();
        assert!(result.valid);
        assert!(result.errors.is_empty());
        assert_eq!(result.schema_id, Some("backup".to_string()));
    }

    #[test]
    fn test_invalid_backup_certificate_missing_field() {
        let temp_dir = setup_test_schemas().unwrap();
        let schema_dir = temp_dir.path().join("certs").join("schemas");
        let validator = CertificateValidator::from_schema_dir(Some(schema_dir)).unwrap();

        let invalid_cert = json!({
            "cert_type": "backup",
            "cert_id": "backup_123"
            // Missing created_at and device
        });

        let result = validator.validate_certificate(&invalid_cert).unwrap();
        assert!(!result.valid);
        assert!(!result.errors.is_empty());
        assert!(result.errors.iter().any(|e| e.contains("created_at")));
        assert!(result.errors.iter().any(|e| e.contains("device")));
    }

    #[test]
    fn test_invalid_certificate_type() {
        let temp_dir = setup_test_schemas().unwrap();
        let schema_dir = temp_dir.path().join("certs").join("schemas");
        let validator = CertificateValidator::from_schema_dir(Some(schema_dir)).unwrap();

        let invalid_cert = json!({
            "cert_type": "invalid_type",
            "cert_id": "test_123"
        });

        let result = validator.validate_certificate(&invalid_cert);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported certificate type"));
    }

    #[test]
    fn test_valid_wipe_certificate() {
        let temp_dir = setup_test_schemas().unwrap();
        let schema_dir = temp_dir.path().join("certs").join("schemas");
        let validator = CertificateValidator::from_schema_dir(Some(schema_dir)).unwrap();

        let valid_cert = json!({
            "cert_type": "wipe",
            "cert_id": "wipe_456",
            "created_at": "2023-12-05T15:00:30Z",
            "device": {
                "model": "Test SSD",
                "serial": "ABC123"
            }
        });

        let result = validator.validate_certificate(&valid_cert).unwrap();
        assert!(result.valid);
        assert!(result.errors.is_empty());
        assert_eq!(result.schema_id, Some("wipe".to_string()));
    }

    #[test]
    fn test_certificate_json_validation() {
        let temp_dir = setup_test_schemas().unwrap();
        let schema_dir = temp_dir.path().join("certs").join("schemas");
        let validator = CertificateValidator::from_schema_dir(Some(schema_dir)).unwrap();

        let cert_json = r#"{
            "cert_type": "backup",
            "cert_id": "backup_789",
            "created_at": "2023-12-05T16:00:00Z",
            "device": {
                "model": "Test SSD",
                "serial": "DEF456"
            }
        }"#;

        let result = validator.validate_certificate_json(cert_json).unwrap();
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validation_result_success() {
        let result = ValidationResult::success(Some("backup".to_string()));
        assert!(result.valid);
        assert!(result.errors.is_empty());
        assert_eq!(result.schema_id, Some("backup".to_string()));
    }

    #[test]
    fn test_validation_result_failure() {
        let errors = vec!["Error 1".to_string(), "Error 2".to_string()];
        let result = ValidationResult::failure(errors.clone(), Some("wipe".to_string()));
        assert!(!result.valid);
        assert_eq!(result.errors, errors);
        assert_eq!(result.schema_id, Some("wipe".to_string()));
    }

    #[test]
    fn test_convenience_functions() {
        let temp_dir = setup_test_schemas().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let valid_cert = json!({
            "cert_type": "backup",
            "cert_id": "backup_conv",
            "created_at": "2023-12-05T17:00:00Z",
            "device": {
                "model": "Test SSD",
                "serial": "GHI789"
            }
        });

        let result = validate_certificate(&valid_cert).unwrap();
        assert!(result.valid);

        let cert_json = serde_json::to_string(&valid_cert).unwrap();
        let result = validate_certificate_json(&cert_json).unwrap();
        assert!(result.valid);
    }
}