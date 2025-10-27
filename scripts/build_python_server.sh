#!/bin/bash

# Build Python OCR Server with PyInstaller
# This script bundles the Python server into a single executable

set -e

echo "🐍 Building Python OCR Server..."

# Navigate to project root
cd "$(dirname "$0")/.."

# Check if Python is available
if ! command -v python3 &> /dev/null; then
    echo "❌ Python 3 is not installed"
    echo "   Please install Python 3.10 or higher"
    exit 1
fi

# Check if uv is installed, offer to install if not
USE_UV=false
if command -v uv &> /dev/null; then
    USE_UV=true
    echo "✓ uv is installed"
else
    echo "📦 uv is not installed"
    echo ""
    echo "uv is a fast Python package installer (recommended)"
    read -p "   Would you like to install uv? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "   Installing uv..."
        if curl -LsSf https://astral.sh/uv/install.sh | sh; then
            # Add uv to current shell PATH
            export PATH="$HOME/.local/bin:$PATH"
            USE_UV=true
            echo "✅ uv installed successfully"
        else
            echo "⚠️  uv installation failed, will use python venv instead"
            USE_UV=false
        fi
    else
        echo "   Using python venv instead"
        USE_UV=false
    fi
fi

# Check if virtual environment exists, create if not
if [ ! -d ".venv" ]; then
    echo "📦 Creating virtual environment..."

    # Remove any incomplete venv
    rm -rf .venv

    if [ "$USE_UV" = true ]; then
        echo "   Using uv..."
        uv venv
        source .venv/bin/activate
    else
        echo "   Using python venv..."

        # Try standard venv creation first
        if python3 -m venv .venv 2>/dev/null; then
            echo "   ✓ Virtual environment created"
            source .venv/bin/activate
        elif python3 -m venv --without-pip .venv 2>/dev/null; then
            # Fallback: create without pip and install manually
            echo "   ✓ Virtual environment created (without pip)"
            source .venv/bin/activate

            echo "   Installing pip..."
            if ! curl -sS https://bootstrap.pypa.io/get-pip.py | python; then
                echo "❌ Failed to install pip"
                exit 1
            fi
        else
            echo "❌ Failed to create virtual environment"
            exit 1
        fi
    fi

    echo "✅ Virtual environment created at .venv"
else
    # Activate existing virtual environment
    source .venv/bin/activate
fi

# Determine which pip to use
if command -v uv &> /dev/null; then
    echo "📦 Using uv for package management"
    PIP_INSTALL="uv pip install"
else
    echo "📦 Using pip for package management"

    # Ensure pip is available
    if ! command -v pip &> /dev/null; then
        echo "   Pip not found, installing..."
        curl -sS https://bootstrap.pypa.io/get-pip.py | python
    fi

    PIP_INSTALL="pip install"
fi

# Install dependencies if needed
if [ ! -f ".venv/.deps_installed" ]; then
    echo "📥 Installing dependencies..."
    $PIP_INSTALL -r python_ocr_server/requirements.txt
    touch .venv/.deps_installed
    echo "✅ Dependencies installed"
fi

# Install PyInstaller if not already installed
if ! command -v pyinstaller &> /dev/null; then
    echo "📦 Installing PyInstaller..."
    $PIP_INSTALL pyinstaller
fi

# Navigate to server directory
cd python_ocr_server

# Generate OS-specific PyInstaller spec
echo "📝 Generating PyInstaller spec for current OS..."
python generate_spec.py
if [ $? -ne 0 ]; then
    echo "❌ Failed to generate spec file"
    exit 1
fi

# Clean previous builds
echo "🧹 Cleaning previous builds..."
rm -rf build dist

# Build with PyInstaller
echo "🔨 Building with PyInstaller..."
pyinstaller ocr_server.spec

# Check if build succeeded
if [ -d "dist/ocr_server" ] && [ -f "dist/ocr_server/ocr_server" ]; then
    echo "✅ Python OCR server built successfully!"
    echo "📦 Bundle location: python_ocr_server/dist/ocr_server/"

    # Copy to resources directory for Tauri
    mkdir -p ../src-tauri/resources
    rm -rf ../src-tauri/resources/ocr_server
    cp -r dist/ocr_server ../src-tauri/resources/
    echo "✅ Copied to src-tauri/resources/ocr_server/"
else
    echo "❌ Build failed!"
    exit 1
fi

echo "🎉 Build complete!"
