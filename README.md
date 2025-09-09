# EraseSure

EraseSure is a secure data wiping and certificate generation tool, built with Electron and Node.js.  
It follows NIST SP 800-88 guidelines to ensure compliant, verifiable media sanitization.  

## Project Structure

```python
secure-wipe-mvp/
├── package.json                      # Dependencies and scripts
├── main.js                           # Electron main process
├── src/
│   ├── core/                         # Shared business logic
│   │   ├── PolicyEngine.js           # NIST SP 800-88 implementation
│   │   ├── MediaDiscovery.js         # Device detection and classification
│   │   ├── WipeOrchestrator.js       # Wipe execution coordinator
│   │   ├── CertificateGenerator.js   # JSON + PDF certificate creation
│   │   └── EventBus.js               # Inter-module communication
│   ├── backends/
│   │   ├── LinuxBackend.js           # Linux-specific wipe implementations
│   │   └── ShellExecutor.js          # Safe shell command execution
│   ├── ui/
│   │   ├── index.html                # Main application UI
│   │   ├── renderer.js               # UI logic (Electron renderer)
│   │   └── styles.css                # Application styling
│   └── verification/
│       ├── server.js                 # Certificate verification web server
│       └── validator.js              # Certificate validation logic
├── certificates/                     # Generated certificates storage
├── logs/                             # Wipe execution logs
└── keys/                             # Private/public keys for signing
    ├── private.pem
    └── public.pem
```

# 🔐 Technical Architecture & Core Stack

## 1. Storage Media Basics
- **HDD (Hard Disk Drive)**: Magnetic platters + read/write heads. Supports ATA/SATA commands like `SECURE ERASE`.
- **SSD (Solid State Drive)**: NAND flash with a controller. Supports NVMe sanitize or ATA commands.
- **NVMe (Non-Volatile Memory Express)**: Fast SSD protocol; supports `block_erase`, `overwrite`, `crypto_erase`.
- **eMMC / UFS**: Embedded flash for mobile devices.

## 2. Wipe Methods (Standards & Commands)
- **DoD 5220.22-M**: Old US DoD overwrite method (multi-pass).
- **NIST SP 800-88 Rev.1**: Current sanitization standard.
  - **Clear**: Logical overwrite/reset.
  - **Purge**: Cryptographic erase, block erase, secure erase.
  - **Destroy**: Physical destruction (shred/melt/degauss).
- **ATA Secure Erase / Enhanced Secure Erase**: SATA commands.
- **NVMe Sanitize**: Commands: `block_erase`, `overwrite`, `crypto_erase`.

## 3. Device Interfaces & Tools
- **SATA / PATA**: HDD/SSD interfaces.
- **PCIe**: NVMe SSD connection.
- **lsblk**: List block devices (Linux).
- **smartctl (smartmontools)**: Drive health/info/firmware.
- **hdparm**: ATA secure erase.
- **nvme-cli**: NVMe sanitize/management.

## 4. Operating System & Execution Environment
- **Live Boot / Live USB**: Run OS from USB, safe testing.
- **Virtual Machine (VM)**: Simulated hardware; firmware commands often limited.
- **Bare Metal**: Running directly on physical hardware.

## 5. Programming & Architecture
- **Rust (Core Engine)**: Secure wipe logic, system-level access.
- **Tauri + React (UI)**: Lightweight cross-platform desktop UI.
- **Cross-Platform Binaries**: Linux, Windows, macOS (maybe Android).
- **System Calls**: OS-level commands for wiping.

## 6. Certification & Compliance
- **Certificate of Erasure**: Proof of secure wipe.
- **JSON Schema**: Defines certificate data structure.
- **Digital Signature (X.509 / RSA / ECDSA)**: Cryptographic verification.

## 7. Android / Embedded Cooperation
- **ADB (Android Debug Bridge)**: Interface for Android.
- **Fastboot**: Low-level flashing/wiping.
- **eMMC Secure Erase**: Mobile-specific erase.
- **Cross-compilation**: Build Rust for ARM/embedded.

## 8. Security Concepts
- **Firmware vs Software Erase**: Firmware is stronger.
- **Cryptographic Erase**: Delete encryption keys → instant wipe.
- **Forensics Resistance**: Prevent lab-level recovery.
- **Chain of Custody**: Track devices from collection to recycling.

## 9. DevOps / Build
- **Cargo**: Rust package manager & build tool.
- **Brew (Homebrew)**: macOS package manager.
- **Docker**: Containerized build/testing.
- **Cross / Musl**: Tools for cross-platform Rust compilation.
