#!/bin/bash

# Kali Linux SecureWipe Demo Setup Script
# Run this on Kali to prepare the ultimate demo environment

echo "🔐 Setting up Kali Linux for SecureWipe Enterprise Demo"
echo "=================================================="

# Update Kali and install additional tools
echo "📦 Installing additional forensic tools..."
sudo apt update
sudo apt install -y \
    nvme-cli \
    hdparm \
    smartmontools \
    testdisk \
    foremost \
    scalpel \
    sleuthkit \
    autopsy \
    dc3dd \
    dcfldd \
    qrencode \
    zbar-tools

# Create demo workspace
echo "📁 Creating demo workspace..."
mkdir -p ~/SecureWipe-Demo/{target-devices,evidence,certificates,reports}

# Download and setup SecureWipe
echo "⬇️  Setting up SecureWipe..."
cd ~/SecureWipe-Demo
# git clone your repository here when ready

# Create test datasets
echo "🧪 Creating forensic test datasets..."

# Dataset 1: Financial records
mkdir -p ~/SecureWipe-Demo/test-data/financial
cat > ~/SecureWipe-Demo/test-data/financial/accounts.csv << 'EOF'
Account,Balance,SSN,Credit_Card
John Doe,50000,123-45-6789,4532-1234-5678-9012
Jane Smith,75000,987-65-4321,5555-4444-3333-2222
Bob Johnson,125000,456-78-9123,4111-1111-1111-1111
EOF

# Dataset 2: Medical records
mkdir -p ~/SecureWipe-Demo/test-data/medical
cat > ~/SecureWipe-Demo/test-data/medical/patient_records.txt << 'EOF'
CONFIDENTIAL MEDICAL RECORDS
Patient: John Doe
DOB: 1980-01-15
SSN: 123-45-6789
Diagnosis: Hypertension
Medications: Lisinopril 10mg
Insurance: Blue Cross Blue Shield
Policy: BCBS-123456789
EOF

# Dataset 3: Source code (trade secrets)
mkdir -p ~/SecureWipe-Demo/test-data/source-code
cat > ~/SecureWipe-Demo/test-data/source-code/encryption_key.h << 'EOF'
// PROPRIETARY ENCRYPTION ALGORITHM
// Company: SecureCorpTech
// Classification: TOP SECRET
#define SECRET_KEY "AES256_MASTER_KEY_DO_NOT_DISTRIBUTE"
#define BACKDOOR_CODE "DEBUG_ACCESS_2024"
static const char* api_keys[] = {
    "sk_live_51234567890abcdef",
    "pk_test_98765432109876543"
};
EOF

echo "🎯 Creating demonstration scenarios..."

# Scenario 1: Data breach simulation
cat > ~/SecureWipe-Demo/demo_scenarios.md << 'EOF'
# SecureWipe Demo Scenarios

## Scenario 1: Data Breach Prevention
**Story**: Employee laptop being decommissioned
**Data**: Financial records, medical data, source code
**Threat**: Standard deletion leaves recoverable data
**Demo**: Show data recovery before SecureWipe, complete erasure after

## Scenario 2: Compliance Audit
**Story**: Healthcare organization disposing of storage
**Requirement**: HIPAA-compliant data destruction
**Demo**: Generate certificates, verify compliance, show audit trail

## Scenario 3: Corporate Espionage Protection
**Story**: Competitor attempting data recovery
**Threat**: HPA/DCO hidden areas contain trade secrets
**Demo**: Detect hidden data, show complete clearing with SecureWipe

## Scenario 4: Government/Military Standard
**Story**: Classified data destruction requirement
**Standard**: NIST SP 800-88 Rev. 1 PURGE level
**Demo**: Multi-pass overwrite, cryptographic verification
EOF

# Create data recovery demonstration script
cat > ~/SecureWipe-Demo/demo_data_recovery.sh << 'EOF'
#!/bin/bash

echo "🕵️  DEMO: Data Recovery from 'Deleted' Files"
echo "============================================="

# Create test USB with sensitive data
echo "Creating test USB with sensitive data..."
TARGET_DEVICE="$1"

if [[ -z "$TARGET_DEVICE" ]]; then
    echo "Usage: $0 /dev/sdX"
    exit 1
fi

# Copy sensitive test data
sudo mount ${TARGET_DEVICE}1 /mnt
sudo cp -r ~/SecureWipe-Demo/test-data/* /mnt/
sudo umount /mnt

echo "✅ Test data written to USB"

# Standard deletion
echo "🗑️  Performing 'standard' deletion..."
sudo mount ${TARGET_DEVICE}1 /mnt
sudo rm -rf /mnt/*
sudo umount /mnt

echo "📁 Files 'deleted' - device appears empty"

# Attempt data recovery
echo "🔍 Attempting data recovery with forensic tools..."
sudo foremost -i $TARGET_DEVICE -o ~/SecureWipe-Demo/evidence/recovered/

echo "📊 Recovery Results:"
find ~/SecureWipe-Demo/evidence/recovered/ -type f -name "*.txt" -o -name "*.csv" | head -10

echo ""
echo "⚠️  CRITICAL: Sensitive data recoverable after 'deletion'!"
echo "💡 Solution: Use SecureWipe for guaranteed destruction"
EOF

chmod +x ~/SecureWipe-Demo/demo_data_recovery.sh

echo ""
echo "🎉 Kali SecureWipe Demo Environment Ready!"
echo ""
echo "📋 Demo Workflow:"
echo "1. Boot target machine with this Kali USB"
echo "2. Connect test USB drive"
echo "3. Run data recovery demo: ./demo_data_recovery.sh /dev/sdX"
echo "4. Show recovered sensitive data"
echo "5. Run SecureWipe with DESTROY policy"
echo "6. Attempt recovery again - show nothing found"
echo "7. Generate compliance certificates"
echo ""
echo "🔧 Tools Available:"
echo "- foremost, scalpel (data recovery)"
echo "- hdparm, nvme-cli (HPA/DCO testing)"
echo "- autopsy (forensic analysis)"
echo "- SecureWipe (secure destruction)"
echo ""
echo "💼 Perfect for enterprise demos!"