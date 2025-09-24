#!/bin/bash

# SecureWipe GUI Launcher with Elevated Privileges
# This script helps users run the SecureWipe GUI with proper permissions for destructive operations

echo "🔐 SecureWipe GUI - Elevated Privileges Launcher"
echo "=================================================="
echo ""
echo "This script will launch the SecureWipe GUI with administrator privileges."
echo "You will be prompted for your password to enable destructive wipe operations."
echo ""

# Check if we're already running as root
if [ "$EUID" -eq 0 ]; then
    echo "✅ Already running with administrator privileges"
else
    echo "🔑 Administrator privileges required for destructive wipe operations"
    echo "   (Discovery and backup operations work without sudo)"
    echo ""
fi

# Find the built GUI executable
GUI_PATH=""
if [ -f "target/release/secure-wipe" ]; then
    GUI_PATH="target/release/secure-wipe"
elif [ -f "target/debug/secure-wipe" ]; then
    GUI_PATH="target/debug/secure-wipe"
elif [ -f "ui/src-tauri/target/release/secure-wipe" ]; then
    GUI_PATH="ui/src-tauri/target/release/secure-wipe"
elif [ -f "ui/src-tauri/target/debug/secure-wipe" ]; then
    GUI_PATH="ui/src-tauri/target/debug/secure-wipe"
else
    echo "❌ Error: Could not find SecureWipe GUI executable"
    echo "   Please build the application first with: cd ui && npm run tauri build"
    exit 1
fi

echo "🚀 Launching SecureWipe GUI: $GUI_PATH"

# Check if we're in a graphical environment
if [ -z "$DISPLAY" ] && [ -z "$WAYLAND_DISPLAY" ]; then
    echo "❌ Error: No graphical display detected"
    echo "   This script requires a graphical environment to run the GUI"
    echo "   Use the CLI instead: sudo ./core/target/debug/securewipe wipe --help"
    exit 1
fi

# Set up environment for GUI sudo prompts
export SUDO_ASKPASS="$(which zenity 2>/dev/null || which kdialog 2>/dev/null || echo '')"

if [ "$EUID" -eq 0 ]; then
    # Already root, run directly
    exec "./$GUI_PATH"
else
    # Need privileges, but GUI privilege escalation is problematic
    # Provide clear guidance instead
    echo ""
    echo "⚠️  GUI Privilege Escalation Limitations"
    echo "   Running GUI applications with elevated privileges can cause display access issues."
    echo ""
    echo "🎯 Recommended Approaches:"
    echo ""
    echo "   1. 🖥️  Launch from terminal with sudo:"
    echo "      sudo DISPLAY=$DISPLAY ./$GUI_PATH"
    echo ""
    echo "   2. 🔧 Use CLI for destructive operations:"
    echo "      sudo ./core/target/debug/securewipe discover"
    echo "      sudo ./core/target/debug/securewipe wipe --device /dev/sdX --policy CLEAR --danger-allow-wipe"
    echo ""
    echo "   3. 🚀 Run GUI normally (discovery/backup work, wipe shows permission error):"
    echo "      ./$GUI_PATH"
    echo ""
    echo -n "Choose an option (1/2/3) or press Enter for option 3: "
    read choice
    
    case "$choice" in
        1)
            echo "🔐 Launching with sudo and display preservation..."
            exec sudo DISPLAY="$DISPLAY" XAUTHORITY="$XAUTHORITY" "./$GUI_PATH"
            ;;
        2)
            echo "🔧 CLI mode selected. Example commands:"
            echo "  sudo ./core/target/debug/securewipe discover"
            echo "  sudo ./core/target/debug/securewipe backup --device /dev/sdX --dest /path/to/backup"
            echo "  sudo ./core/target/debug/securewipe wipe --device /dev/sdX --policy CLEAR --danger-allow-wipe"
            exit 0
            ;;
        3|"")
            echo "� Launching GUI without privileges..."
            echo "   Discovery and backup will work normally."
            echo "   Wipe operations will show permission error with guidance."
            exec "./$GUI_PATH"
            ;;
        *)
            echo "❌ Invalid choice. Launching GUI without privileges..."
            exec "./$GUI_PATH"
            ;;
    esac
fi