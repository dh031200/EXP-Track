@echo off
REM Build Python OCR Server with PyInstaller for Windows
REM This script bundles the Python server into a single executable

setlocal enabledelayedexpansion

echo Building Python OCR Server...

REM Navigate to project root
cd /d "%~dp0\.."

REM Check if uv is installed, offer to install if not
set USE_UV=false
where uv >nul 2>&1
if errorlevel 1 (
    echo [INFO] uv is not installed
    echo.
    echo uv is a fast Python package installer (recommended)
    set /p "INSTALL_UV=   Would you like to install uv? (y/N): "

    if /i "!INSTALL_UV!"=="y" (
        echo [INFO] Installing uv...
        powershell -ExecutionPolicy ByPass -Command "irm https://astral.sh/uv/install.ps1 | iex"

        REM Add uv to PATH for current session
        set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"

        where uv >nul 2>&1
        if errorlevel 1 (
            echo [WARN] uv installation failed, will use python instead
            set USE_UV=false
        ) else (
            echo [OK] uv installed successfully
            set USE_UV=true
        )
    ) else (
        echo [INFO] Using python instead
        set USE_UV=false
    )
) else (
    echo [OK] uv is installed
    set USE_UV=true
)

REM Check if Python is available (only if not using uv)
if "%USE_UV%"=="false" (
    python --version >nul 2>&1
    if errorlevel 1 (
        echo [ERROR] Python 3 is not installed
        echo         Please install Python 3.10 or higher from https://www.python.org
        echo         Or install uv by running this script again and choosing 'y'
        exit /b 1
    )
)

REM Check if virtual environment exists, create if not
if not exist ".venv" (
    echo [INFO] Creating virtual environment...

    if "%USE_UV%"=="true" (
        echo [INFO] Using uv...
        uv venv
        if errorlevel 1 (
            echo [ERROR] Failed to create virtual environment
            exit /b 1
        )
    ) else (
        echo [INFO] Using python venv...
        python -m venv .venv
        if errorlevel 1 (
            echo [ERROR] Failed to create virtual environment
            exit /b 1
        )
    )

    echo [OK] Virtual environment created at .venv
)

REM Activate virtual environment
call .venv\Scripts\activate.bat
if errorlevel 1 (
    echo [ERROR] Failed to activate virtual environment
    exit /b 1
)

REM Determine which pip to use
if "%USE_UV%"=="true" (
    echo [INFO] Using uv for package management
    set PIP_INSTALL=uv pip install
) else (
    echo [INFO] Using pip for package management

    REM Ensure pip is available
    python -m pip --version >nul 2>&1
    if errorlevel 1 (
        echo [INFO] Pip not found, installing...
        python -m ensurepip --upgrade
    )

    set PIP_INSTALL=pip install
)

REM Install dependencies if needed
if not exist ".venv\.deps_installed" (
    echo [INFO] Installing dependencies...

    if "%USE_UV%"=="false" (
        python -m pip install --upgrade pip
    )

    %PIP_INSTALL% -r python_ocr_server\requirements.txt
    if errorlevel 1 (
        echo [ERROR] Failed to install dependencies
        exit /b 1
    )
    type nul > .venv\.deps_installed
    echo [OK] Dependencies installed
)

REM Install PyInstaller if not already installed
python -m PyInstaller --version >nul 2>&1
if errorlevel 1 (
    echo [INFO] Installing PyInstaller...
    %PIP_INSTALL% pyinstaller
    if errorlevel 1 (
        echo [ERROR] Failed to install PyInstaller
        exit /b 1
    )
)

REM Navigate to server directory
cd python_ocr_server

REM Generate OS-specific PyInstaller spec
echo [INFO] Generating PyInstaller spec for Windows...
python generate_spec.py
if errorlevel 1 (
    echo [ERROR] Failed to generate spec file
    exit /b 1
)

REM Clean previous builds
echo [INFO] Cleaning previous builds...
if exist "build" rmdir /s /q build
if exist "dist" rmdir /s /q dist

REM Build with PyInstaller
echo [INFO] Building with PyInstaller...
pyinstaller ocr_server.spec
if errorlevel 1 (
    echo [ERROR] Build failed!
    exit /b 1
)

REM Check if build succeeded
if exist "dist\ocr_server\ocr_server.exe" (
    echo [OK] Python OCR server built successfully!
    echo [INFO] Bundle location: python_ocr_server\dist\ocr_server\

    REM Copy to resources directory for Tauri
    if not exist "..\src-tauri\resources" mkdir ..\src-tauri\resources
    if exist "..\src-tauri\resources\ocr_server" rmdir /s /q ..\src-tauri\resources\ocr_server
    xcopy /E /I /Y dist\ocr_server ..\src-tauri\resources\ocr_server
    echo [OK] Copied to src-tauri\resources\ocr_server\
) else (
    echo [ERROR] Build failed! Executable not found.
    exit /b 1
)

echo [SUCCESS] Build complete!
