"""
SecureWipe Verification Portal - FastAPI Application

Validates JSON certificates (backup or wipe) against schemas and provides
verification results with schema validation.

The routes live here. Trust material loads in app.keys, signatures are checked
in app.signature, and app.verification combines the checks.
"""

import json

from fastapi import FastAPI, HTTPException, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import HTMLResponse

from app import keys
from app.pages import HOME_PAGE
from app.signature import canonicalize_json, verify_ed25519_signature
from app.verification import VerificationResult, validate_certificate

__all__ = ["app", "canonicalize_json", "validate_certificate", "verify_ed25519_signature"]

# Initialize FastAPI app
app = FastAPI(
    title="SecureWipe Verification Portal",
    description="Validate JSON certificates (backup or wipe) with schema validation",
    version="1.0.0",
)

# Add CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # Configure appropriately for production
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


@app.get("/", response_class=HTMLResponse)
async def get_home():
    """Provide minimal HTML page showing how to POST"""
    return HTMLResponse(content=HOME_PAGE)


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
            request_data = json.loads(body.decode("utf-8"))
        except json.JSONDecodeError as e:
            raise HTTPException(status_code=400, detail=f"Invalid JSON: {str(e)}") from e

        if not isinstance(request_data, dict):
            raise HTTPException(status_code=400, detail="Certificate must be a JSON object")

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
        raise HTTPException(status_code=500, detail=f"Internal server error: {str(e)}") from e


@app.get("/verify/{cert_id}")
async def get_certificate_info(cert_id: str):
    """
    Get certificate information by ID (placeholder for future implementation)

    In the MVP, this is a placeholder. In the future, this could lookup
    stored certificate metadata from a database.
    """
    return {
        "message": "Certificate lookup by ID not implemented in MVP. Use POST /verify to validate certificates.",
        "cert_id": cert_id,
        "hint": "To verify a certificate, POST the JSON data to /verify endpoint",
    }


@app.get("/health")
async def health_check():
    """Health check endpoint"""
    return {
        "status": "healthy",
        "schemas_loaded": keys.BACKUP_SCHEMA is not None and keys.WIPE_SCHEMA is not None,
        "public_key_loaded": keys.PUBLIC_KEY_BYTES is not None,
        "version": "1.0.0",
    }


if __name__ == "__main__":
    import uvicorn

    uvicorn.run(app, host="0.0.0.0", port=8000)
