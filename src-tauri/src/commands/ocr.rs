use crate::models::ocr_result::{CombinedOcrResult, ExpResult, LevelResult, MapResult};
use crate::services::ocr::HttpOcrClient;
use base64::Engine as _;
use image::DynamicImage;
use parking_lot::Mutex;
use std::sync::Arc;
use tauri::State;

/// State wrapper for OCR service (Arc for async sharing, parking_lot::Mutex for performance)
pub type OcrServiceState = Arc<Mutex<OcrService>>;

/// OCR service using HTTP client to communicate with Python server
pub struct OcrService {
    http_client: HttpOcrClient,
}

impl OcrService {
    /// Create a new OCR service with HTTP client
    pub fn new() -> Result<Self, String> {
        let http_client = HttpOcrClient::new()?;

        Ok(Self {
            http_client,
        })
    }

    /// Recognize and parse level from image
    pub async fn recognize_level(&self, image: &DynamicImage) -> Result<LevelResult, String> {
        self.http_client.recognize_level(image).await
    }

    /// Recognize and parse EXP from image
    pub async fn recognize_exp(&self, image: &DynamicImage) -> Result<ExpResult, String> {
        self.http_client.recognize_exp(image).await
    }

    /// Recognize and parse map name from image
    pub async fn recognize_map(&self, _image: &DynamicImage) -> Result<MapResult, String> {
        // TODO: Implement map recognition in Python server
        Err("Map recognition not yet implemented in HTTP OCR server".to_string())
    }

    /// Recognize HP potion count from inventory image (numbers only)
    pub async fn recognize_hp_potion_count(&self, image: &DynamicImage) -> Result<u32, String> {
        self.http_client.recognize_hp_potion_count(image).await
    }

    /// Recognize MP potion count from inventory image (numbers only)
    pub async fn recognize_mp_potion_count(&self, image: &DynamicImage) -> Result<u32, String> {
        self.http_client.recognize_mp_potion_count(image).await
    }
}

/// Initialize OCR service state
pub fn init_ocr_service() -> Result<OcrServiceState, String> {
    let service = OcrService::new()?;
    Ok(Arc::new(Mutex::new(service)))
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

/// Recognize level from base64-encoded image (async to prevent UI blocking)
#[tauri::command]
pub async fn recognize_level(
    state: State<'_, OcrServiceState>,
    image_base64: String,
) -> Result<LevelResult, String> {
    let http_client = {
        let service = state.inner().lock();
        service.http_client.clone()
    };
    let image = decode_base64_image(&image_base64)?;
    http_client.recognize_level(&image).await
}

/// Recognize EXP from base64-encoded image (async to prevent UI blocking)
#[tauri::command]
pub async fn recognize_exp(
    state: State<'_, OcrServiceState>,
    image_base64: String,
) -> Result<ExpResult, String> {
    let http_client = {
        let service = state.inner().lock();
        service.http_client.clone()
    };
    let image = decode_base64_image(&image_base64)?;
    http_client.recognize_exp(&image).await
}

/// Recognize map name from base64-encoded image (async to prevent UI blocking)
#[tauri::command]
pub async fn recognize_map(
    _state: State<'_, OcrServiceState>,
    _image_base64: String,
) -> Result<MapResult, String> {
    // TODO: Implement map recognition in Python server
    Err("Map recognition not yet implemented in HTTP OCR server".to_string())
}

/// Tauri command: Recognize HP potion count from base64 image
#[tauri::command]
pub async fn recognize_hp_potion_count(
    state: State<'_, OcrServiceState>,
    image_base64: String,
) -> Result<u32, String> {
    let http_client = {
        let service = state.inner().lock();
        service.http_client.clone()
    };
    let image = decode_base64_image(&image_base64)?;
    http_client.recognize_hp_potion_count(&image).await
}

/// Tauri command: Recognize MP potion count from base64 image
#[tauri::command]
pub async fn recognize_mp_potion_count(
    state: State<'_, OcrServiceState>,
    image_base64: String,
) -> Result<u32, String> {
    let http_client = {
        let service = state.inner().lock();
        service.http_client.clone()
    };
    let image = decode_base64_image(&image_base64)?;
    http_client.recognize_mp_potion_count(&image).await
}

/// Tauri command: Recognize all 4 OCR operations in parallel
/// Each operation is independent - failures don't block others
#[tauri::command]
pub async fn recognize_all_parallel(
    state: State<'_, OcrServiceState>,
    level_base64: String,
    exp_base64: String,
    hp_base64: String,
    mp_base64: String,
) -> Result<CombinedOcrResult, String> {
    let http_client = {
        let service = state.inner().lock();
        service.http_client.clone()
    };

    // Decode images
    let level_image = decode_base64_image(&level_base64).ok();
    let exp_image = decode_base64_image(&exp_base64).ok();
    let hp_image = decode_base64_image(&hp_base64).ok();
    let mp_image = decode_base64_image(&mp_base64).ok();

    // Run all 4 OCR operations in parallel
    let (level_result, exp_result, hp_potion_result, mp_potion_result) = tokio::join!(
        async {
            match level_image {
                Some(ref img) => http_client.recognize_level(img).await.ok(),
                None => None,
            }
        },
        async {
            match exp_image {
                Some(ref img) => http_client.recognize_exp(img).await.ok(),
                None => None,
            }
        },
        async {
            match hp_image {
                Some(ref img) => http_client.recognize_hp_potion_count(img).await.ok(),
                None => None,
            }
        },
        async {
            match mp_image {
                Some(ref img) => http_client.recognize_mp_potion_count(img).await.ok(),
                None => None,
            }
        },
    );

    Ok(CombinedOcrResult {
        level: level_result,
        exp: exp_result,
        hp: hp_potion_result,
        mp: mp_potion_result,
    })
}

/// Tauri command: Check OCR server health
#[tauri::command]
pub async fn check_ocr_health(state: State<'_, OcrServiceState>) -> Result<bool, String> {
    let http_client = {
        let service = state.inner().lock();
        service.http_client.clone()
    };
    
    match http_client.health_check().await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

