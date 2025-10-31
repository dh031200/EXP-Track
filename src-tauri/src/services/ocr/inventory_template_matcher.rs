use image::{DynamicImage, GrayImage, ImageBuffer, Luma, imageops};
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

        // Step 6: Final threshold for OCR (threshold 1)
        // Dark pixels (< 1) become white (255) - digits
        // Bright pixels (≥ 1) become black (0) - background
        let final_binary = ImageBuffer::from_fn(inv_width, inv_height, |x, y| {
            let pixel = cropped_gray.get_pixel(x, y);
            if pixel[0] < 1 {
                Luma([255u8])  // Dark pixels → white
            } else {
                Luma([0u8])    // Bright pixels → black
            }
        });

        Ok((DynamicImage::ImageLuma8(final_binary), (*left, *top, *right, *bottom)))
    }

    /// Detect inventory region from full screenshot
    /// Returns original-size inventory image (no resize)
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

        // Convert to grayscale
        let gray = inventory_image.to_luma8();

        // Detect digits in ROI
        let detections = self.detect_digits_in_roi(&gray, roi)?;

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
        // Calculate slot ROIs dynamically based on actual inventory size
        let width = inventory_image.width();
        let height = inventory_image.height();
        let slot_rois = Self::calculate_slot_rois(width, height);

        let mut results = HashMap::new();
        let slots = vec!["shift", "ins", "home", "pup", "ctrl", "del", "end", "pdn"];

        #[cfg(debug_assertions)]
        println!("    📦 Inventory slots ({}x{}):", width, height);

        for slot in slots {
            // Recognize count in this slot, default to 0 if recognition fails
            let count = self.recognize_count_in_slot(inventory_image, &slot_rois, slot).unwrap_or(0);

            #[cfg(debug_assertions)]
            println!("       {} = {}", slot, count);

            results.insert(slot.to_string(), count);
        }

        Ok(results)
    }

    /// Detect all digits in ROI using multi-scale template matching
    fn detect_digits_in_roi(&self, gray: &GrayImage, roi: &SlotRoi) -> Result<Vec<DigitDetection>, String> {
        // Extract ROI
        let roi_image = image::imageops::crop_imm(
            gray,
            roi.x,
            roi.y,
            roi.width,
            roi.height,
        ).to_image();

        // Multi-scale template matching
        let scales = vec![0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2, 1.3];
        let threshold = 0.7;

        // Use rayon for parallel template matching across scales
        use rayon::prelude::*;

        // Create all (template, scale) combinations for parallel processing
        let mut combinations = Vec::new();
        for template in &self.templates {
            for &scale in &scales {
                combinations.push((template, scale));
            }
        }

        let all_detections: Vec<DigitDetection> = combinations.par_iter()
            .flat_map(|(template, scale)| {
                // Resize template
                let (tmpl_width, tmpl_height) = template.image.dimensions();
                let new_width = (tmpl_width as f32 * scale) as u32;
                let new_height = (tmpl_height as f32 * scale) as u32;

                if new_width < 5 || new_height < 5 {
                    return Vec::new();
                }
                if new_width > roi.width || new_height > roi.height {
                    return Vec::new();
                }

                let scaled_template = image::imageops::resize(
                    &template.image,
                    new_width,
                    new_height,
                    image::imageops::FilterType::Lanczos3,  // High quality for accurate recognition
                );

                // Template matching
                let matches = self.match_template(&roi_image, &scaled_template, threshold);

                matches.into_iter().map(|(x, y, score)| {
                    DigitDetection {
                        digit: template.digit,
                        x: x + roi.x,
                        y: y + roi.y,
                        width: new_width,
                        height: new_height,
                        score,
                        scale: *scale,
                    }
                }).collect()
            })
            .collect();

        // Apply NMS to remove overlapping detections
        let filtered = self.non_maximum_suppression(all_detections, 0.05)?;

        // Filter by height consistency
        let height_filtered = self.filter_by_height(filtered, 0.2)?;

        // Remove duplicates
        let final_detections = self.remove_duplicates(height_filtered, 5)?;

        Ok(final_detections)
    }

    /// Template matching using normalized cross-correlation
    fn match_template(&self, image: &GrayImage, template: &GrayImage, threshold: f32) -> Vec<(u32, u32, f32)> {
        let (img_width, img_height) = image.dimensions();
        let (tmpl_width, tmpl_height) = template.dimensions();

        if tmpl_width > img_width || tmpl_height > img_height {
            return Vec::new();
        }

        let mut matches = Vec::new();

        for y in 0..=(img_height - tmpl_height) {
            for x in 0..=(img_width - tmpl_width) {
                let score = self.calculate_ncc(image, template, x, y);
                if score >= threshold {
                    matches.push((x, y, score));
                }
            }
        }

        matches
    }

    /// Calculate normalized cross-correlation
    fn calculate_ncc(&self, image: &GrayImage, template: &GrayImage, x: u32, y: u32) -> f32 {
        let (tmpl_width, tmpl_height) = template.dimensions();

        let mut sum_img = 0.0;
        let mut sum_tmpl = 0.0;
        let mut sum_img_sq = 0.0;
        let mut sum_tmpl_sq = 0.0;
        let mut sum_prod = 0.0;
        let n = (tmpl_width * tmpl_height) as f32;

        for ty in 0..tmpl_height {
            for tx in 0..tmpl_width {
                let img_val = image.get_pixel(x + tx, y + ty)[0] as f32;
                let tmpl_val = template.get_pixel(tx, ty)[0] as f32;

                sum_img += img_val;
                sum_tmpl += tmpl_val;
                sum_img_sq += img_val * img_val;
                sum_tmpl_sq += tmpl_val * tmpl_val;
                sum_prod += img_val * tmpl_val;
            }
        }

        let mean_img = sum_img / n;
        let mean_tmpl = sum_tmpl / n;

        let numer = sum_prod - n * mean_img * mean_tmpl;
        let denom = ((sum_img_sq - n * mean_img * mean_img) * (sum_tmpl_sq - n * mean_tmpl * mean_tmpl)).sqrt();

        if denom == 0.0 {
            return 0.0;
        }

        (numer / denom).max(0.0)
    }

    /// Non-maximum suppression to remove overlapping detections
    fn non_maximum_suppression(&self, mut detections: Vec<DigitDetection>, overlap_threshold: f32) -> Result<Vec<DigitDetection>, String> {
        if detections.is_empty() {
            return Ok(Vec::new());
        }

        // Sort by score (descending)
        detections.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        let mut kept = Vec::new();

        for detection in detections {
            let mut overlaps = false;

            for kept_det in &kept {
                let overlap_area = self.calculate_overlap(&detection, kept_det);
                let min_area = (detection.width * detection.height).min(kept_det.width * kept_det.height) as f32;

                if overlap_area > overlap_threshold * min_area {
                    overlaps = true;
                    break;
                }
            }

            if !overlaps {
                kept.push(detection);
            }
        }

        Ok(kept)
    }

    /// Calculate overlap area between two detections
    fn calculate_overlap(&self, d1: &DigitDetection, d2: &DigitDetection) -> f32 {
        let x_overlap = (d1.x + d1.width).min(d2.x + d2.width) as i32 - d1.x.max(d2.x) as i32;
        let y_overlap = (d1.y + d1.height).min(d2.y + d2.height) as i32 - d1.y.max(d2.y) as i32;

        if x_overlap > 0 && y_overlap > 0 {
            (x_overlap * y_overlap) as f32
        } else {
            0.0
        }
    }

    /// Filter detections by height consistency
    fn filter_by_height(&self, detections: Vec<DigitDetection>, tolerance: f32) -> Result<Vec<DigitDetection>, String> {
        if detections.is_empty() {
            return Ok(Vec::new());
        }

        // Calculate median height
        let mut heights: Vec<u32> = detections.iter().map(|d| d.height).collect();
        heights.sort();
        let median_height = heights[heights.len() / 2];

        // Filter detections within tolerance
        let filtered: Vec<DigitDetection> = detections.into_iter()
            .filter(|d| {
                let diff = (d.height as i32 - median_height as i32).abs() as f32;
                diff <= median_height as f32 * tolerance
            })
            .collect();

        Ok(filtered)
    }

    /// Remove duplicate detections at same position
    fn remove_duplicates(&self, mut detections: Vec<DigitDetection>, position_tolerance: u32) -> Result<Vec<DigitDetection>, String> {
        if detections.is_empty() {
            return Ok(Vec::new());
        }

        // Sort by score (descending)
        detections.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        let mut kept: Vec<DigitDetection> = Vec::new();

        for detection in detections {
            let mut is_duplicate = false;

            for kept_det in &kept {
                // Check position proximity
                let x_diff = (detection.x as i32 - kept_det.x as i32).abs() as u32;
                let y_diff = (detection.y as i32 - kept_det.y as i32).abs() as u32;

                if x_diff <= position_tolerance && y_diff <= position_tolerance {
                    is_duplicate = true;
                    break;
                }

                // Check overlap
                let overlap = self.calculate_overlap(&detection, kept_det);
                let current_area = (detection.width * detection.height) as f32;
                let kept_area = (kept_det.width * kept_det.height) as f32;

                if overlap > 0.3 * current_area.min(kept_area) {
                    is_duplicate = true;
                    break;
                }
            }

            if !is_duplicate {
                kept.push(detection);
            }
        }

        Ok(kept)
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
