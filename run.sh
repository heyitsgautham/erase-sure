#!/bin/bash
# filepath: run.sh
set -e  # Exit on any error

echo "Building core (debug)..."
cd core && cargo build

echo "Building core (release)..."
cargo build --release

echo "Running core tests..."
cargo test

echo "Checking UI Tauri backend..."
cd ../ui/src-tauri/ && cargo check

echo "Installing UI dependencies (if needed)..."
cd ../ && npm install

echo "Starting Tauri dev server..."
npm run tauri dev

cd ..
echo "Done!"
