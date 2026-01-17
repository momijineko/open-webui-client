@echo off
REM Open WebUI Desktop Release Script for Windows
REM
REM This script creates a complete release of the Open WebUI desktop client
REM including the backend and frontend builds
REM
REM Usage:
REM   scripts\release.bat

setlocal enabledelayedexpansion

echo ========================================
echo   Open WebUI Desktop Release Script
echo ========================================
echo.

REM Set variables
set "VERSION=%~1"
if "%VERSION%"=="" (
    REM Get version from package.json
    for /f "tokens=2 delims==" %%a in ('findstr "version" "%~dp0..\package.json"') do (
        set "VERSION=%%~a"
        set "VERSION=!VERSION:" "=!
        set "VERSION=!VERSION:,=!"
    )
)

set "PROJECT_ROOT=%~dp0.."
set "DIST_DIR=%PROJECT_ROOT%\dist\release\v%VERSION%"

echo [INFO] Version: %VERSION%
echo [INFO] Distribution directory: %DIST_DIR%
echo.

REM Create distribution directory
if exist "%DIST_DIR%" rmdir /s /q "%DIST_DIR%"
mkdir "%DIST_DIR%"

REM Step 1: Build backend
echo ========================================
echo Step 1: Building Backend
echo ========================================
echo.

call "%~dp0build-backend.bat"

if errorlevel 1 (
    echo [ERROR] Backend build failed!
    exit /b 1
)

echo.
echo [INFO] Copying backend to release directory...
xcopy /e /i /y "%PROJECT_ROOT%\dist\backend\open_webui_backend" "%DIST_DIR%\backend\"

REM Step 2: Build desktop app
echo.
echo ========================================
echo Step 2: Building Desktop App
echo ========================================
echo.

call "%~dp0build-desktop.bat" release

if errorlevel 1 (
    echo [ERROR] Desktop build failed!
    exit /b 1
)

REM Step 3: Copy installer
echo.
echo [INFO] Copying installer to release directory...
set "APP_DIR=%PROJECT_ROOT%\apps\desktop"

if exist "%APP_DIR%\src-tauri\target\release\bundle\msi\" (
    xcopy /y "%APP_DIR%\src-tauri\target\release\bundle\msi\*.msi" "%DIST_DIR%\" 2>nul
)

if exist "%APP_DIR%\src-tauri\target\release\bundle\nsis\" (
    xcopy /y "%APP_DIR%\src-tauri\target\release\bundle\nsis\*.exe" "%DIST_DIR%\" 2>nul
)

REM Step 4: Create README
echo.
echo [INFO] Creating README...
(
echo # Open WebUI Desktop v%VERSION% Release
echo.
echo ## Contents
echo.
echo This package contains:
echo.
echo ### Backend
echo - Open WebUI Python backend ^(PyInstaller^)
echo.
echo ### Desktop Client
echo - Tauri desktop application installer
echo.
echo ## Installation
echo.
echo 1. Run the installer ^(MSI or NSIS^)
echo 2. The backend will be bundled with the application
echo.
echo ## Configuration
echo.
echo On first launch, you will be prompted to:
echo - Choose installation mode ^(Local or Remote^)
echo - Configure local backend or connect to remote instance
echo.
echo ## Support
echo.
echo - GitHub: https://github.com/open-webui/open-webui
echo - Documentation: https://docs.openwebui.com
echo.
) > "%DIST_DIR%\README.txt"

REM Summary
echo.
echo ========================================
echo   Release Created Successfully!
echo ========================================
echo.
echo [INFO] Version: %VERSION%
echo [INFO] Location: %DIST_DIR%
echo.
echo [INFO] Contents:
echo   - Backend: Open WebUI Python backend
echo   - Desktop: Tauri application installer
echo   - Docs: README.txt
echo.

endlocal
