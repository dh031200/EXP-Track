use image::DynamicImage;

/// OCR Engine trait - abstraction for different OCR implementations
pub trait OcrEngine: Send + Sync {
    /// Recognize text from image with default language (English)
    fn recognize(&self, image: &DynamicImage) -> Result<String, String>;

    /// Recognize text with specific language
    fn recognize_with_lang(&self, image: &DynamicImage, lang: &str) -> Result<String, String>;

    /// Check if the OCR engine is available
    fn is_available() -> bool
    where
        Self: Sized;
}
