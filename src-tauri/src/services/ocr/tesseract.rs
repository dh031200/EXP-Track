use image::DynamicImage;
use tesseract::{Tesseract, PageSegMode};
use super::engine::OcrEngine;

/// Tesseract OCR engine implementation
pub struct TesseractEngine {
    // Tesseract instance will be created per-call for thread safety
}

impl TesseractEngine {
    /// Create a new Tesseract engine instance
    pub fn new() -> Result<Self, String> {
        // Verify Tesseract is available
        if !Self::is_available() {
            return Err("Tesseract not available on system".to_string());
        }

        Ok(Self {})
    }

    /// Configure Tesseract for single line recognition (game UI)
    fn configure_single_line(mut tesseract: Tesseract) -> Result<Tesseract, String> {
        tesseract.set_page_seg_mode(PageSegMode::PsmSingleLine);
        Ok(tesseract)
    }

    /// Set character whitelist for better accuracy
    fn set_whitelist(tesseract: Tesseract, whitelist: &str) -> Result<Tesseract, String> {
        tesseract
            .set_variable("tessedit_char_whitelist", whitelist)
            .map_err(|e| format!("Failed to set whitelist: {}", e))
    }
}

impl OcrEngine for TesseractEngine {
    fn recognize(&self, image: &DynamicImage) -> Result<String, String> {
        self.recognize_with_lang(image, "eng")
    }

    fn recognize_with_lang(&self, image: &DynamicImage, lang: &str) -> Result<String, String> {
        // Convert DynamicImage to bytes (PNG format for Tesseract)
        let mut img_bytes: Vec<u8> = Vec::new();
        image
            .write_to(&mut std::io::Cursor::new(&mut img_bytes), image::ImageFormat::Png)
            .map_err(|e| format!("Failed to encode image: {}", e))?;

        // Create and configure Tesseract instance
        let tesseract = Tesseract::new(None, Some(lang))
            .map_err(|e| format!("Failed to create Tesseract instance: {}", e))?;

        // Configure for single line (game UI text)
        let tesseract = Self::configure_single_line(tesseract)?;

        // Set image data and recognize text
        let text = tesseract
            .set_image_from_mem(&img_bytes)
            .map_err(|e| format!("Failed to set image: {}", e))?
            .get_text()
            .map_err(|e| format!("Failed to recognize text: {}", e))?;

        Ok(text)
    }

    fn is_available() -> bool {
        // Try to create a Tesseract instance with English
        Tesseract::new(None, Some("eng")).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{RgbImage, Rgb, DynamicImage as DynImg};

    /// Helper: Create test image with text (simulated)
    /// Note: This creates a simple colored rectangle, not actual text
    /// Real OCR testing requires actual images with text
    fn create_simple_test_image() -> DynImg {
        let img = RgbImage::from_fn(200, 50, |x, y| {
            if x % 2 == 0 {
                Rgb([255, 255, 255]) // White
            } else {
                Rgb([0, 0, 0]) // Black
            }
        });
        DynImg::ImageRgb8(img)
    }

    /// Helper: Load actual test image from fixtures
    fn load_fixture(filename: &str) -> Result<DynImg, String> {
        let path = format!("../../tests/fixtures/{}", filename);
        image::open(&path).map_err(|e| format!("Failed to load fixture {}: {}", filename, e))
    }

    // 🔴 RED Phase Tests - These should FAIL initially

    #[test]
    fn test_tesseract_engine_creation() {
        let result = TesseractEngine::new();
        assert!(result.is_ok(), "TesseractEngine creation should succeed");
    }

    #[test]
    fn test_tesseract_is_available() {
        let available = TesseractEngine::is_available();
        assert!(available, "Tesseract should be available on the system");
    }

    #[test]
    fn test_recognize_basic_text() {
        let engine = TesseractEngine::new().unwrap();
        let image = create_simple_test_image();

        let result = engine.recognize(&image);

        // Should return something (even if empty for simple pattern)
        assert!(result.is_ok(), "Recognition should not fail");
    }

    #[test]
    #[ignore] // Ignore until we have real test images
    fn test_recognize_level_from_fixture() {
        let engine = TesseractEngine::new().unwrap();

        // Try to load the level image
        if let Ok(image) = load_fixture("level_126.png") {
            let result = engine.recognize(&image);

            assert!(result.is_ok(), "Recognition should succeed");
            let text = result.unwrap();

            // Should contain "126"
            assert!(text.contains("126"), "Should recognize level number 126, got: {}", text);
        } else {
            // Skip test if fixture not available
            println!("Skipping: level_126.png not found");
        }
    }

    #[test]
    #[ignore] // Ignore until we have real test images
    fn test_recognize_korean_from_fixture() {
        let engine = TesseractEngine::new().unwrap();

        if let Ok(image) = load_fixture("map_korean.png") {
            let result = engine.recognize_with_lang(&image, "kor");

            assert!(result.is_ok(), "Korean recognition should succeed");
            let text = result.unwrap();

            // Should contain some Korean characters
            assert!(!text.is_empty(), "Should recognize Korean text, got: {}", text);
            // Note: Exact matching is hard due to OCR errors, just verify non-empty
        } else {
            println!("Skipping: map_korean.png not found");
        }
    }

    #[test]
    fn test_recognize_with_english_lang() {
        let engine = TesseractEngine::new().unwrap();
        let image = create_simple_test_image();

        let result = engine.recognize_with_lang(&image, "eng");
        assert!(result.is_ok(), "English language recognition should succeed");
    }

    #[test]
    fn test_empty_result_on_blank_image() {
        let engine = TesseractEngine::new().unwrap();

        // Create blank white image
        let blank = RgbImage::from_pixel(100, 50, Rgb([255, 255, 255]));
        let blank_img = DynImg::ImageRgb8(blank);

        let result = engine.recognize(&blank_img);

        assert!(result.is_ok(), "Recognition should succeed even on blank image");
        // Result might be empty or whitespace
    }
}
