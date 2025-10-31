use image::{DynamicImage, GrayImage, ImageBuffer, Luma};
use std::path::Path;
use rayon::prelude::*;

/// Template for digit recognition
#[derive(Debug, Clone)]
pub struct Template {
    pub digit: u8,
    pub image: GrayImage,
    pub name: String,
}

/// Bounding box for detected digits
#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Digit match result with confidence
#[derive(Debug, Clone)]
pub struct DigitMatch {
    pub digit: u8,
    pub similarity: f32,
    pub template_name: String,
    pub position: (u32, u32),
}

/// Template matcher for OCR using template matching
pub struct TemplateMatcher {
    templates: Vec<Template>,
}

impl TemplateMatcher {
    /// Create a new template matcher
    pub fn new() -> Self {
        Self {
            templates: Vec::new(),
        }
    }

    /// Load templates from a directory
    pub fn load_templates<P: AsRef<Path>>(&mut self, template_dir: P) -> Result<(), String> {
        let template_dir = template_dir.as_ref();

        if !template_dir.exists() {
            return Err(format!("Template directory not found: {:?}", template_dir));
        }

        let mut _loaded_count = 0;
        let entries = std::fs::read_dir(template_dir)
            .map_err(|e| format!("Failed to read template directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            // Only process PNG files
            if path.extension().and_then(|s| s.to_str()) != Some("png") {
                continue;
            }

            // Extract digit from filename (first character)
            if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                if let Some(digit_char) = filename.chars().next() {
                    if let Some(digit) = digit_char.to_digit(10) {
                        // Load image
                        let img = image::open(&path)
                            .map_err(|e| format!("Failed to load template {:?}: {}", path, e))?;
                        
                        // Convert to grayscale
                        let gray = img.to_luma8();
                        
                        // Verify dimensions (35x41)
                        if gray.width() != 35 || gray.height() != 41 {
                            continue;
                        }

                        self.templates.push(Template {
                            digit: digit as u8,
                            image: gray,
                            name: filename.to_string(),
                        });
                        
                        _loaded_count += 1;
                    }
                }
            }
        }

        Ok(())
    }

    /// Extract orange boxes from image using HSV color filtering (parallel processing)
    pub fn extract_orange_boxes(&self, image: &DynamicImage) -> Result<GrayImage, String> {
        let rgb_image = image.to_rgb8();
        let (width, height) = rgb_image.dimensions();

        // Process rows in parallel
        let mask_data: Vec<u8> = (0..height)
            .into_par_iter()
            .flat_map(|y| {
                let mut row_data = Vec::with_capacity(width as usize);
                for x in 0..width {
                    let pixel = rgb_image.get_pixel(x, y);
                    let (h, s, v) = rgb_to_hsv(pixel[0], pixel[1], pixel[2]);

                    // Orange color range (matching Python OpenCV)
                    // H[8-28]: orange hue range
                    // S[180-255]: high saturation
                    // V[180-255]: high brightness
                    if h >= 8.0 && h <= 28.0 && s >= 180.0 && v >= 180.0 {
                        row_data.push(255u8);
                    } else {
                        row_data.push(0u8);
                    }
                }
                row_data
            })
            .collect();

        // Create mask from processed data
        let mask = GrayImage::from_raw(width, height, mask_data)
            .ok_or("Failed to create mask from parallel processing")?;

        Ok(mask)
    }

    /// Find digit boxes with aspect ratio filtering
    pub fn find_digit_boxes(&self, mask: &GrayImage) -> Result<Vec<BoundingBox>, String> {
        // Find connected components (simple flood fill approach)
        let contours = find_contours(mask);

        let mut digit_boxes = Vec::new();

        // Aspect ratio range: 0.800 to 0.900
        let min_ratio = 0.800;
        let max_ratio = 0.900;

        for contour in contours.iter() {
            let bbox = get_bounding_box(&contour);

            if bbox.width == 0 || bbox.height == 0 {
                continue;
            }

            let ratio = bbox.width as f32 / bbox.height as f32;

            // Check aspect ratio only (no position filter)
            if ratio >= min_ratio && ratio <= max_ratio {
                digit_boxes.push(bbox.clone());
            }
        }

        Ok(digit_boxes)
    }

    /// Old method for backward compatibility (deprecated)
    pub fn find_digit_boxes_with_log(&self, _image: &DynamicImage, mask: &GrayImage) -> Result<(Vec<BoundingBox>, String), String> {
        let boxes = self.find_digit_boxes(mask)?;
        Ok((boxes, String::new()))
    }

    /// Extract white digit from box image (resize + binarize)
    pub fn extract_white_digit(&self, box_image: &DynamicImage) -> Result<GrayImage, String> {
        // Step 1: Resize to 35x41 (NEAREST for sharp edges)
        let resized = image::imageops::resize(
            &box_image.to_rgb8(),
            35,
            41,
            image::imageops::FilterType::Nearest,
        );
        
        // Step 2: Convert to grayscale
        let gray = DynamicImage::ImageRgb8(resized).to_luma8();
        
        // Step 3: Binarize with threshold 200
        let binary = ImageBuffer::from_fn(35, 41, |x, y| {
            let pixel = gray.get_pixel(x, y);
            if pixel[0] > 200 {
                Luma([255u8])
            } else {
                Luma([0u8])
            }
        });
        
        Ok(binary)
    }

    /// Calculate exact pixel similarity between two images
    pub fn calculate_similarity(&self, img1: &GrayImage, img2: &GrayImage) -> f32 {
        if img1.dimensions() != img2.dimensions() {
            return 0.0;
        }
        
        let total_pixels = (img1.width() * img1.height()) as f32;
        let mut exact_match = 0;
        
        for (p1, p2) in img1.pixels().zip(img2.pixels()) {
            if p1[0] == p2[0] {
                exact_match += 1;
            }
        }
        
        (exact_match as f32 / total_pixels) * 100.0
    }

    /// Match digit with highest similarity template (must be >= 95%)
    pub fn match_digit(&self, digit_image: &GrayImage) -> Result<Option<DigitMatch>, String> {
        let mut max_similarity = 0.0;
        let mut best_digit = None;
        let mut best_template_name = None;
        
        for template in &self.templates {
            let similarity = self.calculate_similarity(digit_image, &template.image);
            
            if similarity > max_similarity {
                max_similarity = similarity;
                best_digit = Some(template.digit);
                best_template_name = Some(template.name.clone());
            }
        }
        
        // Reject if similarity is below 95%
        if max_similarity < 95.0 {
            return Ok(None);
        }
        
        Ok(Some(DigitMatch {
            digit: best_digit.unwrap(),
            similarity: max_similarity,
            template_name: best_template_name.unwrap(),
            position: (0, 0), // Will be set by caller
        }))
    }

    /// Recognize level number from image
    pub fn recognize_level(&self, image: &DynamicImage) -> Result<u32, String> {
        // Find orange boxes
        let mask = self.extract_orange_boxes(image)?;

        // Find boxes
        let mut boxes = self.find_digit_boxes(&mask)?;

        if boxes.is_empty() {
            return Err("No digit boxes found".to_string());
        }

        // Sort left to right
        boxes.sort_by_key(|b| b.x);

        // Match each digit
        let mut digits = Vec::new();

        for bbox in boxes.iter() {
            // Extract box without padding
            let box_img = image.crop_imm(
                bbox.x,
                bbox.y,
                bbox.width,
                bbox.height,
            );

            // Extract white digit
            let white_digit = self.extract_white_digit(&box_img)?;

            // Check white pixel ratio (9% ~ 21.5%)
            const MIN_WHITE_RATIO: f32 = 9.0;
            const MAX_WHITE_RATIO: f32 = 21.5;

            let total_pixels = (35 * 41) as f32;
            let white_pixels = white_digit.pixels().filter(|p| p[0] == 255).count() as f32;
            let white_ratio = (white_pixels / total_pixels) * 100.0;

            if white_ratio < MIN_WHITE_RATIO || white_ratio > MAX_WHITE_RATIO {
                continue; // Skip this box
            }

            // Match digit
            if let Some(mut digit_match) = self.match_digit(&white_digit)? {
                digit_match.position = (bbox.x, bbox.y);
                digits.push(digit_match.digit);
            }
        }

        if digits.is_empty() {
            return Err("No digits matched with sufficient similarity".to_string());
        }
        
        // Combine digits to form level number
        let level_str: String = digits.iter().map(|d| d.to_string()).collect();
        let level = level_str.parse::<u32>()
            .map_err(|e| format!("Failed to parse level number: {}", e))?;

        // Validate level range (1-300 for MapleStory)
        if level < 1 || level > 300 {
            return Err(format!("Invalid level range: {} (expected 1-300)", level));
        }

        Ok(level)
    }
}

/// Convert RGB to HSV color space
/// Returns (H: 0-360, S: 0-255, V: 0-255)
fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
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
    
    // Saturation calculation
    let s = if max == 0.0 { 0.0 } else { (delta / max) * 255.0 };
    
    // Value calculation
    let v = max * 255.0;
    
    (h, s, v)
}

/// Find connected components in binary mask (simple approach)
fn find_contours(mask: &GrayImage) -> Vec<Vec<(u32, u32)>> {
    let (width, height) = mask.dimensions();
    let mut visited = vec![vec![false; width as usize]; height as usize];
    let mut contours = Vec::new();
    
    for y in 0..height {
        for x in 0..width {
            if mask.get_pixel(x, y)[0] > 128 && !visited[y as usize][x as usize] {
                let contour = flood_fill(mask, &mut visited, x, y);
                if !contour.is_empty() {
                    contours.push(contour);
                }
            }
        }
    }
    
    contours
}

/// Flood fill to find connected component
fn flood_fill(
    mask: &GrayImage,
    visited: &mut Vec<Vec<bool>>,
    start_x: u32,
    start_y: u32,
) -> Vec<(u32, u32)> {
    let (width, height) = mask.dimensions();
    let mut stack = vec![(start_x, start_y)];
    let mut contour = Vec::new();
    
    while let Some((x, y)) = stack.pop() {
        if x >= width || y >= height || visited[y as usize][x as usize] {
            continue;
        }
        
        if mask.get_pixel(x, y)[0] <= 128 {
            continue;
        }
        
        visited[y as usize][x as usize] = true;
        contour.push((x, y));
        
        // Add neighbors
        if x > 0 { stack.push((x - 1, y)); }
        if x < width - 1 { stack.push((x + 1, y)); }
        if y > 0 { stack.push((x, y - 1)); }
        if y < height - 1 { stack.push((x, y + 1)); }
    }
    
    contour
}

/// Get bounding box from contour points
fn get_bounding_box(contour: &[(u32, u32)]) -> BoundingBox {
    if contour.is_empty() {
        return BoundingBox { x: 0, y: 0, width: 0, height: 0 };
    }
    
    let min_x = contour.iter().map(|(x, _)| *x).min().unwrap();
    let max_x = contour.iter().map(|(x, _)| *x).max().unwrap();
    let min_y = contour.iter().map(|(_, y)| *y).min().unwrap();
    let max_y = contour.iter().map(|(_, y)| *y).max().unwrap();
    
    BoundingBox {
        x: min_x,
        y: min_y,
        width: max_x - min_x + 1,
        height: max_y - min_y + 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_to_hsv() {
        // Test pure red
        let (h, s, v) = rgb_to_hsv(255, 0, 0);
        assert!((h - 0.0).abs() < 1.0);
        assert!((s - 255.0).abs() < 1.0);
        assert!((v - 255.0).abs() < 1.0);
        
        // Test orange (typical game UI)
        let (h, s, v) = rgb_to_hsv(255, 140, 0);
        assert!(h >= 8.0 && h <= 40.0); // Orange hue range
        assert!(s > 180.0); // High saturation
        assert!(v > 180.0); // High value
    }

    #[test]
    fn test_similarity_calculation() {
        let matcher = TemplateMatcher::new();
        
        // Create two identical 10x10 images
        let img1 = GrayImage::from_pixel(10, 10, Luma([255u8]));
        let img2 = GrayImage::from_pixel(10, 10, Luma([255u8]));
        
        let similarity = matcher.calculate_similarity(&img1, &img2);
        assert_eq!(similarity, 100.0);
        
        // Create different images
        let img3 = GrayImage::from_pixel(10, 10, Luma([0u8]));
        let similarity = matcher.calculate_similarity(&img1, &img3);
        assert_eq!(similarity, 0.0);
    }
}
