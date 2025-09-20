use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, warn, error};
use serde_json::Value;

/// Python-based PDF generator that calls the high-quality Python scripts
pub struct PythonPdfGenerator {
    project_root: PathBuf,
}

impl PythonPdfGenerator {
    pub fn new() -> Result<Self> {
        // Find project root by looking for the tests directory
        let current_dir = std::env::current_dir()?;
        let project_root = find_project_root(&current_dir)
            .ok_or_else(|| anyhow::anyhow!("Could not find project root"))?;
        
        Ok(Self { project_root })
    }

    /// Generate backup certificate PDF using Python script
    pub fn generate_backup_pdf(
        &self,
        cert_json: &str,
        output_path: &Path,
    ) -> Result<PathBuf> {
        info!("Generating backup PDF using Python script");
        
        // Create a temporary JSON file for the certificate
        let temp_cert_file = self.create_temp_cert_file(cert_json)?;
        
        // Path to the Python PDF generator script
        let python_script = self.project_root.join("core/src/python_pdf_generator.py");
        
        // Ensure the Python script exists, if not create it
        if !python_script.exists() {
            self.create_python_pdf_script(&python_script)?;
        }
        
        // Call Python script
        let output = Command::new("python3")
            .arg(&python_script)
            .arg("--cert-file")
            .arg(&temp_cert_file)
            .arg("--output")
            .arg(output_path)
            .arg("--type")
            .arg("backup")
            .current_dir(&self.project_root)
            .output()
            .context("Failed to execute Python PDF generator")?;
        
        // Clean up temp file
        if let Err(e) = std::fs::remove_file(&temp_cert_file) {
            warn!("Failed to clean up temp file: {}", e);
        }
        
        if !output.status.success() {
            error!("Python script failed: {}", String::from_utf8_lossy(&output.stderr));
            return Err(anyhow::anyhow!(
                "Python PDF generation failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        
        info!("Python PDF generation completed successfully");
        info!("Output: {}", String::from_utf8_lossy(&output.stdout));
        
        Ok(output_path.to_path_buf())
    }

    /// Generate wipe certificate PDF using Python script
    pub fn generate_wipe_pdf(
        &self,
        cert_json: &str,
        output_path: &Path,
    ) -> Result<PathBuf> {
        info!("Generating wipe PDF using Python script");
        
        // Create a temporary JSON file for the certificate
        let temp_cert_file = self.create_temp_cert_file(cert_json)?;
        
        // Path to the Python PDF generator script
        let python_script = self.project_root.join("core/src/python_pdf_generator.py");
        
        // Ensure the Python script exists
        if !python_script.exists() {
            self.create_python_pdf_script(&python_script)?;
        }
        
        // Call Python script
        let output = Command::new("python3")
            .arg(&python_script)
            .arg("--cert-file")
            .arg(&temp_cert_file)
            .arg("--output")
            .arg(output_path)
            .arg("--type")
            .arg("wipe")
            .current_dir(&self.project_root)
            .output()
            .context("Failed to execute Python PDF generator")?;
        
        // Clean up temp file
        if let Err(e) = std::fs::remove_file(&temp_cert_file) {
            warn!("Failed to clean up temp file: {}", e);
        }
        
        if !output.status.success() {
            error!("Python script failed: {}", String::from_utf8_lossy(&output.stderr));
            return Err(anyhow::anyhow!(
                "Python PDF generation failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        
        info!("Python PDF generation completed successfully");
        Ok(output_path.to_path_buf())
    }

    fn create_temp_cert_file(&self, cert_json: &str) -> Result<PathBuf> {
        use std::io::Write;
        
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join(format!("cert_{}.json", uuid::Uuid::new_v4()));
        
        let mut file = std::fs::File::create(&temp_file)?;
        file.write_all(cert_json.as_bytes())?;
        file.flush()?;
        
        Ok(temp_file)
    }

    fn create_python_pdf_script(&self, script_path: &Path) -> Result<()> {
        let script_content = include_str!("python_pdf_generator.py");
        std::fs::write(script_path, script_content)?;
        info!("Created Python PDF generator script at: {}", script_path.display());
        Ok(())
    }
}

fn find_project_root(start_dir: &Path) -> Option<PathBuf> {
    let mut current = start_dir;
    
    loop {
        // Look for indicators of project root
        if current.join("tests").exists() && 
           current.join("core").exists() && 
           current.join("certs").exists() {
            return Some(current.to_path_buf());
        }
        
        match current.parent() {
            Some(parent) => current = parent,
            None => return None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_python_pdf_generator_creation() {
        let generator = PythonPdfGenerator::new();
        assert!(generator.is_ok());
    }

    #[test]
    fn test_find_project_root() {
        // This test might fail in some environments, but it's useful for development
        let current_dir = std::env::current_dir().unwrap();
        let root = find_project_root(&current_dir);
        
        // In the project directory, this should find the root
        if let Some(root_path) = root {
            assert!(root_path.join("tests").exists());
            assert!(root_path.join("core").exists());
        }
    }

    #[test]
    fn test_temp_cert_file_creation() {
        let generator = PythonPdfGenerator::new().unwrap();
        let test_json = r#"{"cert_type": "backup", "cert_id": "test"}"#;
        
        let temp_file = generator.create_temp_cert_file(test_json).unwrap();
        assert!(temp_file.exists());
        
        let content = std::fs::read_to_string(&temp_file).unwrap();
        assert_eq!(content, test_json);
        
        // Clean up
        std::fs::remove_file(temp_file).unwrap();
    }
}
