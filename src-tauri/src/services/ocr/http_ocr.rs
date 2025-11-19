use crate::models::ocr_result::{ExpResult, LevelResult};
use super::template_matcher::TemplateMatcher;
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use base64::{Engine as _, engine::general_purpose};
use regex::Regex;
use std::sync::Arc;

/// HTTP OCR client that communicates with Python FastAPI server
#[derive(Clone)]
pub struct HttpOcrClient {
    client: reqwest::Client,
    base_url: String,
    template_matcher: Option<Arc<TemplateMatcher>>,
}

#[derive(Serialize)]
struct ImageRequest {
    image_base64: String,
}

/// Single text box with bounding box coordinates
#[derive(Deserialize, Clone, Debug)]
struct TextBox {
    #[serde(rename = "box")]
    bbox: Vec<Vec<f64>>,  // 4 corner points [[x1,y1], [x2,y2], [x3,y3], [x4,y4]]
    text: String,
    score: f64,
}

/// Unified OCR response from Python server
#[derive(Deserialize)]
struct OcrResponse {
    boxes: Vec<TextBox>,
    raw_text: String,  // Legacy concatenated text
}

impl TextBox {
    /// Get bounding box as (x_min, y_min, x_max, y_max)
    fn get_bbox_rect(&self) -> (f64, f64, f64, f64) {
        let xs: Vec<f64> = self.bbox.iter().map(|p| p[0]).collect();
        let ys: Vec<f64> = self.bbox.iter().map(|p| p[1]).collect();

        let x_min = xs.iter().cloned().fold(f64::INFINITY, f64::min);
        let x_max = xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let y_min = ys.iter().cloned().fold(f64::INFINITY, f64::min);
        let y_max = ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        (x_min, y_min, x_max, y_max)
    }

    /// Compute IoU (Intersection over Union) with another box
    fn iou(&self, other: &TextBox) -> f64 {
        let (x1_min, y1_min, x1_max, y1_max) = self.get_bbox_rect();
        let (x2_min, y2_min, x2_max, y2_max) = other.get_bbox_rect();

        // Intersection
        let inter_x_min = x1_min.max(x2_min);
        let inter_y_min = y1_min.max(y2_min);
        let inter_x_max = x1_max.min(x2_max);
        let inter_y_max = y1_max.min(y2_max);

        if inter_x_max <= inter_x_min || inter_y_max <= inter_y_min {
            return 0.0; // No overlap
        }

        let inter_area = (inter_x_max - inter_x_min) * (inter_y_max - inter_y_min);

        // Union
        let box1_area = (x1_max - x1_min) * (y1_max - y1_min);
        let box2_area = (x2_max - x2_min) * (y2_max - y2_min);
        let union_area = box1_area + box2_area - inter_area;

        if union_area <= 0.0 {
            return 0.0;
        }

        inter_area / union_area
    }

    /// Get leftmost x-coordinate (for left-to-right sorting)
    fn left_x(&self) -> f64 {
        self.bbox.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min)
    }

    /// Get box area
    fn area(&self) -> f64 {
        let (x_min, y_min, x_max, y_max) = self.get_bbox_rect();
        (x_max - x_min) * (y_max - y_min)
    }
}

/// Response for number classification
#[derive(Deserialize)]
struct NumberResponse {
    value: u32,
}

impl HttpOcrClient {
    /// Create a new HTTP OCR client
    pub fn new() -> Result<Self, String> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        Ok(Self {
            client,
            base_url: "http://127.0.0.1:39835".to_string(),
            template_matcher: None,
        })
    }

    /// Initialize template matcher with resource directory
    pub fn init_template_matcher(&mut self, template_dir: &str) -> Result<(), String> {
        let mut matcher = TemplateMatcher::new();
        matcher.load_templates(template_dir)
            .map_err(|e| format!("Failed to load templates: {}", e))?;

        self.template_matcher = Some(Arc::new(matcher));
        Ok(())
    }

    /// Detect Level ROI by recognizing level digits
    /// Returns (left, top, right, bottom, matched_boxes) where matched_boxes are successfully recognized digit boxes
    pub fn detect_level_roi_with_boxes(&self, image: &DynamicImage) -> Result<(u32, u32, u32, u32, Vec<super::template_matcher::BoundingBox>), String> {
        let matcher = self.template_matcher.as_ref()
            .ok_or("Template matcher not initialized")?;

        // Recognize level and get matched boxes
        let (_level, matched_boxes) = matcher.recognize_level_with_boxes(image)?;

        if matched_boxes.is_empty() {
            return Err("No digit boxes matched for ROI detection".to_string());
        }

        // Compute overall bounding box from matched boxes only
        let min_x = matched_boxes.iter().map(|b| b.x).min().unwrap();
        let min_y = matched_boxes.iter().map(|b| b.y).min().unwrap();
        let max_x = matched_boxes.iter().map(|b| b.x + b.width).max().unwrap();
        let max_y = matched_boxes.iter().map(|b| b.y + b.height).max().unwrap();

        // Add padding (10 pixels on each side)
        let padding = 10;
        let left = min_x.saturating_sub(padding);
        let top = min_y.saturating_sub(padding);
        let right = (max_x + padding).min(image.width() - 1);
        let bottom = (max_y + padding).min(image.height() - 1);

        Ok((left, top, right, bottom, matched_boxes))
    }

    /// Detect Level ROI (backward compatibility)
    pub fn detect_level_roi(&self, image: &DynamicImage) -> Result<(u32, u32, u32, u32), String> {
        let (left, top, right, bottom, _boxes) = self.detect_level_roi_with_boxes(image)?;
        Ok((left, top, right, bottom))
    }

    /// Apply NMS-like filtering to remove overlapping boxes
    /// Keep larger boxes when IoU > threshold
    fn filter_overlapping_boxes(boxes: Vec<TextBox>, iou_threshold: f64) -> Vec<TextBox> {
        if boxes.is_empty() {
            return boxes;
        }

        let mut filtered = Vec::new();
        let mut remaining = boxes;

        // Sort by area (largest first) to keep bigger boxes
        remaining.sort_by(|a, b| b.area().partial_cmp(&a.area()).unwrap_or(std::cmp::Ordering::Equal));

        while let Some(current) = remaining.pop() {
            // Keep current box
            filtered.push(current.clone());

            // Remove boxes that overlap significantly with current
            remaining.retain(|other| current.iou(other) <= iou_threshold);
        }

        filtered
    }

    /// Process OCR boxes: filter overlapping, sort left-to-right, concatenate text
    fn process_ocr_boxes(boxes: Vec<TextBox>) -> String {
        if boxes.is_empty() {
            return String::new();
        }

        // Step 1: Filter overlapping boxes (IoU > 0.3 = overlapping)
        let mut filtered = Self::filter_overlapping_boxes(boxes, 0.3);

        // Step 2: Sort left-to-right by x-coordinate
        filtered.sort_by(|a, b| a.left_x().partial_cmp(&b.left_x()).unwrap_or(std::cmp::Ordering::Equal));

        // Step 3: Concatenate text (no spaces, just join)
        filtered.iter().map(|b| b.text.as_str()).collect::<Vec<_>>().join("")
    }

    /// Check if server is healthy
    pub async fn health_check(&self) -> Result<(), String> {
        let url = format!("{}/health", self.base_url);
        self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Health check failed: {}", e))?;
        Ok(())
    }

    /// Encode image to base64
    fn encode_image(image: &DynamicImage) -> Result<String, String> {
        let mut buffer = Vec::new();
        image
            .write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Png)
            .map_err(|e| format!("Failed to encode image: {}", e))?;
        Ok(general_purpose::STANDARD.encode(&buffer))
    }

    /// Call unified OCR endpoint and get processed text
    /// Returns text after NMS filtering and left-to-right sorting
    async fn recognize_text(&self, image: &DynamicImage) -> Result<String, String> {
        let image_base64 = Self::encode_image(image)?;
        let url = format!("{}/ocr", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&ImageRequest { image_base64 })
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("OCR server error: {}", error_text));
        }

        let data: OcrResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        // Process boxes: filter overlapping, sort left-to-right, concatenate
        let processed_text = Self::process_ocr_boxes(data.boxes);
        Ok(processed_text)
    }

    /// Recognize number from image using custom ONNX model
    pub async fn recognize_number(&self, image: &DynamicImage) -> Result<u32, String> {
        let image_base64 = Self::encode_image(image)?;
        let url = format!("{}/ocr/predict_number", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&ImageRequest { image_base64 })
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("OCR server error: {}", error_text));
        }

        let data: NumberResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(data.value)
    }

    /// Parse level from OCR text
    fn parse_level(text: &str) -> Result<u32, String> {
        // Strip all non-digits
        let digits: String = text.chars().filter(|c| c.is_ascii_digit()).collect();

        if digits.is_empty() {
            return Err(format!("No digits found in level text: '{}'", text));
        }

        let level = digits.parse::<u32>()
            .map_err(|e| format!("Failed to parse level '{}': {}", digits, e))?;

        if level < 1 || level > 300 {
            return Err(format!("Level {} out of valid range (1-300)", level));
        }

        Ok(level)
    }

    /// Parse EXP from OCR text
    fn parse_exp(text: &str) -> Result<(u64, f64), String> {
        // Remove "EXP" prefix, spaces, and commas
        let cleaned = text.replace("EXP", "").replace(" ", "").replace(",", "");

        // Extract absolute value and percentage: "1234567[12.34%]" or "1234567[12.34]"
        let re = Regex::new(r"(\d+)\[?([\d.]+)%?\]?")
            .map_err(|e| format!("Regex error: {}", e))?;

        let caps = re.captures(&cleaned)
            .ok_or_else(|| format!("Failed to parse EXP format: '{}'", text))?;

        let absolute = caps.get(1)
            .ok_or("Missing absolute value")?
            .as_str()
            .parse::<u64>()
            .map_err(|e| format!("Failed to parse absolute: {}", e))?;

        let percentage = caps.get(2)
            .ok_or("Missing percentage")?
            .as_str()
            .parse::<f64>()
            .map_err(|e| format!("Failed to parse percentage: {}", e))?;

        Ok((absolute, percentage))
    }

    /// Recognize level from image using template matching (with RapidOCR fallback)
    pub async fn recognize_level(&self, image: &DynamicImage) -> Result<LevelResult, String> {
        // Try template matching first if available
        if let Some(matcher) = &self.template_matcher {
            let matcher = Arc::clone(matcher);
            let image = image.clone();

            // Run blocking template matching in dedicated thread pool
            let result = tokio::task::spawn_blocking(move || {
                matcher.recognize_level(&image)
            }).await.map_err(|e| format!("Template matching task failed: {}", e))?;

            match result {
                Ok(level) => {
                    return Ok(LevelResult {
                        level,
                        raw_text: format!("LV. {}", level),
                    });
                }
                Err(_e) => {
                    // Fall back to RapidOCR
                }
            }
        }

        // Fall back to RapidOCR
        let text = self.recognize_text(image).await?;
        let level = Self::parse_level(&text)?;

        Ok(LevelResult {
            level,
            raw_text: format!("LV. {}", level),
        })
    }

    /// Recognize EXP from image
    pub async fn recognize_exp(&self, image: &DynamicImage) -> Result<ExpResult, String> {
        let text = self.recognize_text(image).await?;
        let (absolute, percentage) = Self::parse_exp(&text)?;

        Ok(ExpResult {
            absolute,
            percentage,
            raw_text: text,
        })
    }

    /// Recognize HP potion count from image
    pub async fn recognize_hp_potion_count(&self, image: &DynamicImage) -> Result<u32, String> {
        self.recognize_number(image).await
    }

    /// Recognize MP potion count from image
    pub async fn recognize_mp_potion_count(&self, image: &DynamicImage) -> Result<u32, String> {
        self.recognize_number(image).await
    }
}
