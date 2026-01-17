#!/bin/bash
# Open WebUI Desktop Release Script for Unix/Linux/macOS
#
# This script creates a complete release of the Open WebUI desktop client
# including the backend and frontend builds
#
# Usage:
#   chmod +x scripts/release.sh
#   ./scripts/release.sh [version]

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Set variables
VERSION="${1:-$(node -p "require('$PWD/package.json').version")}"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$PROJECT_ROOT/dist/release/v$VERSION"

echo -e "${CYAN}========================================"
echo -e "  Open WebUI Desktop Release Script"
echo -e "========================================${NC}"
echo

echo -e "${BLUE}[INFO] Version: $VERSION${NC}"
echo -e "${BLUE}[INFO] Distribution directory: $DIST_DIR${NC}"
echo

# Create distribution directory
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

# Step 1: Build backend
echo -e "${CYAN}========================================"
echo -e "Step 1: Building Backend"
echo -e "========================================${NC}"
echo

"$SCRIPTS_DIR/build-backend.sh"

# Copy backend to release directory
echo
echo -e "${BLUE}[INFO] Copying backend to release directory...${NC}"
cp -r "$PROJECT_ROOT/dist/backend/open_webui_backend" "$DIST_DIR/backend"

# Step 2: Build desktop app
echo
echo -e "${CYAN}========================================"
echo -e "Step 2: Building Desktop App"
echo -e "========================================${NC}"
echo

"$SCRIPTS_DIR/build-desktop.sh" release

# Copy installer to release directory
echo
echo -e "${BLUE}[INFO] Copying installer to release directory...${NC}"
APP_DIR="$PROJECT_ROOT/apps/desktop"

# Platform-specific installers
case "$(uname -s)" in
    Linux*)
        if [ -d "$APP_DIR/src-tauri/target/release/bundle/deb" ]; then
            cp -v "$APP_DIR/src-tauri/target/release/bundle/deb"/*.deb "$DIST_DIR/" 2>/dev/null || true
        fi
        if [ -d "$APP_DIR/src-tauri/target/release/bundle/appimage" ]; then
            cp -v "$APP_DIR/src-tauri/target/release/bundle/appimage"/*.AppImage "$DIST_DIR/" 2>/dev/null || true
        fi
        ;;
    Darwin*)
        if [ -d "$APP_DIR/src-tauri/target/release/bundle/dmg" ]; then
            cp -v "$APP_DIR/src-tauri/target/release/bundle/dmg"/*.dmg "$DIST_DIR/" 2>/dev/null || true
        fi
        if [ -d "$APP_DIR/src-tauri/target/release/bundle/macos" ]; then
            cp -rv "$APP_DIR/src-tauri/target/release/bundle/macos/Open WebUI.app" "$DIST_DIR/" 2>/dev/null || true
        fi
        ;;
esac

# Step 3: Create README
echo
echo -e "${BLUE}[INFO] Creating README...${NC}"
cat > "$DIST_DIR/README.md" << EOF
# Open WebUI Desktop v$VERSION Release

## Contents

This package contains:

### Backend
- Open WebUI Python backend (PyInstaller)

### Desktop Client
- Tauri desktop application installer

## Installation

### Linux
\`\`\`bash
# For .deb package
sudo dpkg -i open-webui_*.deb

# For .AppImage
chmod +x Open-WebUI*.AppImage
./Open-WebUI*.AppImage
\`\`\`

### macOS
\`\`\`bash
# For .dmg installer
open Open-WebUI*.dmg

# Or use the .app directly
open "Open WebUI.app"
\`\`\`

## Configuration

On first launch, you will be prompted to:
- Choose installation mode (Local or Remote)
- Configure local backend or connect to remote instance

## Support

- GitHub: https://github.com/open-webui/open-webui
- Documentation: https://docs.openwebui.com
EOF

# Summary
echo
echo -e "${GREEN}========================================"
echo -e "  Release Created Successfully!"
echo -e "========================================${NC}"
echo
echo -e "${BLUE}[INFO] Version: $VERSION${NC}"
echo -e "${BLUE}[INFO] Location: $DIST_DIR${NC}"
echo
echo -e "${CYAN}[INFO] Contents:${NC}"
echo -e "  - Backend: Open WebUI Python backend"
echo -e "  - Desktop: Tauri application installer"
echo -e "  - Docs: README.md"
echo
