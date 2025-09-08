# Copilot Instructions for SecureWipe

## Purpose

These are **repository-wide Copilot guidelines** for the SecureWipe project (SIH PS 25070).  
They ensure all Copilot-generated code is **consistent, secure, and aligned** with project goals.

---

## General Rules

- **Languages**:
  - `/core`: Rust (for erasure & backup engine).
  - `/ui`: TypeScript + React with Tauri (for desktop UI).
  - `/portal`: Python + FastAPI (for verification service).
- **Certificates**: Must strictly follow JSON Schemas in `/certs/schemas/*.json`.
- **Documentation**: Reference `/docs/PRD.md` and `/docs/schemas.md` for requirements and data models.

---

## Security Rules

- **Signing**: Use **Ed25519** for certificate signatures.
  - `pubkey_id` must be `"sih_root_v1"`.
  - Do **not** generate or embed private keys in the code.
  - Expect `pubkey.pem` at runtime for verification.
- **Backup**:
  - Always encrypt with **AES-256-CTR**.
  - Always generate a SHA-256 manifest of files.
  - Post-backup verification must sample at least N=5 random files.
- **Wipe**:
  - Default wipe policy is **PURGE** (use controller sanitize commands where available).
  - If sanitize is unsupported, fallback to CLEAR (overwrite) and record rationale in the certificate.
- **Guard Rails**:
  - Never allow wiping CRITICAL/system disks unless in ISO mode with explicit confirmation.
  - Always show user confirmation dialogs for destructive actions.

---

## Logging & Testing

- All code must emit **structured JSON logs** with timestamps and step identifiers.
- Errors should always include **actionable messages** (not just stack traces).
- Unit tests required for:
  - Hashing utilities
  - Ed25519 signing/verification
  - Schema validation
- Integration tests required for:
  - Backup → Wipe → Certificate → Verify flow

---

## Out of Scope for MVP

- Cloud backup integrations (Google Drive, Dropbox, OneDrive).
- Blockchain anchoring of certificates.
- Enterprise fleet orchestration.
- Multi-language/localization.
- Dockerized builds (add to roadmap, not MVP).

---

## Style Rules

- Prefer clean, modular code with descriptive names.
- Avoid hardcoding paths; use config/env variables where possible.
- Certificates (JSON and PDF) must include:
  - Device details
  - Policy and method
  - Verification results
  - Digital signature
  - QR code for verification

---

## Deliverables

- **Rust crate** in `/core` exposing CLI: `discover`, `backup`, `wipe`, `cert`.
- **React/Tauri UI** in `/ui` with one-click flows and risk badges.
- **FastAPI app** in `/portal` with `/verify` endpoint and simple green-tick UI.
- **Schemas** validated against `/certs/schemas/*.json`.
- **Docs** updated in `/docs/`.

---

### Reminder

Copilot should **always follow this file** as the single source of truth.  
When in doubt: prefer **security, simplicity, and NIST SP 800-88 alignment**.
