# SecureWipe — GitHub Copilot Repository Instructions (Single Source of Truth)

Purpose

- These are the only instructions Copilot will automatically read across the repository.
- They apply to inline suggestions, Copilot Chat, and (where available) PR review.
- Keep this file authoritative and up to date with PRD and schemas.

Project Overview

- SIH PS 25070 — Secure, NIST-aligned data wiping with tamper-proof certificates.
- MVP Platforms: Linux (full), Android technician path, Windows simulated.
- Modules: /core (Rust), /ui (Tauri + React), /portal (FastAPI).

Authoritative References (open these files when prompting)

- /docs/PRD.md (product & flows)
- /docs/schemas.md (reference)
- /certs/schemas/backup_schema.json (machine schema)
- /certs/schemas/wipe_schema.json (machine schema)

Global Security & Compliance Rules (must follow everywhere)

- Certificates: Must conform exactly to JSON Schemas in /certs/schemas/\*.json.
- Signing: Ed25519 only. pubkey_id must be "sih_root_v1". Never embed or generate private keys in code. Load keys at runtime from a secure path; only bundle public key for verification when needed.
- Backup: Always AES-256-CTR encryption. Always produce SHA-256 manifest. Verify at least N=5 random files post-copy.
- Wipe: Default policy PURGE (controller sanitize preferred). If sanitize unsupported, fallback to CLEAR (single overwrite + sampled verification) and record the fallback rationale in the certificate.
- Guard rails: Never wipe CRITICAL/system disks unless running from bootable ISO mode with explicit double confirmation (the UI should not expose this in MVP).
- Destructive actions: Always require a two-step confirmation in UI and clear warnings in CLI/UX.
- Logging: Emit structured JSON logs with timestamps, step identifiers, and actionable messages (no silent failures).
- Config: Avoid hardcoded paths; prefer config/env vars with safe defaults (e.g., ~/SecureWipe/certificates/ for artifacts).
- PDFs: Must be nicely styled, include logo/title, certificate ID, tables, and a QR code linking to verification (URL configurable) or encoding the certificate ID.

Testing & Validation (global expectations)

- Unit tests: hashing utilities, manifest generation, schema validation, Ed25519 verify.
- Integration tests: backup → wipe → certificate → verify (CLI + portal).
- Schema validation: Sample cert JSON must validate against /certs/schemas/\*.json in tests.
- Error handling: Nonzero exit codes on failure, with clear user-facing messages.

Out of Scope for MVP (do not generate unless explicitly requested)

- Cloud backup (Drive/Dropbox/OneDrive).
- Blockchain anchoring.
- Enterprise fleet orchestration and role-based portal.
- Multi-language/i18n.
- Docker and automated ISO build pipelines (document in roadmap only).

---

## Module Section — Core (/core, Rust)

Purpose

- Secure, NIST-aligned backup and wiping operations on Linux.
- Generate signed JSON and nicely styled PDF certificates.
- Provide a CLI used by the Tauri UI.

Language & Tools

- Rust only for /core.
- Use system tools read-only where applicable: lsblk (JSON), smartctl -i, hdparm -I, nvme id-ctrl.
- Use hdparm and nvme-cli for controller actions only in destructive flows (not in discovery/planning).

Core Requirements

1. Device Discovery (read-only)

- Parse lsblk -J output to enumerate block devices.
- Enrich with optional smartctl/hdparm/nvme id-ctrl (non-fatal if missing).
- Determine bus (SATA/NVMe/USB), capacity_bytes, serial, mountpoints.
- Risk levels:
  - CRITICAL: contains root "/" (system disk).
  - HIGH: any mounted writable volume.
  - SAFE: raw/unmounted.
- Output device list as JSON for the UI.

2. Backup (encrypted)

- Inputs: --device <dev> --dest <path> [--paths <p1> <p2> ...] defaulting to $HOME/Documents,Desktop,Pictures.
- AES-256-CTR encryption (session key generated at runtime). No cloud.
- Create manifest.json mapping relative_path -> sha256 of original file content.
- Copy encrypted data to destination; post-copy verify at least 5 random files against manifest hashes.
- Emit Backup Certificate JSON conforming to /certs/schemas/backup_schema.json.
- Initially create unsigned JSON; signing can be a distinct step; save to ~/SecureWipe/certificates/.

3. Wipe (NIST aligned)

- Default policy PURGE. Prefer controller sanitize (nvme sanitize / format; hdparm secure-erase / sanitize-crypto-erase).
- CLEAR fallback for HDD or unsupported controllers: single overwrite + random-sector verification sampling.
- Clear HPA/DCO if present (record commands used).
- Log exact commands, exit codes, elapsed milliseconds, verification sampling and results.
- Emit Wipe Certificate JSON conforming to /certs/schemas/wipe_schema.json; include linkage.backup_cert_id when available.

4. Certificates (JSON + PDF)

- JSON fields must match schemas exactly. Include device details, method, logs, verify results, linkage, created_at.
- Sign JSON with Ed25519 (signature.alg="Ed25519"; signature.pubkey_id="sih_root_v1").
- PDF must mirror JSON summary, styled with header logo/title, tables, and QR code to verification portal or containing cert ID.
- Save artifacts to ~/SecureWipe/certificates/.

5. CLI

- Subcommands: discover, backup, wipe, cert.
- discover: print device list + risk JSON.
- backup: run encrypted backup, create manifest, verify N samples, emit Backup Certificate JSON (and later PDF/sign).
- wipe: enforce guard rails, plan + execute steps, emit Wipe Certificate JSON (and later PDF/sign).
- cert: show or export stored certificate JSON/PDF by ID.
- All commands: structured JSON logging, clear error codes/messages.

Constraints

- Never wipe CRITICAL disks unless ISO mode is detected and user provides explicit override; default build should not expose this path.
- No cloud or network backup in MVP.
- Private keys must never be committed or embedded; signer loads externally at runtime.

---

## Module Section — UI (/ui, Tauri + React)

Purpose

- Provide a simple, safe, one-click experience to perform Backup → Wipe → Certificates.

Language & Framework

- TypeScript + React with Tauri.

Views & Interactions

1. Welcome

- SIH banner and mission statement.
- Primary CTA: “Backup & Wipe”; Secondary: “Wipe Only”.

2. Device Discovery

- Render device cards: model, capacity, bus, masked serial, Risk Badge (CRITICAL/HIGH/SAFE).
- If CRITICAL, show a blocked state with guidance to use the bootable ISO for system disks.

3. Backup

- Destination selector (USB/NAS path).
- Display “AES-256 encryption ON (mandatory)”.
- Progress: bytes copied, ETA; post-run integrity sampling results; retry on failure or allow “Skip and Wipe” with warning.

4. Wipe

- Policy chooser: CLEAR, PURGE (default), DESTROY (info only).
- Two-step confirmation modal before starting destructive actions.
- Stream and display steps: HPA/DCO clear, sanitize/overwrite, verification sampling.
- Clear failure messages with recovery suggestions.

5. Certificates

- Show four buttons: Backup JSON, Backup PDF, Wipe JSON, Wipe PDF.
- Display QR preview and instructions to verify (CLI and portal URL).
- “Open Folder” to ~/SecureWipe/certificates/.

IPC & State

- Invoke /core CLI subcommands; stream JSON progress into state.
- Support cancellation with confirmation where destructive steps are in progress.
- Persist last-used destination and simple UI preferences locally.

Constraints

- No cloud backup UI in MVP.
- Must refuse CRITICAL disk wipes unless ISO mode flag is set in config/env.

---

## Module Section — Portal (/portal, FastAPI)

Purpose

- Verify certificates via public key: validate schema, signature, hashes, and linkage.

Language & Framework

- Python + FastAPI.

Endpoints

- POST /verify:

  - Accept JSON certificate (backup or wipe).
  - Detect type via cert_type and validate against /certs/schemas/\*.json.
  - Verify Ed25519 signature using pubkey.pem from configurable path.
  - Recompute hashes (e.g., manifest_sha256) and compare.
  - If wipe certificate includes linkage.backup_cert_id and a corresponding backup cert is provided, validate the linkage consistency.
  - Return JSON: signature_valid, hash_valid, chain_valid, cert_summary (key fields).

- GET / (optional)

  - Minimal HTML page documenting how to POST for verification.

- GET /verify/{cert_id} (optional MVP)
  - If a store is later added, return stored metadata; otherwise return an instructional stub.

UI Output (optional HTML)

- Show green ticks for Signature, Hash, Chain Link.
- Display key certificate fields in a simple table.
- Provide download links if certificate files are provided.

Constraints

- Stateless MVP: no DB, no authentication.
- Never handle private keys; verification uses only public key.

---

## Prompts & How To Use Copilot

Reusable Chat Prompts (already provided as .github/prompts/\*.prompt.md)

- /prompt core: Use when working in /core to scaffold CLI, discovery, backup, wipe, cert, and tests.
- /prompt ui: Use when working in /ui to scaffold screens, device cards, progress views, and IPC wiring.
- /prompt portal: Use when working in /portal to scaffold FastAPI endpoints and verification logic.

Good Prompting Habits

- Keep /docs/PRD.md and /certs/schemas/\*.json open in editor tabs while generating related code.
- Ask for small, testable increments (single function/file/endpoint at a time).
- Restate critical constraints in your request (PURGE default, AES-256-CTR, Ed25519, no CRITICAL disk wipe).

Verification That Copilot Sees These Instructions

- In Copilot Chat, with a file open in /core (or /ui, /portal), ask:
  “Which repository instructions are active for this file?”
- You should see this file (.github/copilot-instructions.md) listed.
- Path-specific instruction files are NOT supported natively; keep everything consolidated here.

Maintenance

- Update this file when PRD or schemas change.
- Keep scope sections current (what’s in/out for MVP).
- Reject Copilot suggestions that violate constraints and explain why; Copilot will adapt within the session.
