@echo off
REM Open WebUI Desktop Build Script for Windows
REM
REM This script builds the Open WebUI desktop client using Tauri
REM
REM Usage:
REM   scripts\build-desktop.bat
REM   scripts\build-desktop.bat release

setlocal enabledelayedexpansion

echo ========================================
echo Open WebUI Desktop Build Script
echo ========================================
echo.

REM Set variables
set "BUILD_TYPE=%1"
if "%BUILD_TYPE%"=="" set "BUILD_TYPE=debug"

set "PROJECT_ROOT=%~dp0.."
set "APP_DIR=%PROJECT_ROOT%\apps\desktop"

REM Check if Node.js is available
node --version >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Node.js is not installed or not in PATH
    exit /b 1
)

REM Check if pnpm is available
pnpm --version >nul 2>&1
if errorlevel 1 (
    echo [ERROR] pnpm is not installed or not in PATH
    echo [INFO] Installing pnpm...
    npm install -g pnpm
)

REM Navigate to project root
cd /d "%PROJECT_ROOT%" || exit /b 1

echo [INFO] Build type: %BUILD_TYPE%
echo [INFO] Project root: %PROJECT_ROOT%
echo.

REM Install dependencies if needed
if not exist "node_modules\" (
    echo [INFO] Installing dependencies...
    pnpm install
)

REM Build frontend
echo [INFO] Building frontend...
pnpm build

if errorlevel 1 (
    echo [ERROR] Frontend build failed!
    exit /b 1
)

REM Build Tauri app
echo [INFO] Building Tauri desktop app...
if "%BUILD_TYPE%"=="release" (
    pnpm tauri build
) else (
    pnpm tauri build --debug
)

if errorlevel 1 (
    echo [ERROR] Tauri build failed!
    exit /b 1
)

echo.
echo ========================================
echo Build completed successfully!
echo ========================================
echo.

if "%BUILD_TYPE%"=="release" (
    echo [INFO] Release build output:
    echo   %APP_DIR%\src-tauri\target\release\
    echo.
    echo [INFO] Bundler output:
    echo   %APP_DIR%\src-tauri\target\release\bundle\
) else (
    echo [INFO] Debug build output:
    echo   %APP_DIR%\src-tauri\target\debug\
)

endlocal
