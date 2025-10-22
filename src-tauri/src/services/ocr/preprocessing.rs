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

    /// Invert image colors (white text becomes black, black becomes white)
    pub fn invert(&self, image: &DynamicImage) -> DynamicImage {
        use image::imageops;
        let mut img = image.clone();
        imageops::invert(&mut img);
        img
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

    /// Extract white pixels from HSV color space (for level/exp UI)
    /// Matches legacy Python implementation exactly:
    /// lower_white = np.array([0, 0, 180])
    /// upper_white = np.array([179, 50, 255])
    pub fn extract_white_hsv(&self, image: &DynamicImage) -> DynamicImage {
        let rgb_img = image.to_rgb8();
        let (width, height) = rgb_img.dimensions();

        // Create mask for white pixels
        let mask = ImageBuffer::from_fn(width, height, |x, y| {
            let pixel = rgb_img.get_pixel(x, y);
            // Convert RGB to BGR order for OpenCV-compatible HSV (R=pixel[0], G=pixel[1], B=pixel[2])
            let (h, s, v) = Self::rgb_to_hsv(pixel[2], pixel[1], pixel[0]);

            // White range in HSV (legacy exact values): S=[0,50], V=[180,255]
            let is_white = s <= 50 && v >= 180;

            if is_white {
                Luma([255u8])
            } else {
                Luma([0u8])
            }
        });

        DynamicImage::ImageLuma8(mask)
    }

    /// Preprocess for level ROI (white + orange extraction)
    /// Orange numbers (88) + white text (LV.)
    pub fn preprocess_level(&self, image: &DynamicImage) -> Result<DynamicImage, String> {
        let rgb_img = image.to_rgb8();
        let (width, height) = rgb_img.dimensions();

        // Create mask for white + orange pixels
        let mask = ImageBuffer::from_fn(width, height, |x, y| {
            let pixel = rgb_img.get_pixel(x, y);
            // Convert RGB to BGR order for OpenCV-compatible HSV (R=pixel[0], G=pixel[1], B=pixel[2])
            let (h, s, v) = Self::rgb_to_hsv(pixel[2], pixel[1], pixel[0]);

            // White: S=[0,50], V=[180,255]
            let is_white = s <= 50 && v >= 180;

            // Orange (level numbers): H=[10,25], S=[100,255], V=[150,255]
            // Orange is red-yellow with high saturation
            let is_orange = h >= 10 && h <= 25 && s >= 100 && v >= 150;

            if is_white || is_orange {
                Luma([255u8])
            } else {
                Luma([0u8])
            }
        });

        let mask_img = DynamicImage::ImageLuma8(mask);

        // Resize 2x for better OCR (legacy uses fx=2, fy=2, INTER_CUBIC)
        let resized = self.scale(&mask_img, 2.0);

        // GaussianBlur (3x3, sigma=0) - legacy final step
        let blurred = self.gaussian_blur(&resized, 3);

        // Debug: Save processed image
        if let Err(e) = blurred.save("/tmp/level.png") {
            eprintln!("❌ Failed to save level.png: {}", e);
        } else {
            println!("✅ Saved level.png (preprocessed)");
        }

        Ok(blurred)
    }

    /// Preprocess for EXP ROI (white + green extraction)
    /// Matches legacy Python preprocessing exactly
    pub fn preprocess_exp(&self, image: &DynamicImage) -> Result<DynamicImage, String> {
        let rgb_img = image.to_rgb8();
        let (width, height) = rgb_img.dimensions();

        // Create mask for white + green pixels
        let mask = ImageBuffer::from_fn(width, height, |x, y| {
            let pixel = rgb_img.get_pixel(x, y);
            // Convert RGB to BGR order for OpenCV-compatible HSV (R=pixel[0], G=pixel[1], B=pixel[2])
            let (h, s, v) = Self::rgb_to_hsv(pixel[2], pixel[1], pixel[0]);

            // White: exact legacy threshold S=[0,50], V=[180,255]
            let is_white = s <= 50 && v >= 180;

            // Green (brackets): H=[35,85], S=[40,255], V=[80,255]
            let is_green = h >= 35 && h <= 85 && s >= 40 && v >= 80;

            if is_white || is_green {
                Luma([255u8])
            } else {
                Luma([0u8])
            }
        });

        let mask_img = DynamicImage::ImageLuma8(mask);

        // Resize 2x for better OCR (legacy uses fx=2, fy=2, INTER_CUBIC)
        let resized = self.scale(&mask_img, 2.0);

        // GaussianBlur (3x3, sigma=0) - legacy final step
        let blurred = self.gaussian_blur(&resized, 3);

        // Debug: Save processed image
        if let Err(e) = blurred.save("/tmp/exp.png") {
            eprintln!("❌ Failed to save exp.png: {}", e);
        } else {
            println!("✅ Saved exp.png (preprocessed)");
        }

        Ok(blurred)
    }

    /// Apply Gaussian blur (matches legacy cv2.GaussianBlur)
    pub fn gaussian_blur(&self, image: &DynamicImage, kernel_size: u32) -> DynamicImage {
        use imageproc::filter::gaussian_blur_f32;

        let gray_img = image.to_luma8();

        // Legacy uses (3,3) kernel with sigma=0 (auto-calculated from kernel size)
        // imageproc uses sigma directly, so we calculate it
        // OpenCV sigma calculation: sigma = 0.3*((ksize-1)*0.5 - 1) + 0.8
        let sigma = 0.3 * ((kernel_size as f32 - 1.0) * 0.5 - 1.0) + 0.8;

        let blurred = gaussian_blur_f32(&gray_img, sigma);

        DynamicImage::ImageLuma8(blurred)
    }

    /// Preprocess for HP/MP ROI (white text with black outline extraction)
    /// Extracts bright pixels while excluding pure white background
    ///
    /// Image type detection:
    /// - If image is roughly square or close to 112x112 (HiDPI): full HP/MP box
    ///   → Crop to bottom half (where numbers are)
    /// - Otherwise: user selected numbers only
    ///   → Use full image
    pub fn preprocess_hp_mp(&self, image: &DynamicImage) -> Result<DynamicImage, String> {
        let (width, height) = image.dimensions();

        // Detect if this is a full HP/MP box or numbers-only selection
        // Full box: roughly square (aspect ratio close to 1.0) or close to 112x112
        let aspect_ratio = width as f32 / height as f32;
        let is_square = aspect_ratio >= 0.8 && aspect_ratio <= 1.25;
        let is_near_112 = (width as i32 - 112).abs() < 20 && (height as i32 - 112).abs() < 20;
        let is_full_box = is_square || is_near_112;

        // If full box, crop to bottom-right (numbers area, excluding icon on left)
        let cropped_image = if is_full_box {
            // Crop to bottom half
            let crop_y = height / 2;
            let crop_height = height - crop_y;

            // Also crop from left to remove icon (icon is roughly 30-40% of width)
            let crop_x = width * 35 / 100;  // Start at 35% from left
            let crop_width = width - crop_x;

            image.crop_imm(crop_x, crop_y, crop_width, crop_height)
        } else {
            image.clone()
        };

        let rgb_img = cropped_image.to_rgb8();
        let (crop_width, crop_height) = rgb_img.dimensions();

        // Apply binary threshold - keep bright pixels, exclude pure white background
        let mask = ImageBuffer::from_fn(crop_width, crop_height, |x, y| {
            let pixel = rgb_img.get_pixel(x, y);
            let r = pixel[0];
            let g = pixel[1];
            let b = pixel[2];

            // Calculate min and max channels
            let min_val = r.min(g).min(b);

            // Keep pixels that are bright (200+) but NOT pure white
            // Pure white has all channels very similar and very high (250+)
            let is_very_bright = min_val >= 250;
            let is_bright = min_val >= 200;

            if is_bright && !is_very_bright {
                Luma([255u8])
            } else {
                Luma([0u8])
            }
        });

        let mask_img = DynamicImage::ImageLuma8(mask);

        // Apply dilation to expand white pixels (make text thicker and more connected)
        let dilated = self.dilate(&mask_img, 1);

        // Resize 5x for better OCR (these images are very small, need more scaling)
        let resized = self.scale(&dilated, 5.0);

        // Light blur to smooth edges
        let blurred = self.gaussian_blur(&resized, 3);

        Ok(blurred)
    }

    /// Convert RGB to HSV
    fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
        let r = r as f32 / 255.0;
        let g = g as f32 / 255.0;
        let b = b as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        // Hue calculation
        let h = if delta == 0.0 {
            0.0
        } else if max == r {
            60.0 * (((g - b) / delta) % 6.0)
        } else if max == g {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };

        let h = if h < 0.0 { h + 360.0 } else { h };
        let h = (h / 2.0) as u8; // OpenCV uses 0-179 for H

        // Saturation calculation
        let s = if max == 0.0 {
            0.0
        } else {
            delta / max
        };
        let s = (s * 255.0) as u8;

        // Value calculation
        let v = (max * 255.0) as u8;

        (h, s, v)
    }

    /// Apply dilation to image (expands white areas, shrinks black areas)
    /// Makes white text thicker and more connected
    fn dilate(&self, image: &DynamicImage, iterations: u32) -> DynamicImage {
        use imageproc::morphology::dilate;
        use imageproc::distance_transform::Norm;

        let mut result = image.to_luma8();

        for _ in 0..iterations {
            result = dilate(&result, Norm::L1, 1);
        }

        DynamicImage::ImageLuma8(result)
    }

    /// Apply erosion to image (thickens black areas, shrinks white areas)
    fn erode(&self, image: &DynamicImage, iterations: u32) -> DynamicImage {
        use imageproc::morphology::erode;
        use imageproc::distance_transform::Norm;

        let mut result = image.to_luma8();

        for _ in 0..iterations {
            result = erode(&result, Norm::L1, 1);
        }

        DynamicImage::ImageLuma8(result)
    }

    /// Morphological opening (erosion followed by dilation)
    fn morphology_open(&self, image: &DynamicImage, iterations: u32) -> DynamicImage {
        use imageproc::morphology::{erode, dilate};
        use imageproc::distance_transform::Norm;

        let mut result = image.to_luma8();

        for _ in 0..iterations {
            result = erode(&result, Norm::L1, 1);
        }
        for _ in 0..iterations {
            result = dilate(&result, Norm::L1, 1);
        }

        DynamicImage::ImageLuma8(result)
    }

    /// Morphological closing (dilation followed by erosion)
    fn morphology_close(&self, image: &DynamicImage, iterations: u32) -> DynamicImage {
        use imageproc::morphology::{erode, dilate};
        use imageproc::distance_transform::Norm;

        let mut result = image.to_luma8();

        for _ in 0..iterations {
            result = dilate(&result, Norm::L1, 1);
        }
        for _ in 0..iterations {
            result = erode(&result, Norm::L1, 1);
        }

        DynamicImage::ImageLuma8(result)
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
