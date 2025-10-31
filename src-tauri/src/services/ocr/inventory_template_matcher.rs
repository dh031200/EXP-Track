use image::{DynamicImage, GrayImage, ImageBuffer, Luma, Rgb, RgbImage, imageops};
use std::path::Path;
use std::collections::HashMap;
use rayon::prelude::*;

/// Template for digit recognition (inventory numbers)
#[derive(Debug, Clone)]
pub struct InventoryTemplate {
    pub digit: u8,
    pub image: GrayImage,
    pub name: String,
}

/// ROI (Region of Interest) for inventory slots
#[derive(Debug, Clone, Copy)]
pub struct SlotRoi {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Detection result for a single digit
#[derive(Debug, Clone)]
pub struct DigitDetection {
    pub digit: u8,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub score: f32,
    pub scale: f32,
}

/// Inventory template matcher for potion counting
pub struct InventoryTemplateMatcher {
    templates: Vec<InventoryTemplate>,
}

impl InventoryTemplateMatcher {
    /// Create a new inventory template matcher
    pub fn new() -> Self {
        Self {
            templates: Vec::new(),
        }
    }

    /// Calculate slot ROIs dynamically based on actual inventory image size
    /// Reference: 522x255px inventory with 4x2 grid layout
    fn calculate_slot_rois(width: u32, height: u32) -> HashMap<String, SlotRoi> {
        let mut rois = HashMap::new();

        // Row proportions (based on 522x255 reference)
        // Row 0: y=64-125 → top=0.2510, bottom=0.4902
        // Row 1: y=196-254 → top=0.7686, bottom=0.9961
        let row0_top = (height as f32 * 0.2510) as u32;
        let row0_bottom = (height as f32 * 0.4902) as u32;
        let row1_top = (height as f32 * 0.7686) as u32;
        let row1_bottom = (height as f32 * 0.9961) as u32;

        let row0_height = row0_bottom - row0_top;
        let row1_height = row1_bottom - row1_top;

        // Column proportions (based on 522 width reference)
        // Boundaries: 0, 130, 261, 391, 521 → 0.0, 0.2490, 0.5000, 0.7491, 0.9981
        let col0 = 0;
        let col1 = (width as f32 * 0.2490) as u32;
        let col2 = (width as f32 * 0.5000) as u32;
        let col3 = (width as f32 * 0.7491) as u32;
        let col4 = (width as f32 * 0.9981) as u32;

        // Row 0 (top row)
        rois.insert("shift".to_string(), SlotRoi { x: col0, y: row0_top, width: col1 - col0, height: row0_height });
        rois.insert("ins".to_string(),   SlotRoi { x: col1, y: row0_top, width: col2 - col1, height: row0_height });
        rois.insert("home".to_string(),  SlotRoi { x: col2, y: row0_top, width: col3 - col2, height: row0_height });
        rois.insert("pup".to_string(),   SlotRoi { x: col3, y: row0_top, width: col4 - col3, height: row0_height });

        // Row 1 (bottom row)
        rois.insert("ctrl".to_string(),  SlotRoi { x: col0, y: row1_top, width: col1 - col0, height: row1_height });
        rois.insert("del".to_string(),   SlotRoi { x: col1, y: row1_top, width: col2 - col1, height: row1_height });
        rois.insert("end".to_string(),   SlotRoi { x: col2, y: row1_top, width: col3 - col2, height: row1_height });
        rois.insert("pdn".to_string(),   SlotRoi { x: col3, y: row1_top, width: col4 - col3, height: row1_height });

        rois
    }

    /// Load digit templates from directory
    pub fn load_templates<P: AsRef<Path>>(&mut self, template_dir: P) -> Result<(), String> {
        let template_dir = template_dir.as_ref();

        if !template_dir.exists() {
            return Err(format!("Template directory not found: {:?}", template_dir));
        }

        let mut loaded_count = 0;
        let entries = std::fs::read_dir(template_dir)
            .map_err(|e| format!("Failed to read template directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            // Only process PNG files with "_threshold_1" suffix
            if path.extension().and_then(|s| s.to_str()) != Some("png") {
                continue;
            }

            if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                // Parse digit from filename (e.g., "0_threshold_1" -> 0)
                if let Some(digit_char) = filename.chars().next() {
                    if let Some(digit) = digit_char.to_digit(10) {
                        // Load image
                        let img = image::open(&path)
                            .map_err(|e| format!("Failed to load template {:?}: {}", path, e))?;

                        // Convert to grayscale
                        let gray = img.to_luma8();

                        self.templates.push(InventoryTemplate {
                            digit: digit as u8,
                            image: gray,
                            name: filename.to_string(),
                        });

                        loaded_count += 1;
                    }
                }
            }
        }

        if loaded_count == 0 {
            return Err("No templates loaded".to_string());
        }

        #[cfg(debug_assertions)]
        println!("✅ Loaded {} inventory digit templates", loaded_count);

        Ok(())
    }

    /// Detect inventory region from full screenshot with debug info
    /// Returns (inventory_image, (left, top, right, bottom))
    /// Note: Resizes to standard 522x255 for consistent template matching
    pub fn detect_inventory_region_with_coords(&self, image: &DynamicImage) -> Result<(DynamicImage, (u32, u32, u32, u32)), String> {
        // Step 1: Convert to grayscale
        let gray = image.to_luma8();
        let (width, height) = gray.dimensions();

        // Step 2: Binarization (threshold 70) - parallel processing
        let gray_data = gray.as_raw();
        let binary_data: Vec<u8> = gray_data
            .par_iter()
            .map(|&pixel| {
                if pixel < 70 {
                    255u8
                } else {
                    0u8
                }
            })
            .collect();

        let binary = GrayImage::from_raw(width, height, binary_data)
            .ok_or("Failed to create binary image from parallel processing")?;

        // Step 3: Find candidate regions via connected components
        let candidates = self.find_candidate_regions(&binary)?;

        if candidates.is_empty() {
            return Err("No inventory region candidates found".to_string());
        }

        // Step 4: Select rightmost-bottom region (inventory is at bottom-right)
        let (left, top, right, bottom) = candidates.iter()
            .max_by_key(|(_l, _t, r, b)| r + b)
            .ok_or("Failed to select inventory region")?;

        let inv_width = right - left + 1;
        let inv_height = bottom - top + 1;

        // Step 5: Crop inventory region from original grayscale
        let cropped_gray = imageops::crop_imm(&gray, *left, *top, inv_width, inv_height).to_image();

        // Step 6: Resize to standard 522x255 for consistent template matching
        // NEAREST preserves sharp edges for better digit recognition
        let resized_gray = image::imageops::resize(
            &cropped_gray,
            522,
            255,
            image::imageops::FilterType::Nearest,
        );

        // Step 7: Final threshold for OCR (threshold 20)
        // Dark pixels (< 20) become white (255) - digits
        // Bright pixels (≥ 20) become black (0) - background
        let final_binary = ImageBuffer::from_fn(522, 255, |x, y| {
            let pixel = resized_gray.get_pixel(x, y);
            if pixel[0] < 20 {
                Luma([255u8])  // Dark pixels → white
            } else {
                Luma([0u8])    // Bright pixels → black
            }
        });

        Ok((DynamicImage::ImageLuma8(final_binary), (*left, *top, *right, *bottom)))
    }

    /// Detect inventory region from full screenshot
    /// Returns 522x255 standardized inventory image
    pub fn detect_inventory_region(&self, image: &DynamicImage) -> Result<DynamicImage, String> {
        let (inventory_image, _coords) = self.detect_inventory_region_with_coords(image)?;
        Ok(inventory_image)
    }

    /// Apply morphological operations (closing: dilation -> erosion)
    fn apply_morphology(&self, binary: &GrayImage, kernel_size: u32) -> Result<GrayImage, String> {
        let (width, height) = binary.dimensions();

        // Dilation
        let mut dilated = binary.clone();
        for y in 0..height {
            for x in 0..width {
                if self.has_white_neighbor(binary, x, y, kernel_size) {
                    dilated.put_pixel(x, y, Luma([255u8]));
                }
            }
        }

        // Erosion
        let mut eroded = dilated.clone();
        for y in 0..height {
            for x in 0..width {
                if !self.all_white_neighbors(&dilated, x, y, kernel_size) {
                    eroded.put_pixel(x, y, Luma([0u8]));
                }
            }
        }

        Ok(eroded)
    }

    /// Check if pixel has at least one white neighbor in kernel
    fn has_white_neighbor(&self, image: &GrayImage, x: u32, y: u32, kernel_size: u32) -> bool {
        let (width, height) = image.dimensions();
        let half_k = kernel_size as i32 / 2;
        
        for dy in 0..kernel_size as i32 {
            for dx in 0..kernel_size as i32 {
                let nx = x as i32 - half_k + dx;
                let ny = y as i32 - half_k + dy;
                
                if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                    if image.get_pixel(nx as u32, ny as u32)[0] == 255 {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check if all neighbors in kernel are white
    fn all_white_neighbors(&self, image: &GrayImage, x: u32, y: u32, kernel_size: u32) -> bool {
        let (width, height) = image.dimensions();
        let half_k = kernel_size as i32 / 2;
        
        for dy in 0..kernel_size as i32 {
            for dx in 0..kernel_size as i32 {
                let nx = x as i32 - half_k + dx;
                let ny = y as i32 - half_k + dy;
                
                if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                    if image.get_pixel(nx as u32, ny as u32)[0] != 255 {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Find candidate regions using connected components
    fn find_candidate_regions(&self, binary: &GrayImage) -> Result<Vec<(u32, u32, u32, u32)>, String> {
        let (width, height) = binary.dimensions();
        let mut visited = vec![vec![false; width as usize]; height as usize];
        let mut candidates = Vec::new();

        for y in 0..height {
            for x in 0..width {
                if binary.get_pixel(x, y)[0] == 255 && !visited[y as usize][x as usize] {
                    let component = self.flood_fill(binary, &mut visited, x, y);

                    if component.is_empty() {
                        continue;
                    }

                    // Calculate bounding box
                    let left = component.iter().map(|(x, _)| *x).min().unwrap();
                    let right = component.iter().map(|(x, _)| *x).max().unwrap();
                    let top = component.iter().map(|(_, y)| *y).min().unwrap();
                    let bottom = component.iter().map(|(_, y)| *y).max().unwrap();

                    let comp_width = right - left + 1;
                    let comp_height = bottom - top + 1;

                    // Filter by size (150-600 width, 80-400 height)
                    if comp_width < 150 || comp_width > 600 {
                        continue;
                    }
                    if comp_height < 80 || comp_height > 400 {
                        continue;
                    }

                    // Filter by aspect ratio (1.5-2.5)
                    let ratio = comp_width as f32 / comp_height as f32;
                    if ratio < 1.5 || ratio > 2.5 {
                        continue;
                    }

                    candidates.push((left, top, right, bottom));
                }
            }
        }

        Ok(candidates)
    }

    /// Flood fill to find connected component
    fn flood_fill(&self, binary: &GrayImage, visited: &mut Vec<Vec<bool>>, start_x: u32, start_y: u32) -> Vec<(u32, u32)> {
        let (width, height) = binary.dimensions();
        let mut stack = vec![(start_x, start_y)];
        let mut component = Vec::new();

        while let Some((x, y)) = stack.pop() {
            if x >= width || y >= height || visited[y as usize][x as usize] {
                continue;
            }

            if binary.get_pixel(x, y)[0] != 255 {
                continue;
            }

            visited[y as usize][x as usize] = true;
            component.push((x, y));

            // Add 4-connected neighbors
            if x > 0 { stack.push((x - 1, y)); }
            if x < width - 1 { stack.push((x + 1, y)); }
            if y > 0 { stack.push((x, y - 1)); }
            if y < height - 1 { stack.push((x, y + 1)); }
        }

        component
    }

    /// Recognize potion count in specific slot
    pub fn recognize_count_in_slot(&self, inventory_image: &DynamicImage, slot_rois: &HashMap<String, SlotRoi>, slot: &str) -> Result<u32, String> {
        // Get ROI for slot
        let roi = slot_rois.get(slot)
            .ok_or(format!("Invalid slot: {}", slot))?;

        #[cfg(debug_assertions)]
        println!("    🎯 Processing slot [{}]: ROI(x={}, y={}, w={}, h={})",
            slot, roi.x, roi.y, roi.width, roi.height);

        // Convert to grayscale
        let gray = inventory_image.to_luma8();

        // Detect digits in ROI
        let detections = self.detect_digits_in_roi(&gray, roi, slot)?;

        if detections.is_empty() {
            return Ok(0); // Empty slot
        }

        // Sort detections left to right
        let mut sorted = detections;
        sorted.sort_by_key(|d| d.x);

        // Concatenate digits to form number
        let number_str: String = sorted.iter().map(|d| d.digit.to_string()).collect();
        let count = number_str.parse::<u32>()
            .map_err(|e| format!("Failed to parse potion count: {}", e))?;

        Ok(count)
    }

    /// Recognize counts in all 8 inventory slots
    /// Returns HashMap with slot names as keys and item counts as values
    pub fn recognize_all_slots(&self, inventory_image: &DynamicImage) -> Result<HashMap<String, u32>, String> {
        // Inventory image is always 522x255 after standardization
        let slot_rois = Self::calculate_slot_rois(522, 255);

        let mut results = HashMap::new();
        let slots = vec!["shift", "ins", "home", "pup", "ctrl", "del", "end", "pdn"];

        #[cfg(debug_assertions)]
        println!("    📦 Inventory slots (522x255):");

        for slot in slots {
            // Recognize count in this slot, default to 0 if recognition fails
            let count = self.recognize_count_in_slot(inventory_image, &slot_rois, slot).unwrap_or(0);

            #[cfg(debug_assertions)]
            println!("       {} = {}", slot, count);

            results.insert(slot.to_string(), count);
        }

        Ok(results)
    }

    /// Detect all digits in ROI using coordinate-based direct extraction
    /// Uses known digit positions based on center-alignment formula
    fn detect_digits_in_roi(&self, gray: &GrayImage, roi: &SlotRoi, _slot_name: &str) -> Result<Vec<DigitDetection>, String> {
        // Constants from coordinate analysis
        const WIDTH_1: f32 = 18.68;
        const WIDTH_OTHER: f32 = 30.0;
        const HEIGHT: u32 = 42;

        // Extract ROI
        let roi_image = image::imageops::crop_imm(
            gray,
            roi.x,
            roi.y,
            roi.width,
            roi.height,
        ).to_image();

        // Step 1: Find number region (white pixels bounding box)
        let number_bbox = match self.find_number_region(&roi_image) {
            Some(bbox) => bbox,
            None => return Ok(Vec::new()), // No digits found
        };

        let (bbox_x, bbox_y, bbox_width, bbox_height) = number_bbox;

        #[cfg(debug_assertions)]
        println!("      📏 Number region: x={}, y={}, w={}, h={}", bbox_x, bbox_y, bbox_width, bbox_height);

        // Step 2: Estimate digit count from width
        let digit_count = self.estimate_digit_count(bbox_width);

        #[cfg(debug_assertions)]
        println!("      🔢 Estimated digit count: {}", digit_count);

        if digit_count == 0 {
            return Ok(Vec::new());
        }

        // Step 3: Calculate start position (center alignment)
        let start_x = bbox_x;
        let start_y = bbox_y;

        // Step 4: Extract and match each digit position
        let mut detections = Vec::new();
        let mut current_x = start_x;

        for digit_idx in 0..digit_count {
            // Try both widths: 18.68 (digit 1) and 30.0 (others)
            let candidates = vec![
                (WIDTH_1, "narrow"),
                (WIDTH_OTHER, "wide"),
            ];

            let mut best_match: Option<(u8, f32, f32)> = None; // (digit, score, width)

            for (width, _width_type) in &candidates {
                let digit_width = *width as u32;

                // Check bounds
                if current_x + digit_width > roi_image.width() {
                    continue;
                }

                // Extract digit region
                let digit_region = imageops::crop_imm(
                    &roi_image,
                    current_x,
                    start_y,
                    digit_width,
                    HEIGHT.min(roi_image.height() - start_y),
                ).to_image();

                // Match against all templates
                for template in &self.templates {
                    // Resize template to match digit region size
                    let resized_template = imageops::resize(
                        &template.image,
                        digit_region.width(),
                        digit_region.height(),
                        imageops::FilterType::Lanczos3,
                    );

                    let score = self.calculate_similarity(&digit_region, &resized_template);

                    if score > best_match.as_ref().map(|(_, s, _)| *s).unwrap_or(0.0) {
                        best_match = Some((template.digit, score, *width));
                    }
                }
            }

            // Accept if score >= 0.70
            if let Some((digit, score, width)) = best_match {
                if score >= 0.70 {
                    #[cfg(debug_assertions)]
                    println!("      ✅ Position {}: digit={}, score={:.3}, width={:.2}",
                        digit_idx, digit, score, width);

                    detections.push(DigitDetection {
                        digit,
                        x: current_x + roi.x,
                        y: start_y + roi.y,
                        width: width as u32,
                        height: HEIGHT,
                        score,
                        scale: 1.0,
                    });

                    current_x += width as u32;
                } else {
                    #[cfg(debug_assertions)]
                    println!("      ❌ Position {}: score too low ({:.3})", digit_idx, score);
                    break;
                }
            } else {
                #[cfg(debug_assertions)]
                println!("      ❌ Position {}: no match found", digit_idx);
                break;
            }
        }

        Ok(detections)
    }

    /// Find number region in ROI (white pixels bounding box)
    fn find_number_region(&self, image: &GrayImage) -> Option<(u32, u32, u32, u32)> {
        let (width, height) = image.dimensions();

        let mut min_x = width;
        let mut max_x = 0;
        let mut min_y = height;
        let mut max_y = 0;
        let mut found = false;

        for y in 0..height {
            for x in 0..width {
                if image.get_pixel(x, y)[0] == 255 { // White pixel
                    min_x = min_x.min(x);
                    max_x = max_x.max(x);
                    min_y = min_y.min(y);
                    max_y = max_y.max(y);
                    found = true;
                }
            }
        }

        if found {
            let bbox_width = max_x - min_x + 1;
            let bbox_height = max_y - min_y + 1;
            Some((min_x, min_y, bbox_width, bbox_height))
        } else {
            None
        }
    }

    /// Estimate digit count from number region width
    fn estimate_digit_count(&self, width: u32) -> usize {
        // Based on coordinate analysis:
        // 1 digit: 18-30px
        // 2 digits: 37-60px
        // 3 digits: 56-90px
        // 4 digits: 74-120px

        if width >= 18 && width <= 30 {
            1
        } else if width >= 37 && width <= 60 {
            2
        } else if width >= 56 && width <= 90 {
            3
        } else if width >= 74 && width <= 120 {
            4
        } else {
            0 // Invalid width
        }
    }

    /// Calculate similarity between two images (exact pixel matching)
    fn calculate_similarity(&self, img1: &GrayImage, img2: &GrayImage) -> f32 {
        if img1.dimensions() != img2.dimensions() {
            return 0.0;
        }

        let total_pixels = (img1.width() * img1.height()) as f32;
        let mut matched_pixels = 0;

        for (p1, p2) in img1.pixels().zip(img2.pixels()) {
            if p1[0] == p2[0] {
                matched_pixels += 1;
            }
        }

        (matched_pixels as f32 / total_pixels) * 100.0
    }


    /// Get available slot names
    pub fn get_available_slots(&self) -> Vec<String> {
        vec![
            "shift".to_string(),
            "ins".to_string(),
            "home".to_string(),
            "pup".to_string(),
            "ctrl".to_string(),
            "del".to_string(),
            "end".to_string(),
            "pdn".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slot_rois_calculation() {
        // Test with reference size 522x255
        let slot_rois = InventoryTemplateMatcher::calculate_slot_rois(522, 255);
        assert_eq!(slot_rois.len(), 8);

        // Test specific slots
        assert!(slot_rois.contains_key("shift"));
        assert!(slot_rois.contains_key("pdn"));
    }

    #[test]
    fn test_get_available_slots() {
        let matcher = InventoryTemplateMatcher::new();
        let slots = matcher.get_available_slots();
        assert_eq!(slots.len(), 8);
        assert!(slots.contains(&"shift".to_string()));
    }
}
