#!/usr/bin/env python3
"""
FastAPI OCR Server using RapidOCR
Provides REST API for OCR operations
"""
import base64
import io
import os
import sys
from pathlib import Path
from typing import Optional
from concurrent.futures import ThreadPoolExecutor
import asyncio

import numpy as np
from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from PIL import Image
from pydantic import BaseModel
from rapidocr import RapidOCR, EngineType, LangDet, LangRec, ModelType, OCRVersion

app = FastAPI(title="EXP Tracker OCR Server", version="1.0.0")

# CORS middleware for Tauri app
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Global OCR engine (initialized once)
ocr_engine: Optional[RapidOCR] = None

# Thread pool for CPU-intensive OCR operations
executor: Optional[ThreadPoolExecutor] = None


@app.on_event("startup")
async def startup_event():
    """Initialize OCR engine on startup"""
    global ocr_engine, executor
    print("🚀 Initializing RapidOCR engine...")

    # Determine models directory path (works for both dev and bundled app)
    if getattr(sys, 'frozen', False):
        # Running as PyInstaller bundle
        base_path = Path(sys._MEIPASS)
    else:
        # Running as script
        base_path = Path(__file__).parent

    models_dir = base_path / "rapidocr" / "models"

    # Model file paths
    det_model_path = models_dir / "ch_PP-OCRv5_mobile_det.onnx"
    cls_model_path = models_dir / "ch_ppocr_mobile_v2.0_cls_infer.onnx"
    rec_model_path = models_dir / "korean_PP-OCRv5_rec_mobile_infer.onnx"

    print(f"📁 Models directory: {models_dir}")
    print(f"   Det model: {det_model_path.exists()}")
    print(f"   Cls model: {cls_model_path.exists()}")
    print(f"   Rec model: {rec_model_path.exists()}")

    # Initialize RapidOCR engine with explicit model paths
    ocr_engine = RapidOCR(
        params={
            "Det.engine_type": EngineType.ONNXRUNTIME,
            "Det.lang_type": LangDet.CH,
            "Det.model_type": ModelType.MOBILE,
            "Det.ocr_version": OCRVersion.PPOCRV5,
            "Det.model_path": str(det_model_path),
            "Rec.engine_type": EngineType.ONNXRUNTIME,
            "Rec.lang_type": LangRec.KOREAN,
            "Rec.model_type": ModelType.MOBILE,
            "Rec.ocr_version": OCRVersion.PPOCRV5,
            "Rec.model_path": str(rec_model_path),
            "Cls.engine_type": EngineType.ONNXRUNTIME,
            "Cls.model_path": str(cls_model_path),
        }
    )
    # Create thread pool with 4 workers for parallel OCR
    executor = ThreadPoolExecutor(max_workers=4)
    print("✅ RapidOCR engine ready with 4 worker threads")


@app.on_event("shutdown")
async def shutdown_event():
    """Cleanup on shutdown"""
    global executor
    if executor:
        executor.shutdown(wait=True)
        print("🛑 Thread pool shutdown complete")


# Request/Response models
class ImageRequest(BaseModel):
    image_base64: str


class OcrResponse(BaseModel):
    """Unified OCR response - returns raw text only"""
    text: str
    confidence: Optional[float] = None


# Legacy response models (not used anymore)
# class LevelResponse(BaseModel):
#     level: int
#     raw_text: str
#
# class ExpResponse(BaseModel):
#     absolute: int
#     percentage: float
#     raw_text: str
#
# class HpMpResponse(BaseModel):
#     value: int
#     raw_text: str


# Helper functions
def decode_base64_image(base64_str: str) -> np.ndarray:
    """Decode base64 string to numpy array"""
    image_bytes = base64.b64decode(base64_str)
    image = Image.open(io.BytesIO(image_bytes))
    return np.array(image)


def extract_text_from_result(result) -> str:
    """Extract text from RapidOCR result"""
    if not result or not hasattr(result, 'txts'):
        return ""
    
    if isinstance(result.txts, tuple):
        return " ".join(result.txts)
    return ""


# OCR Endpoints - Legacy endpoints (not used, parsing done in Rust)
# @app.post("/ocr/level", response_model=LevelResponse)
# @app.post("/ocr/exp", response_model=ExpResponse)
# @app.post("/ocr/hp", response_model=HpMpResponse)
# @app.post("/ocr/mp", response_model=HpMpResponse)


def _run_ocr_sync(image: np.ndarray) -> str:
    """Synchronous OCR function to run in thread pool"""
    # Call with text_score=0.65 for lower detection threshold
    result = ocr_engine(image, text_score=0.65)
    return extract_text_from_result(result)


@app.post("/ocr", response_model=OcrResponse)
async def recognize_text(request: ImageRequest):
    """
    Unified OCR endpoint - returns raw text only.
    Parsing is handled by the Rust client.
    Runs OCR in thread pool for true parallel processing.
    """
    try:
        image = decode_base64_image(request.image_base64)

        # Run CPU-intensive OCR in thread pool
        loop = asyncio.get_event_loop()
        raw_text = await loop.run_in_executor(executor, _run_ocr_sync, image)

        # Return raw text without any parsing
        return OcrResponse(
            text=raw_text,
            confidence=None  # RapidOCR doesn't provide confidence in simple mode
        )

    except Exception as e:
        raise HTTPException(status_code=500, detail=f"OCR failed: {str(e)}")


@app.get("/health")
async def health_check():
    """Health check endpoint"""
    return {"status": "ok", "engine": "RapidOCR"}


@app.post("/shutdown")
async def shutdown():
    """Graceful shutdown endpoint"""
    import asyncio
    import signal
    import os
    
    async def shutdown_task():
        await asyncio.sleep(0.5)  # Give time to send response
        os.kill(os.getpid(), signal.SIGTERM)
    
    asyncio.create_task(shutdown_task())
    return {"status": "shutting down"}


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="127.0.0.1", port=39835, log_level="info")
