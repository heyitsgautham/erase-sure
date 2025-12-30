# 🔒 SecureWipe — NIST-Compliant Data Sanitization Platform

<div align="center">

**Transforming e-waste management through verifiable, tamper-proof data wiping**

[![NIST SP 800-88](https://img.shields.io/badge/NIST-SP%20800--88%20Rev.1-blue)](https://csrc.nist.gov/publications/detail/sp/800-88/rev-1/final)
[![Ed25519 Signatures](https://img.shields.io/badge/Signatures-Ed25519-green)](https://ed25519.cr.yp.to/)
[![Rust](https://img.shields.io/badge/Core-Rust%201.70+-orange)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/UI-Tauri%20%2B%20React-blueviolet)](https://tauri.app/)
[![FastAPI](https://img.shields.io/badge/Portal-FastAPI-009688)](https://fastapi.tiangolo.com/)
[![SIH 2024](https://img.shields.io/badge/SIH-Problem%2025070-red)](https://www.sih.gov.in/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)

[Features](#-features) • [Quick Start](#-quick-start) • [Architecture](#-architecture) • [CLI Reference](#-cli-reference) • [API Docs](#-verification-portal-api) • [Contributing](#-contributing)

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

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        SecureWipe Platform                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   ┌──────────────┐    ┌──────────────┐    ┌──────────────────────┐     │
│   │  🦀 Core     │    │  🖥️ UI       │    │  🌐 Portal           │     │
│   │  Engine      │◄───│  Desktop     │    │  Verification        │     │
│   │  (Rust)      │    │  (Tauri)     │    │  (FastAPI)           │     │
│   └──────┬───────┘    └──────────────┘    └──────────────────────┘     │
│          │                                                              │
│   ┌──────▼───────────────────────────────────────────────────────┐     │
│   │                    System Layer (Linux)                       │     │
│   ├──────────────┬──────────────┬──────────────┬─────────────────┤     │
│   │  lsblk       │  nvme-cli    │  hdparm      │  smartmontools  │     │
│   │  (discovery) │  (NVMe ops)  │  (ATA ops)   │  (health info)  │     │
│   └──────────────┴──────────────┴──────────────┴─────────────────┘     │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Technology Stack

| Layer | Technology | Purpose |
|-------|------------|---------|
| **Core Engine** | Rust 1.70+ | Device control, NIST algorithms, certificate generation |
| **Desktop UI** | Tauri 1.6 + React 18 + TypeScript | Cross-platform GUI with native performance |
| **Verification Portal** | Python 3.11 + FastAPI | Certificate validation REST API |
| **Cryptography** | Ed25519 (ed25519-dalek) | Digital signatures |
| **Encryption** | AES-256-CTR | Backup data protection |
| **Hashing** | SHA-256 (sha2) | File integrity & certificate hashing |
| **PDF Generation** | printpdf + ReportLab | Styled certificate documents |
| **QR Codes** | qrcode crate + Pillow | Portal verification links |
| **Schema Validation** | JSON Schema Draft-07 | Certificate format compliance |

### Core Rust Dependencies

```toml
clap = "4.0"              # CLI argument parsing
serde = "1.0"             # Serialization framework
tokio = "1.0"             # Async runtime
ed25519-dalek = "2.0"     # Ed25519 signatures
aes = "0.8" + ctr = "0.9" # AES-256-CTR encryption
sha2 = "0.10"             # SHA-256 hashing
printpdf = "0.7"          # PDF generation
qrcode = "0.14"           # QR code generation
jsonschema = "0.17"       # Schema validation
chrono = "0.4"            # Timestamp handling
uuid = "1.0"              # Unique identifiers
```

### UI Stack

```json
{
  "@tauri-apps/api": "^1.6.0",     // Native OS integration
  "react": "^18.2.0",              // Component framework
  "react-router-dom": "^6.8.1",    // Navigation
  "lucide-react": "^0.294.0",      // Icon library
  "qrcode": "^1.5.4",              // QR generation
  "typescript": "^5.2.2"           // Type safety
}
```

### Portal Dependencies

```txt
fastapi==0.115.6          # Web framework
pynacl==1.5.0             # Ed25519 verification
jsonschema==4.25.1        # Schema validation
reportlab==4.2.5          # PDF generation
qrcode[pil]==8.0          # QR codes
cryptography==45.0.7      # Key handling
```

---

## 🚀 Quick Start

### Prerequisites

```bash
# Required tools
rustc --version     # Rust 1.70+ (rustup.rs)
node --version      # Node.js 18+ (nodejs.org)
python --version    # Python 3.11+ (python.org)

# Linux system tools (for full functionality)
sudo apt install lsblk hdparm nvme-cli smartmontools
```

### 1️⃣ Clone Repository

```bash
git clone https://github.com/heyitsgautham/erase-sure.git
cd erase-sure
```

### 2️⃣ Build Core Engine (Rust)

```bash
cd core
cargo build --release
cargo test

# Quick test - discover devices (safe, read-only)
cargo run -- discover
```

### 3️⃣ Setup Verification Portal (Python)

```bash
cd portal
python -m venv venv
source venv/bin/activate  # Windows: venv\Scripts\activate
pip install -r requirements.txt

# Start server
python run.py
# → http://localhost:8000
```

### 4️⃣ Build Desktop UI (Tauri + React)

```bash
cd ui
npm install
npm run dev          # Development mode with hot reload
npm run tauri dev    # Run as desktop app
npm run build        # Production build
```

### 5️⃣ Setup Signing Keys (Development)

```bash
# Generate development Ed25519 keypair
cd keys
openssl genpkey -algorithm ED25519 -out dev_private.pem
openssl pkey -in dev_private.pem -pubout -out dev_public.pem

# Set environment variable
export SECUREWIPE_PRIVATE_KEY_PATH=$(pwd)/dev_private.pem
export SECUREWIPE_PUBKEY_PATH=$(pwd)/dev_public.pem
```

---

## 💻 CLI Reference

### Device Discovery (Safe - Read Only)

```bash
# List all storage devices with risk assessment
cargo run -- discover

# JSON output for programmatic use
cargo run -- discover --format json

# Verbose mode with SMART data
cargo run -- discover --verbose
```

**Output Example:**
```
📀 Discovered Devices:
┌────────────────────────────────────────────────────────────────┐
│ /dev/nvme0n1 │ Samsung SSD 980 PRO │ 1.0 TB │ NVMe │ ⚠️ HIGH  │
│ /dev/sda     │ WD Blue 2TB         │ 2.0 TB │ SATA │ ✅ SAFE  │
│ /dev/sdb     │ SanDisk USB         │ 64 GB  │ USB  │ ✅ SAFE  │
└────────────────────────────────────────────────────────────────┘
```

### Secure Backup

```bash
# Basic backup with AES-256-CTR encryption
cargo run -- backup \
  --device /dev/sda \
  --destination /media/backup-usb \
  --paths ~/Documents ~/Pictures ~/Desktop

# Custom verification sample count
cargo run -- backup \
  --device /dev/sda \
  --destination /media/backup \
  --verify-samples 10
```

### Secure Wipe (⚠️ Destructive!)

```bash
# PURGE level - NVMe Sanitize (recommended for SSDs)
sudo cargo run -- wipe \
  --device /dev/nvme0n1 \
  --policy PURGE \
  --method nvme_sanitize_crypto_erase

# CLEAR level - Overwrite with verification (HDDs)
sudo cargo run -- wipe \
  --device /dev/sda \
  --policy CLEAR \
  --method overwrite_verify

# Link to backup certificate
sudo cargo run -- wipe \
  --device /dev/sda \
  --policy PURGE \
  --backup-cert-id BCK_2024_001
```

### Certificate Management

```bash
# View certificate details
cargo run -- cert --show ./certificates/WPE_2024_001.json

# Generate PDF from JSON certificate
cargo run -- cert \
  --json-path ./certificates/WPE_2024_001.json \
  --output-pdf ./certificates/WPE_2024_001.pdf

# Verify certificate signature
cargo run -- cert --verify ./certificates/WPE_2024_001.json
```

---

## 🌐 Verification Portal API

### Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/` | Documentation and web interface |
| `GET` | `/health` | Health check |
| `POST` | `/verify` | Validate certificate JSON |
| `GET` | `/verify/{cert_id}` | Lookup by certificate ID (future) |

### Verify Certificate

```bash
curl -X POST http://localhost:8000/verify \
  -H "Content-Type: application/json" \
  -d @certificate.json
```

**Response:**
```json
{
  "schema_valid": true,
  "signature_valid": true,
  "hash_valid": true,
  "chain_valid": true,
  "cert_summary": {
    "cert_id": "WPE_2024_001",
    "cert_type": "wipe",
    "device_model": "Samsung SSD 980 PRO",
    "device_serial": "S***7890",
    "nist_level": "PURGE",
    "method": "nvme_sanitize_crypto_erase",
    "created_at": "2024-12-31T10:30:00Z",
    "result": "PASS"
  }
}
```

---

## 📁 Project Structure

```
erase-sure/
├── 🦀 core/                      # Rust core engine
│   ├── src/
│   │   ├── main.rs              # CLI entry point
│   │   ├── cli.rs               # Command-line interface
│   │   ├── device.rs            # Device discovery (lsblk, smartctl)
│   │   ├── backup.rs            # AES-256-CTR encrypted backup
│   │   ├── wipe.rs              # NIST sanitization methods
│   │   ├── cert.rs              # JSON certificate generation
│   │   ├── cert_pdf.rs          # PDF certificate generation
│   │   ├── signer.rs            # Ed25519 signing
│   │   ├── schema.rs            # JSON Schema validation
│   │   └── logging.rs           # Structured logging
│   ├── tests/                   # Rust unit tests
│   └── Cargo.toml               # Rust dependencies
│
├── 🖥️ ui/                        # Tauri + React desktop app
│   ├── src/
│   │   ├── screens/             # UI views
│   │   │   ├── Home.tsx         # Landing page
│   │   │   ├── Discover.tsx     # Device discovery
│   │   │   ├── Backup.tsx       # Backup workflow
│   │   │   ├── DestructiveWipe.tsx  # Wipe workflow
│   │   │   └── Certificates.tsx # Certificate viewer
│   │   ├── components/          # Reusable components
│   │   │   ├── DeviceCard.tsx   # Device display card
│   │   │   ├── Progress.tsx     # Progress indicators
│   │   │   ├── QRPreview.tsx    # QR code display
│   │   │   └── WipeConfirmationModal.tsx
│   │   └── hooks/               # React hooks
│   ├── src-tauri/               # Tauri backend
│   └── package.json             # Node dependencies
│
├── 🌐 portal/                    # FastAPI verification service
│   ├── app/
│   │   └── main.py              # API endpoints
│   ├── requirements.txt         # Python dependencies
│   └── examples/                # Sample certificates
│
├── 📋 certs/                     # Certificate schemas
│   └── schemas/
│       ├── backup_schema.json   # Backup certificate schema
│       └── wipe_schema.json     # Wipe certificate schema
│
├── 🔑 keys/                      # Signing keys (git-ignored)
│   ├── dev_private.pem          # Development private key
│   └── dev_public.pem           # Development public key
│
├── 🧪 tests/                     # Integration tests
│   ├── test_backup_schema.py    # Schema validation tests
│   ├── test_wipe_schema.py
│   ├── test_pdf_certificates.py # PDF generation tests
│   ├── test_qr_codes.py         # QR code tests
│   └── scripts/                 # Shell test scripts
│
├── 📚 docs/                      # Documentation
│   ├── PRD.md                   # Product requirements
│   ├── schemas.md               # Certificate format guide
│   ├── CERTIFICATE_HANDLING.md  # Certificate implementation
│   └── KALI_DEMO_GUIDE.md       # Demo setup guide
│
└── 🐧 iso/                       # Bootable ISO configuration
    └── build.md                 # ISO build instructions
```

---

## 🏆 Compliance Standards

| Standard | Implementation | Status |
|----------|----------------|--------|
| **NIST SP 800-88 Rev.1** | PURGE/CLEAR/DESTROY levels | ✅ Full |
| **DoD 5220.22-M** | 3-pass overwrite option | ✅ Supported |
| **ISO/IEC 27040** | Information security for storage | ✅ Compliant |
| **Ed25519 (RFC 8032)** | Digital signature algorithm | ✅ Implemented |
| **JSON Schema Draft-07** | Certificate validation | ✅ Enforced |
| **AES-256 (FIPS 197)** | Backup encryption | ✅ Implemented |

---

## 🧪 Testing

### Run All Tests

```bash
# Rust unit tests
cd core && cargo test

# Python schema validation tests
cd tests && python -m pytest -v

# Integration tests
./tests/scripts/test_certificate_flows.sh
```

### Test Coverage

```bash
# Rust coverage (requires cargo-tarpaulin)
cd core && cargo tarpaulin --out Html

# Python coverage
cd tests && python -m pytest --cov=portal --cov-report=html
```

### Manual Testing

```bash
# Test device discovery (safe)
cargo run -- discover --format json

# Test certificate validation
curl -X POST http://localhost:8000/verify \
  -H "Content-Type: application/json" \
  -d @tests/samples/valid_backup_cert.json
```

---

## 🔒 Security Considerations

### Key Management

- **Private keys** are NEVER committed to the repository
- Keys are loaded at runtime from environment variables or secure paths
- Only **public keys** are bundled for verification

### Guard Rails

- **CRITICAL disks** (system/root) are blocked from wiping unless in ISO mode
- **Two-step confirmation** required for all destructive operations
- **Risk badges** clearly indicate device danger levels

### Cryptographic Choices

| Component | Algorithm | Key Size | Rationale |
|-----------|-----------|----------|-----------|
| Signatures | Ed25519 | 256-bit | Modern, fast, small signatures |
| Encryption | AES-256-CTR | 256-bit | NIST approved, streaming mode |
| Hashing | SHA-256 | 256-bit | Industry standard, collision resistant |

---

## 🤝 Contributing

### Development Setup

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install development tools
cargo install cargo-watch cargo-tarpaulin

# Setup Python environment
cd portal && pip install -r requirements.txt pytest-cov

# Install Node dependencies
cd ui && npm install
```

### Code Style

- **Rust**: `cargo fmt` and `cargo clippy`
- **Python**: `black` and `ruff`
- **TypeScript**: ESLint + Prettier

### Pull Request Checklist

- [ ] All tests pass (`cargo test`, `pytest`)
- [ ] Code formatted (`cargo fmt`, `black`)
- [ ] Schema validation updated if certificate format changed
- [ ] Documentation updated for new features
- [ ] Security review for cryptographic changes

---

## 📄 License

**MIT License** — see [LICENSE](./LICENSE)

---

## ⚠️ Disclaimer

This tool performs **irreversible data destruction**. Always:

- ✅ **Backup critical data** before wiping
- ✅ **Test on non-production devices** first
- ✅ **Verify device paths** carefully to avoid accidental wipes
- ✅ **Run from bootable media** for system drives
- ✅ **Review certificates** to confirm successful sanitization

**NIST Compliance Note**: Implements NIST SP 800-88 Rev.1 guidelines. Users are responsible for ensuring compliance with their specific regulatory requirements.

---

## 🆘 Support

### Quick Help

```bash
cargo run -- --help
cargo run -- wipe --help
cargo run -- backup --help
```

### Common Issues

| Issue | Solution |
|-------|----------|
| Permission denied | Run with `sudo` or from bootable ISO |
| Device not found | Check `lsblk` output and device permissions |
| Certificate verification failed | Ensure public key matches signing key |
| NVMe sanitize not supported | Use CLEAR policy with overwrite fallback |

### Contact

- 🐛 **Bug Reports**: [GitHub Issues](https://github.com/heyitsgautham/erase-sure/issues)
- 💡 **Feature Requests**: [GitHub Discussions](https://github.com/heyitsgautham/erase-sure/discussions)
- 📧 **Security Issues**: Contact maintainers directly

---

## 🗺️ Roadmap

### Phase 1: MVP ✅
- [x] Linux NIST sanitization (PURGE/CLEAR)
- [x] AES-256-CTR encrypted backup
- [x] JSON + PDF certificate generation
- [x] Ed25519 digital signatures
- [x] FastAPI verification portal
- [x] Tauri desktop UI

### Phase 2: Enterprise 🚧
- [ ] Windows native support
- [ ] Batch processing workflows
- [ ] LDAP/SSO integration
- [ ] Compliance reporting dashboard

### Phase 3: Scale 🔮
- [ ] Blockchain certificate anchoring
- [ ] Cloud verification service
- [ ] Mobile companion app
- [ ] Enterprise API for integrators

---

<div align="center">

**Built with ❤️ for secure e-waste recycling and data protection**

🏆 **SIH 2024 • Problem Statement 25070 • Ministry of Mines, JNARDDC**

</div>
