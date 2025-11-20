#!/usr/bin/env python3
"""
Generate PyInstaller spec file for OCR server with PaddleOCR
Uses PyInstaller's collect_all to automatically gather all necessary files
"""
import platform
import sys

def generate_spec():
    """Generate PyInstaller spec file for current OS."""
    system = platform.system()
    machine = platform.machine().lower()

    spec_content = f'''# -*- mode: python ; coding: utf-8 -*-
"""
PyInstaller spec file for PaddleOCR Server
Auto-generated for {system} ({machine})
"""
from PyInstaller.utils.hooks import collect_all

block_cipher = None

# Automatically collect all PaddleOCR and Paddle files
datas = []
binaries = []
hiddenimports = []

# Collect PaddleOCR (models, configs, utils, etc.)
try:
    tmp_ret = collect_all('paddleocr')
    datas += tmp_ret[0]
    binaries += tmp_ret[1]
    hiddenimports += tmp_ret[2]
    print("OK Collected PaddleOCR files")
except Exception as e:
    print(f"Warning: Could not collect paddleocr: {{e}}")

# Collect Paddle (framework, ops, etc.)
try:
    tmp_ret = collect_all('paddle')
    datas += tmp_ret[0]
    binaries += tmp_ret[1]
    hiddenimports += tmp_ret[2]
    print("OK Collected Paddle files")
except Exception as e:
    print(f"Warning: Could not collect paddle: {{e}}")

a = Analysis(
    ['main.py'],
    pathex=[],
    binaries=binaries,
    datas=datas,
    hiddenimports=hiddenimports + [
        'fastapi',
        'uvicorn',
        'uvicorn.lifespan',
        'uvicorn.lifespan.on',
        'pydantic',
        'pydantic_core',
        'PIL',
        'PIL.Image',
        'numpy',
        'cv2',
        'shapely',
        'shapely.geometry',
        'pyclipper',
    ],
    hookspath=[],
    hooksconfig={{}},
    runtime_hooks=[],
    excludes=[
        'matplotlib',
        'scipy',
        'pandas',
        'pytest',
        'setuptools',
    ],
    win_no_prefer_redirects=False,
    win_private_assemblies=False,
    cipher=block_cipher,
    noarchive=False,
)

pyz = PYZ(a.pure, a.zipped_data, cipher=block_cipher)

exe = EXE(
    pyz,
    a.scripts,
    [],
    exclude_binaries=True,
    name='ocr_server',
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    console=True,  # Keep console for debugging
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
)

coll = COLLECT(
    exe,
    a.binaries,
    a.zipfiles,
    a.datas,
    strip=False,
    upx=True,
    upx_exclude=[],
    name='ocr_server',
)
'''

    return spec_content


if __name__ == "__main__":
    print(f"Generating PyInstaller spec for {platform.system()} ({platform.machine()})...")

    spec_content = generate_spec()
    spec_path = Path(__file__).parent / "ocr_server.spec"

    with open(spec_path, 'w', encoding='utf-8') as f:
        f.write(spec_content)

    print(f"OK Spec file generated: {spec_path}")
    print("\nTo build the executable:")
    print("  1. Activate virtual environment")
    print("  2. Run: pyinstaller ocr_server.spec")


from pathlib import Path
