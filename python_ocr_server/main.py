#!/usr/bin/env python3
"""
FastAPI OCR Server using RapidOCR
Provides REST API for OCR operations
"""
import base64
import io
import re
from typing import Optional
from concurrent.futures import ThreadPoolExecutor
import asyncio

import numpy as np
from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from PIL import Image
from pydantic import BaseModel
from rapidocr import RapidOCR

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
    ocr_engine = RapidOCR()
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


class LevelResponse(BaseModel):
    level: int
    raw_text: str


class ExpResponse(BaseModel):
    absolute: int
    percentage: float
    raw_text: str


class HpMpResponse(BaseModel):
    value: int
    raw_text: str


class OcrResponse(BaseModel):
    """Unified OCR response - returns raw text only"""
    text: str
    confidence: Optional[float] = None


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


# OCR Endpoints
@app.post("/ocr/level", response_model=LevelResponse)
async def recognize_level(request: ImageRequest):
    """Recognize level from image"""
    try:
        image = decode_base64_image(request.image_base64)
        result = ocr_engine(image)
        raw_text = extract_text_from_result(result)
        
        # Parse level (strip all non-digits)
        digits = "".join(filter(str.isdigit, raw_text))
        
        if not digits:
            raise HTTPException(status_code=400, detail=f"No digits found in OCR output: '{raw_text}'")
        
        level = int(digits)
        
        # Validate range
        if level < 1 or level > 300:
            raise HTTPException(status_code=400, detail=f"Level {level} out of valid range (1-300)")
        
        return LevelResponse(
            level=level,
            raw_text=f"LV. {level}"
        )
    
    except ValueError as e:
        raise HTTPException(status_code=400, detail=f"Failed to parse level: {str(e)}")
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"OCR failed: {str(e)}")


@app.post("/ocr/exp", response_model=ExpResponse)
async def recognize_exp(request: ImageRequest):
    """Recognize EXP from image"""
    try:
        image = decode_base64_image(request.image_base64)
        result = ocr_engine(image)
        raw_text = extract_text_from_result(result)
        
        # Parse EXP format: "EXP 1,234,567 [12.34%]" or "1234567[12.34%]"
        # Remove "EXP" prefix and spaces
        cleaned = raw_text.replace("EXP", "").replace(" ", "").replace(",", "")
        
        # Extract absolute value and percentage
        match = re.search(r'(\d+)\[?([\d.]+)%?\]?', cleaned)
        if not match:
            raise HTTPException(status_code=400, detail=f"Failed to parse EXP format: '{raw_text}'")
        
        absolute = int(match.group(1))
        percentage = float(match.group(2))
        
        return ExpResponse(
            absolute=absolute,
            percentage=percentage,
            raw_text=raw_text
        )
    
    except ValueError as e:
        raise HTTPException(status_code=400, detail=f"Failed to parse EXP: {str(e)}")
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"OCR failed: {str(e)}")


@app.post("/ocr/hp", response_model=HpMpResponse)
async def recognize_hp(request: ImageRequest):
    """Recognize HP from image"""
    try:
        image = decode_base64_image(request.image_base64)
        result = ocr_engine(image)
        raw_text = extract_text_from_result(result)
        
        # Extract digits only
        digits = "".join(filter(str.isdigit, raw_text))
        
        if not digits:
            raise HTTPException(status_code=400, detail=f"No digits found in HP image: '{raw_text}'")
        
        hp = int(digits)
        
        return HpMpResponse(
            value=hp,
            raw_text=raw_text
        )
    
    except ValueError as e:
        raise HTTPException(status_code=400, detail=f"Failed to parse HP: {str(e)}")
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"OCR failed: {str(e)}")


@app.post("/ocr/mp", response_model=HpMpResponse)
async def recognize_mp(request: ImageRequest):
    """Recognize MP from image"""
    try:
        image = decode_base64_image(request.image_base64)
        result = ocr_engine(image)
        raw_text = extract_text_from_result(result)
        
        # Extract digits only
        digits = "".join(filter(str.isdigit, raw_text))
        
        if not digits:
            raise HTTPException(status_code=400, detail=f"No digits found in MP image: '{raw_text}'")
        
        mp = int(digits)
        
        return HpMpResponse(
            value=mp,
            raw_text=raw_text
        )
    
    except ValueError as e:
        raise HTTPException(status_code=400, detail=f"Failed to parse MP: {str(e)}")
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"OCR failed: {str(e)}")


def _run_ocr_sync(image: np.ndarray) -> str:
    """Synchronous OCR function to run in thread pool"""
    result = ocr_engine(image)
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
