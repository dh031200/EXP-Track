use crate::models::ocr_result::{CombinedOcrResult, ExpResult, LevelResult, MapResult};
use crate::services::ocr::{HttpOcrClient, InventoryTemplateMatcher};
use base64::Engine as _;
use image::DynamicImage;
use parking_lot::Mutex;
use std::sync::Arc;
use std::collections::HashMap;
use tauri::State;

/// State wrapper for OCR service (Arc for async sharing, parking_lot::Mutex for performance)
pub type OcrServiceState = Arc<Mutex<OcrService>>;

/// OCR service using HTTP client to communicate with Python server
pub struct OcrService {
    pub http_client: HttpOcrClient,  // Public for cloning in async tasks
    pub inventory_matcher: Option<Arc<InventoryTemplateMatcher>>,  // Rust native inventory recognition
}

impl OcrService {
    /// Create a new OCR service with HTTP client
    pub fn new() -> Result<Self, String> {
        println!("🔧 Initializing OCR Service...");
        let mut http_client = HttpOcrClient::new()?;

        // Try to initialize level template matcher (non-fatal if it fails)
        Self::try_init_template_matcher(&mut http_client).ok();

        // Try to initialize inventory template matcher (Rust native)
        let inventory_matcher = Self::try_init_inventory_matcher().ok();

        Ok(Self {
            http_client,
            inventory_matcher,
        })
    }

    /// Try to initialize template matcher from bundled resources
    fn try_init_template_matcher(http_client: &mut HttpOcrClient) -> Result<(), String> {
        // Try multiple possible template paths
        let possible_paths = vec![
            "src-tauri/resources/level_template", // Development (from project root)
            "resources/level_template",           // Development (from src-tauri)
            "../Resources/level_template",        // macOS bundled
            "./resources/level_template",         // Windows/Linux bundled
        ];

        for path in possible_paths.iter() {
            if std::path::Path::new(path).exists() {
                return http_client.init_template_matcher(path);
            }
        }

        Err("Template directory not found in any expected location".to_string())
    }

    /// Try to initialize inventory template matcher (Rust native)
    fn try_init_inventory_matcher() -> Result<Arc<InventoryTemplateMatcher>, String> {
        println!("🔧 Initializing Inventory Template Matcher (Rust native)...");

        // Try multiple possible template paths for inventory digit templates
        let possible_paths = vec![
            "src-tauri/resources/item_template",   // Development (from project root)
            "resources/item_template",             // Development (from src-tauri)
            "../Resources/item_template",          // macOS bundled
            "./resources/item_template",           // Windows/Linux bundled
        ];

        let mut matcher = InventoryTemplateMatcher::new();

        for path in possible_paths.iter() {
            #[cfg(debug_assertions)]
            println!("🔍 Trying inventory template path: {}", path);

            if std::path::Path::new(path).exists() {
                println!("📂 Loading inventory templates from: {}", path);
                match matcher.load_templates(path) {
                    Ok(_) => {
                        println!("✅ Inventory template matcher initialized successfully");
                        return Ok(Arc::new(matcher));
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to load templates from {}: {}", path, e);
                        continue;
                    }
                }
            } else {
                #[cfg(debug_assertions)]
                println!("❌ Path does not exist: {}", path);
            }
        }

        Err("Inventory template directory not found in any expected location".to_string())
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

    /// Recognize all 8 inventory slots (Rust native implementation)
    /// Returns HashMap with slot names as keys and item counts as values
    pub fn recognize_inventory(&self, image: &DynamicImage) -> Result<HashMap<String, u32>, String> {
        // Try Rust native template matching first
        if let Some(matcher) = &self.inventory_matcher {
            #[cfg(debug_assertions)]
            {
                let t_start = std::time::Instant::now();
                println!("🔍 Using Rust native inventory recognition (image: {}x{})", image.width(), image.height());

                // Step 1: Detect inventory region (522x255) with coordinates
                let t1 = std::time::Instant::now();
                let detection_result = matcher.detect_inventory_region_with_coords(image);
                let t2 = std::time::Instant::now();
                println!("    ⏱️  detect_inventory_region_with_coords: {}ms", (t2 - t1).as_millis());

                match detection_result {
                    Ok((inventory_image, _coords)) => {
                        // Step 2: Recognize all 8 slots
                        let t3 = std::time::Instant::now();
                        let recognition_result = matcher.recognize_all_slots(&inventory_image);
                        let t4 = std::time::Instant::now();
                        println!("    ⏱️  recognize_all_slots: {}ms", (t4 - t3).as_millis());

                        match recognition_result {
                            Ok(results) => {
                                let t_end = std::time::Instant::now();
                                println!("✅ Inventory recognition successful (total: {}ms): {:?}", (t_end - t_start).as_millis(), results);
                                return Ok(results);
                            }
                            Err(e) => {
                                eprintln!("❌ Slot recognition failed: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Inventory region detection failed: {}", e);
                    }
                }
            }

            #[cfg(not(debug_assertions))]
            {
                // Non-debug version without timing
                match matcher.detect_inventory_region_with_coords(image) {
                    Ok((inventory_image, _coords)) => {
                        if let Ok(results) = matcher.recognize_all_slots(&inventory_image) {
                            return Ok(results);
                        }
                    }
                    Err(_) => {}
                }
            }
        } else {
            #[cfg(debug_assertions)]
            println!("⚠️  Inventory matcher not initialized");
        }

        // Fallback: Return empty inventory (Python HTTP fallback can be added later if needed)
        #[cfg(debug_assertions)]
        println!("⚠️  Falling back to empty inventory");

        let mut empty = HashMap::new();
        for slot in &["shift", "ins", "home", "pup", "ctrl", "del", "end", "pdn"] {
            empty.insert(slot.to_string(), 0);
        }
        Ok(empty)
    }

    /// Check if OCR server is healthy
    pub async fn health_check(&self) -> Result<(), String> {
        self.http_client.health_check().await
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

