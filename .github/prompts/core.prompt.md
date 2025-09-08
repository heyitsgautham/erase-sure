---
title: "SecureWipe Core — Rust Erasure + Backup"
description: "Implement secure backup and disk wiping on Linux, emit signed JSON/PDF certificates, expose a CLI for the Tauri UI."
labels: ["rust", "security", "cli", "storage", "nist-800-88", "linux"]
audience: ["copilot", "developer"]
---

## Context

You are generating **Rust** code for the SecureWipe project’s core module located in `/core`.  
This module must:

- Discover storage devices on **Linux**.
- Perform **encrypted backups** to external USB/NAS.
- Execute **NIST SP 800-88–aligned** disk wipe operations (CLEAR, PURGE; DESTROY documented only).
- Produce **tamper-proof certificates** (JSON + nicely styled PDF) signed with **Ed25519**.
- Expose a **CLI** used by the Tauri UI.

**Authoritative specs to follow** (read from repo):

- JSON Schemas: `/certs/schemas/backup_schema.json`, `/certs/schemas/wipe_schema.json`
- Reference docs: `/docs/PRD.md`, `/docs/schemas.md`

> Do not implement cloud backup in MVP. Do not embed private keys. Only load public key `pubkey.pem` at runtime for verification; keep private key external.

## Tasks

1. **Device Discovery**

   - Enumerate devices using:
     - `lsblk -J` (JSON), parse and filter block devices.
     - Enrich with `smartctl -i`, `hdparm -I`, `nvme id-ctrl` when available.
   - Output device metadata: `{ model, serial, capacity_bytes, bus }`.
   - Classify risk:
     - `CRITICAL`: system/root disk (e.g., contains `/`).
     - `HIGH`: mounted writable volumes.
     - `SAFE`: raw/unmounted disks.

2. **Backup (Encrypted)**

   - Detect “personal data” folders (e.g., `$HOME/Documents`, `Desktop`, `Pictures`), exclude OS/program files and caches.
   - Compress and encrypt using **AES-256-CTR** with a fresh random key (persist key material safely for restore only if required by spec; otherwise ephemeral).
   - Generate **SHA-256 manifest** of all original files (path → hash).
   - Copy to **external USB/NAS** path provided (`--dest`).
   - Integrity check: verify **N=5 random files** post-copy.
   - Emit **Backup Certificate JSON** that **validates against** `/certs/schemas/backup_schema.json`.

3. **Wipe (NIST-aligned)**

   - Default policy = **PURGE**.
   - **HPA/DCO**: detect and clear (where supported) via `hdparm`.
   - **SSD/NVMe**:
     - Use controller methods where supported: `nvme sanitize` (or `nvme format --ses=1`), or `hdparm --security-erase` / `--sanitize-crypto-erase`.
   - **HDD**:
     - CLEAR fallback: 1-pass random overwrite at full device, then **sampled verification** of random sectors.
   - Log **exact commands**, **exit codes**, **elapsed ms**, **verification outcomes**.
   - Emit **Wipe Certificate JSON** that validates against `/certs/schemas/wipe_schema.json` and includes `linkage.backup_cert_id` when available.

4. **Certificates (JSON + PDF)**

   - Sign JSON with **Ed25519**: `signature.alg="Ed25519"`, `signature.pubkey_id="sih_root_v1"`, `signature.sig=<base64>`.
   - Generate a **styled PDF**:
     - Header with logo/title, certificate ID and created_at.
     - Tables for device details, methods, logs, and verification summary.
     - **QR code** linking to the verification portal URL (configurable) or containing the certificate ID.
   - Save artifacts to `~/SecureWipe/certificates/`.

5. **CLI**
   - `discover` → prints JSON array of devices with risk classification.
   - `backup --device <dev> --dest <path> [--paths <p1> <p2> ...]` → performs encrypted backup.
   - `wipe --device <dev> --policy <CLEAR|PURGE>` → executes wipe with guard rails.
   - `cert --show <cert_id>` → prints a human-readable summary (from stored JSON).
   - All commands must:
     - Emit **structured JSON logs** to stderr (or a log file) with timestamps and step IDs.
     - Return nonzero exit codes on failure with actionable messages.

## Guard Rails & Constraints

- **Never wipe CRITICAL/system disks** unless an explicit `--i-know-what-im-doing` and **ISO mode** flag are both set (MVP UI will not expose these).
- AES-256-CTR is **mandatory** for backups.
- Default wipe policy is **PURGE**; if controller sanitize is unsupported, fall back to CLEAR and **record rationale** in the certificate.
- Private keys are **never** embedded; the signer must load from a secure runtime location.

## Deliverables

- Rust crates in `/core` with `Cargo.toml`.
- Unit tests for hashing/signing/schema validation.
- Integration tests for backup→wipe→cert path (with stub devices/mocks).
- Sample certs for demo.

---

### Example Prompt to Start

Generate a Rust CLI skeleton in `/core` with subcommands `discover`, `backup`, `wipe`, and `cert`, wiring structured logging and argument parsing. Include trait abstractions for device I/O so we can mock tests.
