# -*- mode: python ; coding: utf-8 -*-

block_cipher = None

a = Analysis(
    ['main.py'],
    pathex=[],
    binaries=[
        # ONNX Runtime native libraries - CRITICAL for inference
        ('../.venv/lib/python3.12/site-packages/onnxruntime/capi/libonnxruntime.1.23.2.dylib', 'onnxruntime/capi'),
        ('../.venv/lib/python3.12/site-packages/onnxruntime/capi/onnxruntime_pybind11_state.so', 'onnxruntime/capi'),
    ],
    datas=[
        # Include entire RapidOCR package with all resources
        ('../.venv/lib/python3.12/site-packages/rapidocr', 'rapidocr'),
    ],
    hiddenimports=[
        # RapidOCR core
        'rapidocr',
        'rapidocr.main',
        'rapidocr.ch_ppocr_det',
        'rapidocr.ch_ppocr_rec',
        'rapidocr.ch_ppocr_cls',
        'rapidocr.cal_rec_boxes',
        'rapidocr.cal_rec_boxes.main',
        'rapidocr.ch_ppocr_rec.utils',
        'rapidocr.utils',
        # ONNX Runtime
        'onnxruntime',
        'onnxruntime.capi',
        'onnxruntime.capi.onnxruntime_pybind11_state',
        # FastAPI & dependencies
        'fastapi',
        'fastapi.responses',
        'uvicorn',
        'uvicorn.protocols',
        'uvicorn.protocols.http',
        'uvicorn.protocols.websockets',
        'uvicorn.lifespan',
        'uvicorn.lifespan.on',
        'starlette',
        'starlette.responses',
        'starlette.middleware',
        'starlette.middleware.cors',
        'pydantic',
        'pydantic.types',
        # Image processing
        'PIL',
        'PIL.Image',
        'PIL.ImageDraw',
        'PIL.ImageFont',
        'numpy',
        'numpy.core',
        'numpy.core._multiarray_umath',
        'cv2',
        # Other
        'yaml',
        'base64',
        'pathlib',
    ],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[],
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
    console=True,
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
    name='ocr_server'
)
