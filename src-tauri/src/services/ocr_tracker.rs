use crate::commands::ocr::OcrServiceState;
use crate::models::exp_data::ExpData;
use crate::models::roi::Roi;
use crate::models::config::PotionConfig;
use crate::services::exp_calculator::ExpCalculator;
use crate::services::hp_potion_calculator::HpPotionCalculator;
use crate::services::mp_potion_calculator::MpPotionCalculator;
use crate::services::screen_capture::ScreenCapture;
use crate::services::config::ConfigManager;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Mutex;
use tokio::time::sleep;
use image::DynamicImage;
use std::fs;

/// Current tracking statistics
#[derive(Debug, Clone, Serialize)]
pub struct TrackingStats {
    pub level: Option<i32>,
    pub exp: Option<i64>,
    pub percentage: Option<f64>,
    pub hp_potion_count: Option<i32>,
    pub mp_potion_count: Option<i32>,
    pub total_exp: i64,
    pub total_percentage: f64,
    pub elapsed_seconds: i64,
    pub exp_per_hour: i64,
    pub percentage_per_hour: f64,
    pub is_tracking: bool,
    pub error: Option<String>,
    pub hp_potions_used: i32,
    pub mp_potions_used: i32,
    pub hp_potions_per_minute: f64,
    pub mp_potions_per_minute: f64,
    pub ocr_server_healthy: bool,
}

/// OCR Tracker state
struct TrackerState {
    level: Option<u32>,
    exp: Option<u64>,
    percentage: Option<f64>,
    hp_potion_count: Option<u32>,
    mp_potion_count: Option<u32>,
    // Independent calculators - each tracks its own data
    exp_calculator: ExpCalculator,
    hp_calculator: HpPotionCalculator,
    mp_calculator: MpPotionCalculator,
    is_tracking: bool,
    error: Option<String>,
    // Level stability tracking
    prev_level: Option<u32>,
    level_match_count: u32,
    // Session started flag
    session_started: bool,
    // OCR server health status
    ocr_server_healthy: bool,
    // Latest stats cache - each calculator updates its own fields
    latest_stats: TrackingStats,
}

impl TrackerState {
    fn new() -> Result<Self, String> {
        Ok(Self {
            level: None,
            exp: None,
            percentage: None,
            hp_potion_count: None,
            mp_potion_count: None,
            exp_calculator: ExpCalculator::new()?,
            hp_calculator: HpPotionCalculator::new(),
            mp_calculator: MpPotionCalculator::new(),
            is_tracking: false,
            error: None,
            prev_level: None,
            level_match_count: 0,
            session_started: false,
            ocr_server_healthy: true,
            latest_stats: TrackingStats {
                level: None,
                exp: None,
                percentage: None,
                hp_potion_count: None,
                mp_potion_count: None,
                total_exp: 0,
                total_percentage: 0.0,
                elapsed_seconds: 0,
                exp_per_hour: 0,
                percentage_per_hour: 0.0,
                is_tracking: false,
                error: None,
                hp_potions_used: 0,
                mp_potions_used: 0,
                hp_potions_per_minute: 0.0,
                mp_potions_per_minute: 0.0,
                ocr_server_healthy: true,
            },
        })
    }

    /// Update level - emit immediately for UI responsiveness
    fn update_level(&mut self, new_level: u32) -> bool {
        let should_emit = match self.prev_level {
            Some(prev) if prev == new_level => {
                // Same as before - already displayed in UI, no need to re-emit
                self.level_match_count += 1;
                false
            }
            _ => {
                // New value - emit immediately to UI
                self.prev_level = Some(new_level);
                self.level_match_count = 1;
                self.level = Some(new_level);
                true
            }
        };
        should_emit
    }

    /// Update EXP and trigger calculator update - returns true if changed
    fn update_exp_data(&mut self, exp: u64, percentage: f64) -> bool {
        let changed = self.exp != Some(exp) || self.percentage != Some(percentage);
        self.exp = Some(exp);
        self.percentage = Some(percentage);

        // Update ExpCalculator if level is stable
        if let Some(level) = self.level {
            let data = ExpData {
                level,
                exp,
                percentage,
                meso: None,
            };

            if !self.session_started {
                self.exp_calculator.start(data);
                self.session_started = true;
            } else {
                // Update session with EXP tracking - ORIGINAL WORKING MECHANISM
                let result = self.exp_calculator.update(data);

                match result {
                    Ok(stats) => {
                        // Cache ONLY EXP stats - HP/MP have their own calculators now
                        self.latest_stats.total_exp = stats.total_exp as i64;
                        self.latest_stats.total_percentage = stats.total_percentage;
                        self.latest_stats.elapsed_seconds = stats.elapsed_seconds as i64;
                        self.latest_stats.exp_per_hour = stats.exp_per_hour as i64;
                        self.latest_stats.percentage_per_hour = stats.percentage_per_hour;
                        self.error = None;
                    }
                    Err(e) => {
                        self.error = Some(e);
                    }
                }
            }
        }
        changed
    }

    fn to_stats(&self) -> TrackingStats {
        // ORIGINAL EXP MECHANISM: Read from cached latest_stats
        // All trackers use the same mechanism now
        TrackingStats {
            level: self.level.map(|l| l as i32),
            exp: self.exp.map(|e| e as i64),
            percentage: self.percentage,
            hp_potion_count: self.hp_potion_count.map(|h| h as i32),
            mp_potion_count: self.mp_potion_count.map(|m| m as i32),
            // Read from cache (same as original EXP mechanism)
            total_exp: self.latest_stats.total_exp,
            total_percentage: self.latest_stats.total_percentage,
            elapsed_seconds: self.latest_stats.elapsed_seconds,
            exp_per_hour: self.latest_stats.exp_per_hour,
            percentage_per_hour: self.latest_stats.percentage_per_hour,
            is_tracking: self.is_tracking,
            error: self.error.clone(),
            hp_potions_used: self.latest_stats.hp_potions_used,
            mp_potions_used: self.latest_stats.mp_potions_used,
            hp_potions_per_minute: self.latest_stats.hp_potions_per_minute,
            mp_potions_per_minute: self.latest_stats.mp_potions_per_minute,
            ocr_server_healthy: self.ocr_server_healthy,
        }
    }
}

/// Event payloads for Frontend updates
#[derive(Clone, Serialize)]
struct LevelUpdate {
    level: u32,
}

#[derive(Clone, Serialize)]
struct ExpUpdate {
    exp: u64,
    percentage: f64,
}

#[derive(Clone, Serialize)]
struct HpPotionUpdate {
    hp_potion_count: u32,
}

#[derive(Clone, Serialize)]
struct MpPotionUpdate {
    mp_potion_count: u32,
}

/// Global OCR Tracker instance
pub struct OcrTracker {
    state: Arc<Mutex<TrackerState>>,
    stop_signal: Arc<Mutex<bool>>,
    screen_capture: Arc<ScreenCapture>,
    app: AppHandle,
    ocr_service: OcrServiceState,  // Shared OCR service instance
}

impl OcrTracker {
    pub fn new(app: AppHandle, ocr_service: OcrServiceState) -> Result<Self, String> {
        Ok(Self {
            state: Arc::new(Mutex::new(TrackerState::new()?)),
            stop_signal: Arc::new(Mutex::new(false)),
            screen_capture: Arc::new(ScreenCapture::new()?),
            app,
            ocr_service,  // Store shared OCR service
        })
    }

    /// Start OCR tracking with 3 independent parallel tasks (Level, EXP, Inventory)
    /// Inventory recognition uses automatic ROI detection
    pub async fn start_tracking(
        &self,
        level_roi: Roi,
        exp_roi: Roi,
    ) -> Result<(), String> {
        // Check if already tracking - prevent reinitialization
        let mut state = self.state.lock().await;
        if state.is_tracking {
            return Ok(());
        }

        // Check if this is a resume (session_started = true) or new session
        let is_resume = state.session_started;

        if !is_resume {
            // New session - reset state completely
            *state = TrackerState::new()?;
        }

        // Set tracking flag
        state.is_tracking = true;
        drop(state);

        // Reset stop signal
        *self.stop_signal.lock().await = false;

        // Spawn OCR tasks: combined Level+Inventory (shared capture), separate EXP, health check
        self.spawn_combined_level_inventory_loop(level_roi, self.app.clone());
        self.spawn_exp_loop(exp_roi, self.app.clone());
        self.spawn_health_check_loop(self.app.clone());
        Ok(())
    }

    /// Stop all OCR loops
    pub async fn stop_tracking(&self) {
        *self.stop_signal.lock().await = true;
        let mut state = self.state.lock().await;
        state.is_tracking = false;
    }

    /// Get current tracking statistics
    pub async fn get_stats(&self) -> TrackingStats {
        let state = self.state.lock().await;
        state.to_stats()
    }

    /// Reset tracking session
    pub async fn reset(&self) -> Result<(), String> {
        *self.stop_signal.lock().await = true;
        let mut state = self.state.lock().await;
        *state = TrackerState::new()?;
        Ok(())
    }

    /// Combined Level + Inventory OCR loop (shares full screen capture for efficiency)
    fn spawn_combined_level_inventory_loop(&self, _roi: Roi, app: AppHandle) {
        let state = Arc::clone(&self.state);
        let stop_signal = Arc::clone(&self.stop_signal);
        let screen_capture = Arc::clone(&self.screen_capture);
        let ocr_service = Arc::clone(&self.ocr_service);

        tokio::spawn(async move {
            // Image cache for duplicate detection
            let mut last_image_bytes: Option<Vec<u8>> = None;

            // ROI memoization for performance (caches detected regions)
            let mut memoized_level_roi: Option<(u32, u32, u32, u32)> = None;
            let mut memoized_inventory_roi: Option<(u32, u32, u32, u32)> = None;

            while !*stop_signal.lock().await {
                let start = std::time::Instant::now();

                // Single full screen capture for both Level and Inventory
                match screen_capture.capture_full() {
                    Ok(image) => {
                        // Convert image to raw bytes for comparison
                        let current_bytes = image.as_bytes().to_vec();

                        // Check if image is identical to last capture
                        if let Some(ref last_bytes) = last_image_bytes {
                            if current_bytes == *last_bytes {
                                sleep(Duration::from_millis(500)).await;
                                continue;
                            }
                        }

                        // Process Level and Inventory independently (not waiting for each other)
                        // Share captured image via Arc to avoid cloning full image
                        let image = Arc::new(image);

                        // Spawn Level OCR as independent task with ROI memoization
                        {
                            let http_client = {
                                let service = ocr_service.lock();
                                service.http_client.clone()
                            };
                            let image = Arc::clone(&image);
                            let app = app.clone();
                            let state = Arc::clone(&state);
                            let memoized_roi = memoized_level_roi.clone();

                            let updated_roi = tokio::spawn(async move {
                                // Try memoized ROI first (fast path)
                                if let Some((left, top, right, bottom)) = memoized_roi {
                                    let width = right - left + 1;
                                    let height = bottom - top + 1;
                                    let cropped = image.crop_imm(left, top, width, height);

                                    if let Ok(result) = http_client.recognize_level(&cropped).await {
                                        return (Ok(result), Some((left, top, right, bottom)));
                                    }
                                }

                                // Fallback: Full detection
                                match http_client.recognize_level(&*image).await {
                                    Ok(result) => {
                                        // Try to detect ROI for memoization
                                        let roi = http_client.detect_level_roi(&*image).ok();
                                        (Ok(result), roi)
                                    }
                                    Err(e) => (Err(e), None)
                                }
                            }).await;

                            let (level_result, new_roi) = match updated_roi {
                                Ok((result, roi)) => (result, roi),
                                Err(e) => (Err(format!("Task failed: {}", e)), None)
                            };

                            // Update memoized ROI if we got a new one
                            if new_roi.is_some() {
                                memoized_level_roi = new_roi;
                            }

                            match level_result {
                                Ok(result) => {
                                    println!("📊 [LEVEL] {} (text: '{}')", result.level, result.raw_text);
                                    
                                    let should_emit = {
                                        let mut state = state.lock().await;
                                        state.update_level(result.level)
                                    };

                                    if should_emit {
                                        if let Err(e) = app.emit("ocr:level-update", LevelUpdate { level: result.level }) {
                                            eprintln!("Failed to emit level update: {}", e);
                                        }
                                    }
                                }
                                Err(_e) => {
                                    // Level OCR failed, will retry on next cycle
                                }
                            }
                        }

                        // Spawn Inventory OCR as independent task with ROI memoization
                        {
                            let ocr_service_clone = Arc::clone(&ocr_service);
                            let image = Arc::clone(&image);
                            let app = app.clone();
                            let state = Arc::clone(&state);
                            let memoized_roi = memoized_inventory_roi.clone();

                            let updated_roi = tokio::spawn(async move {
                                let inventory_result = tokio::task::spawn_blocking(move || {
                                    let service = ocr_service_clone.lock();

                                    // Try memoized ROI first (fast path)
                                    if let Some((left, top, right, bottom)) = memoized_roi {
                                        let padding = 100;
                                        let img_width = image.width();
                                        let img_height = image.height();
                                        let padded_left = left.saturating_sub(padding);
                                        let padded_top = top.saturating_sub(padding);
                                        let padded_right = (right + padding).min(img_width - 1);
                                        let padded_bottom = (bottom + padding).min(img_height - 1);

                                        let crop_width = padded_right - padded_left + 1;
                                        let crop_height = padded_bottom - padded_top + 1;
                                        let cropped = image.crop_imm(padded_left, padded_top, crop_width, crop_height);

                                        if let Ok(results) = service.recognize_inventory(&cropped) {
                                            return Ok((results, Some((left, top, right, bottom))));
                                        }
                                    }

                                    // Fallback: Full detection
                                    match service.recognize_inventory(&*image) {
                                        Ok(results) => {
                                            // Try to get ROI coordinates for memoization
                                            if let Some(matcher) = &service.inventory_matcher {
                                                if let Ok((_, coords)) = matcher.detect_inventory_region_with_coords(&*image) {
                                                    let (left, top, right, bottom) = coords;
                                                    let width = right - left + 1;
                                                    let height = bottom - top + 1;
                                                    let cropped_original = image::imageops::crop_imm(&*image, left, top, width, height);
                                                    let dynamic_img = DynamicImage::ImageRgba8(cropped_original.to_image());
                                                    save_inventory_preview(&dynamic_img);
                                                    
                                                    return Ok((results, Some(coords)));
                                                }
                                            }
                                            Ok((results, None))
                                        }
                                        Err(e) => Err(e)
                                    }
                                }).await;

                                match inventory_result {
                                    Ok(Ok((results, roi))) => (Ok(results), roi),
                                    Ok(Err(e)) => (Err(e), None),
                                    Err(e) => (Err(format!("Task failed: {}", e)), None)
                                }
                            }).await;

                            let (inventory_result, new_roi) = match updated_roi {
                                Ok(result) => result,
                                Err(e) => (Err(format!("Spawn failed: {}", e)), None)
                            };

                            // Update memoized ROI if we got a new one
                            if new_roi.is_some() {
                                memoized_inventory_roi = new_roi;
                            }

                            match inventory_result {
                                Ok(inventory) => {
                                    let potion_config = {
                                        if let Some(config_state) = app.try_state::<std::sync::Mutex<ConfigManager>>() {
                                            match config_state.lock() {
                                                Ok(manager) => match manager.load() {
                                                    Ok(config) => config.potion,
                                                    Err(_) => PotionConfig::default()
                                                },
                                                Err(_) => PotionConfig::default()
                                            }
                                        } else {
                                            PotionConfig::default()
                                        }
                                    };

                                    let hp_potion_count = *inventory.get(&potion_config.hp_potion_slot).unwrap_or(&0);
                                    let mp_potion_count = *inventory.get(&potion_config.mp_potion_slot).unwrap_or(&0);

                                    let mut state = state.lock().await;
                                    state.hp_potion_count = Some(hp_potion_count);
                                    state.mp_potion_count = Some(mp_potion_count);

                                    let (hp_used, hp_per_min) = state.hp_calculator.update(hp_potion_count);
                                    state.latest_stats.hp_potions_used = hp_used as i32;
                                    state.latest_stats.hp_potions_per_minute = hp_per_min;

                                    let (mp_used, mp_per_min) = state.mp_calculator.update(mp_potion_count);
                                    state.latest_stats.mp_potions_used = mp_used as i32;
                                    state.latest_stats.mp_potions_per_minute = mp_per_min;

                                    drop(state);

                                    // Emit events to Frontend
                                    if let Err(e) = app.emit("ocr:hp-potion-update", HpPotionUpdate { hp_potion_count }) {
                                        eprintln!("Failed to emit HP potion update: {}", e);
                                    }

                                    if let Err(e) = app.emit("ocr:mp-potion-update", MpPotionUpdate { mp_potion_count }) {
                                        eprintln!("Failed to emit MP potion update: {}", e);
                                    }
                                }
                                Err(_e) => {
                                    // Inventory OCR failed, will retry on next cycle
                                }
                            }
                        }

                        // Update cache
                        last_image_bytes = Some(current_bytes);
                    }
                    Err(_e) => {
                        // Full screen capture failed, will retry on next cycle
                    }
                }

                // Dynamic sleep based on config
                let interval_ms = {
                    if let Some(config_state) = app.try_state::<std::sync::Mutex<ConfigManager>>() {
                        match config_state.lock() {
                            Ok(manager) => match manager.load() {
                                Ok(config) => (config.tracking.update_interval.max(1) as f64 * 1000.0) as u64,
                                Err(_) => 1000
                            },
                            Err(_) => 1000
                        }
                    } else {
                        1000
                    }
                };
                sleep(Duration::from_millis(interval_ms)).await;
            }
        });
    }

    // Independent Level OCR loop with shared OCR service + image caching
    // NOTE: Template matching uses FULL SCREEN, not ROI (roi param unused)
    fn spawn_level_loop(&self, _roi: Roi, app: AppHandle) {
        let state = Arc::clone(&self.state);
        let stop_signal = Arc::clone(&self.stop_signal);
        let screen_capture = Arc::clone(&self.screen_capture);
        let ocr_service = Arc::clone(&self.ocr_service);  // Use shared service

        tokio::spawn(async move {
            #[cfg(debug_assertions)]
            println!("🚀 LEVEL OCR task started - using shared OCR service (FULL SCREEN capture for template matching)");

            // Image cache for duplicate detection
            let mut last_image_bytes: Option<Vec<u8>> = None;

            while !*stop_signal.lock().await {
                let start = std::time::Instant::now();

                // For template matching: capture FULL SCREEN (not ROI)
                // Template matching needs full screen to find orange boxes
                match screen_capture.capture_full() {
                    Ok(image) => {
                        // Convert image to raw bytes for comparison
                        let current_bytes = image.as_bytes().to_vec();

                        // Check if image is identical to last capture
                        if let Some(ref last_bytes) = last_image_bytes {
                            if current_bytes == *last_bytes {
                                #[cfg(debug_assertions)]
                                println!("⏭️  LEVEL: Skipped (identical image)");
                                sleep(Duration::from_millis(500)).await;
                                continue;
                            }
                        }

                        // Image changed - run OCR with FULL SCREEN
                        let http_client = {
                            let service = ocr_service.lock();
                            service.http_client.clone()
                        };
                        match http_client.recognize_level(&image).await {
                            Ok(result) => {
                                let mut state = state.lock().await;
                                state.update_level(result.level);

                                // Emit event to Frontend if level is confirmed (stable)
                                if let Some(level) = state.level {
                                    app.emit("ocr:level-update", LevelUpdate { level }).ok();
                                }

                                #[cfg(debug_assertions)]
                                {
                                    let elapsed = start.elapsed().as_millis();
                                    println!("✅ LEVEL OCR: {} ({}ms)", result.level, elapsed);
                                }
                            }
                            Err(e) => {
                                #[cfg(debug_assertions)]
                                eprintln!("❌ LEVEL OCR failed: {}", e);
                            }
                        }

                        // Update cache
                        last_image_bytes = Some(current_bytes);
                    }
                    Err(e) => {
                        #[cfg(debug_assertions)]
                        eprintln!("❌ LEVEL full screen capture failed: {}", e);
                    }
                }

                sleep(Duration::from_millis(500)).await;
            }

            #[cfg(debug_assertions)]
            println!("⏹️  LEVEL OCR task stopped");
        });
    }

    // Independent EXP OCR loop with shared OCR service + image caching
    fn spawn_exp_loop(&self, roi: Roi, app: AppHandle) {
        let state = Arc::clone(&self.state);
        let stop_signal = Arc::clone(&self.stop_signal);
        let screen_capture = Arc::clone(&self.screen_capture);
        let ocr_service = Arc::clone(&self.ocr_service);  // Use shared service

        tokio::spawn(async move {
            // Image cache for duplicate detection
            let mut last_image_bytes: Option<Vec<u8>> = None;

            while !*stop_signal.lock().await {
                match screen_capture.capture_region(&roi) {
                    Ok(image) => {
                        let current_bytes = image.as_bytes().to_vec();

                        // Check if image is identical to last capture
                        if let Some(ref last_bytes) = last_image_bytes {
                            if current_bytes == *last_bytes {
                                sleep(Duration::from_millis(500)).await;
                                continue;
                            }
                        }

                        // Image changed - run OCR
                        let http_client = {
                            let service = ocr_service.lock();
                            service.http_client.clone()
                        };
                        
                        match http_client.recognize_exp(&image).await {
                            Ok(result) => {
                                println!("📊 [EXP] {} [{:.2}%] (text: '{}')", 
                                    result.absolute, result.percentage, result.raw_text);
                                
                                let should_emit = {
                                    let mut state_guard = state.lock().await;
                                    state_guard.update_exp_data(result.absolute, result.percentage)
                                };

                                // Emit event to Frontend if EXP changed
                                if should_emit {
                                    if let Err(e) = app.emit("ocr:exp-update", ExpUpdate {
                                        exp: result.absolute,
                                        percentage: result.percentage
                                    }) {
                                        eprintln!("Failed to emit EXP update: {}", e);
                                    }
                                }
                            }
                            Err(_e) => {
                                // EXP OCR failed, will retry on next cycle
                            }
                        }

                        // Update cache
                        last_image_bytes = Some(current_bytes);
                    }
                    Err(_e) => {
                        // EXP capture failed, will retry on next cycle
                    }
                }

                // Dynamic sleep based on config
                let interval_ms = {
                    if let Some(config_state) = app.try_state::<std::sync::Mutex<ConfigManager>>() {
                        match config_state.lock() {
                            Ok(manager) => match manager.load() {
                                Ok(config) => (config.tracking.update_interval.max(1) as f64 * 1000.0) as u64,
                                Err(_) => 1000
                            },
                            Err(_) => 1000
                        }
                    } else {
                        1000
                    }
                };
                sleep(Duration::from_millis(interval_ms)).await;
            }
        });
    }

    // Unified Inventory OCR loop - Rust native with automatic ROI detection
    fn spawn_inventory_loop(&self, app: AppHandle) {
        let state = Arc::clone(&self.state);
        let stop_signal = Arc::clone(&self.stop_signal);
        let screen_capture = Arc::clone(&self.screen_capture);
        let ocr_service = Arc::clone(&self.ocr_service);

        tokio::spawn(async move {
            // Image cache for duplicate detection
            let mut last_image_bytes: Option<Vec<u8>> = None;

            while !*stop_signal.lock().await {
                // Capture full screen for automatic inventory detection
                match screen_capture.capture_full() {
                    Ok(image) => {
                        // Convert image to raw bytes for comparison
                        let current_bytes = image.as_bytes().to_vec();

                        // Check if image is identical to last capture
                        if let Some(ref last_bytes) = last_image_bytes {
                            if current_bytes == *last_bytes {
                                sleep(Duration::from_millis(500)).await;
                                continue;
                            }
                        }

                        // Run Rust native inventory recognition (async, non-blocking)
                        let ocr_service_clone = Arc::clone(&ocr_service);
                        let image_clone = image.clone();
                        let inventory_results = match tokio::task::spawn_blocking(move || {
                            let service = ocr_service_clone.lock();
                            service.recognize_inventory(&image_clone)
                        }).await {
                            Ok(result) => result,
                            Err(e) => Err(format!("Inventory recognition task failed: {}", e))
                        };

                        match inventory_results {
                            Ok(inventory) => {
                                // Load potion config from app state
                                let potion_config = {
                    if let Some(config_state) = app.try_state::<std::sync::Mutex<ConfigManager>>() {
                        match config_state.lock() {
                            Ok(manager) => match manager.load() {
                                Ok(config) => config.potion,
                                Err(_) => PotionConfig::default()
                            },
                            Err(_) => PotionConfig::default()
                        }
                    } else {
                        PotionConfig::default()
                    }
                };

                                // Extract HP and MP counts from inventory
                                let hp_potion_count = *inventory.get(&potion_config.hp_potion_slot).unwrap_or(&0);
                                let mp_potion_count = *inventory.get(&potion_config.mp_potion_slot).unwrap_or(&0);

                                // Update state and calculators
                                let mut state = state.lock().await;
                                state.hp_potion_count = Some(hp_potion_count);
                                state.mp_potion_count = Some(mp_potion_count);

                                // Update HP calculator
                                let (hp_used, hp_per_min) = state.hp_calculator.update(hp_potion_count);
                                state.latest_stats.hp_potions_used = hp_used as i32;
                                state.latest_stats.hp_potions_per_minute = hp_per_min;

                                // Update MP calculator
                                let (mp_used, mp_per_min) = state.mp_calculator.update(mp_potion_count);
                                state.latest_stats.mp_potions_used = mp_used as i32;
                                state.latest_stats.mp_potions_per_minute = mp_per_min;

                                drop(state);

                                // Emit events to Frontend
                                app.emit("ocr:hp-potion-update", HpPotionUpdate { hp_potion_count }).ok();
                                app.emit("ocr:mp-potion-update", MpPotionUpdate { mp_potion_count }).ok();
                            }
                            Err(_e) => {
                                // Inventory OCR failed, will retry on next cycle
                            }
                        }

                        // Update cache
                        last_image_bytes = Some(current_bytes);
                    }
                    Err(_e) => {
                        // Full screen capture failed, will retry on next cycle
                    }
                }

                sleep(Duration::from_millis(500)).await;
            }
        });
    }


    /// Spawn health check loop - monitors OCR server health
    fn spawn_health_check_loop(&self, _app: AppHandle) {
        let state = Arc::clone(&self.state);
        let stop_signal = Arc::clone(&self.stop_signal);
        let ocr_service = Arc::clone(&self.ocr_service);  // Use shared service

        tokio::spawn(async move {
            while !*stop_signal.lock().await {
                // Use shared OCR service for health check
                let http_client = {
                    let service = ocr_service.lock();
                    service.http_client.clone()
                };
                match http_client.health_check().await {
                    Ok(_) => {
                        let mut state = state.lock().await;
                        state.ocr_server_healthy = true;
                        state.latest_stats.ocr_server_healthy = true;
                    }
                    Err(_e) => {
                        let mut state = state.lock().await;
                        state.ocr_server_healthy = false;
                        state.latest_stats.ocr_server_healthy = false;
                    }
                }

                // Check every 2 seconds
                sleep(Duration::from_secs(2)).await;
            }
        });
    }
}

/// Helper function to save inventory preview image
fn save_inventory_preview(image: &DynamicImage) {
    let temp_dir = std::env::temp_dir().join("exp-tracker-previews");
    if fs::create_dir_all(&temp_dir).is_err() {
        return;
    }

    let file_path = temp_dir.join("inventory_preview.png");
    let _ = image.save(&file_path);
}
