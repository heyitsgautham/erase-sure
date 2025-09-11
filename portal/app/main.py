"""
SecureWipe Verification Portal - FastAPI Application

Validates JSON certificates (backup or wipe) against schemas and provides
verification results with schema validation.
"""

import json
import os
import hashlib
import base64
from pathlib import Path
from typing import Dict, Any, List, Optional, Union

from fastapi import FastAPI, HTTPException, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import HTMLResponse
from pydantic import BaseModel, ValidationError
import jsonschema
from jsonschema import validate, ValidationError as JsonSchemaValidationError
from nacl.signing import VerifyKey
from nacl.exceptions import BadSignatureError
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey


# Initialize FastAPI app
app = FastAPI(
    title="SecureWipe Verification Portal",
    description="Validate JSON certificates (backup or wipe) with schema validation",
    version="1.0.0"
)

# Add CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # Configure appropriately for production
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Global variables for schemas and configuration
BACKUP_SCHEMA = None
WIPE_SCHEMA = None
PUBLIC_KEY_BYTES = None


def load_public_key():
    """Load Ed25519 public key from PEM file"""
    global PUBLIC_KEY_BYTES
    
    # Get public key path from environment or use default
    pubkey_path = os.environ.get("SECUREWIPE_PUBKEY_PATH", "keys/dev_public.pem")
    
    # Try keys/ folder as fallback
    if not os.path.exists(pubkey_path):
        current_dir = Path(__file__).parent.parent.parent
        fallback_path = current_dir / "keys" / "dev_public.pem"
        if fallback_path.exists():
            pubkey_path = str(fallback_path)
    
    try:
        with open(pubkey_path, 'r') as f:
            pem_content = f.read()
        
        # Parse PEM public key
        public_key = serialization.load_pem_public_key(pem_content.encode())
        
        if not isinstance(public_key, Ed25519PublicKey):
            raise RuntimeError(f"Public key is not Ed25519: {type(public_key)}")
        
        # Extract raw 32-byte key
        PUBLIC_KEY_BYTES = public_key.public_bytes(
            encoding=serialization.Encoding.Raw,
            format=serialization.PublicFormat.Raw
        )
        
    except FileNotFoundError:
        raise RuntimeError(f"Public key file not found: {pubkey_path}")
    except Exception as e:
        raise RuntimeError(f"Failed to load public key: {e}")


def load_schemas():
    """Load JSON schemas from files"""
    global BACKUP_SCHEMA, WIPE_SCHEMA
    
    # Get the project root directory
    current_dir = Path(__file__).parent.parent.parent
    backup_schema_path = current_dir / "certs" / "schemas" / "backup_schema.json"
    wipe_schema_path = current_dir / "certs" / "schemas" / "wipe_schema.json"
    
    try:
        with open(backup_schema_path, 'r') as f:
            BACKUP_SCHEMA = json.load(f)
        
        with open(wipe_schema_path, 'r') as f:
            WIPE_SCHEMA = json.load(f)
            
    except FileNotFoundError as e:
        raise RuntimeError(f"Schema file not found: {e}")
    except json.JSONDecodeError as e:
        raise RuntimeError(f"Invalid JSON in schema file: {e}")


def canonicalize_json(value: Dict[str, Any]) -> bytes:
    """
    Canonicalize JSON according to RFC 8785 JSON Canonicalization Scheme (JCS)
    
    This mirrors the implementation in core/src/signer.rs:
    - UTF-8 encoding
    - Sorted object keys
    - No insignificant whitespace
    - Consistent number formatting
    """
    def canonicalize_value(val):
        if isinstance(val, dict):
            # Sort keys and canonicalize all values
            return {k: canonicalize_value(v) for k, v in sorted(val.items())}
        elif isinstance(val, list):
            # Canonicalize array elements
            return [canonicalize_value(item) for item in val]
        else:
            # Primitive values are already canonical
            return val
    
    canonical = canonicalize_value(value)
    
    # Serialize without whitespace for true RFC 8785 compliance
    canonical_json = json.dumps(canonical, separators=(',', ':'), ensure_ascii=False)
    
    return canonical_json.encode('utf-8')


# Load schemas and public key on startup
load_schemas()
load_public_key()


class VerificationResult(BaseModel):
    """Response model for certificate verification"""
    schema_valid: bool
    signature_valid: Optional[bool] = None
    hash_valid: Optional[bool] = None
    chain_valid: Optional[bool] = None
    cert_summary: Dict[str, Any]
    computed: Dict[str, Optional[str]]
    errors: List[str]


class CertificateSummary(BaseModel):
    """Summary of key certificate fields"""
    cert_id: str
    cert_type: str
    device_model: str
    policy_method: Optional[str] = None  # For wipe certificates
    destination: Optional[str] = None    # For backup certificates


def extract_cert_summary(cert_data: Dict[str, Any]) -> Dict[str, Any]:
    """Extract key fields for certificate summary"""
    summary = {
        "cert_id": cert_data.get("cert_id", "unknown"),
        "cert_type": cert_data.get("cert_type", "unknown"),
        "device_model": cert_data.get("device", {}).get("model", "unknown"),
    }
    
    # Add type-specific fields
    if cert_data.get("cert_type") == "wipe":
        policy = cert_data.get("policy", {})
        summary["policy_method_or_destination"] = f"{policy.get('nist_level', 'unknown')} - {policy.get('method', 'unknown')}"
    elif cert_data.get("cert_type") == "backup":
        destination = cert_data.get("destination", {})
        if destination:
            summary["policy_method_or_destination"] = f"{destination.get('type', 'unknown')} ({destination.get('label', 'unlabeled')})"
        else:
            summary["policy_method_or_destination"] = "backup"
    
    return summary


def verify_ed25519_signature(cert_data: Dict[str, Any]) -> Optional[bool]:
    """
    Verify Ed25519 signature on certificate
    
    Returns:
        True if signature is valid
        False if signature is invalid
        None if no signature is present
    """
    signature_obj = cert_data.get("signature")
    if not signature_obj:
        return None
    
    # Check required signature fields
    if (signature_obj.get("alg") != "Ed25519" or 
        signature_obj.get("pubkey_id") != "sih_root_v1"):
        return False
    
    try:
        # Get signature bytes
        sig_b64 = signature_obj.get("sig")
        if not sig_b64:
            return False
        
        signature_bytes = base64.b64decode(sig_b64)
        
        # Create unsigned certificate for canonicalization
        unsigned_cert = cert_data.copy()
        if "signature" in unsigned_cert:
            del unsigned_cert["signature"]
        
        # Canonicalize the unsigned certificate
        canonical_bytes = canonicalize_json(unsigned_cert)
        
        # Verify signature using PyNaCl
        verify_key = VerifyKey(PUBLIC_KEY_BYTES)
        verify_key.verify(canonical_bytes, signature_bytes)
        return True
        
    except (BadSignatureError, ValueError, Exception):
        return False


def compute_certificate_hash(cert_json_bytes: bytes) -> str:
    """Compute SHA256 hash of certificate JSON bytes"""
    return hashlib.sha256(cert_json_bytes).hexdigest()


def validate_chain_linkage(wipe_cert: Dict[str, Any], linked_backup_cert: Optional[Dict[str, Any]]) -> Optional[bool]:
    """
    Validate chain linkage between wipe and backup certificates
    
    Returns:
        True if linkage is valid
        False if linkage is invalid
        None if no linkage information provided
    """
    if not linked_backup_cert:
        return None
    
    linkage = wipe_cert.get("linkage", {})
    backup_cert_id = linkage.get("backup_cert_id")
    
    if not backup_cert_id:
        return None
    
    # Check if backup cert ID matches
    linked_cert_id = linked_backup_cert.get("cert_id")
    return backup_cert_id == linked_cert_id


def validate_certificate(cert_data: Dict[str, Any], cert_json_bytes: bytes, linked_backup_cert: Optional[Dict[str, Any]] = None) -> VerificationResult:
    """
    Validate a certificate against the appropriate schema and verify signature
    
    Args:
        cert_data: The certificate JSON data
        cert_json_bytes: Raw JSON bytes as received
        linked_backup_cert: Optional linked backup certificate for chain validation
        
    Returns:
        VerificationResult with validation status and summary
    """
    errors = []
    schema_valid = False
    signature_valid = None
    hash_valid = None
    chain_valid = None
    cert_summary = {}
    computed = {
        "certificate_json_sha256": compute_certificate_hash(cert_json_bytes),
        "linked_backup_sha256": None
    }
    
    try:
        # Detect certificate type
        cert_type = cert_data.get("cert_type")
        if not cert_type:
            errors.append("Missing 'cert_type' field")
            return VerificationResult(
                schema_valid=False,
                signature_valid=None,
                hash_valid=None,
                chain_valid=None,
                errors=errors,
                cert_summary=cert_summary,
                computed=computed
            )
        
        # Select appropriate schema
        if cert_type == "backup":
            schema = BACKUP_SCHEMA
        elif cert_type == "wipe":
            schema = WIPE_SCHEMA
        else:
            errors.append(f"Unknown certificate type: {cert_type}")
            return VerificationResult(
                schema_valid=False,
                signature_valid=None,
                hash_valid=None,
                chain_valid=None,
                errors=errors,
                cert_summary=cert_summary,
                computed=computed
            )
        
        # Validate against schema
        try:
            validate(instance=cert_data, schema=schema)
            schema_valid = True
        except JsonSchemaValidationError as e:
            errors.append(f"Schema validation error: {e.message}")
            if hasattr(e, 'path') and e.path:
                errors.append(f"At path: {' -> '.join(str(p) for p in e.path)}")
        
        # Verify Ed25519 signature
        try:
            signature_valid = verify_ed25519_signature(cert_data)
            if signature_valid is False:
                errors.append("Invalid Ed25519 signature")
        except Exception as e:
            signature_valid = False
            errors.append(f"Signature verification error: {str(e)}")
        
        # Hash verification for backup certificates
        if cert_type == "backup":
            metadata = cert_data.get("metadata", {})
            expected_hash = metadata.get("certificate_json_sha256")
            if expected_hash:
                computed_hash = computed["certificate_json_sha256"]
                hash_valid = expected_hash.lower() == computed_hash.lower()
                if not hash_valid:
                    errors.append(f"Certificate hash mismatch: expected {expected_hash}, got {computed_hash}")
        
        # Chain linkage validation for wipe certificates
        if cert_type == "wipe" and linked_backup_cert:
            try:
                # Validate linked backup cert against backup schema
                validate(instance=linked_backup_cert, schema=BACKUP_SCHEMA)
                
                # Compute linked backup hash
                linked_backup_bytes = json.dumps(linked_backup_cert, separators=(',', ':')).encode('utf-8')
                computed["linked_backup_sha256"] = compute_certificate_hash(linked_backup_bytes)
                
                # Validate linkage
                chain_valid = validate_chain_linkage(cert_data, linked_backup_cert)
                if chain_valid is False:
                    errors.append("Chain linkage validation failed: backup_cert_id mismatch")
                
            except JsonSchemaValidationError as e:
                chain_valid = False
                errors.append(f"Linked backup certificate schema error: {e.message}")
            except Exception as e:
                chain_valid = False
                errors.append(f"Chain validation error: {str(e)}")
        
        # Extract summary (even if validation failed, try to get what we can)
        cert_summary = extract_cert_summary(cert_data)
        
    except Exception as e:
        errors.append(f"Unexpected error during validation: {str(e)}")
    
    return VerificationResult(
        schema_valid=schema_valid,
        signature_valid=signature_valid,
        hash_valid=hash_valid,
        chain_valid=chain_valid,
        errors=errors,
        cert_summary=cert_summary,
        computed=computed
    )


@app.get("/", response_class=HTMLResponse)
async def get_home():
    """Provide minimal HTML page showing how to POST"""
    html_content = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>SecureWipe Verification Portal</title>
        <style>
            body { font-family: Arial, sans-serif; margin: 40px; background-color: #f5f5f5; }
            .container { max-width: 800px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
            .header { text-align: center; color: #2c3e50; margin-bottom: 30px; }
            .endpoint { background: #ecf0f1; padding: 20px; border-radius: 5px; margin: 20px 0; }
            .method { background: #27ae60; color: white; padding: 5px 10px; border-radius: 3px; font-weight: bold; }
            code { background: #f8f9fa; padding: 2px 6px; border-radius: 3px; font-family: monospace; }
            .example { background: #f8f9fa; padding: 15px; border-radius: 5px; overflow-x: auto; }
            pre { margin: 0; }
            ul { padding-left: 20px; }
        </style>
    </head>
    <body>
        <div class="container">
            <div class="header">
                <h1>🔒 SecureWipe Verification Portal</h1>
                <p>Validate JSON certificates with schema validation</p>
            </div>
            
            <div class="endpoint">
                <h2><span class="method">POST</span> /verify</h2>
                <p>Validate a JSON certificate (backup or wipe) with schema validation, Ed25519 signature verification, hash recomputation, and optional chain linkage checks.</p>
                
                <h3>Request</h3>
                <ul>
                    <li><strong>Content-Type:</strong> <code>application/json</code></li>
                    <li><strong>Body:</strong> Raw JSON certificate data</li>
                    <li><strong>Optional:</strong> For wipe certificates, include <code>linked_backup_cert</code> field with the referenced backup certificate for chain validation</li>
                </ul>
                
                <h3>Response</h3>
                <div class="example">
                    <pre>{
  "schema_valid": true,
  "signature_valid": true,
  "hash_valid": true,
  "chain_valid": null,
  "cert_summary": {
    "cert_id": "backup_20231205_143022_f4a2b8c1",
    "cert_type": "backup",
    "device_model": "Samsung SSD 980 PRO 1TB",
    "policy_method_or_destination": "usb (External Drive)"
  },
  "computed": {
    "certificate_json_sha256": "a1b2c3...",
    "linked_backup_sha256": null
  },
  "errors": []
}</pre>
                </div>
                
                <h3>Verification Features</h3>
                <ul>
                    <li><strong>Schema Validation:</strong> JSON structure validation against backup/wipe schemas</li>
                    <li><strong>Ed25519 Signature:</strong> Cryptographic signature verification using RFC 8785 JSON canonicalization</li>
                    <li><strong>Hash Verification:</strong> SHA256 hash recomputation and comparison</li>
                    <li><strong>Chain Linkage:</strong> Validate backup_cert_id linkage between wipe and backup certificates</li>
                </ul>
            </div>
            
            <div class="endpoint">
                <h2><span class="method">GET</span> /verify/{cert_id}</h2>
                <p>Get information about certificate verification by ID (placeholder for future implementation).</p>
            </div>
            
            <h3>Example Usage with curl</h3>
            <div class="example">
                <pre># Verify a single certificate
curl -X POST "http://localhost:8000/verify" \\
     -H "Content-Type: application/json" \\
     -d @certificate.json

# Verify wipe certificate with linked backup
curl -X POST "http://localhost:8000/verify" \\
     -H "Content-Type: application/json" \\
     -d '{
       "cert_type": "wipe",
       "cert_id": "wipe_001",
       "linkage": {"backup_cert_id": "backup_001"},
       "signature": {...},
       "linked_backup_cert": {
         "cert_type": "backup",
         "cert_id": "backup_001",
         ...
       }
     }'</pre>
            </div>
            
            <h3>Supported Certificate Types</h3>
            <ul>
                <li><strong>backup:</strong> Encrypted backup certificates with AES-256-CTR</li>
                <li><strong>wipe:</strong> NIST-aligned wipe certificates with CLEAR/PURGE/DESTROY policies</li>
            </ul>
        </div>
    </body>
    </html>
    """
    return HTMLResponse(content=html_content)


@app.post("/verify", response_model=VerificationResult)
async def verify_certificate(request: Request):
    """
    Verify a JSON certificate against the appropriate schema with Ed25519 signature verification
    
    Accepts raw JSON certificate data and validates it against either backup_schema.json 
    or wipe_schema.json based on cert_type. Performs Ed25519 signature verification using 
    RFC 8785 JSON canonicalization, hash recomputation, and optional chain linkage validation.
    
    For wipe certificates, include 'linked_backup_cert' in the JSON body to enable
    chain linkage validation.
    """
    try:
        # Get raw JSON body
        body = await request.body()
        if not body:
            raise HTTPException(status_code=400, detail="Empty request body")
        
        # Parse JSON
        try:
            request_data = json.loads(body.decode('utf-8'))
        except json.JSONDecodeError as e:
            raise HTTPException(
                status_code=400, 
                detail=f"Invalid JSON: {str(e)}"
            )
        
        # Extract main certificate and optional linked backup
        cert_data = request_data.copy()
        linked_backup_cert = None
        
        # Handle linked backup certificate for wipe certs
        if "linked_backup_cert" in cert_data:
            linked_backup_cert = cert_data.pop("linked_backup_cert")
        
        # Validate certificate (pass original body bytes for hash computation)
        result = validate_certificate(cert_data, body, linked_backup_cert)
        return result
        
    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(
            status_code=500,
            detail=f"Internal server error: {str(e)}"
        )


@app.get("/verify/{cert_id}")
async def get_certificate_info(cert_id: str):
    """
    Get certificate information by ID (placeholder for future implementation)
    
    In the MVP, this is a placeholder. In the future, this could lookup
    stored certificate metadata from a database.
    """
    return {
        "message": f"Certificate lookup by ID not implemented in MVP. Use POST /verify to validate certificates.",
        "cert_id": cert_id,
        "hint": "To verify a certificate, POST the JSON data to /verify endpoint"
    }


@app.get("/health")
async def health_check():
    """Health check endpoint"""
    return {
        "status": "healthy",
        "schemas_loaded": BACKUP_SCHEMA is not None and WIPE_SCHEMA is not None,
        "public_key_loaded": PUBLIC_KEY_BYTES is not None,
        "version": "1.0.0"
    }


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)