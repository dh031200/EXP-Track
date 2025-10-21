use crate::models::ocr_result::{ExpResult, LevelResult, MapResult};
use crate::services::ocr::{OcrEngine, PreprocessingService, TesseractEngine};
use crate::services::ocr::{parse_exp, parse_level, parse_map};
use base64::Engine as _;
use image::DynamicImage;
use std::sync::Mutex;
use tauri::State;

/// State wrapper for OCR service
pub type OcrServiceState = Mutex<OcrService>;

/// OCR service that combines preprocessing, OCR engine, and parsing
pub struct OcrService {
    preprocessor: PreprocessingService,
    engine: TesseractEngine,
}

impl OcrService {
    /// Create a new OCR service
    pub fn new() -> Result<Self, String> {
        let preprocessor = PreprocessingService::default();
        let engine = TesseractEngine::new()?;

        Ok(Self {
            preprocessor,
            engine,
        })
    }

    /// Recognize and parse level from image
    pub fn recognize_level(&self, image: &DynamicImage) -> Result<LevelResult, String> {
        // Preprocess image
        let processed = self.preprocessor.preprocess(image)?;

        // OCR recognition (English)
        let raw_text = self.engine.recognize(&processed)?;

        // Parse level
        let level = parse_level(&raw_text)?;

        Ok(LevelResult { level, raw_text })
    }

    /// Recognize and parse EXP from image
    pub fn recognize_exp(&self, image: &DynamicImage) -> Result<ExpResult, String> {
        // Preprocess image
        let processed = self.preprocessor.preprocess(image)?;

        // OCR recognition (English)
        let raw_text = self.engine.recognize(&processed)?;

        // Parse EXP
        let exp_data = parse_exp(&raw_text)?;

        Ok(ExpResult {
            absolute: exp_data.absolute,
            percentage: exp_data.percentage,
            raw_text,
        })
    }

    /// Recognize and parse map name from image
    pub fn recognize_map(&self, image: &DynamicImage) -> Result<MapResult, String> {
        // Preprocess image
        let processed = self.preprocessor.preprocess(image)?;

        // OCR recognition (Korean)
        let raw_text = self.engine.recognize_with_lang(&processed, "kor")?;

        // Parse map name
        let map_name = parse_map(&raw_text)?;

        Ok(MapResult {
            map_name,
            raw_text,
        })
    }
}

/// Initialize OCR service state
pub fn init_ocr_service() -> Result<OcrServiceState, String> {
    let service = OcrService::new()?;
    Ok(Mutex::new(service))
}

/// Decode base64 image to DynamicImage
fn decode_base64_image(base64_data: &str) -> Result<DynamicImage, String> {
    let image_bytes = base64::engine::general_purpose::STANDARD
        .decode(base64_data)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;

    let image = image::load_from_memory(&image_bytes)
        .map_err(|e| format!("Failed to load image: {}", e))?;

    Ok(image)
}

// ============================================================
// Tauri Commands
// ============================================================

/// Recognize level from base64-encoded image
#[tauri::command]
pub fn recognize_level(
    state: State<OcrServiceState>,
    image_base64: String,
) -> Result<LevelResult, String> {
    let service = state
        .lock()
        .map_err(|e| format!("Failed to lock OCR service: {}", e))?;

    let image = decode_base64_image(&image_base64)?;
    service.recognize_level(&image)
}

/// Recognize EXP from base64-encoded image
#[tauri::command]
pub fn recognize_exp(
    state: State<OcrServiceState>,
    image_base64: String,
) -> Result<ExpResult, String> {
    let service = state
        .lock()
        .map_err(|e| format!("Failed to lock OCR service: {}", e))?;

    let image = decode_base64_image(&image_base64)?;
    service.recognize_exp(&image)
}

/// Recognize map name from base64-encoded image
#[tauri::command]
pub fn recognize_map(
    state: State<OcrServiceState>,
    image_base64: String,
) -> Result<MapResult, String> {
    let service = state
        .lock()
        .map_err(|e| format!("Failed to lock OCR service: {}", e))?;

    let image = decode_base64_image(&image_base64)?;
    service.recognize_map(&image)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage};

    /// Helper: Create a simple test image
    fn create_test_image() -> DynamicImage {
        let img = RgbImage::from_fn(100, 50, |_x, _y| Rgb([255, 255, 255]));
        DynamicImage::ImageRgb8(img)
    }

    /// Helper: Encode image to base64
    fn encode_image_base64(image: &DynamicImage) -> String {
        let mut bytes: Vec<u8> = Vec::new();
        image
            .write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png)
            .unwrap();
        base64::engine::general_purpose::STANDARD.encode(&bytes)
    }

    // 🔴 RED Phase Tests

    #[test]
    fn test_ocr_service_creation() {
        let result = OcrService::new();
        assert!(result.is_ok(), "OcrService creation should succeed");
    }

    #[test]
    fn test_init_ocr_service_state() {
        let result = init_ocr_service();
        assert!(result.is_ok(), "OCR service state initialization should succeed");
    }

    #[test]
    fn test_decode_base64_image() {
        let image = create_test_image();
        let base64 = encode_image_base64(&image);

        let result = decode_base64_image(&base64);
        assert!(result.is_ok(), "Should decode base64 image");

        let decoded = result.unwrap();
        assert_eq!(decoded.width(), 100);
        assert_eq!(decoded.height(), 50);
    }

    #[test]
    fn test_decode_base64_image_invalid() {
        let result = decode_base64_image("invalid_base64!!!");
        assert!(result.is_err(), "Should fail on invalid base64");
    }

    #[test]
    #[ignore] // Requires actual test images with text
    fn test_recognize_level_integration() {
        // This test would need actual fixture images
        // Skipped for now, will test with real images later
    }

    #[test]
    #[ignore] // Requires actual test images with text
    fn test_recognize_exp_integration() {
        // This test would need actual fixture images
        // Skipped for now
    }

    #[test]
    #[ignore] // Requires actual test images with text
    fn test_recognize_map_integration() {
        // This test would need actual fixture images
        // Skipped for now
    }
}
