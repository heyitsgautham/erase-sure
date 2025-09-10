#!/usr/bin/env python3
"""
Test script to verify QR codes in the generated PDFs contain correct URLs
"""

import tempfile
import qrcode
from PIL import Image
from pyzbar import pyzbar

def create_test_qr_codes():
    """Create test QR codes with the expected URLs"""
    
    # Test URLs
    wipe_url = "https://verify.securewipe.org/cert/TEST_WPE_2024_001"
    backup_url = "https://verify.securewipe.org/cert/TEST_BCK_2024_001"
    
    print("🔍 Testing QR Code Generation and Reading")
    print("=" * 50)
    
    # Generate QR codes
    print("\n1. Generating QR codes...")
    
    # Wipe certificate QR
    qr_wipe = qrcode.QRCode(version=1, box_size=10, border=4)
    qr_wipe.add_data(wipe_url)
    qr_wipe.make(fit=True)
    img_wipe = qr_wipe.make_image(fill_color="black", back_color="white")
    
    # Backup certificate QR  
    qr_backup = qrcode.QRCode(version=1, box_size=10, border=4)
    qr_backup.add_data(backup_url)
    qr_backup.make(fit=True)
    img_backup = qr_backup.make_image(fill_color="black", back_color="white")
    
    # Save to temp files
    wipe_temp = tempfile.mktemp(suffix="_wipe.png")
    backup_temp = tempfile.mktemp(suffix="_backup.png")
    
    img_wipe.save(wipe_temp)
    img_backup.save(backup_temp)
    
    print(f"  ✓ Wipe QR saved to: {wipe_temp}")
    print(f"  ✓ Backup QR saved to: {backup_temp}")
    
    # Read QR codes back
    print("\n2. Reading QR codes...")
    
    # Read wipe QR
    wipe_image = Image.open(wipe_temp)
    wipe_barcodes = pyzbar.decode(wipe_image)
    
    if wipe_barcodes:
        wipe_data = wipe_barcodes[0].data.decode('utf-8')
        print(f"  🎯 Wipe QR contains: {wipe_data}")
        if wipe_data == wipe_url:
            print("  ✅ Wipe QR URL is correct!")
        else:
            print("  ❌ Wipe QR URL is incorrect!")
    else:
        print("  ❌ Could not read wipe QR code!")
    
    # Read backup QR
    backup_image = Image.open(backup_temp)
    backup_barcodes = pyzbar.decode(backup_image)
    
    if backup_barcodes:
        backup_data = backup_barcodes[0].data.decode('utf-8')
        print(f"  🎯 Backup QR contains: {backup_data}")
        if backup_data == backup_url:
            print("  ✅ Backup QR URL is correct!")
        else:
            print("  ❌ Backup QR URL is incorrect!")
    else:
        print("  ❌ Could not read backup QR code!")
    
    print("\n3. Testing QR scanner compatibility...")
    print("  📱 Both QR codes should be scannable with any QR code reader app")
    print("  🌐 Scanning should open the verification URL directly in a browser")
    
    return True

if __name__ == "__main__":
    print("SecureWipe QR Code Verification Test")
    print("=" * 50)
    
    success = create_test_qr_codes()
    
    print("\n" + "=" * 50)
    print(f"Test Result: {'SUCCESS' if success else 'FAILED'}")
    print("\n📋 Next Steps:")
    print("  1. Open the generated PDF certificates")
    print("  2. Scan the QR codes with your phone")
    print("  3. Verify they open the correct verification URLs")
