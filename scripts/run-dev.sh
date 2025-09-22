#!/bin/bash
# Quick development script (minimal setup)
set -e

echo "🚀 Quick SecureWipe dev setup..."
cd core && cargo build && cargo test --lib
cd ../ui && npm install 
echo "Starting Tauri dev..."
npm run tauri dev
