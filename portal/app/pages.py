"""The portal's one HTML page: how to POST a certificate for verification."""

HOME_PAGE = """
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
                <li><strong>wipe:</strong> NIST-aligned wipe certificates with CLEAR/PURGE policies</li>
            </ul>
        </div>
    </body>
    </html>
    """
