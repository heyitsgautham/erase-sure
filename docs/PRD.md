# File: docs/PRD.md

# Product Requirements Document (PRD)

**Project Title**: Secure Data Wiping for Trustworthy IT Asset Recycling  
**PS ID**: 25070 (Ministry of Mines, JNARDDC)  
**MVP Timeline**: 1 week (demo-first, extensible architecture)  
**Primary Platforms (MVP)**: Linux (full), Android technician path (ADB/Recovery), Windows (simulated flow)  
**Language**: English only (MVP)  
**Certificates**: Digitally signed JSON + **nicely styled** PDF with logo + QR for verification  
**Offline**: Bootable ISO/USB (manual build for MVP, scripts later)

---

## 1) Problem Overview

India generates ~1.75M tonnes e-waste annually, with ~₹50,000+ crore of IT assets hoarded due to fear of data leaks. Existing tools are complex/expensive or lack verifiable proof of erasure.  
Goal: A user-friendly, **NIST SP 800-88 aligned** wiping tool with **tamper-proof certificates** to boost recycling confidence.

---

## 2) Objectives, Scope, Non-Goals

### 2.1 Objectives (MVP)

- End-to-end flow: **Backup → Wipe → Dual Certificates → Verify**.
- **Linux** full support (HDD overwrite, NVMe sanitize, HPA/DCO clear).
- **Android technician path** (ADB/Recovery wipe) and device attestation fields.
- **Windows** simulated path (device discovery + wipe plan + guard rails).
- Certificates: **JSON** and **styled PDF** (with logo, tables, QR), **Ed25519** signatures.
- **Offline-first**: Bootable ISO with preinstalled tools.
- **Verification**: Offline CLI + lightweight **FastAPI** portal (signature/hash checks).
- Clear, simple **one-click UI** (Tauri + React).

### 2.2 Out-of-Scope for MVP (Roadmap)

- Cloud backup (Drive/Dropbox/OneDrive).
- Blockchain anchoring (Merkle-root anchoring).
- Multi-language i18n.
- Enterprise fleet orchestration.
- Automated ISO build pipelines/Dockerized DevOps.

---

## 3) Personas

- **Citizen Recycler**: backs up personal files and wipes device safely.
- **Service Technician**: bulk wipes; needs auditable proof per device.
- **Auditor/Verifier**: validates wipe claims using certificates.
- **Enterprise IT (Future)**: large-scale compliance, dashboards.

---

## 4) Key Requirements

### 4.1 Functional Requirements

1. **Device Discovery (Linux)**
   - Enumerate disks via `lsblk -J`, `smartctl -i`, `hdparm -I`, `nvme id-ctrl`.
   - Display: model, serial, capacity, bus, health (if available).
   - Risk classification:
     - **CRITICAL**: system/root disk (block wipe unless ISO mode).
     - **HIGH**: mounted writable volumes.
     - **SAFE**: unmounted raw disks.
2. **Intelligent Backup (USB/NAS only for MVP)**
   - Classify directories (Documents/Pictures/Desktop) and exclude OS files.
   - Compress + **AES-256-CTR** encrypt; produce **SHA-256** manifest.
   - Post-copy integrity: verify N random files (default N=5).
   - Emit **Backup Certificate** (JSON + styled PDF).
3. **Secure Wipe (NIST Mapping)**
   - **CLEAR**: single overwrite + sample verify (HDD fallback).
   - **PURGE (Default)**: controller-level erase (ATA Secure Erase / NVMe Sanitize).
   - **DESTROY**: documented physical methods (info-only for MVP).
   - Remove **HPA/DCO** (where applicable) prior to wipe.
   - Verification: random sector reads (sampled) with hex diff logs.
   - Emit **Wipe Certificate** (JSON + styled PDF), linked to backup cert ID.
4. **Android Technician Path**
   - Recovery/ADB factory reset; relock bootloader where applicable.
   - Record device identifiers (IMEI\*, serial, attestation digest where feasible).
   - Emit wipe certificate with Android metadata and method.
5. **Verification**
   - **CLI**: Provide `verify --file <cert.json> --pub <pubkey.pem>` (offline).
   - **Portal (FastAPI)**:
     - Upload JSON or open via `GET /verify/{cert_id}`.
     - Verify signature, recompute hash, validate chain link.
     - Show green ticks (Signature, Hash, Chain Link), display cert fields.
6. **Bootable ISO**
   - Live Linux with `hdparm`, `nvme-cli`, `smartmontools`, `parted`, `wipefs`.
   - Prevent auto-mount, autostart UI, run as non-root with privileged helpers.
   - Save certificates to chosen external media.

### 4.2 Non-Functional Requirements

- **Security**:
  - AES-256-CTR for backup; ephemeral keys in memory; zeroize secrets.
  - **Ed25519** signatures; offline root private key; bundle only **pubkey** for verification.
  - Privileged operations gated, explicit confirmation flows.
- **Performance**:
  - Wipe operations near device throughput (bounded by controller).
  - Certificate verification < 200 ms on reasonable hardware.
- **Reliability**:
  - Power loss handling: detect incomplete runs, require re-verify.
  - Atomic file writes for manifests and certs.
- **Usability**:
  - One primary CTA; destructive actions guarded by 2-step confirmation.
  - Plain-English summaries, progress bars, success/failure states.
- **Observability**:
  - Structured logs (JSON lines) with timestamps + step IDs.

---

## 5) Detailed Flows

### 5.1 Backup Flow

- Detect candidate folders → estimate size → recommend destination (USB).
- Encrypt (AES-256-CTR) + compress → copy to destination.
- Compute manifest (SHA-256 per file) and store.
- Sample verify (N files, random).
- Generate **Backup Certificate** (JSON + PDF) with: device info, totals, destination, manifest hash, timestamp, signature.

### 5.2 Wipe Flow (Linux)

- Pre-flight: unmount targets, confirm not CRITICAL unless ISO mode.
- HPA/DCO clear (where supported) using `hdparm`.
- NVMe: `nvme sanitize` (method selection per controller capabilities).
- SATA SSD: `hdparm --security-erase` / `--sanitize-crypto-erase`.
- HDD: 1-pass random overwrite + sampled verification.
- Log exact commands, exit codes, elapsed ms, and verification samples.
- Generate **Wipe Certificate** (JSON + PDF), linked to backup cert ID.

### 5.3 Android Flow

- Recovery reset/ADB factory reset; optionally relock bootloader.
- Capture device IDs (IMEI\*/serial/Android ID/attestation digest where feasible).
- Generate Wipe Certificate with Android metadata + method details.

### 5.4 Verification

- **CLI**: Input JSON cert + `pubkey.pem` → verify signature + recompute embedded content hash.
- **Portal**: show result with green ticks; display structured data; allow PDF/JSON download.

---

## 6) Data Contracts (Summaries)

See `/docs/schemas.md` for full JSON Schemas. High-level fields:

**Backup Certificate (JSON)**

- `cert_type` = `"backup"`
- `cert_id`, `created_at`, `device{}` (model, serial, capacity_bytes, bus)
- `files_summary{count, personal_bytes}`
- `destination{type,label,fs}`
- `crypto{alg, manifest_sha256}`
- `signature{alg, sig, pubkey_id}`

**Wipe Certificate (JSON)**

- `cert_type` = `"wipe"`
- `cert_id`, `created_at`, `device{}`
- `policy{nist_level, method}`
- `hpa_dco{cleared, commands[]?}`
- `commands[]` with `cmd`, `exit`, `ms`
- `verify{strategy, samples, failures}`
- `linkage{backup_cert_id}`
- `signature{alg, sig, pubkey_id}`

---

## 7) UX Requirements

- **Home**: “Backup & Wipe” primary CTA; “Wipe Only” secondary.
- **Device Cards**: model, capacity, bus, serial (masked), **Risk Badge**.
- **Progress**: percent + step name + estimated time (where possible).
- **Completion**: four buttons (Backup JSON/PDF, Wipe JSON/PDF) + **QR** previews.
- **PDF Style**: header logo + title + certificate ID; two-column tables; QR on first page; footer with timestamp and signature summary.

---

## 8) Acceptance Criteria (MVP)

- Linux: successful detection, backup (encrypted), NVMe sanitize or HDD overwrite, and certificate generation with valid signatures.
- Android: technician path demonstration with issued wipe certificate.
- Verification: CLI and portal both validate signatures and hashes; chain link check passes when both certs present.
- Guard rails: cannot wipe CRITICAL disk unless booted from ISO; confirmation dialogs present.

---

## 9) Risks & Mitigations

- **Controller not supporting sanitize** → fallback to overwrite with CLEAR mapping and explain in cert.
- **Power loss** → resumable logs; re-run requires re-verification.
- **User selects wrong disk** → CRITICAL guard rails + confirmations + risk badges.
- **Privileged operations** → require sudo; show explicit warnings.

---

## 10) Testing Strategy

- **Unit**: cryptography utils, manifest hashing, schema validation, signature routines.
- **Integration**: full backup→wipe→cert→verify loop with stub disks; real device smoke tests.
- **Manual Demo**:
  - USB backup of 1–2 GB sample.
  - HDD overwrite on external drive.
  - NVMe sanitize on test machine.
  - Android reset via ADB.
  - Verify certs on portal + CLI.
- **Artifacts**: store sample certs for fallback.

---

## 11) Roadmap

- **Phase 2**: Windows/macOS backends; cloud backup integrations; automated ISO build.
- **Phase 3**: Blockchain anchoring; enterprise fleet; role-based portal.
- **Phase 4**: i18n; incentives/gamification; telemetry (opt-in).

---

## 12) Repo Structure (Monorepo)

See root `README.md` and detailed tree below:

- `/core` (Rust) – erasure + backup engine
- `/ui` (Tauri + React) – desktop UI
- `/certs` – schemas + signing helpers
- `/portal` (FastAPI) – verifier
- `/iso` – manual build docs
- `/docs` – PRD, schemas, prompts
- `/.github` – Copilot instructions
- `/.cursorrules` – editor guidance

---

## 13) Definition of Done (MVP)

- Runs on Linux host and ISO; completes end-to-end flows; produces verifiable certs; passes acceptance tests; clean demo narrative with slides and sample artifacts.

---

# File: docs/schemas.md

# Certificate Schemas (JSON Schema Draft 2020-12)

Note: Keep `pubkey_id` stable for MVP (`"sih_root_v1"`). All timestamps ISO 8601 with timezone.

## Backup Certificate Schema

(Also saved as `/certs/schemas/backup_schema.json`)
{
"$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://example.org/schemas/backup_certificate.json",
"type": "object",
"required": ["cert_type","cert_id","device","files_summary","destination","crypto","created_at","signature"],
"properties": {
"cert_type": { "const": "backup" },
"cert_id": { "type": "string", "minLength": 3 },
"device": {
"type": "object",
"required": ["model","capacity_bytes"],
"properties": {
"model": { "type": "string" },
"serial": { "type": "string" },
"bus": { "type": "string" },
"capacity_bytes": { "type": "integer", "minimum": 0 }
}
},
"files_summary": {
"type": "object",
"required": ["count","personal_bytes"],
"properties": {
"count": { "type": "integer", "minimum": 0 },
"personal_bytes": { "type": "integer", "minimum": 0 }
}
},
"destination": {
"type": "object",
"required": ["type"],
"properties": {
"type": { "type": "string", "enum": ["usb","nas","cloud","other"] },
"label": { "type": "string" },
"fs": { "type": "string" }
}
},
"crypto": {
"type": "object",
"required": ["alg","manifest_sha256"],
"properties": {
"alg": { "type": "string", "enum": ["AES-256-CTR"] },
"manifest_sha256": { "type": "string", "pattern": "^[a-f0-9]{64}$" }
}
},
"created_at": { "type": "string", "format": "date-time" },
"signature": {
"type": "object",
"required": ["alg","sig","pubkey_id"],
"properties": {
"alg": { "type": "string", "enum": ["Ed25519"] },
"sig": { "type": "string" },
"pubkey_id": { "type": "string" }
}
}
},
"additionalProperties": true
}

## Wipe Certificate Schema

(Also saved as `/certs/schemas/wipe_schema.json`)
{
"$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://example.org/schemas/wipe_certificate.json",
"type": "object",
"required": ["cert_type","cert_id","device","policy","verify","created_at","signature"],
"properties": {
"cert_type": { "const": "wipe" },
"cert_id": { "type": "string", "minLength": 3 },
"device": {
"type": "object",
"required": ["model"],
"properties": {
"model": { "type": "string" },
"serial": { "type": "string" },
"bus": { "type": "string" },
"capacity_bytes": { "type": "integer", "minimum": 0 }
}
},
"policy": {
"type": "object",
"required": ["nist_level","method"],
"properties": {
"nist_level": { "type": "string", "enum": ["CLEAR","PURGE","DESTROY"] },
"method": { "type": "string" }
}
},
"hpa_dco": {
"type": "object",
"properties": {
"cleared": { "type": "boolean" },
"commands": {
"type": "array",
"items": { "type": "string" }
}
},
"additionalProperties": true
},
"commands": {
"type": "array",
"items": {
"type": "object",
"required": ["cmd","exit"],
"properties": {
"cmd": { "type": "string" },
"exit": { "type": "integer" },
"ms": { "type": "integer", "minimum": 0 }
}
}
},
"verify": {
"type": "object",
"required": ["strategy","samples","failures"],
"properties": {
"strategy": { "type": "string", "enum": ["random_sectors","full_readback","controller_status"] },
"samples": { "type": "integer", "minimum": 0 },
"failures": { "type": "integer", "minimum": 0 }
}
},
"linkage": {
"type": "object",
"properties": {
"backup_cert_id": { "type": "string" }
}
},
"created_at": { "type": "string", "format": "date-time" },
"signature": {
"type": "object",
"required": ["alg","sig","pubkey_id"],
"properties": {
"alg": { "type": "string", "enum": ["Ed25519"] },
"sig": { "type": "string" },
"pubkey_id": { "type": "string" }
}
}
},
"additionalProperties": true
}

---

# File: .github/copilot-instructions.md

# Copilot Instructions for SecureWipe (MVP)

- Use **Rust** for core wiping/backup (`/core`), **Tauri + React** for desktop UI (`/ui`), and **FastAPI** for verification portal (`/portal`).
- Certificates must **strictly** follow `/docs/schemas.md` and `/certs/schemas/*.json`.
- Always add structured logging and robust error handling. Never run destructive actions without risk/guard checks.
- **Out of scope (MVP)**: cloud backup, blockchain anchoring, enterprise fleet orchestration, multi-language.
- Default wipe policy = **PURGE**. If sanitize not supported, fallback to CLEAR (overwrite) and document rationale in the certificate.
- Backup must be encrypted with **AES-256-CTR** and must generate a SHA-256 manifest. Perform N=5 random file sample verification.
- PDF certificates must be **nicely styled** (logo header, tables, QR to verifier).
- Guard rails: do not wipe CRITICAL/system disks unless running in bootable ISO mode and user confirms twice.
- Use **Ed25519** signatures; do not embed private keys in repo; only reference `pubkey_id = "sih_root_v1"` and expect `pubkey.pem` at runtime.

---

# File: .cursorrules

# Cursor/Editor Guidance

- Language: Rust (core), TypeScript/React (UI), Python (FastAPI portal).
- Security hard rules:
  - Use Ed25519 for signatures; never generate keys in code; load from secure path.
  - AES-256-CTR only; zeroize buffers after use if possible.
- UX:
  - Primary CTA is “Backup & Wipe”.
  - Show risk badges on device cards: CRITICAL / HIGH / SAFE.
  - Two-step confirmation before destructive actions.
- Testing:
  - Unit tests for hashing, signing, schema validation.
  - Integration test for backup→wipe→cert→verify loop.
- Files:
  - Tests in `/core/tests` and `/portal/tests`.
  - Schemas live in `/certs/schemas` and `/docs/schemas.md` (reference, not code).
- Out of scope for MVP: cloud OAuth, Docker, i18n, blockchain.

---

# File: docs/copilot_prompt_core.md

# Copilot Prompt — Core (Rust Erasure + Backup)

**Goals**

- Implement secure backup and wipe operations on Linux, exposing a CLI interface callable by the Tauri UI.
- Generate JSON + styled PDF certificates with Ed25519 signatures.

**Key Requirements**

- Device discovery: parse `lsblk -J`; enrich with `smartctl -i`, `hdparm -I`, `nvme id-ctrl`.
- Risk classification: CRITICAL (system/root), HIGH (mounted), SAFE (raw).
- Backup:
  - Select common personal dirs; exclude OS/program files.
  - Compress + **AES-256-CTR** encrypt; generate **SHA-256 manifest**.
  - Sample verify N=5 random files.
  - Emit Backup Certificate JSON conforming to `/certs/schemas/backup_schema.json`.
- Wipe:
  - HPA/DCO removal where supported (hdparm).
  - NVMe sanitize or SATA secure-erase for PURGE; HDD overwrite for CLEAR.
  - Random sector verification sampling; log commands with exit codes and ms timings.
  - Emit Wipe Certificate JSON conforming to `/certs/schemas/wipe_schema.json` and link to backup cert ID.
- Certificates:
  - Sign JSON with Ed25519 using offline key (provided at runtime).
  - Generate **nicely styled PDF** mirroring the JSON (logo, tables, QR pointing to portal/CLI verify).
- CLI commands:
  - `discover` → JSON device list with risk.
  - `backup --device <dev> --dest <mount> [--paths ...]`
  - `wipe --device <dev> --policy <CLEAR|PURGE>`
  - `cert --print --id <cert_id>`
- Logging:
  - Structured JSON logs; errors carry actionable messages.

---

# File: docs/copilot_prompt_ui.md

# Copilot Prompt — UI (Tauri + React)

**Goals**

- Provide a one-click, non-technical UI to run the **Backup → Wipe → Certificates** flow safely.

**Views**

- Welcome (SIH banner + “Backup & Wipe” primary CTA, “Wipe Only” secondary).
- Device Discovery:
  - Cards: model, capacity, bus, masked serial, **Risk Badge**.
  - Block CRITICAL unless ISO mode; show guidance tooltip.
- Backup:
  - Show recommended destination (USB/NAS), always-on **AES-256** note.
  - Progress with bytes transferred; post-verify results.
- Wipe:
  - Policy picker with default **PURGE**; plain-English explanations.
  - Two-step confirmation for destructive action.
  - Real-time progress and step logs.
- Certificates:
  - Show **four buttons**: Backup JSON/PDF, Wipe JSON/PDF.
  - Render QR previews; show verification instructions (CLI + portal).

**Tauri IPC**

- Call `/core` CLI with structured payloads.
- Stream progress events to UI; handle cancellations safely (confirm required).

**Accessibility & Style**

- Clear warnings; readable status colors; large action buttons.
- PDF downloads accessible via file pickers; default save folder `~/SecureWipe/certificates`.

---

# File: docs/copilot_prompt_portal.md

# Copilot Prompt — Verification Portal (FastAPI)

**Goals**

- Provide lightweight verification of JSON certificates with public key, and display a human-readable result.

**Endpoints**

- `POST /verify` — body: JSON certificate; returns signature/hash/chain-link results + parsed data.
- `GET /verify/{cert_id}` — fetch stored metadata (optional for MVP; can parse uploaded JSON).

**Logic**

- Load JSON, validate against schema (backup or wipe).
- Verify **Ed25519** signature using provided `pubkey.pem`.
- Recompute hash for any hash fields and compare.
- If wipe cert includes `linkage.backup_cert_id`, validate presence/consistency (if backup cert provided).
- Return structured response; surface green ticks in simple HTML UI.

**Non-Goals**

- No authentication, no role-based access, no database (MVP can be stateless with in-memory or file cache).

---

# File: iso/build.md

# Bootable ISO — Manual Build Notes (MVP)

**Objective**: Provide a live Linux ISO/USB that runs the UI and supports privileged wipe commands offline.

**Base Options**

- Ubuntu (minimal) or TinyCore Linux.
- Include tools: `hdparm`, `nvme-cli`, `smartmontools`, `parted`, `wipefs`, `lsblk`.
- Autostart the Tauri UI on login; disable auto-mount of newly attached disks (udev rules).

**Steps (high-level)**

1. Create minimal live image; install required packages.
2. Add a non-root user; configure `sudo` for specific commands.
3. Copy `/core` and `/ui` binaries; set UI to autostart with `.desktop` entry.
4. Add `public/pubkey.pem` in read-only location; private keys never included.
5. Test on target hardware: detect disks, perform PURGE on NVMe, CLEAR on HDD.

**Roadmap**

- Scripted builds (Packer/Buildroot).
- Secure Boot signing (MOK enrollment flow).

---

# File: README.md

# SecureWipe (MVP)

**Purpose**: NIST-aligned secure wipe with verifiable certificates to enable safe IT asset recycling (SIH PS 25070).  
**Platforms**: Linux full; Android technician path; Windows simulated.  
**Core Tech**: Rust core, Tauri + React UI, FastAPI verification portal.

## Quick Start (Dev)

- See `docs/PRD.md` for product details and `docs/schemas.md` for certificate schemas.
- Build `/core` (Rust) and `/ui` (Tauri).
- Run `/portal` (FastAPI) locally for verification testing.

## Certificates

- JSON + **styled PDF** with QR.
- Signed with **Ed25519**; verify using CLI or portal:
  - CLI: `verify --file <cert.json> --pub pubkey.pem`
  - Portal: `POST /verify` with JSON or open `GET /verify/{cert_id}` (if enabled)

## Safety

- Guard rails prevent wiping system disks unless in ISO mode.
- Always back up before wiping; encryption is mandatory for backups.

---

# File: certs/schemas/backup_schema.json

{ Refer to the “Backup Certificate Schema” block in docs/schemas.md — paste the same JSON here. }

# File: certs/schemas/wipe_schema.json

{ Refer to the “Wipe Certificate Schema” block in docs/schemas.md — paste the same JSON here. }

---

# File: repo-structure.txt

securewipe/
├─ core/
│ ├─ src/
│ ├─ Cargo.toml
│ └─ tests/
├─ ui/
│ ├─ src/
│ ├─ package.json
│ └─ tauri.conf.json
├─ certs/
│ ├─ schemas/
│ │ ├─ backup_schema.json
│ │ └─ wipe_schema.json
│ └─ (signing helpers live with core; private keys never committed)
├─ portal/
│ ├─ app/
│ ├─ requirements.txt
│ └─ tests/
├─ iso/
│ └─ build.md
├─ docs/
│ ├─ PRD.md
│ ├─ schemas.md
│ ├─ copilot_prompt_core.md
│ ├─ copilot_prompt_ui.md
│ └─ copilot_prompt_portal.md
├─ .github/
│ └─ copilot-instructions.md
├─ .cursorrules
├─ README.md
└─ LICENSE
