# 🔒 SecureWipe — NIST-Compliant Data Sanitization Platform

<div align="center">

**Transforming e-waste management through verifiable, tamper-proof data wiping**

[![NIST SP 800-88](https://img.shields.io/badge/NIST-SP%20800--88%20Rev.1-blue)](https://csrc.nist.gov/publications/detail/sp/800-88/rev-1/final)
[![Ed25519 Signatures](https://img.shields.io/badge/Signatures-Ed25519-green)](https://ed25519.cr.yp.to/)
[![Rust](https://img.shields.io/badge/Core-Rust%201.82+-orange)](https://www.rust-lang.org/)
[![CI](https://github.com/heyitsgautham/erase-sure/actions/workflows/ci.yml/badge.svg)](https://github.com/heyitsgautham/erase-sure/actions/workflows/ci.yml)
[![Tauri](https://img.shields.io/badge/UI-Tauri%20%2B%20React-blueviolet)](https://tauri.app/)
[![FastAPI](https://img.shields.io/badge/Portal-FastAPI-009688)](https://fastapi.tiangolo.com/)
[![SIH 2024](https://img.shields.io/badge/SIH-Problem%2025070-red)](https://www.sih.gov.in/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)

[Features](#-features) • [Quick Start](#quick-start) • [Architecture](#architecture) • [CLI Reference](#cli-reference) • [API Docs](#verification-portal-api) • [Contributing](#contributing)

</div>

---

## 🎯 The Problem We Solve

**India generates ~1.75M tonnes of e-waste annually**, with **₹50,000+ crore worth of IT assets hoarded** due to fear of data breaches during recycling.

| Challenge | SecureWipe Solution |
|-----------|---------------------|
| **Trust Crisis** | Cryptographically signed certificates with QR verification |
| **Compliance Gaps** | NIST SP 800-88 Rev.1 aligned sanitization methods |
| **Technical Complexity** | One-click UI with intelligent device discovery |
| **Chain of Custody** | End-to-end audit trail from backup to wipe |
| **Verification** | Offline CLI + web portal for certificate validation |

---

## ⚡ Features

### 🛡️ NIST SP 800-88 Rev.1 Compliant Sanitization

| Method | Description | Use Case |
|--------|-------------|----------|
| **PURGE** | Controller-level sanitize commands | NVMe/SSD with crypto erase |
| **CLEAR** | Single overwrite + verification | HDD or unsupported controllers |
| **DESTROY** | Physical destruction (documented) | End-of-life compliance |

- **NVMe Sanitize** — Cryptographic/Block erase via `nvme-cli`
- **ATA Secure Erase** — Controller-level wipe via `hdparm`
- **HPA/DCO Clearing** — Remove hidden protected areas before wipe
- **Verification Sampling** — Random sector reads with hex diff logging

### 💾 Intelligent Encrypted Backup

- **AES-256-CTR Encryption** — Military-grade protection for backup data
- **SHA-256 Manifest** — Per-file integrity hashes
- **Smart Path Detection** — Auto-detects Documents, Pictures, Desktop
- **Post-Copy Verification** — Random sample integrity checks (configurable N)

### 📜 Tamper-Proof Certificate System

```
┌─────────────────────────────────────────────────────────────────┐
│  🔐 SecureWipe Certificate                                      │
├─────────────────────────────────────────────────────────────────┤
│  cert_id: WPE_2024_001                                          │
│  device:  Samsung SSD 980 PRO 1TB (NVMe)                        │
│  policy:  PURGE (nvme_sanitize_crypto_erase)                    │
│  status:  ✅ VERIFIED                                            │
├─────────────────────────────────────────────────────────────────┤
│  signature: Ed25519 (sih_root_v1)  │  ████████  QR Code         │
│  hash:      SHA-256 ✓              │  ████████  for Portal      │
│  chain:     BCK_2024_001 linked    │  ████████  Verification    │
└─────────────────────────────────────────────────────────────────┘
```

- **JSON Certificates** — Machine-readable, schema-validated audit records
- **Styled PDF Reports** — Professional documents with logo, tables, QR codes
- **Ed25519 Digital Signatures** — Cryptographic tamper detection
- **Certificate Linking** — Chain wipe certificates to backup certificates

### 🖥️ Cross-Platform Desktop UI

- **Device Discovery Cards** — Model, capacity, bus type, risk badges
- **Risk Classification** — CRITICAL (system) / HIGH (mounted) / SAFE (unmounted)
- **Two-Step Confirmation** — Guard rails for destructive operations
- **Real-Time Progress** — Live streaming of wipe steps and verification
- **Certificate Viewer** — JSON/PDF export with QR preview

### 🌐 Verification Portal

- **Schema Validation** — JSON Schema Draft-07 compliance checking
- **Signature Verification** — Ed25519 cryptographic proof validation
- **Hash Recomputation** — Integrity verification of embedded data
- **Chain Link Validation** — Verify backup-to-wipe certificate relationships
- **REST API** — Programmatic access for integration

---

## Architecture

SecureWipe is a Linux disk sanitization tool with three parts in one repository.

| Part | Folder | Language | What it does |
|---|---|---|---|
| Core engine and CLI | `core/` | Rust | Discovers drives, plans and runs NIST SP 800-88 CLEAR and PURGE wipes, makes encrypted backups, writes and signs JSON and PDF certificates |
| Desktop UI | `ui/` | Tauri 1 + React 18 + TypeScript | Spawns the `securewipe` binary and streams its output into a guided workflow |
| Verification portal | `portal/` | Python + FastAPI | Checks a certificate against its JSON schema and its Ed25519 signature |

```
          +---------------------+        spawns, streams stdout/stderr
          |  ui/ (Tauri, React) | ------------------------------------+
          +---------------------+                                     |
                                                                      v
+---------------------------+    lsblk, hdparm, nvme,    +----------------------+
| Linux block devices       | <------ dd, smartctl ----- | core/ securewipe CLI |
+---------------------------+                            +----------------------+
                                                                      |
                                     signed JSON + PDF certificates   |
                                     in ~/SecureWipe/certificates     v
                                                         +----------------------+
                                                         | portal/ FastAPI      |
                                                         | POST /verify         |
                                                         +----------------------+
```

### Core modules

| File | Responsibility |
|---|---|
| `core/src/main.rs`, `cmd.rs` | clap CLI with the `discover`, `backup`, `wipe` and `cert` subcommands |
| `core/src/device.rs` | Parses `lsblk -J`, enriches with `smartctl`, `hdparm` and `nvme`, and classifies each drive as CRITICAL, HIGH or SAFE |
| `core/src/wipe.rs` | `plan_wipe` and the `NistAlignedWipe` engine; every external command goes through a `CommandRunner` so tests can fake it |
| `core/src/backup.rs` | AES-256-CTR encrypted backup with a SHA-256 manifest and sampled verification |
| `core/src/cert.rs`, `schema.rs` | Builds schema-shaped certificates and validates them against `certs/schemas/*.json` |
| `core/src/signer.rs` | RFC 8785 style canonical JSON and Ed25519 signing and verification |
| `core/src/pdf.rs`, `cert_pdf.rs`, `python_pdf_generator.py` | PDF certificates, from printpdf or from ReportLab |
| `core/src/logging.rs` | One JSON object per log line on stderr |

### Safety model

The CLI plans by default and only writes to a device when every guard passes.

1. `securewipe wipe --device X` without `--danger-allow-wipe` prints a plan and touches nothing.
2. With `--danger-allow-wipe`, the environment must also hold `SECUREWIPE_DANGER=1`.
3. A drive classified CRITICAL (one of its partitions is mounted at `/`) is blocked unless `--iso-mode` says you booted from the SecureWipe ISO.
4. The operator types `WIPE <serial>` (or `CONFIRM WIPE` when the serial is unknown), or passes the same text in `SECUREWIPE_CONFIRM_TOKEN`.
5. The engine unmounts the device's partitions, tries controller sanitize (`nvme sanitize`, `hdparm --security-erase`), falls back to a `dd` overwrite, then reads random sectors back to verify.

Tests never reach a real device.
Wipe tests run against temp files through a fake `CommandRunner`, CI refuses to run as root, and the Docker images run unprivileged with no block devices mapped in.

---

## Quick start

### With Docker

```bash
git clone https://github.com/heyitsgautham/erase-sure.git
cd erase-sure
docker compose up --build
curl http://localhost:8000/health
```

`docker compose up` builds the portal image, writes a development key pair into the `keys` volume on first start, and serves the portal on port 8000.
Set `PORTAL_PORT` to use another port.
The CLI image runs on demand.

```bash
docker compose run --rm cli cert validate --file /data/latest_cert.json
```

The CLI container is unprivileged and sees no block devices, so it cannot wipe.

### From source

Prerequisites are Rust 1.82 or newer, Python 3.11 or newer, Node.js 20, and on Linux `util-linux`, `hdparm`, `nvme-cli` and `smartmontools` for real drives.

Build and test the core.

```bash
cd core
cargo build --release
cargo test
./target/release/securewipe discover --format human   # read-only
```

One core test renders a PDF through `python3`, so install the portal packages first (next step) or that test fails with a missing `reportlab`.

Set up the portal.

```bash
cd portal
python3 -m venv .venv && source .venv/bin/activate
pip install --require-hashes -r requirements-lock.txt
python keygen.py                 # writes ../keys/dev_private.pem and ../keys/dev_public.pem once
python run.py                    # http://localhost:8000
```

Run the desktop UI.

```bash
cd ui
npm ci
npm run tauri dev                # needs the core binary in core/target/debug or core/target/release
```

`npm run dev` serves the React app alone on http://localhost:1420, but every action calls the Tauri backend.
Wiping from the UI needs root, see `run-gui-sudo.sh` and `SUDO_REQUIREMENTS.md`.

---

## Configuration

Copy `.env.example` to `.env` in the folder you run the CLI from, because `securewipe` loads `.env` from its working directory.
The portal reads its variable from the process environment.

| Variable | Read by | Purpose |
|---|---|---|
| `SECUREWIPE_SIGN_KEY_PATH` | CLI | Ed25519 PKCS#8 PEM used when `--sign` is set and `--sign-key-path` is not |
| `SECUREWIPE_PUBKEY_PATH` | portal | Trusted Ed25519 public key, default `keys/dev_public.pem` |
| `SECUREWIPE_DANGER` | CLI | Must be `1` for `--danger-allow-wipe` to proceed |
| `SECUREWIPE_CONFIRM_TOKEN` | CLI | Non-interactive confirmation, `WIPE <serial>` or `CONFIRM WIPE` |
| `PORTAL_PORT` | docker compose | Host port for the portal, default 8000 |

Keys live in `keys/`, which git ignores.
Never commit a private key.

---

## CLI reference

Every command prints JSON on stdout and JSON log lines on stderr.

```bash
# Discover drives with risk levels (read-only)
securewipe discover --format human

# Encrypted backup of chosen folders to a destination
securewipe backup --device /dev/sdb --dest /media/usb --paths ~/Documents --paths ~/Pictures --sign

# Plan a wipe (safe, prints the plan only)
securewipe wipe --device /dev/sdb --policy PURGE --format human

# Destructive wipe, as root, after a typed confirmation
sudo SECUREWIPE_DANGER=1 securewipe wipe --device /dev/sdb --policy CLEAR \
  --danger-allow-wipe --sign --backup-cert-id <backup-cert-id>

# Certificates in ~/SecureWipe/certificates
securewipe cert --show <cert-id>
securewipe cert --export-pdf <cert-id>
securewipe cert sign --file cert.json --key keys/dev_private.pem
securewipe cert verify --file cert.json --pubkey keys/dev_public.pem
securewipe cert validate --file cert.json
```

`--policy` takes `CLEAR` (one zero pass) or `PURGE` (HPA/DCO reset, then controller sanitize or one random pass).
`--samples` sets how many sectors the planner reports for verification.

---

## Verification portal API

| Method | Path | Description |
|---|---|---|
| `GET` | `/` | HTML page with usage notes |
| `GET` | `/health` | `status`, `schemas_loaded`, `public_key_loaded`, `version` |
| `POST` | `/verify` | Validate one certificate; add `linked_backup_cert` to a wipe certificate to check the chain |
| `GET` | `/verify/{cert_id}` | Placeholder that points to `POST /verify` |

```bash
curl -X POST http://localhost:8000/verify \
  -H "Content-Type: application/json" \
  -d @tests/test_backup_cert.json
```

The response holds `schema_valid`, `signature_valid`, `hash_valid`, `chain_valid`, a `cert_summary`, the `computed` SHA-256 of the body and a list of `errors`.
A body that is not a JSON object gets a 400.

---

## Testing

| Part | Command | Notes |
|---|---|---|
| core | `cd core && cargo test` | 118 tests; needs `python3` with the portal packages for the PDF test |
| core lint | `cargo fmt --check && cargo clippy --all-targets -- -D warnings` | |
| core coverage | `cargo llvm-cov --all-targets --summary-only --fail-under-lines 65` | needs `cargo-llvm-cov` |
| portal | `cd portal && python -m pytest --cov` | fails under 80% coverage |
| portal lint | `ruff check . && ruff format --check .` | config in `portal/pyproject.toml` |
| ui | `cd ui && npm ci && npm run lint && npm run build` | ESLint with zero warnings, then `tsc` and Vite |

GitHub Actions runs all of these on every push and pull request, plus `cargo audit`, `pip-audit`, `npm audit`, a gitleaks scan and a Docker build with a health check.
See `.github/workflows/ci.yml`.

The shell scripts in `tests/` and `tests/scripts/` exercise a real Linux machine, and `test_destructive_wipe_integration.sh`, `test_hpa_dco_demo.sh` and `test_wipe_cli_unit.sh` call the destructive wipe path.
Read each one before running it, and never point one at a disk you care about.

---

## Project structure

```
erase-sure/
├── core/                  Rust engine and securewipe CLI
│   ├── src/               modules listed above
│   ├── tests/             integration tests (CLI, signing, fake-runner wipes)
│   └── Dockerfile         CLI image
├── ui/                    Tauri + React desktop app
│   ├── src/               screens, components, hooks, context
│   └── src-tauri/         Rust side of the Tauri app
├── portal/                FastAPI verification portal
│   ├── app/main.py        endpoints, schema and signature checks
│   ├── keygen.py          development key pair helper
│   └── Dockerfile         portal image
├── certs/schemas/         backup and wipe certificate JSON schemas
├── tests/                 PDF generators, fixtures and Linux shell test scripts
├── docs/, instructions/   design notes, PRD and implementation write-ups
├── iso/                   bootable ISO notes
└── docker-compose.yml     portal plus on-demand CLI
```

---

## Standards

| Standard | What the code does |
|---|---|
| NIST SP 800-88 Rev.1 | CLEAR is one overwrite pass; PURGE resets HPA/DCO and uses controller sanitize or one random pass; both sample sectors afterwards |
| Ed25519 (RFC 8032) | Certificate signatures, `ed25519-dalek` in the CLI and PyNaCl in the portal |
| JSON Schema Draft-07 | `certs/schemas/backup_schema.json` and `wipe_schema.json` |
| AES-256 (FIPS 197) | Backup encryption in CTR mode |

---

## Security

- Private keys are loaded at runtime from a path and are never committed; `keys/`, `*.pem` and `.env` are gitignored and kept out of Docker builds.
- CRITICAL drives are blocked unless `--iso-mode` is set, and every destructive run needs the flag, the environment variable and the typed confirmation.
- CI runs gitleaks over the full history and audits Rust, Python and npm dependencies on every push.

Report a vulnerability privately to the maintainers instead of opening a public issue.

---

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for setup, checks and commit style.

---

## License

MIT, see [LICENSE](./LICENSE).

---

## Disclaimer

This tool destroys data and the destruction cannot be undone.
Back up first, test on spare drives, check the device path twice, and wipe system drives from bootable media.
SecureWipe follows NIST SP 800-88 Rev.1, and you remain responsible for meeting your own regulatory requirements.

---

## Roadmap

Done in the MVP:

- [x] Linux NIST sanitization (PURGE and CLEAR)
- [x] AES-256-CTR encrypted backup
- [x] JSON and PDF certificates with Ed25519 signatures
- [x] FastAPI verification portal
- [x] Tauri desktop UI

Next:

- [ ] Windows support
- [ ] Batch workflows
- [ ] Certificate lookup by ID in the portal

---

<div align="center">

**SIH 2024 • Problem Statement 25070 • Ministry of Mines, JNARDDC**

</div>
