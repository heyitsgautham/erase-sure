# 🔒 SecureWipe - NIST-Compliant Data Sanitization Platform

> **Transforming e-waste management through verifiable, tamper-proof data wiping**

[![NIST SP 800-88](https://img.shields.io/badge/NIST-SP%20800--88%20Rev.1-blue)](https://csrc.nist.gov/publications/detail/sp/800-88/rev-1/final)
[![Ed25519](https://img.shields.io/badge/Signatures-Ed25519-green)](https://ed25519.cr.yp.to/)
[![SIH 2024](https://img.shields.io/badge/SIH-Problem%2025070-orange)](https://www.sih.gov.in/)

## 🚀 Mission

**India generates ~1.75M tonnes of e-waste annually**, with **₹50,000+ crore worth of IT assets hoarded** due to fear of data breaches during recycling. SecureWipe bridges this trust gap with **NIST-aligned sanitization** and **cryptographically-signed certificates**, enabling confident IT asset recycling.

### 🎯 What We Solve
- **Trust Crisis**: Organizations hoard old devices instead of recycling
python run.py
# Server runs at: http://localhost:8000
```

### 4️⃣ Build Desktop UI (Tauri + React)
python run.py
# Server runs at: http://localhost:8000
```

### 4️⃣ Build Desktop UI (Tauri + React)
- **Compliance Gaps**: Lack of verifiable proof of secure data destruction  
- **Technical Complexity**: Existing tools are expensive or hard to use
- **Chain of Custody**: No auditable trail from wipe to recycling

## ⭐ Key Features

### 🛡️ **NIST SP 800-88 Rev.1 Compliant**
python run.py
# Server runs at: http://localhost:8000
```

### 4️⃣ Build Desktop UI (Tauri + React)
- **PURGE** level sanitization (cryptographic erase, block erase)
- **CLEAR** level fallback (secure overwrite + verification)
- **HPA/DCO clearing** for hidden disk areas>
- **NVMe sanitize** and **ATA secure erase** support

### 📜 **Tamper-Proof Certificates**
- **JSON certificates** with complete audit trail
- **Styled PDF reports** with logo, QR codes, and verification URLs
- **Ed25519 digital signatures** for cryptographic integrity
- **Blockchain-ready** format for future anchoring

### 🔄 **End-to-End Workflow**
```
📂 Backup → 🔥 Sanitize → 📋 Certificate → ✅ Verify
```

### 🌐 **Multi-Platform Support**
- **Linux**: Full hardware control (NVMe, SATA, HPA/DCO)
- **Android**: ADB/Recovery technician workflows  
- **Windows**: Simulated flows with real device discovery
- **Bootable ISO**: Hardware-level access without OS interference

---

## 🏗️ Architecture

```
SecureWipe Platform
├── 🦀 Core Engine (Rust)           # Device control, NIST algorithms, certificates
├── 🖥️  Desktop UI (Tauri + React)   # Cross-platform user interface
├── 🌐 Verification Portal (FastAPI) # Certificate validation service
├── 📱 Mobile Integration (ADB)      # Android device sanitization
└── 🔐 Certificate System (Ed25519)  # Cryptographic proof generation
```

### **Technology Stack**
- **Backend**: Rust (performance, memory safety, system access)
- **Frontend**: Tauri + React TypeScript (lightweight, secure)
- **Verification**: Python FastAPI (web standards, JSON Schema)
- **Certificates**: Ed25519 + JSON Schema + ReportLab PDFs
- **Standards**: NIST SP 800-88 Rev.1, ISO 27040, DoD 5220.22-M

---

## 🚀 Quick Start

### Prerequisites
```bash
# Ensure you have the required tools
rust --version     # Rust 1.70+
node --version     # Node.js 18+
python --version   # Python 3.11+
```

### 1️⃣ Clone and Setup
```bash
git clone https://github.com/heyitsgautham/erase-sure.git
cd erase-sure
```

### 2️⃣ Build Core Engine (Rust)
```bash
cd core
cargo build --release
cargo test

# Test device discovery (read-only)
cargo run -- discover
```

### 3️⃣ Setup Verification Portal (Python)
```bash
cd portal
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate
python run.py
# Server runs at: http://localhost:8000
```

### 4️⃣ Build Desktop UI (Tauri + React)
pip install -r requirements.txt

# Start verification server
python run.py
# Server runs at: http://localhost:8000
```

### 4️⃣ Build Desktop UI (Tauri + React)
```bash
cd ui
npm install
npm run dev        # Development mode
npm run build      # Production build
```

---

## 💻 Usage Commands

### 🔍 **Device Discovery** (Safe - Read Only)
```bash
# List all storage devices with risk assessment
cd core
cargo run -- discover

# Get detailed device information
cargo run -- discover --verbose

# Export device list as JSON
cargo run -- discover --format json > devices.json
```

### 💾 **Secure Backup** (Before Wiping)
```bash
# Backup personal files with AES-256 encryption
python run.py
# Server runs at: http://localhost:8000
```

### 4️⃣ Build Desktop UI (Tauri + React)
cargo run -- backup \
  --device /dev/sda \
  --destination /media/backup-usb \
  --paths ~/Documents ~/Pictures ~/Desktop

# Backup with custom encryption key
cargo run -- backup \
  --device /dev/sda \
- **Compliance Gaps**: Lack of verifiable proof of secure data destruction  
- **Technical Complexity**: Existing tools are expensive or hard to use
- **Chain of Custody**: No auditable trail from wipe to recycling

## ⭐ Key Features

### 🛡️ **NIST SP 800-88 Rev.1 Compliant**
- **PURGE** level sanitization (cryptographic erase, block erase)
- **CLEAR** level fallback (secure overwrite + verification)
- **HPA/DCO clearing** for hidden disk areas>
- **NVMe sanitize** and **ATA secure erase** support

### 📜 **Tamper-Proof Certificates**
- **JSON certificates** with complete audit trail
- **Styled PDF reports** with logo, QR codes, and verification URLs
- **Ed25519 digital signatures** for cryptographic integrity
- **Blockchain-ready** format for future anchoring

### 🔄 **End-to-End Workflow**
```
📂 Backup → 🔥 Sanitize → 📋 Certificate → ✅ Verify
```

### 🌐 **Multi-Platform Support**
- **Linux**: Full hardware control (NVMe, SATA, HPA/DCO)
- **Android**: ADB/Recovery technician workflows  
- **Windows**: Simulated flows with real device discovery
- **Bootable ISO**: Hardware-level access without OS interference

---
python run.py
# Server runs at: http://localhost:8000
```

### 4️⃣ Build Desktop UI (Tauri + React)

## 🏗️ Architecture

```
SecureWipe Platform
├── 🦀 Core Engine (Rust)           # Device control, NIST algorithms, certificates
├── 🖥️  Desktop UI (Tauri + React)   # Cross-platform user interface
├── 🌐 Verification Portal (FastAPI) # Certificate validation service
├── 📱 Mobile Integration (ADB)      # Android device sanitization
└── 🔐 Certificate System (Ed25519)  # Cryptographic proof generation
```

### **Technology Stack**
- **Backend**: Rust (performance, memory safety, system access)
- **Frontend**: Tauri + React TypeScript (lightweight, secure)
- **Verification**: Python FastAPI (web standards, JSON Schema)
- **Certificates**: Ed25519 + JSON Schema + ReportLab PDFs
- **Standards**: NIST SP 800-88 Rev.1, ISO 27040, DoD 5220.22-M

---

## 🚀 Quick Start

### Prerequisites
```bash
# Ensure you have the required tools
rust --version     # Rust 1.70+
node --version     # Node.js 18+
python --version   # Python 3.11+
```

python run.py
# Server runs at: http://localhost:8000
```

### 4️⃣ Build Desktop UI (Tauri + React)
### 1️⃣ Clone and Setup
```bash
git clone https://github.com/heyitsgautham/erase-sure.git
cd erase-sure
```

### 2️⃣ Build Core Engine (Rust)
```bash
cd core
cargo build --release
cargo test

# Test device discovery (read-only)
cargo run -- discover
```

### 3️⃣ Setup Verification Portal (Python)
```bash
cd portal
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate
pip install -r requirements.txt

# Start verification server
python run.py
# Server runs at: http://localhost:8000
```

### 4️⃣ Build Desktop UI (Tauri + React)
```bash
cd ui
npm install
npm run dev        # Development mode
npm run build      # Production build
```

---

## 💻 Usage Commands

### 🔍 **Device Discovery** (Safe - Read Only)
```bash
# List all storage devices with risk assessment
cd core
cargo run -- discover

# Get detailed device information
cargo run -- discover --verbose

# Export device list as JSON
cargo run -- discover --format json > devices.json
```

### 💾 **Secure Backup** (Before Wiping)
```bash
# Backup personal files with AES-256 encryption
cargo run -- backup \
  --device /dev/sda \
  --destination /media/backup-usb \
  --paths ~/Documents ~/Pictures ~/Desktop

# Backup with custom encryption key
cargo run -- backup \
  --device /dev/sda \
  --destination /media/backup \
  --key-file ./backup.key \
  --verify-samples 10
```

### 🔥 **Secure Wipe** (Destructive - Use Carefully!)
```bash
# PURGE level wipe (recommended for SSDs)
cargo run -- wipe \
  --device /dev/sda \
  --policy PURGE \
  --method nvme_sanitize_crypto_erase

# CLEAR level wipe (for HDDs or unsupported controllers)  
cargo run -- wipe \
  --device /dev/sda \
  --policy CLEAR \
  --method overwrite_verify

# Link to previous backup certificate
cargo run -- wipe \
  --device /dev/sda \
  --policy PURGE \
  --backup-cert-id BCK_2024_001
```

### 📋 **Certificate Management**
```bash
# Generate signed PDF certificate
cargo run -- cert \
  --json-path ./certificates/WPE_2024_001.json \
  --output-pdf ./certificates/WPE_2024_001.pdf \
  --sign

# Verify certificate integrity  
cargo run -- cert \
  --verify ./certificates/WPE_2024_001.json

# Export for portal verification
cargo run -- cert \
  --export ./certificates/WPE_2024_001.json
```

---

## 🧪 Testing & Validation

### **Schema Validation Tests**
```bash
cd tests

# Test backup certificate schema
python test_backup_schema.py

# Test wipe certificate schema  
python test_wipe_schema.py

# Validate all sample certificates
python -m pytest test_*.py -v
```

### **PDF Certificate Generation**
```bash
# Generate test backup certificate PDF
python test_pdf_certificates.py

# Generate test wipe certificate PDF
python test_wipe_pdf_certificates.py

# Test QR code generation and scanning
python test_qr_codes.py
```

### **Integration Testing**
```bash
cd tests/scripts

# Full backup → wipe → verify workflow
./test_backup_integration.sh

# Certificate generation pipeline
./test_certificates.sh

# PDF generation with all styling
./test_pdf_generation.sh
```

### **Portal API Testing**
```bash
cd portal

# Run unit tests
python -m pytest test_main.py -v

# Test certificate verification endpoint
curl -X POST http://localhost:8000/verify \
  -H "Content-Type: application/json" \
  -d @examples/valid_backup_cert.json

# Health check
curl http://localhost:8000/health
```

---

## 🔒 Security & Compliance

### **Certificate Validation**
```bash
# Verify certificate with portal
curl -X POST http://localhost:8000/verify \
  -H "Content-Type: application/json" \
  -d @path/to/certificate.json

# Manual signature verification
cd core
cargo run -- verify-signature \
  --cert ./certificates/WPE_2024_001.json \
  --pubkey ./keys/sih_root_v1.pem
```

### **Key Management** 
```bash
# Generate new signing keys (production)
cargo run -- generate-keys \
  --output ./keys/ \
  --key-id "production_v1"

# Rotate keys (advanced)
cargo run -- rotate-keys \
  --old-key ./keys/old.pem \
  --new-key ./keys/new.pem
```

---

## 📊 Real-World Examples

### **Enterprise Laptop Retirement**
```bash
# 1. Discover devices
cargo run -- discover --filter laptops

# 2. Backup critical data  
cargo run -- backup --device /dev/nvme0n1 --destination /mnt/backup

# 3. NIST PURGE level sanitization
cargo run -- wipe --device /dev/nvme0n1 --policy PURGE

# 4. Generate compliance certificate
cargo run -- cert --device /dev/nvme0n1 --pdf --qr
```

### **Android Device Sanitization**
```bash
# Enable ADB debugging, connect device
adb devices

# Factory reset + secure wipe  
cargo run -- android-wipe --device-id 1234567890ABCDEF

# Generate mobile certificate
cargo run -- cert --android --device-id 1234567890ABCDEF
```

### **Bulk Processing** 
```bash
# Process multiple devices from CSV
cargo run -- batch-wipe --input devices.csv --policy PURGE

# Generate compliance report
cargo run -- compliance-report --batch-id BATCH_2024_Q1
```

---

## 🌐 Verification Portal

### **Start Portal**
```bash
cd portal
source venv/bin/activate
python run.py --host 0.0.0.0 --port 8000
```

### **Portal Endpoints**
- `GET /` - Documentation and upload interface
- `POST /verify` - Certificate validation API  
- `GET /health` - System health check
- `GET /verify/{cert_id}` - Certificate lookup (future)

### **API Usage**
```bash
# Verify backup certificate
curl -X POST http://localhost:8000/verify \
  -H "Content-Type: application/json" \
  -d @certificate.json

# Expected response:
{
  "schema_valid": true,
  "signature_valid": true,  
  "hash_valid": true,
  "cert_summary": {
    "cert_id": "BCK_2024_001",
    "device_model": "Samsung SSD 980 PRO", 
    "result": "PASS"
  }
}
```

---

## 📁 Project Structure

```
erase-sure/
├── 🦀 core/                    # Rust engine - device control, NIST algorithms
│   ├── src/
│   │   ├── main.rs            # CLI entry point
│   │   ├── device.rs          # Hardware discovery (lsblk, smartctl)  
│   │   ├── backup.rs          # AES-256-CTR encrypted backup
│   │   ├── wipe.rs            # NIST sanitization methods
│   │   ├── cert.rs            # JSON certificate generation
│   │   └── pdf.rs             # Styled PDF with QR codes
│   └── tests/                 # Rust unit & integration tests
│
├── 🌐 portal/                  # FastAPI verification service
│   ├── app/main.py            # Certificate validation endpoints
│   ├── requirements.txt       # Python dependencies
│   └── examples/              # Sample certificates for testing
│
├── 🖥️ ui/                      # Tauri + React desktop application  
│   ├── src/main.tsx           # UI components and state management
│   └── tauri.conf.json        # Desktop app configuration
│
├── 📋 tests/                   # Comprehensive testing suite
│   ├── scripts/               # Integration test scripts
│   ├── samples/               # Reference certificate data
│   └── outputs/               # Generated test artifacts
│
├── 🔐 certs/                   # Certificate schemas & examples
│   └── schemas/               # JSON Schema definitions
│
└── 📚 docs/                    # Documentation & specifications
    ├── PRD.md                 # Product requirements  
    └── schemas.md             # Certificate format guide
```

---

## 🏆 Compliance Standards

| Standard | Level | Implementation |
|----------|-------|---------------|
| **NIST SP 800-88 Rev.1** | PURGE/CLEAR | ✅ Full implementation |
| **DoD 5220.22-M** | 3-pass overwrite | ✅ Legacy support |
| **ISO/IEC 27040** | Information security | ✅ Certificate format |
| **Ed25519** | Digital signatures | ✅ Tamper-proof certs |
| **JSON Schema Draft-07** | Data validation | ✅ Audit-ready format |

---

## 🤝 Contributing

### **Development Setup**
```bash
# Install development dependencies
cd core && cargo install cargo-watch cargo-tarpaulin
cd portal && pip install -r requirements.txt pytest-cov  
cd tests && pip install -r requirements-test.txt

# Run development watchers
cargo watch -x "test"           # Auto-test Rust changes
python -m pytest --cov         # Python test coverage
```

### **Pull Request Checklist**
- [ ] All tests pass (`cargo test`, `pytest`)
- [ ] Schema validation updated if certificate format changed
- [ ] Documentation updated for new features
- [ ] Security review for cryptographic changes
- [ ] Cross-platform testing (Linux/Windows/macOS)

---

## 📄 License & Legal

**License**: MIT License - see [LICENSE](./LICENSE)

**Disclaimer**: This tool performs **irreversible data destruction**. Always:
- ✅ **Backup critical data** before wiping
- ✅ **Test on non-production devices** first  
- ✅ **Verify device paths** to avoid accidental wipes
- ✅ **Run from bootable media** for system drives

**NIST Compliance**: Implements NIST SP 800-88 Rev.1 guidelines. Users are responsible for ensuring compliance with their specific regulatory requirements.

---

## 🆘 Support

### **Quick Help**
```bash
# Get command help
cargo run -- --help
cargo run -- wipe --help

# Check system requirements
cargo run -- system-check

# Generate diagnostic report
cargo run -- diagnose > system-report.txt
```

### **Common Issues**
- **Permission denied**: Run with `sudo` or from bootable ISO
- **Device not found**: Check `lsblk` and device permissions
- **Certificate verification failed**: Verify signature and schema compliance

### **Contact & Issues**
- 🐛 **Bug Reports**: [GitHub Issues](https://github.com/heyitsgautham/erase-sure/issues)
- 💡 **Feature Requests**: [GitHub Discussions](https://github.com/heyitsgautham/erase-sure/discussions)  
- 📧 **Security Issues**: Contact maintainers directly

---

## 🎯 Roadmap

### **Phase 1: MVP** ✅
- [x] Linux NIST sanitization
- [x] Certificate generation (JSON + PDF)
- [x] Verification portal
- [x] Basic UI framework

### **Phase 2: Enterprise** 🚧  
- [ ] Windows native support
- [ ] Batch processing workflows
- [ ] LDAP/SSO integration
- [ ] Compliance reporting dashboard

### **Phase 3: Cloud & Scale** 🔮
- [ ] Blockchain certificate anchoring  
- [ ] Cloud verification service
- [ ] Mobile app companion
- [ ] API for system integrators

---

*Built with ❤️ for secure e-waste recycling and data protection*

**🏆 SIH 2024 Problem Statement 25070 - Ministry of Mines, JNARDDC**
