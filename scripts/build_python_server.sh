#!/bin/bash

# Build Python OCR Server with PyInstaller
# This script bundles the Python server into a single executable

set -e

echo "🐍 Building Python OCR Server..."

# Navigate to project root
cd "$(dirname "$0")/.."

# Check if virtual environment exists
if [ ! -d ".venv" ]; then
    echo "❌ Virtual environment not found at .venv"
    echo "   Please run: uv venv && source .venv/bin/activate && uv pip install -r python_ocr_server/requirements.txt"
    exit 1
fi

# Activate virtual environment
source .venv/bin/activate

# Check if uv is installed
if ! command -v uv &> /dev/null; then
    echo "❌ uv is not installed"
    echo "   Please install: curl -LsSf https://astral.sh/uv/install.sh | sh"
    exit 1
fi

# Install PyInstaller if not already installed
if ! command -v pyinstaller &> /dev/null; then
    echo "📦 Installing PyInstaller..."
    uv pip install pyinstaller
fi

# Navigate to server directory
cd python_ocr_server

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
