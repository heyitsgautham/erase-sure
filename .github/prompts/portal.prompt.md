---
title: "SecureWipe Verification Portal — FastAPI"
description: "Validate JSON certificates (Ed25519) and display human-readable verification results with green ticks."
labels: ["python", "fastapi", "security", "verification", "schemas"]
audience: ["copilot", "developer"]
---

## Context

You are generating **Python (FastAPI)** code for the verification portal under `/portal`.  
Purpose: Accept **JSON certificates** (backup or wipe), verify **Ed25519 signatures**, recompute hashes, and display a simple results page.

**Authoritative specs to follow**:

- Schemas: `/certs/schemas/backup_schema.json`, `/certs/schemas/wipe_schema.json`
- Product: `/docs/PRD.md`, `/docs/schemas.md`

## Requirements

### Endpoints

1. `POST /verify`

   - Accepts a JSON certificate (raw body).
   - Determine cert type by `cert_type`.
   - Validate against the correct JSON Schema.
   - Verify **Ed25519** signature using `pubkey.pem` from a configurable path.
   - Recompute any embedded hashes (e.g., `manifest_sha256`) and compare.
   - If wipe cert includes `linkage.backup_cert_id`, allow optional inclusion of the referenced backup cert to validate linkage consistency.
   - Return a JSON summary:
     ```json
     {
       "signature_valid": true,
       "hash_valid": true,
       "chain_valid": true,
       "cert_summary": { ... key fields ... }
     }
     ```

2. `GET /verify/{cert_id}` (Optional MVP)
   - If a store is added, lookup by `cert_id`.
   - Otherwise, respond with a simple “provide JSON via POST /verify” message.

### UI

- Provide a minimal HTML page (Jinja2 or pure FastAPI HTMLResponse) that:
  - Shows **green ticks** for Signature, Hash, Chain Link.
  - Displays key certificate fields in a table.
  - Offers links or buttons to download the original JSON/PDF if provided.

### Implementation Notes

- Use a robust **Ed25519** library; never generate or store private keys here.
- Keep the service **stateless** for MVP (no DB required).
- Strictly validate requests; handle malformed JSON gracefully with helpful messages.
- Add simple logging (request ID, timing, result flags).

## Constraints

- **No authentication** in MVP.
- **No blockchain anchoring** in MVP.
- Public key path is configurable; private keys are never part of this service.

---

### Example Prompt to Start

Create a FastAPI app with `POST /verify` that:

- Detects backup vs wipe cert,
- Validates the JSON using the schema files,
- Loads `pubkey.pem`, verifies the Ed25519 signature,
- Returns a JSON object with `signature_valid`, `hash_valid`, `chain_valid`, and a `cert_summary`.
