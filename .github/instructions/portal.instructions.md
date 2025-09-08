---
applyTo:
  - "portal/**"
---

# Copilot Instructions for Portal (FastAPI)

- Use **Python with FastAPI** for all code under /portal.
- Required endpoints:
  - `POST /verify`: validate JSON certificate, check Ed25519 signature, recompute hashes.
  - `GET /verify/{cert_id}`: optional stub for MVP.
- Input must validate against `/certs/schemas/*.json`.
- Output must include:
  - `signature_valid`
  - `hash_valid`
  - `chain_valid`
  - `cert_summary`
- Web UI must:
  - Render green ticks for verification results.
  - Display key cert fields in a table.
- Must remain stateless (no DB) for MVP.
- Out of scope:
  - Authentication
  - Blockchain anchoring
  - Enterprise fleet features
