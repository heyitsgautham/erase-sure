use std::path::Path;
use serde_json::Value;

fn main() {
    // Load schema
    let schema_path = Path::new("../certs/schemas/backup_schema.json");
    println!("Schema path exists: {}", schema_path.exists());
    
    if let Ok(schema_content) = std::fs::read_to_string(schema_path) {
        println!("Schema loaded, length: {}", schema_content.len());
        
        if let Ok(schema_json) = serde_json::from_str::<Value>(&schema_content) {
            println!("Schema parsed as JSON successfully");
            
            // Try to compile the schema
            match jsonschema::JSONSchema::compile(&schema_json) {
                Ok(_) => println!("Schema compiled successfully"),
                Err(e) => println!("Schema compilation failed: {}", e),
            }
        } else {
            println!("Failed to parse schema as JSON");
        }
    } else {
        println!("Failed to read schema file");
    }
    
    // Test a minimal certificate
    let test_cert = serde_json::json!({
        "cert_type": "backup",
        "cert_id": "test"
    });
    
    println!("Test certificate: {}", test_cert);
}
