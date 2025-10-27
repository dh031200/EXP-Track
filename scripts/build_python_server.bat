@echo off
REM Build Python OCR Server with PyInstaller for Windows
REM This script bundles the Python server into a single executable

setlocal enabledelayedexpansion

echo Building Python OCR Server...
echo.

REM Navigate to project root
cd /d "%~dp0\.."

REM Check if npm is installed
echo [CHECK] Checking for npm...
where npm >nul 2>&1
if errorlevel 1 (
    echo [INFO] npm is not installed
    echo.
    echo npm is required for Node.js development (required for Tauri frontend)
    set /p "INSTALL_NPM=   Would you like to install Node.js (includes npm)? (y/N): "

    if /i "!INSTALL_NPM!"=="y" (
        echo [INFO] Installing Node.js...
        echo [INFO] Checking for package manager...

        REM Try winget first
        where winget >nul 2>&1
        if not errorlevel 1 (
            echo [INFO] Using winget...
            winget install -e --id OpenJS.NodeJS
        ) else (
            REM Try chocolatey
            where choco >nul 2>&1
            if not errorlevel 1 (
                echo [INFO] Using chocolatey...
                choco install nodejs -y
            ) else (
                echo [WARN] No package manager found
                echo        Please install Node.js manually from https://nodejs.org
                echo        Then run this script again
                exit /b 1
            )
        )

        REM Refresh PATH
        echo [INFO] Refreshing environment...
        call refreshenv 2>nul

        where npm >nul 2>&1
        if errorlevel 1 (
            echo [WARN] npm installation may require a new terminal session
            echo        Please restart your terminal and run this script again
            exit /b 1
        ) else (
            echo [OK] Node.js and npm installed successfully
        )
    ) else (
        echo [WARN] npm is required for building Tauri frontend
        echo        You can install it later from https://nodejs.org
    )
) else (
    echo [OK] npm is installed
)
echo.

REM Check if Rust is installed
echo [CHECK] Checking for Rust...
where rustc >nul 2>&1
if errorlevel 1 (
    echo [INFO] Rust is not installed
    echo.
    echo Rust is required for building Tauri backend
    set /p "INSTALL_RUST=   Would you like to install Rust? (y/N): "

    if /i "!INSTALL_RUST!"=="y" (
        echo [INFO] Installing Rust...

        REM Try winget first
        where winget >nul 2>&1
        if not errorlevel 1 (
            echo [INFO] Using winget...
            winget install -e --id Rustlang.Rustup
        ) else (
            echo [INFO] Downloading rustup-init.exe...
            powershell -Command "Invoke-WebRequest -Uri 'https://win.rustup.rs' -OutFile '%TEMP%\rustup-init.exe'"
            if exist "%TEMP%\rustup-init.exe" (
                echo [INFO] Running Rust installer...
                "%TEMP%\rustup-init.exe" -y
                del "%TEMP%\rustup-init.exe"
            ) else (
                echo [ERROR] Failed to download Rust installer
                echo        Please install manually from https://rustup.rs
                exit /b 1
            )
        )

        REM Add Rust to PATH for current session
        set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"

        where rustc >nul 2>&1
        if errorlevel 1 (
            echo [WARN] Rust installation may require a new terminal session
            echo        Please restart your terminal and run this script again
            exit /b 1
        ) else (
            echo [OK] Rust installed successfully
        )
    ) else (
        echo [WARN] Rust is required for building Tauri applications
        echo        You can install it later from https://rustup.rs
    )
) else (
    echo [OK] Rust is installed
)
echo.

REM Check for MSVC linker (link.exe) - required for Rust on Windows
echo [CHECK] Checking for Visual Studio Build Tools...

REM Try multiple ways to find link.exe
set MSVC_FOUND=false

REM Method 1: Check if link.exe is in PATH
where link.exe >nul 2>&1
if not errorlevel 1 (
    set MSVC_FOUND=true
)

REM Method 2: Check common Visual Studio installation paths
if "%MSVC_FOUND%"=="false" (
    for %%V in (2022 2019 2017) do (
        for %%E in (BuildTools Community Professional Enterprise) do (
            if exist "C:\Program Files\Microsoft Visual Studio\%%V\%%E\VC\Tools\MSVC" (
                set MSVC_FOUND=true
                goto :msvc_found
            )
            if exist "C:\Program Files (x86)\Microsoft Visual Studio\%%V\%%E\VC\Tools\MSVC" (
                set MSVC_FOUND=true
                goto :msvc_found
            )
        )
    )
)
:msvc_found

if "%MSVC_FOUND%"=="false" (
    echo [WARN] Visual Studio Build Tools not detected
    echo.
    echo MSVC linker is required for building Rust/Tauri applications on Windows
    set /p "INSTALL_BUILDTOOLS=   Would you like to install Visual Studio Build Tools? (y/N): "

    if /i "!INSTALL_BUILDTOOLS!"=="y" (
        echo [INFO] Installing Visual Studio Build Tools...
        echo [INFO] This may take several minutes...

        REM Try winget first
        where winget >nul 2>&1
        if not errorlevel 1 (
            echo [INFO] Using winget...
            winget install Microsoft.VisualStudio.2022.BuildTools --override "--wait --quiet --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
            echo [INFO] Installation complete. Please restart your terminal.
        ) else (
            echo [INFO] Opening download page...
            echo        Please download and install "Build Tools for Visual Studio 2022"
            echo        Make sure to select "Desktop development with C++" workload
            start https://visualstudio.microsoft.com/downloads/
            echo.
            echo        After installation, restart your terminal and run this script again
        )
    ) else (
        echo [WARN] Continuing without Visual Studio Build Tools
        echo        Note: Tauri build will fail without MSVC linker
        echo        You can install it later from: https://visualstudio.microsoft.com/downloads/
    )
) else (
    echo [OK] Visual Studio Build Tools detected
)
echo.

REM Check if uv is installed, offer to install if not
echo [CHECK] Checking for uv...
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
echo.

REM Check if Python is available (only if not using uv)
if "%USE_UV%"=="false" (
    echo [CHECK] Checking for Python...
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
