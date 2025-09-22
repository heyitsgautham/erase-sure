#!/bin/bash

# Quick Demo: Test SecureWipe UI with Safe Loop Device
echo "🎯 Creating safe test target for UI demo..."

# Create a small test disk
sudo fallocate -l 50M /tmp/ui_test_disk.img
LOOP_DEV=$(sudo losetup -fP /tmp/ui_test_disk.img && losetup -l | grep ui_test_disk | awk '{print $1}')

echo "✅ Created test device: $LOOP_DEV"
echo "📝 Writing test data..."

# Write some test data
echo "This is test data that should be wiped" | sudo dd of=$LOOP_DEV bs=1 seek=1024 2>/dev/null
sudo sync

echo "🚀 Test device ready for UI testing"
echo "Device: $LOOP_DEV"
echo "Size: 50MB"
echo "Status: Contains test data"
echo
echo "You can now:"
echo "1. Open SecureWipe UI"
echo "2. Select device $LOOP_DEV"
echo "3. Test the complete wipe workflow"
echo "4. Verify certificate generation"
echo
echo "⚠️  Remember: This is a SAFE test device"
echo "💡 When done, run: sudo losetup -d $LOOP_DEV"

# Keep device active
echo "Press Ctrl+C when done testing..."
while true; do
    sleep 1
done