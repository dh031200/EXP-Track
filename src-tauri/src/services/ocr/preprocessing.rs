use image::{DynamicImage, GenericImageView, ImageBuffer, Luma};
use crate::models::config::PreprocessingConfig;

/// Image preprocessing service for OCR optimization
pub struct PreprocessingService {
    config: PreprocessingConfig,
}

impl PreprocessingService {
    /// Create a new preprocessing service with custom configuration
    pub fn new(config: PreprocessingConfig) -> Self {
        Self { config }
    }

    /// Create a preprocessing service with default configuration
    pub fn default() -> Self {
        Self {
            config: PreprocessingConfig::default(),
        }
    }

    /// Full preprocessing pipeline: grayscale → scale → threshold
    pub fn preprocess(&self, image: &DynamicImage) -> Result<DynamicImage, String> {
        // Step 1: Convert to grayscale
        let gray = self.to_grayscale(image);

        // Step 2: Scale up for better OCR
        let scaled = self.scale(&gray, self.config.scale_factor);

        // Step 3: Binary thresholding
        let binary = self.threshold(&scaled);

        Ok(binary)
    }

    /// Convert image to grayscale
    pub fn to_grayscale(&self, image: &DynamicImage) -> DynamicImage {
        DynamicImage::ImageLuma8(image.to_luma8())
    }

    /// Scale image by factor
    pub fn scale(&self, image: &DynamicImage, factor: f64) -> DynamicImage {
        let (width, height) = image.dimensions();
        let new_width = (width as f64 * factor) as u32;
        let new_height = (height as f64 * factor) as u32;

        image.resize(new_width, new_height, image::imageops::FilterType::Lanczos3)
    }

    /// Apply binary thresholding (Otsu's method)
    pub fn threshold(&self, image: &DynamicImage) -> DynamicImage {
        use imageproc::contrast::otsu_level;

        let gray_img = image.to_luma8();
        let threshold_value = otsu_level(&gray_img);

        let binary = ImageBuffer::from_fn(gray_img.width(), gray_img.height(), |x, y| {
            let pixel = gray_img.get_pixel(x, y);
            if pixel[0] > threshold_value {
                Luma([255u8])
            } else {
                Luma([0u8])
            }
        });

        DynamicImage::ImageLuma8(binary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage, ImageBuffer as _};

    /// Helper: Create test RGB image
    fn create_test_rgb_image() -> DynamicImage {
        let img = RgbImage::from_fn(100, 50, |x, y| {
            let val = ((x + y) % 256) as u8;
            Rgb([val, val, val])
        });
        DynamicImage::ImageRgb8(img)
    }

    /// Helper: Create small test image for scaling
    fn create_small_test_image() -> DynamicImage {
        let img = RgbImage::from_fn(50, 20, |x, y| {
            Rgb([128, 128, 128])
        });
        DynamicImage::ImageRgb8(img)
    }

    // 🔴 RED Phase Tests - These should FAIL initially

    #[test]
    fn test_grayscale_conversion() {
        let service = PreprocessingService::default();
        let rgb_image = create_test_rgb_image();

        let gray = service.to_grayscale(&rgb_image);

        // Verify it's grayscale (Luma8)
        match gray {
            DynamicImage::ImageLuma8(_) => {
                // Success - grayscale image
            }
            _ => panic!("Expected grayscale image (Luma8), got {:?}", gray.color()),
        }
    }

    #[test]
    fn test_grayscale_preserves_dimensions() {
        let service = PreprocessingService::default();
        let rgb_image = create_test_rgb_image();
        let (orig_width, orig_height) = rgb_image.dimensions();

        let gray = service.to_grayscale(&rgb_image);
        let (gray_width, gray_height) = gray.dimensions();

        assert_eq!(gray_width, orig_width, "Width should be preserved");
        assert_eq!(gray_height, orig_height, "Height should be preserved");
    }

    #[test]
    fn test_upscaling_2x() {
        let config = PreprocessingConfig {
            scale_factor: 2.0,
            apply_blur: false,
            blur_radius: 0,
        };
        let service = PreprocessingService::new(config);
        let small = create_small_test_image();

        let scaled = service.scale(&small, 2.0);

        assert_eq!(scaled.width(), 100, "Width should be doubled (50 * 2)");
        assert_eq!(scaled.height(), 40, "Height should be doubled (20 * 2)");
    }

    #[test]
    fn test_upscaling_3x() {
        let service = PreprocessingService::default();
        let small = create_small_test_image();

        let scaled = service.scale(&small, 3.0);

        assert_eq!(scaled.width(), 150, "Width should be tripled (50 * 3)");
        assert_eq!(scaled.height(), 60, "Height should be tripled (20 * 3)");
    }

    #[test]
    fn test_binary_threshold() {
        let service = PreprocessingService::default();
        let rgb_image = create_test_rgb_image();
        let gray = service.to_grayscale(&rgb_image);

        let binary = service.threshold(&gray);

        // Verify all pixels are either 0 or 255 (binary)
        match binary {
            DynamicImage::ImageLuma8(ref img) => {
                for pixel in img.pixels() {
                    let val = pixel[0];
                    assert!(
                        val == 0 || val == 255,
                        "Pixel value should be 0 or 255, got {}",
                        val
                    );
                }
            }
            _ => panic!("Expected Luma8 image after thresholding"),
        }
    }

    #[test]
    fn test_full_preprocessing_pipeline() {
        let service = PreprocessingService::default();
        let rgb_image = create_test_rgb_image();

        let result = service.preprocess(&rgb_image);

        assert!(result.is_ok(), "Preprocessing should succeed");

        let processed = result.unwrap();

        // Should be grayscale
        match processed {
            DynamicImage::ImageLuma8(_) => {},
            _ => panic!("Preprocessed image should be grayscale"),
        }

        // Should be scaled (2x by default)
        assert_eq!(processed.width(), 200, "Should be scaled 2x");
        assert_eq!(processed.height(), 100, "Should be scaled 2x");
    }

    #[test]
    fn test_preprocessing_with_custom_scale() {
        let config = PreprocessingConfig {
            scale_factor: 3.0,
            apply_blur: false,
            blur_radius: 0,
        };
        let service = PreprocessingService::new(config);
        let rgb_image = create_test_rgb_image();

        let result = service.preprocess(&rgb_image);

        assert!(result.is_ok());
        let processed = result.unwrap();

        // Should be scaled 3x
        assert_eq!(processed.width(), 300);
        assert_eq!(processed.height(), 150);
    }
}
