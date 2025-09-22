# Kali Linux SecureWipe Demo Guide

## 🎯 **Why Kali is Perfect for SecureWipe Demos**

### **Professional Context**
- **Penetration testers** already trust Kali
- **Security professionals** recognize the platform
- **IT teams** understand the security implications
- **Executives** see serious security tooling

### **Technical Advantages**
- **Hardware access**: Direct device control without OS interference
- **Forensic tools**: Built-in data recovery tools for before/after demos
- **Network isolation**: Secure environment for certificate verification
- **Portable**: Same demo environment anywhere

---

## 🚀 **Ultimate Kali Demo Workflow**

### **Phase 1: The Problem (5 minutes)**
```bash
# Boot Kali from USB
# Connect "target" USB drive with sensitive data

# Show the threat
./demo_data_recovery.sh /dev/sdb
# Results: Recover "deleted" financial records, medical data, source code
```

**Talking Points:**
- "This laptop was 'wiped' by IT using standard tools"
- "In 2 minutes, I recovered social security numbers, medical records, and trade secrets"
- "This data could be sold on dark web for $50-200 per record"

### **Phase 2: The Solution (10 minutes)**
```bash
# Launch SecureWipe
sudo SECUREWIPE_DANGER=1 ./securewipe wipe \
    --device /dev/sdb \
    --policy DESTROY \
    --danger-allow-wipe \
    --sign

# Show real-time progress
# Generate certificates
# Verify with QR code
```

**Talking Points:**
- "SecureWipe uses military-grade NIST SP 800-88 standards"
- "Multi-pass overwrite + HPA/DCO clearing"
- "Cryptographic proof of complete destruction"

### **Phase 3: The Proof (5 minutes)**
```bash
# Attempt recovery again
foremost -i /dev/sdb -o ./post-wipe-recovery/
scalpel /dev/sdb -o ./scalpel-recovery/

# Show empty results
ls -la ./post-wipe-recovery/
ls -la ./scalpel-recovery/

# Verify certificate
qrencode -t ANSI < certificate.json
```

**Results:** No recoverable data, cryptographic certificate proves compliance

---

## 🎭 **Demo Scenarios by Audience**

### **Healthcare (HIPAA Demo)**
```bash
# Setup: Medical records on USB
echo "Patient: John Doe, SSN: 123-45-6789, Diagnosis: Cancer" > patient.txt

# Standard deletion
rm patient.txt

# Recovery
foremost -t txt /dev/sdb  # Recovers patient data

# SecureWipe
securewipe wipe --device /dev/sdb --policy PURGE --sign

# Result: HIPAA-compliant certificate, no recoverable PHI
```

### **Financial (SOX/PCI Demo)**
```bash
# Setup: Credit card data, financial records
echo "4532-1234-5678-9012,John Doe,12/25,123" > cards.csv

# Show PCI DSS requirement for secure disposal
# Demonstrate certificate compliance
```

### **Government (NIST Demo)**
```bash
# Setup: Classified documents simulation
echo "TOP SECRET: Nuclear launch codes" > classified.txt

# Show NIST SP 800-88 Rev. 1 compliance
# Multi-pass overwrite with verification
# Chain of custody documentation
```

---

## 🛠️ **Advanced Kali Techniques**

### **1. HPA/DCO Detection**
```bash
# Use Kali's built-in tools
hdparm -N /dev/sdb          # Check HPA
hdparm --dco-identify /dev/sdb  # Check DCO
nvme id-ctrl /dev/nvme0n1   # NVMe capabilities

# Show hidden areas that standard tools miss
dd if=/dev/sdb bs=512 skip=1000000 count=1 | hexdump -C
```

### **2. Forensic Analysis**
```bash
# Mount device in read-only mode
mount -o ro /dev/sdb1 /mnt/evidence

# Create forensic image
dc3dd if=/dev/sdb of=evidence.dd hash=sha256

# Analyze with Autopsy
autopsy &  # GUI forensic analysis
```

### **3. Network Verification**
```bash
# Setup local verification server
python3 -m http.server 8080 &

# Generate QR code for certificate
qrencode -o cert_qr.png < wipe_certificate.json

# Verify via mobile device or second machine
```

---

## 📱 **Mobile Integration Demo**

### **QR Code Verification**
```bash
# Generate verification QR
echo "https://verify.securewipe.com/cert/$(cat cert_id)" | qrencode -t ANSI

# Show mobile verification
# Scan with phone, verify signature, show green checkmarks
```

---

## 🎥 **Recording Professional Demos**

### **Screen Recording Setup**
```bash
# Install recording tools
sudo apt install obs-studio simplescreenrecorder

# Professional demo recording
# 1. Split-screen: Terminal + GUI
# 2. Picture-in-picture: Presenter
# 3. Clear audio narration
```

### **Demo Video Structure**
1. **Hook** (0-30s): "Your deleted data isn't really gone..."
2. **Problem** (30s-2m): Show data recovery from "wiped" drive
3. **Solution** (2m-8m): SecureWipe complete demonstration
4. **Proof** (8m-10m): Certificate verification, failed recovery
5. **CTA** (10m+): Contact for enterprise trial

---

## 💼 **Enterprise Presentation Kit**

### **Materials to Bring**
- ✅ Kali USB (bootable demo environment)
- ✅ Test USB drives (various sizes)
- ✅ Laptop for demos
- ✅ Mobile device for QR verification
- ✅ Backup slides (in case of technical issues)
- ✅ Compliance documentation
- ✅ Sample certificates

### **Presentation Flow**
1. **Business case** (5 min): Data breach costs, regulatory fines
2. **Technical demo** (15 min): Live Kali demonstration
3. **Compliance proof** (5 min): Certificates, audit trails
4. **ROI discussion** (10 min): Cost savings, risk reduction
5. **Q&A and next steps** (10 min)

---

## ⚡ **Quick Demo Commands**

```bash
# One-liner data recovery demo
echo "CONFIDENTIAL DATA" > test.txt && rm test.txt && foremost -i /dev/sdb

# One-liner secure wipe demo
sudo SECUREWIPE_DANGER=1 ./securewipe wipe --device /dev/sdb --policy DESTROY --danger-allow-wipe

# One-liner verification
qrencode -t ANSI < wipe_certificate.json && echo "Scan to verify"
```

This Kali approach gives you **maximum credibility** and **technical depth** for enterprise demos!