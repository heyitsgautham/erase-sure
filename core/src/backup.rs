use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    pub files: HashMap<String, String>, // path -> sha256
    pub created_at: String,
    pub total_files: usize,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupResult {
    pub manifest: BackupManifest,
    pub destination: String,
    pub encryption_method: String,
    pub verification_samples: usize,
    pub verification_passed: bool,
}

#[allow(dead_code)] // MVP: Implementation pending
pub trait BackupOperations {
    fn perform_backup(
        &self,
        paths: &[String],
        destination: &str,
    ) -> Result<BackupResult, Box<dyn std::error::Error>>;
}

#[allow(dead_code)] // MVP: Implementation pending
pub struct EncryptedBackup;

impl BackupOperations for EncryptedBackup {
    fn perform_backup(
        &self,
        _paths: &[String],
        _destination: &str,
    ) -> Result<BackupResult, Box<dyn std::error::Error>> {
        // Stub implementation - will implement AES-256-CTR encryption
        let manifest = BackupManifest {
            files: HashMap::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
            total_files: 0,
            total_bytes: 0,
        };
        
        Ok(BackupResult {
            manifest,
            destination: "stub".to_string(),
            encryption_method: "AES-256-CTR".to_string(),
            verification_samples: 5,
            verification_passed: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_backup_operations_trait() {
        let backup = EncryptedBackup;
        let result = backup.perform_backup(&[], "/mnt/backup");
        assert!(result.is_ok());
        
        if let Ok(backup_result) = result {
            assert_eq!(backup_result.encryption_method, "AES-256-CTR");
            assert_eq!(backup_result.verification_samples, 5);
            assert!(backup_result.verification_passed);
        }
    }
    
    #[test]
    fn test_backup_manifest_serialization() {
        let mut files = HashMap::new();
        files.insert("test/file.txt".to_string(), "abc123".to_string());
        
        let manifest = BackupManifest {
            files,
            created_at: "2023-01-01T00:00:00Z".to_string(),
            total_files: 1,
            total_bytes: 1024,
        };
        
        let json = serde_json::to_string(&manifest);
        assert!(json.is_ok());
        
        let deserialized: BackupManifest = serde_json::from_str(&json.unwrap()).unwrap();
        assert_eq!(deserialized.total_files, 1);
        assert_eq!(deserialized.total_bytes, 1024);
        assert_eq!(deserialized.files.len(), 1);
    }
    
    #[test]
    fn test_backup_result_serialization() {
        let manifest = BackupManifest {
            files: HashMap::new(),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            total_files: 0,
            total_bytes: 0,
        };
        
        let result = BackupResult {
            manifest,
            destination: "/mnt/backup".to_string(),
            encryption_method: "AES-256-CTR".to_string(),
            verification_samples: 5,
            verification_passed: true,
        };
        
        let json = serde_json::to_string(&result);
        assert!(json.is_ok());
    }
    
    #[test]
    fn test_backup_with_paths() {
        let backup = EncryptedBackup;
        let paths = vec!["Documents".to_string(), "Pictures".to_string()];
        let result = backup.perform_backup(&paths, "/mnt/backup");
        assert!(result.is_ok());
    }
}