"""
SecureWipe Verification Portal - FastAPI Application

Validates JSON certificates (backup or wipe) against schemas and provides
verification results with schema validation.
"""

import json
import os
from pathlib import Path
from typing import Dict, Any, List, Optional

from fastapi import FastAPI, HTTPException, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import HTMLResponse
from pydantic import BaseModel, ValidationError
import jsonschema
from jsonschema import validate, ValidationError as JsonSchemaValidationError


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

# Global variables for schemas
BACKUP_SCHEMA = None
WIPE_SCHEMA = None


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


# Load schemas on startup
load_schemas()


class VerificationResult(BaseModel):
    """Response model for certificate verification"""
    schema_valid: bool
    errors: List[str]
    cert_summary: Dict[str, Any]


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
        summary["policy_method"] = f"{policy.get('nist_level', 'unknown')} - {policy.get('method', 'unknown')}"
    elif cert_data.get("cert_type") == "backup":
        destination = cert_data.get("destination", {})
        summary["destination"] = f"{destination.get('type', 'unknown')} ({destination.get('label', 'unlabeled')})"
    
    return summary


def validate_certificate(cert_data: Dict[str, Any]) -> VerificationResult:
    """
    Validate a certificate against the appropriate schema
    
    Args:
        cert_data: The certificate JSON data
        
    Returns:
        VerificationResult with validation status and summary
    """
    errors = []
    schema_valid = False
    cert_summary = {}
    
    try:
        # Detect certificate type
        cert_type = cert_data.get("cert_type")
        if not cert_type:
            errors.append("Missing 'cert_type' field")
            return VerificationResult(
                schema_valid=False,
                errors=errors,
                cert_summary=cert_summary
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
                errors=errors,
                cert_summary=cert_summary
            )
        
        # Validate against schema
        try:
            validate(instance=cert_data, schema=schema)
            schema_valid = True
        except JsonSchemaValidationError as e:
            errors.append(f"Schema validation error: {e.message}")
            if hasattr(e, 'path') and e.path:
                errors.append(f"At path: {' -> '.join(str(p) for p in e.path)}")
        
        # Extract summary (even if validation failed, try to get what we can)
        cert_summary = extract_cert_summary(cert_data)
        
    except Exception as e:
        errors.append(f"Unexpected error during validation: {str(e)}")
    
    return VerificationResult(
        schema_valid=schema_valid,
        errors=errors,
        cert_summary=cert_summary
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
                <p>Validate a JSON certificate (backup or wipe) against the appropriate schema.</p>
                
                <h3>Request</h3>
                <ul>
                    <li><strong>Content-Type:</strong> <code>application/json</code></li>
                    <li><strong>Body:</strong> Raw JSON certificate data</li>
                </ul>
                
                <h3>Response</h3>
                <div class="example">
                    <pre>{
  "schema_valid": true,
  "errors": [],
  "cert_summary": {
    "cert_id": "backup_20231205_143022_f4a2b8c1",
    "cert_type": "backup",
    "device_model": "Samsung SSD 980 PRO 1TB",
    "destination": "usb (External Drive)"
  }
}</pre>
                </div>
            </div>
            
            <div class="endpoint">
                <h2><span class="method">GET</span> /verify/{cert_id}</h2>
                <p>Get information about certificate verification by ID (placeholder for future implementation).</p>
            </div>
            
            <h3>Example Usage with curl</h3>
            <div class="example">
                <pre>curl -X POST "http://localhost:8000/verify" \\
     -H "Content-Type: application/json" \\
     -d @certificate.json</pre>
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
    Verify a JSON certificate against the appropriate schema
    
    Accepts raw JSON certificate data and validates it against
    either backup_schema.json or wipe_schema.json based on cert_type.
    """
    try:
        # Get raw JSON body
        body = await request.body()
        if not body:
            raise HTTPException(status_code=400, detail="Empty request body")
        
        # Parse JSON
        try:
            cert_data = json.loads(body.decode('utf-8'))
        except json.JSONDecodeError as e:
            raise HTTPException(
                status_code=400, 
                detail=f"Invalid JSON: {str(e)}"
            )
        
        # Validate certificate
        result = validate_certificate(cert_data)
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
        "version": "1.0.0"
    }


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)