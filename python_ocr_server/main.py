#!/usr/bin/env python3
"""
FastAPI OCR Server using RapidOCR
Provides REST API for OCR operations
"""
import base64
import io
import sys
from pathlib import Path
from typing import Optional, List
from concurrent.futures import ThreadPoolExecutor
import asyncio
from contextlib import asynccontextmanager

import numpy as np
from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from PIL import Image
from pydantic import BaseModel
from rapidocr import RapidOCR
import cv2 # OpenCV for image preprocessing

# Determine models directory path (works for both dev and bundled app)
if getattr(sys, 'frozen', False):
    # Running as PyInstaller bundle
    base_path = Path(sys._MEIPASS)
else:
    # Running as script
    base_path = Path(__file__).parent

models_dir = base_path / "rapidocr" / "models"


# Model file paths - Using English model for better number recognition
det_model_path = models_dir / "ch_PP-OCRv4_det_infer.onnx"
cls_model_path = models_dir / "ch_ppocr_mobile_v2.0_cls_infer.onnx"
rec_model_path = models_dir / "en_PP-OCRv4_rec_infer.onnx"  # English model for numbers

# Character dictionary for numbers only (0-9, [, ], %, .)
dict_path = base_path / "dict_numbers.txt"

print(f"📁 Models directory: {models_dir}")
print(f"   Det model: {det_model_path.exists()}")
print(f"   Cls model: {cls_model_path.exists()}")
print(f"   Rec model (EN): {rec_model_path.exists()}")
print(f"   Number dict: {dict_path.exists()}")


# OCR engine pool - one engine per worker for true parallelism
ocr_engines: List[RapidOCR] = []

# Thread pool for CPU-intensive OCR operations
executor: Optional[ThreadPoolExecutor] = None

# Round-robin index for load balancing
current_engine_idx = 0


def _load_engine(idx: int) -> RapidOCR:
    """Load a single OCR engine (for parallel initialization)"""
    engine = RapidOCR(
        params={
            "Det.model_path": str(det_model_path),
            "Rec.model_path": str(rec_model_path),
            "Cls.model_path": str(cls_model_path),
            # Number-only character dictionary for reduced misrecognition
            "Rec.character_dict_path": str(dict_path) if dict_path.exists() else None,
            # Detection parameter tuning for game UI
            "Det.det_db_thresh": 0.3,       # Binary threshold (lower = more sensitive)
            "Det.det_db_box_thresh": 0.5,   # Box confidence (lower = more boxes detected)
            "Det.det_db_unclip_ratio": 1.6, # Box expansion ratio (higher = larger boxes)
        }
    )
    print(f"   ✅ Engine {idx+1}/4 loaded (~24MB)")
    return engine


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Lifespan context manager for startup and shutdown events"""
    global ocr_engines, executor

    # Startup
    print("🚀 Initializing RapidOCR engine pool...")
    NUM_WORKERS = 4
    print(f"⚙️  Loading {NUM_WORKERS} independent OCR engines in parallel...")

    # Load all 4 engines in parallel (4x faster!)
    load_pool = ThreadPoolExecutor(max_workers=NUM_WORKERS)

    try:
        # Submit all load tasks at once
        futures = [load_pool.submit(_load_engine, i) for i in range(NUM_WORKERS)]

        # Wait for all engines to load
        for future in futures:
            engine = future.result()
            ocr_engines.append(engine)
    finally:
        load_pool.shutdown(wait=True)

    # Create thread pool with 4 workers for OCR operations
    executor = ThreadPoolExecutor(max_workers=NUM_WORKERS)

    total_memory = NUM_WORKERS * 24  # ~24MB per engine
    print(f"✅ OCR engine pool ready: {NUM_WORKERS} engines, ~{total_memory}MB total")
    print(f"🚀 True parallel processing enabled - no GIL contention!")

    yield

    # Shutdown
    if executor:
        executor.shutdown(wait=True)
        print("🛑 Thread pool shutdown complete")

    ocr_engines.clear()
    print("🛑 OCR engine pool cleared")


app = FastAPI(title="EXP Tracker OCR Server", version="1.0.0", lifespan=lifespan)

# CORS middleware for Tauri app
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


# Request/Response models
class ImageRequest(BaseModel):
    image_base64: str


class TextBox(BaseModel):
    """Single OCR text detection with bounding box"""
    box: List[List[float]]  # 4 corner points [[x1,y1], [x2,y2], [x3,y3], [x4,y4]]
    text: str
    score: float

class OcrResponse(BaseModel):
    """Unified OCR response - returns structured text boxes with coordinates"""
    boxes: List[TextBox]  # List of detected text boxes
    raw_text: str  # Legacy: concatenated text for backward compatibility


# Helper functions
def decode_base64_image(base64_str: str) -> np.ndarray:
    """Decode base64 string to numpy array"""
    image_bytes = base64.b64decode(base64_str)
    image = Image.open(io.BytesIO(image_bytes))
    return np.array(image)


def parse_rapidocr_result(result) -> tuple[List[TextBox], str]:
    """
    Parse RapidOCR result into structured TextBox list and concatenated text.

    RapidOCR returns a list of lists: [[box, text, score], ...]
    where box is 4 corner points, text is the recognized string, score is confidence.
    """
    if not result or len(result) == 0:
        return ([], "")

    boxes = []
    texts = []

    for item in result:
        if len(item) >= 3:
            box_coords, text, score = item[0], item[1], item[2]
            boxes.append(TextBox(
                box=box_coords.tolist() if hasattr(box_coords, 'tolist') else box_coords,
                text=text,
                score=float(score)
            ))
            texts.append(text)

    raw_text = " ".join(texts)
    return (boxes, raw_text)

def preprocess_image(image: np.ndarray) -> np.ndarray:
    """
    Preprocess image for better OCR accuracy on game UI text.
    1. Convert to Grayscale
    2. Upscale (2x) - critical for small numbers
    3. Invert if text is white-on-dark (common in games)
    4. Binarize (Threshold) - make text crisp black
    """
    try:
        # Convert to grayscale if color
        if len(image.shape) == 3:
            gray = cv2.cvtColor(image, cv2.COLOR_RGB2GRAY)
        else:
            gray = image

        # Check image size - if it's a small ROI strip, upscale it
        h, w = gray.shape
        scale = 1.0
        if h < 100: # Typical height for Level/EXP number strip is small
            scale = 2.0
            gray = cv2.resize(gray, None, fx=scale, fy=scale, interpolation=cv2.INTER_CUBIC)
        
        # Check brightness to determine if inversion is needed
        # Game UI often has white text on dark background. OCR prefers black text on white.
        mean_brightness = np.mean(gray)
        if mean_brightness < 127:
            # Invert to get black text on white
            gray = 255 - gray
        
        # Enhance contrast/Binarize
        # OTSU thresholding finds optimal separation
        _, binary = cv2.threshold(gray, 0, 255, cv2.THRESH_BINARY + cv2.THRESH_OTSU)
        
        return binary
    except Exception as e:
        print(f"⚠️ Preprocessing failed: {e}, using original image")
        return image

def _run_ocr_sync(image: np.ndarray, engine_idx: int) -> tuple[List[TextBox], str]:
    """
    Synchronous OCR function to run in thread pool.
    Each worker gets its dedicated OCR engine for true parallelism.
    Returns structured TextBox list and concatenated raw text.
    """
    # Use dedicated engine for this worker (no contention)
    engine = ocr_engines[engine_idx]

    # Preprocess image (Upscale, Grayscale, Threshold)
    processed_image = preprocess_image(image)

    # RapidOCR returns a RapidOCROutput object with txts, boxes, scores attributes
    # Lowered text_score for better recall on game UI numbers
    ocr_output = engine(processed_image, text_score=0.75)

    # Extract structured data from RapidOCROutput
    boxes = []
    texts = []

    if ocr_output is not None and hasattr(ocr_output, 'txts') and hasattr(ocr_output, 'boxes') and hasattr(ocr_output, 'scores'):
        # RapidOCR result structure:
        # - txts: tuple of recognized text strings
        # - boxes: list/array of bounding boxes (4 corner points)
        # - scores: list/array of confidence scores

        # Use 'is not None' to avoid numpy array boolean ambiguity
        txts = ocr_output.txts if ocr_output.txts is not None else []
        box_coords = ocr_output.boxes if ocr_output.boxes is not None else []
        scores = ocr_output.scores if ocr_output.scores is not None else []

        # Combine into TextBox objects
        for i, text in enumerate(txts):
            if i < len(box_coords) and i < len(scores):
                boxes.append(TextBox(
                    box=box_coords[i].tolist() if hasattr(box_coords[i], 'tolist') else box_coords[i],
                    text=text,
                    score=float(scores[i])
                ))
                texts.append(text)

    raw_text = " ".join(texts)
    return (boxes, raw_text)


@app.post("/ocr", response_model=OcrResponse)
async def recognize_text(request: ImageRequest):
    """
    Unified OCR endpoint - returns structured text boxes with bounding boxes.
    Rust client will handle NMS filtering and parsing.
    Uses round-robin load balancing across 4 independent OCR engines.
    """
    global current_engine_idx

    try:
        image = decode_base64_image(request.image_base64)

        # Round-robin engine selection for load balancing
        engine_idx = current_engine_idx
        current_engine_idx = (current_engine_idx + 1) % len(ocr_engines)

        # Run CPU-intensive OCR in thread pool with dedicated engine
        loop = asyncio.get_event_loop()
        boxes, raw_text = await loop.run_in_executor(
            executor,
            _run_ocr_sync,
            image,
            engine_idx
        )

        # Return structured boxes with coordinates for NMS processing
        response = OcrResponse(
            boxes=boxes,
            raw_text=raw_text
        )

        return response

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
    import platform
    import logging

    # Fix Windows ProactorEventLoop connection reset errors
    if platform.system() == "Windows":
        # Use SelectorEventLoop instead of ProactorEventLoop on Windows
        # This prevents ConnectionResetError when clients close connections quickly
        asyncio.set_event_loop_policy(asyncio.WindowsSelectorEventLoopPolicy())

    # Fix for PyInstaller builds with console=False
    # When bundled without console, sys.stdout/stderr are None, which breaks uvicorn logging
    if getattr(sys, 'frozen', False) and sys.stdout is None:
        # Create log directory if it doesn't exist
        log_dir = Path(sys._MEIPASS).parent / "logs"
        log_dir.mkdir(exist_ok=True)
        log_file = log_dir / "ocr_server.log"

        # Redirect stdout/stderr to log file
        sys.stdout = open(log_file, 'w', encoding='utf-8', buffering=1)
        sys.stderr = sys.stdout

        # Configure logging to use file handler
        logging.basicConfig(
            level=logging.INFO,
            format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
            handlers=[
                logging.FileHandler(log_file, mode='w', encoding='utf-8')
            ]
        )

        # Disable uvicorn's default logging and use custom config
        uvicorn.run(
            app,
            host="127.0.0.1",
            port=39835,
            log_config=None,  # Disable default logging config
            access_log=False   # Disable access logs for cleaner output
        )
    else:
        # Normal execution with console
        uvicorn.run(app, host="127.0.0.1", port=39835, log_level="info")
