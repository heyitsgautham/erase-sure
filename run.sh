#!/bin/bash
# SecureWipe Development Environment Setup & Run Script
set -e  # Exit on any error

PROJECT_ROOT=$(pwd)

echo "🔧 Setting up SecureWipe development environment..."

# Core (Rust) - Build and Test
echo "📦 Building core (debug & release)..."
cd "$PROJECT_ROOT/core"
cargo build
cargo build --release
echo "🧪 Running core tests..."
cargo test

# UI (Tauri + React) - Setup and Check
echo "🎨 Setting up UI..."
cd "$PROJECT_ROOT/ui/src-tauri"
cargo check
cd "$PROJECT_ROOT/ui"
npm install
npm run lint || echo "⚠️  Lint warnings found (continuing...)"

# Portal (FastAPI) - Setup
echo "🌐 Setting up portal..."
cd "$PROJECT_ROOT/portal"
if [ ! -d "venv" ]; then
    echo "Creating Python virtual environment..."
    python3 -m venv venv
fi
source venv/bin/activate
pip install -r requirements.txt
echo "🧪 Running portal tests..."
python -m pytest tests/ || echo "⚠️  Portal tests failed (continuing...)"
deactivate

# Integration Tests (Optional - can be skipped for dev)
echo "🔗 Running integration tests..."
cd "$PROJECT_ROOT"
if [ -f "tests/scripts/test_backup_integration.sh" ]; then
    chmod +x tests/scripts/test_backup_integration.sh
    ./tests/scripts/test_backup_integration.sh || echo "⚠️  Integration tests failed (continuing...)"
fi

# Start Development Servers
echo "🚀 Starting development servers..."
cd "$PROJECT_ROOT/ui"
echo "Starting Tauri dev server..."
npm run tauri dev

cd "$PROJECT_ROOT"
echo "✅ Done!"
