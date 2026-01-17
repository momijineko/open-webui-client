#!/bin/bash
# Open WebUI Desktop Build Script for Unix/Linux/macOS
#
# This script builds the Open WebUI desktop client using Tauri
#
# Usage:
#   chmod +x scripts/build-desktop.sh
#   ./scripts/build-desktop.sh
#   ./scripts/build-desktop.sh release

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Set variables
BUILD_TYPE="${1:-debug}"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_DIR="$PROJECT_ROOT/apps/desktop"

echo -e "${CYAN}========================================"
echo -e "Open WebUI Desktop Build Script"
echo -e "========================================${NC}"
echo

# Check if Node.js is available
if ! command -v node &> /dev/null; then
    echo -e "${RED}[ERROR] Node.js is not installed${NC}"
    exit 1
fi

# Check if pnpm is available
if ! command -v pnpm &> /dev/null; then
    echo -e "${YELLOW}[INFO] pnpm not found, installing...${NC}"
    npm install -g pnpm
fi

# Navigate to project root
cd "$PROJECT_ROOT" || exit 1

echo -e "${BLUE}[INFO] Build type: $BUILD_TYPE${NC}"
echo -e "${BLUE}[INFO] Project root: $PROJECT_ROOT${NC}"
echo

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo -e "${YELLOW}[INFO] Installing dependencies...${NC}"
    pnpm install
fi

# Build frontend
echo -e "${BLUE}[INFO] Building frontend...${NC}"
pnpm build

if [ $? -ne 0 ]; then
    echo -e "${RED}[ERROR] Frontend build failed!${NC}"
    exit 1
fi

# Build Tauri app
echo -e "${BLUE}[INFO] Building Tauri desktop app...${NC}"
if [ "$BUILD_TYPE" = "release" ]; then
    pnpm tauri build
else
    pnpm tauri build --debug
fi

if [ $? -ne 0 ]; then
    echo -e "${RED}[ERROR] Tauri build failed!${NC}"
    exit 1
fi

echo
echo -e "${GREEN}========================================"
echo -e "Build completed successfully!"
echo -e "========================================${NC}"
echo

if [ "$BUILD_TYPE" = "release" ]; then
    echo -e "${CYAN}[INFO] Release build output:${NC}"
    echo -e "  ${APP_DIR}/src-tauri/target/release/"
    echo
    echo -e "${CYAN}[INFO] Bundler output:${NC}"
    echo -e "  ${APP_DIR}/src-tauri/target/release/bundle/"
else
    echo -e "${CYAN}[INFO] Debug build output:${NC}"
    echo -e "  ${APP_DIR}/src-tauri/target/debug/"
fi
echo
