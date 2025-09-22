use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json_content = fs::read_to_string("../../current_backup_cert.json")?;
    println!("JSON content length: {}", json_content.len());
    
    // Try to parse as generic JSON first
    let generic: serde_json::Value = serde_json::from_str(&json_content)?;
    println!("Signature field: {:?}", generic.get("signature"));
    
    // Try to parse with the actual BackupCertificate
    match securewipe::cert::BackupCertificate::try_from(&json_content) {
        Ok(cert) => println!("Parsed successfully! Signature: {:?}", cert.signature),
        Err(e) => println!("Parse error: {}", e),
    }
    
    Ok(())
}