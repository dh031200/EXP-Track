#!/usr/bin/env python3
"""
FastAPI OCR Server using PaddleOCR
Provides REST API for OCR operations
"""
import base64
import io
import sys
import os
from pathlib import Path
from typing import Optional, Dict
import asyncio
from contextlib import asynccontextmanager

import numpy as np
from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from PIL import Image
from pydantic import BaseModel
from paddleocr import PaddleOCR

# Determine base path (works for both dev and bundled app)
if getattr(sys, 'frozen', False):
    # Running as PyInstaller bundle
    base_path = Path(sys._MEIPASS)
else:
    # Running as script
    base_path = Path(__file__).parent

print(f"📁 Base path: {base_path}")

# OCR engine
ocr_engine: Optional[PaddleOCR] = None

# Confidence threshold for OCR results
CONFIDENCE_THRESHOLD = 0.8


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Lifespan context manager for startup and shutdown events"""
    global ocr_engine

    # Startup
    print("🚀 Initializing PaddleOCR engine...")
    try:
        # Set CPU mode before initializing
        os.environ['CUDA_VISIBLE_DEVICES'] = '-1'  # Force CPU mode
        
        ocr_engine = PaddleOCR(
            use_doc_orientation_classify=False,  # Disable document orientation
            use_doc_unwarping=False,  # Disable document unwarping
            use_textline_orientation=False,  # Disable text line orientation
            lang='en'  # English for numbers
        )
        print("✅ PaddleOCR engine initialized successfully (CPU mode)")
    except Exception as e:
        print(f"❌ Failed to initialize PaddleOCR: {e}")
        raise

    yield

    # Shutdown
    print("🛑 Shutting down OCR server...")
    ocr_engine = None


# FastAPI app
app = FastAPI(title="PaddleOCR Server", lifespan=lifespan)

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


# Request/Response models
class ImageRequest(BaseModel):
    image: str  # Base64 encoded PNG image


class LevelResponse(BaseModel):
    level: int
    raw_text: str
    confidence: float


class ExpResponse(BaseModel):
    absolute: int
    percentage: float
    raw_text: str
    confidence: float


class PotionResponse(BaseModel):
    count: int
    confidence: float


# Helper functions
def decode_base64_image(base64_str: str) -> np.ndarray:
    """Decode base64 string to numpy array (OpenCV format)"""
    image_bytes = base64.b64decode(base64_str)
    image = Image.open(io.BytesIO(image_bytes))
    # Convert to RGB (PaddleOCR expects RGB)
    image_rgb = image.convert('RGB')
    # Convert to numpy array
    return np.array(image_rgb)


def run_ocr_with_confidence(image: np.ndarray) -> tuple[str, float]:
    """
    Run PaddleOCR and return text with confidence score
    Returns: (text, confidence)
    """
    if ocr_engine is None:
        raise RuntimeError("OCR engine not initialized")

    try:
        print(f"🔍 [DEBUG] Image shape: {image.shape}, dtype: {image.dtype}")
        
        # Save temporary image for PaddleOCR.predict()
        import tempfile
        with tempfile.NamedTemporaryFile(suffix='.png', delete=False) as tmp_file:
            tmp_path = tmp_file.name
            from PIL import Image as PILImage
            PILImage.fromarray(image).save(tmp_path)
        
        try:
            # Use predict() API instead of ocr()
            result = ocr_engine.predict(input=tmp_path)
            
            print(f"🔍 [DEBUG] OCR raw result: {result}")
            
            if not result:
                print("⚠️  OCR returned empty result")
                return ("", 0.0)
            
            # Collect all texts with confidence >= threshold
            texts = []
            confidences = []
            
            for res in result:
                # Access rec_texts and rec_scores from result
                if hasattr(res, 'rec_texts') and hasattr(res, 'rec_scores'):
                    for text, score in zip(res.rec_texts, res.rec_scores):
                        print(f"🔍 [DEBUG] Found text: '{text}' (score={score:.3f})")
                        if score >= CONFIDENCE_THRESHOLD:
                            texts.append(text)
                            confidences.append(score)
                        else:
                            print(f"⚠️  Ignoring low confidence result: '{text}' (score={score:.3f})")
            
            if not texts:
                print("⚠️  No text passed confidence threshold")
                return ("", 0.0)
            
            # Combine texts and calculate average confidence
            combined_text = " ".join(texts)
            avg_confidence = sum(confidences) / len(confidences) if confidences else 0.0
            
            print(f"✅ [DEBUG] Final result: '{combined_text}' (avg_conf={avg_confidence:.3f})")
            return (combined_text, avg_confidence)
            
        finally:
            # Clean up temporary file
            import os
            try:
                os.unlink(tmp_path)
            except:
                pass
    
    except Exception as e:
        print(f"❌ OCR error: {e}")
        import traceback
        traceback.print_exc()
        return ("", 0.0)


def parse_level(text: str) -> Optional[int]:
    """Extract level number from OCR text"""
    import re
    # Remove "LV" prefix and extract digits
    digits = re.findall(r'\d+', text)
    if digits:
        try:
            return int(digits[0])
        except ValueError:
            pass
    return None


def parse_exp(text: str) -> tuple[Optional[int], Optional[float]]:
    """Extract EXP absolute and percentage from OCR text"""
    import re
    
    # Look for patterns like: "123,456 [45.67%]" or "123456 45.67%"
    # Extract digits for absolute value
    text_clean = text.replace(',', '').replace(' ', '')
    
    # Find numbers
    numbers = re.findall(r'\d+\.?\d*', text_clean)
    
    if len(numbers) >= 2:
        try:
            absolute = int(float(numbers[0]))
            percentage = float(numbers[1])
            return (absolute, percentage)
        except ValueError:
            pass
    elif len(numbers) == 1:
        # Only one number found, might be absolute or percentage
        try:
            num = float(numbers[0])
            if num <= 100:
                # Likely a percentage
                return (None, num)
            else:
                # Likely absolute
                return (int(num), None)
        except ValueError:
            pass
    
    return (None, None)


def parse_potion_count(text: str) -> Optional[int]:
    """Extract potion count from OCR text"""
    import re
    # Extract only digits
    digits = ''.join(re.findall(r'\d', text))
    if digits:
        try:
            return int(digits)
        except ValueError:
            pass
    return None


# API Endpoints
@app.get("/health")
async def health_check():
    """Health check endpoint"""
    return {
        "status": "healthy",
        "engine": "PaddleOCR",
        "confidence_threshold": CONFIDENCE_THRESHOLD
    }


@app.post("/recognize/level", response_model=LevelResponse)
async def recognize_level(request: ImageRequest):
    """Recognize level from image"""
    try:
        image = decode_base64_image(request.image)
        text, confidence = run_ocr_with_confidence(image)
        
        if confidence < CONFIDENCE_THRESHOLD:
            raise HTTPException(status_code=422, detail=f"Low confidence: {confidence:.3f}")
        
        level = parse_level(text)
        if level is None:
            raise HTTPException(status_code=422, detail=f"Could not parse level from: '{text}'")
        
        return LevelResponse(
            level=level,
            raw_text=text,
            confidence=confidence
        )
    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/recognize/exp", response_model=ExpResponse)
async def recognize_exp(request: ImageRequest):
    """Recognize EXP from image"""
    try:
        image = decode_base64_image(request.image)
        text, confidence = run_ocr_with_confidence(image)
        
        if confidence < CONFIDENCE_THRESHOLD:
            raise HTTPException(status_code=422, detail=f"Low confidence: {confidence:.3f}")
        
        absolute, percentage = parse_exp(text)
        if absolute is None or percentage is None:
            raise HTTPException(status_code=422, detail=f"Could not parse EXP from: '{text}'")
        
        return ExpResponse(
            absolute=absolute,
            percentage=percentage,
            raw_text=text,
            confidence=confidence
        )
    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/recognize/hp_potion")
async def recognize_hp_potion(request: ImageRequest):
    """Recognize HP potion count from image"""
    try:
        image = decode_base64_image(request.image)
        text, confidence = run_ocr_with_confidence(image)
        
        if confidence < CONFIDENCE_THRESHOLD:
            raise HTTPException(status_code=422, detail=f"Low confidence: {confidence:.3f}")
        
        count = parse_potion_count(text)
        if count is None:
            raise HTTPException(status_code=422, detail=f"Could not parse potion count from: '{text}'")
        
        return PotionResponse(count=count, confidence=confidence)
    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/recognize/mp_potion")
async def recognize_mp_potion(request: ImageRequest):
    """Recognize MP potion count from image"""
    try:
        image = decode_base64_image(request.image)
        text, confidence = run_ocr_with_confidence(image)
        
        if confidence < CONFIDENCE_THRESHOLD:
            raise HTTPException(status_code=422, detail=f"Low confidence: {confidence:.3f}")
        
        count = parse_potion_count(text)
        if count is None:
            raise HTTPException(status_code=422, detail=f"Could not parse potion count from: '{text}'")
        
        return PotionResponse(count=count, confidence=confidence)
    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/shutdown")
async def shutdown():
    """Graceful shutdown endpoint"""
    async def shutdown_task():
        await asyncio.sleep(0.5)
        os._exit(0)
    
    asyncio.create_task(shutdown_task())
    return {"status": "shutting down"}


if __name__ == "__main__":
    import uvicorn
    import platform

    print(f"🌐 Starting PaddleOCR server on http://127.0.0.1:39835")
    print(f"📝 Python version: {sys.version}")
    print(f"💻 Platform: {platform.system()} {platform.machine()}")
    print(f"📦 Frozen: {getattr(sys, 'frozen', False)}")
    print(f"🎯 Confidence threshold: {CONFIDENCE_THRESHOLD}")

    # Fix Windows ProactorEventLoop connection reset errors
    if platform.system() == "Windows":
        asyncio.set_event_loop_policy(asyncio.WindowsSelectorEventLoopPolicy())

    try:
        uvicorn.run(app, host="127.0.0.1", port=39835, log_level="info")
    except Exception as e:
        print(f"❌ Failed to start server: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)

