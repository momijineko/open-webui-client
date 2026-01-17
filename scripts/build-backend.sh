#!/bin/bash
# Open WebUI Backend Build Script for Unix/Linux/macOS
#
# This script builds the Open WebUI backend using PyInstaller
#
# Usage:
#   chmod +x scripts/build-backend.sh
#   ./scripts/build-backend.sh

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Set variables
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BACKEND_DIR="$PROJECT_ROOT/backend"
DIST_DIR="$PROJECT_ROOT/dist/backend"
SPEC_FILE="$BACKEND_DIR/open_webui_backend.spec"

echo -e "${CYAN}========================================"
echo -e "Open WebUI Backend Build Script"
echo -e "========================================${NC}"
echo

# Check if Python is available
if ! command -v python3 &> /dev/null; then
    echo -e "${RED}[ERROR] Python 3 is not installed${NC}"
    exit 1
fi

echo -e "${BLUE}[INFO] Python version:${NC}"
python3 --version
echo

# Check if PyInstaller is installed
if ! python3 -c "import PyInstaller" 2>/dev/null; then
    echo -e "${YELLOW}[INFO] PyInstaller not found, installing...${NC}"
    pip3 install pyinstaller
fi

# Create dist directory
mkdir -p "$DIST_DIR"

# Navigate to backend directory
cd "$BACKEND_DIR" || exit 1

echo -e "${BLUE}[INFO] Building Open WebUI backend...${NC}"
echo -e "${BLUE}[INFO] Output directory: $DIST_DIR${NC}"
echo

# Run PyInstaller
python3 -m PyInstaller \
    --clean \
    --workpath "/tmp/pyinstaller/open_webui_backend" \
    --distpath "$DIST_DIR" \
    "$SPEC_FILE"

echo
echo -e "${GREEN}========================================"
echo -e "Build completed successfully!"
echo -e "========================================${NC}"
echo
echo -e "Output location: ${DIST_DIR}/open_webui_backend/"
echo
echo -e "${CYAN}To run the backend:${NC}"
echo -e "  ${DIST_DIR}/open_webui_backend/open-webui-backend"
echo
