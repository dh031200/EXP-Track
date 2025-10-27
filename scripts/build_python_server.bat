@echo off
REM Build Python OCR Server with PyInstaller for Windows
REM This script bundles the Python server into a single executable

setlocal enabledelayedexpansion

echo 🐍 Building Python OCR Server...

REM Navigate to project root
cd /d "%~dp0\.."

REM Check if Python is available
python --version >nul 2>&1
if errorlevel 1 (
    echo ❌ Python 3 is not installed
    echo    Please install Python 3.10 or higher from https://www.python.org
    exit /b 1
)

REM Check if virtual environment exists, create if not
if not exist ".venv" (
    echo 📦 Creating virtual environment...
    python -m venv .venv
    if errorlevel 1 (
        echo ❌ Failed to create virtual environment
        exit /b 1
    )
    echo ✅ Virtual environment created at .venv
)

REM Activate virtual environment
call .venv\Scripts\activate.bat
if errorlevel 1 (
    echo ❌ Failed to activate virtual environment
    exit /b 1
)

REM Install dependencies if needed
if not exist ".venv\.deps_installed" (
    echo 📥 Installing dependencies...
    python -m pip install --upgrade pip
    pip install -r python_ocr_server\requirements.txt
    if errorlevel 1 (
        echo ❌ Failed to install dependencies
        exit /b 1
    )
    type nul > .venv\.deps_installed
    echo ✅ Dependencies installed
)

REM Install PyInstaller if not already installed
python -m PyInstaller --version >nul 2>&1
if errorlevel 1 (
    echo 📦 Installing PyInstaller...
    pip install pyinstaller
    if errorlevel 1 (
        echo ❌ Failed to install PyInstaller
        exit /b 1
    )
)

REM Navigate to server directory
cd python_ocr_server

REM Generate OS-specific PyInstaller spec
echo 📝 Generating PyInstaller spec for Windows...
python generate_spec.py
if errorlevel 1 (
    echo ❌ Failed to generate spec file
    exit /b 1
)

REM Clean previous builds
echo 🧹 Cleaning previous builds...
if exist "build" rmdir /s /q build
if exist "dist" rmdir /s /q dist

REM Build with PyInstaller
echo 🔨 Building with PyInstaller...
pyinstaller ocr_server.spec
if errorlevel 1 (
    echo ❌ Build failed!
    exit /b 1
)

REM Check if build succeeded
if exist "dist\ocr_server\ocr_server.exe" (
    echo ✅ Python OCR server built successfully!
    echo 📦 Bundle location: python_ocr_server\dist\ocr_server\

    REM Copy to resources directory for Tauri
    if not exist "..\src-tauri\resources" mkdir ..\src-tauri\resources
    if exist "..\src-tauri\resources\ocr_server" rmdir /s /q ..\src-tauri\resources\ocr_server
    xcopy /E /I /Y dist\ocr_server ..\src-tauri\resources\ocr_server
    echo ✅ Copied to src-tauri\resources\ocr_server\
) else (
    echo ❌ Build failed! Executable not found.
    exit /b 1
)

echo 🎉 Build complete!
