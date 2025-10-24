use crate::models::ocr_result::{ExpResult, LevelResult};
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use base64::{Engine as _, engine::general_purpose};
use regex::Regex;

/// HTTP OCR client that communicates with Python FastAPI server
#[derive(Clone)]
pub struct HttpOcrClient {
    client: reqwest::Client,
    base_url: String,
}

#[derive(Serialize)]
struct ImageRequest {
    image_base64: String,
}

/// Unified OCR response from Python server
#[derive(Deserialize)]
struct OcrResponse {
    text: String,
    confidence: Option<f64>,
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
        })
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

    /// Call unified OCR endpoint and get raw text
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

        Ok(data.text)
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

    /// Parse HP/MP from OCR text (extract digits only)
    fn parse_hp_mp(text: &str) -> Result<u32, String> {
        let digits: String = text.chars().filter(|c| c.is_ascii_digit()).collect();

        if digits.is_empty() {
            return Err(format!("No digits found in HP/MP text: '{}'", text));
        }

        digits.parse::<u32>()
            .map_err(|e| format!("Failed to parse HP/MP '{}': {}", digits, e))
    }

    /// Recognize level from image
    pub async fn recognize_level(&self, image: &DynamicImage) -> Result<LevelResult, String> {
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

    /// Recognize HP from image
    pub async fn recognize_hp(&self, image: &DynamicImage) -> Result<u32, String> {
        let text = self.recognize_text(image).await?;
        Self::parse_hp_mp(&text)
    }

    /// Recognize MP from image
    pub async fn recognize_mp(&self, image: &DynamicImage) -> Result<u32, String> {
        let text = self.recognize_text(image).await?;
        Self::parse_hp_mp(&text)
    }
}
