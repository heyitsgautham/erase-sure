use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use base64::{engine::general_purpose::STANDARD, Engine};

fn main() {
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();
    
    // Print the key bytes in base64 for easy copying
    println!("Signing key (32 bytes): {}", STANDARD.encode(signing_key.to_bytes()));
    println!("Verifying key (32 bytes): {}", STANDARD.encode(verifying_key.to_bytes()));
    
    // Create a proper PEM format for the public key
    let pubkey_der = format!(
        "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----",
        STANDARD.encode(verifying_key.to_bytes())
    );
    
    println!("\nPublic key PEM format:");
    println!("{}", pubkey_der);
}
