@echo off
REM Open WebUI Backend Build Script for Windows
REM
REM This script builds the Open WebUI backend using PyInstaller
REM
REM Usage:
REM   scripts\build-backend.bat

setlocal enabledelayedexpansion

echo ========================================
echo Open WebUI Backend Build Script
echo ========================================
echo.

REM Set variables
set "PROJECT_ROOT=%~dp0.."
set "BACKEND_DIR=%PROJECT_ROOT%\backend"
set "DIST_DIR=%PROJECT_ROOT%\dist\backend"
set "SPEC_FILE=%BACKEND_DIR%\open_webui_backend.spec"

REM Check if Python is available
python --version >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Python is not installed or not in PATH
    exit /b 1
)

echo [INFO] Python version:
python --version
echo.

REM Check if PyInstaller is installed
python -c "import PyInstaller" >nul 2>&1
if errorlevel 1 (
    echo [INFO] PyInstaller not found, installing...
    pip install pyinstaller
)

REM Create dist directory
if not exist "%DIST_DIR%" mkdir "%DIST_DIR%"

REM Navigate to backend directory
cd /d "%BACKEND_DIR%" || exit /b 1

echo [INFO] Building Open WebUI backend...
echo [INFO] Output directory: %DIST_DIR%
echo.

REM Run PyInstaller
python -m PyInstaller ^
    --clean ^
    --workpath "%TEMP%\pyinstaller\open_webui_backend" ^
    --distpath "%DIST_DIR%" ^
    "%SPEC_FILE%"

if errorlevel 1 (
    echo.
    echo [ERROR] Build failed!
    exit /b 1
)

echo.
echo ========================================
echo Build completed successfully!
echo ========================================
echo.
echo Output location: %DIST_DIR%\open_webui_backend\
echo.
echo To run the backend:
echo   %DIST_DIR%\open_webui_backend\open-webui-backend.exe
echo.

endlocal
