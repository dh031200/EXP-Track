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
                // Start session
                self.exp_calculator.start(data);
                self.session_started = true;
                #[cfg(debug_assertions)]
                println!("✅ Tracking session started: level={}, exp={}, percentage={}", level, exp, percentage);
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
                        #[cfg(debug_assertions)]
                        eprintln!("❌ ExpCalculator error: {}", e);
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
        let state = self.state.lock().await;
        if state.is_tracking {
            #[cfg(debug_assertions)]
            println!("⚠️  Already tracking, ignoring restart request");
            return Ok(());
        }
        drop(state);

        #[cfg(debug_assertions)]
        println!("📋 Starting tracking with ROIs:");
        #[cfg(debug_assertions)]
        println!("   Level ROI: ({}, {}) {}x{}", level_roi.x, level_roi.y, level_roi.width, level_roi.height);
        #[cfg(debug_assertions)]
        println!("   EXP ROI: ({}, {}) {}x{}", exp_roi.x, exp_roi.y, exp_roi.width, exp_roi.height);

        // Reset stop signal
        *self.stop_signal.lock().await = false;

        // Reset state (only if not tracking)
        let mut state = self.state.lock().await;
        *state = TrackerState::new()?;
        state.is_tracking = true;
        drop(state);

        // Spawn OCR tasks: combined Level+Inventory (shared capture), separate EXP, health check
        self.spawn_combined_level_inventory_loop(level_roi, self.app.clone());
        self.spawn_exp_loop(exp_roi, self.app.clone());
        self.spawn_health_check_loop(self.app.clone());

        #[cfg(debug_assertions)]
        println!("🚀 OCR Tracker started with 3 OCR tasks (Level, EXP, Inventory) + health monitor");
        Ok(())
    }

    /// Stop all OCR loops
    pub async fn stop_tracking(&self) {
        *self.stop_signal.lock().await = true;
        let mut state = self.state.lock().await;
        state.is_tracking = false;
        #[cfg(debug_assertions)]
        println!("⏹️  OCR Tracker stopped");
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
        #[cfg(debug_assertions)]
        println!("🔄 OCR Tracker reset");
        Ok(())
    }

    /// Combined Level + Inventory OCR loop (shares full screen capture for efficiency)
    fn spawn_combined_level_inventory_loop(&self, _roi: Roi, app: AppHandle) {
        let state = Arc::clone(&self.state);
        let stop_signal = Arc::clone(&self.stop_signal);
        let screen_capture = Arc::clone(&self.screen_capture);
        let ocr_service = Arc::clone(&self.ocr_service);

        tokio::spawn(async move {
            #[cfg(debug_assertions)]
            println!("🚀 Combined LEVEL + INVENTORY OCR task started (shared capture with ROI memoization)");

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
                                #[cfg(debug_assertions)]
                                println!("⏭️  Level+Inventory: Skipped (identical image)");
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
                            let start = start.clone();
                            let memoized_roi = memoized_level_roi.clone();

                            let updated_roi = tokio::spawn(async move {
                                let level_result = tokio::task::spawn_blocking(move || {
                                    // Try memoized ROI first (fast path)
                                    if let Some((left, top, right, bottom)) = memoized_roi {
                                        #[cfg(debug_assertions)]
                                        println!("🎯 Using memoized Level ROI: ({}, {}, {}, {})", left, top, right, bottom);

                                        // Crop to memoized region
                                        let width = right - left + 1;
                                        let height = bottom - top + 1;
                                        let cropped = image.crop_imm(left, top, width, height);

                                        // Try recognition on cropped region
                                        if let Ok(result) = tokio::runtime::Handle::current().block_on(
                                            http_client.recognize_level(&cropped)
                                        ) {
                                            #[cfg(debug_assertions)]
                                            println!("✅ Level OCR succeeded with memoized ROI");
                                            return Ok((result, Some((left, top, right, bottom))));
                                        }

                                        #[cfg(debug_assertions)]
                                        println!("⚠️  Memoized ROI failed, falling back to full detection");
                                    }

                                    // Fallback: Full detection (slow path)
                                    #[cfg(debug_assertions)]
                                    println!("🔍 Performing full Level detection");

                                    let result = tokio::runtime::Handle::current().block_on(
                                        http_client.recognize_level(&*image)
                                    )?;

                                    // Try to detect ROI for memoization
                                    let roi = http_client.detect_level_roi(&*image).ok();
                                    if let Some(coords) = roi {
                                        #[cfg(debug_assertions)]
                                        println!("💾 Memoizing Level ROI: {:?}", coords);
                                    }

                                    Ok((result, roi))
                                }).await;

                                match level_result {
                                    Ok(Ok((result, roi))) => (Ok(result), roi),
                                    Ok(Err(e)) => (Err(e), None),
                                    Err(e) => (Err(format!("Task failed: {}", e)), None)
                                }
                            }).await;

                            let (level_result, new_roi) = match updated_roi {
                                Ok(result) => result,
                                Err(e) => (Err(format!("Spawn failed: {}", e)), None)
                            };

                            // Update memoized ROI if we got a new one
                            if new_roi.is_some() {
                                memoized_level_roi = new_roi;
                            }

                            match level_result {
                                Ok(result) => {
                                    let should_emit = {
                                        let mut state = state.lock().await;
                                        state.update_level(result.level)
                                    };

                                    // Emit event to Frontend if level changed
                                    if should_emit {
                                        #[cfg(debug_assertions)]
                                        println!("📤 Emitting level update event: {}", result.level);
                                        if let Err(e) = app.emit("ocr:level-update", LevelUpdate { level: result.level }) {
                                            eprintln!("❌ Failed to emit level update event: {}", e);
                                        }
                                    } else {
                                        #[cfg(debug_assertions)]
                                        println!("⏭️  Level {} unchanged, skipping emit", result.level);
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
                        }

                                // Spawn Inventory OCR as independent task with ROI memoization
                        {
                            let ocr_service_clone = Arc::clone(&ocr_service);
                            let ocr_service_clone2 = Arc::clone(&ocr_service); // For outer async block
                            let image = Arc::clone(&image);
                            let app = app.clone();
                            let state = Arc::clone(&state);
                            let start = start.clone();
                            let memoized_roi = memoized_inventory_roi.clone();

                            let updated_roi = tokio::spawn(async move {
                                // We need to run async OCR calls, so we can't use spawn_blocking for the whole thing
                                // But we need blocking for image processing.
                                // Strategy: Do image processing in blocking block, then async OCR.
                                
                                let (inventory_image, roi_coords) = tokio::task::spawn_blocking(move || {
                                    let service = ocr_service_clone.lock();
                                    
                                    // Helper to standardize image to 522x255
                                    let standardize = |img: &DynamicImage| -> DynamicImage {
                                        let gray = img.to_luma8();
                                        let resized = image::imageops::resize(
                                            &gray,
                                            522,
                                            255,
                                            image::imageops::FilterType::Nearest,
                                        );
                                        DynamicImage::ImageLuma8(resized)
                                    };

                                    // Try memoized ROI first (fast path)
                                    if let Some((left, top, right, bottom)) = memoized_roi {
                                        #[cfg(debug_assertions)]
                                        {
                                            println!("🎯 Using memoized Inventory ROI: ({}, {}, {}, {})", left, top, right, bottom);
                                        }

                                        // Add padding
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

                                        // We need to find the exact inventory within this cropped region to get the 522x255 image
                                        // Use the matcher's detection logic
                                        if let Some(matcher) = &service.inventory_matcher {
                                            if let Ok((inv_img, _)) = matcher.detect_inventory_region_with_coords(&cropped) {
                                                return (Some(inv_img), Some((left, top, right, bottom)));
                                            }
                                        }
                                        
                                        #[cfg(debug_assertions)]
                                        println!("⚠️  Memoized ROI failed to re-detect inventory");
                                    }

                                    // Fallback: Full detection
                                    #[cfg(debug_assertions)]
                                    println!("🔍 Performing full Inventory region detection");

                                    if let Some(matcher) = &service.inventory_matcher {
                                        if let Ok((inv_img, coords)) = matcher.detect_inventory_region_with_coords(&*image) {
                                            #[cfg(debug_assertions)]
                                            println!("💾 Memoizing Inventory ROI: {:?}", coords);
                                            return (Some(inv_img), Some(coords));
                                        }
                                    }

                                    (None, None)
                                }).await.unwrap_or((None, None));

                                if let Some(inv_img) = inventory_image {
                                    // Load potion config
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

                                    // Get OCR client
                                    let http_client = {
                                        let service = ocr_service_clone2.lock();
                                        service.http_client.clone()
                                    };

                                    // We need the slot ROIs. Let's get them from the service.
                                    let (hp_roi, mp_roi) = {
                                        let service = ocr_service_clone2.lock();
                                        if let Some(matcher) = &service.inventory_matcher {
                                            (
                                                matcher.slot_rois.get(&potion_config.hp_potion_slot).cloned(),
                                                matcher.slot_rois.get(&potion_config.mp_potion_slot).cloned()
                                            )
                                        } else {
                                            (None, None)
                                        }
                                    };

                                    let mut hp_count = 0;
                                    let mut mp_count = 0;

                                    // Process HP Potion
                                    if let Some(roi) = hp_roi {
                                        let slot_img = inv_img.crop_imm(roi.x, roi.y, roi.width, roi.height);
                                        // Crop bottom 45% for number
let crop_h = (roi.height as f32 * 0.45) as u32;
                                        let crop_y = roi.height - crop_h;
                                        
                                        let number_part = slot_img.crop_imm(0, crop_y, roi.width, crop_h);
                                        
                                        // Resize to 92x43 and apply binary threshold
                                        let resized = image::imageops::resize(
                                            &number_part,
                                            92,
                                            43,
                                            image::imageops::FilterType::Triangle
                                        );
                                        let mut number_img = image::DynamicImage::ImageLuma8(resized);
                                        
                                        // Binary threshold
                                        if let Some(gray) = number_img.as_mut_luma8() {
                                            for p in gray.pixels_mut() {
                                                *p = if p.0[0] > 180 { image::Luma([255]) } else { image::Luma([0]) };
                                            }
                                        }
                                        
                                        if let Ok(count) = http_client.recognize_hp_potion_count(&number_img).await {
                                            hp_count = count;
                                        }
                                    }

                                    // Process MP Potion
                                    if let Some(roi) = mp_roi {
                                        let slot_img = inv_img.crop_imm(roi.x, roi.y, roi.width, roi.height);
                                        // Crop bottom 45% for number
                                        let crop_h = (roi.height as f32 * 0.45) as u32;
                                        let crop_y = roi.height - crop_h;
                                        
                                        let number_part = slot_img.crop_imm(0, crop_y, roi.width, crop_h);
                                        
                                        // Resize to 92x43 and apply binary threshold
                                        let resized = image::imageops::resize(
                                            &number_part,
                                            92,
                                            43,
                                            image::imageops::FilterType::Triangle
                                        );
                                        let mut number_img = image::DynamicImage::ImageLuma8(resized);
                                        
                                        // Binary threshold
                                        if let Some(gray) = number_img.as_mut_luma8() {
                                            for p in gray.pixels_mut() {
                                                *p = if p.0[0] > 180 { image::Luma([255]) } else { image::Luma([0]) };
                                            }
                                        }
                                        
                                        if let Ok(count) = http_client.recognize_mp_potion_count(&number_img).await {
                                            mp_count = count;
                                        }
                                    }

                                    // Update state
                                    let mut state = state.lock().await;
                                    state.hp_potion_count = Some(hp_count);
                                    state.mp_potion_count = Some(mp_count);
                                    
                                    // Update calculators
                                    let (hp_used, hp_per_min) = state.hp_calculator.update(hp_count);
                                    state.latest_stats.hp_potions_used = hp_used as i32;
                                    state.latest_stats.hp_potions_per_minute = hp_per_min;

                                    let (mp_used, mp_per_min) = state.mp_calculator.update(mp_count);
                                    state.latest_stats.mp_potions_used = mp_used as i32;
                                    state.latest_stats.mp_potions_per_minute = mp_per_min;
                                    drop(state);

                                    // Emit events
                                    #[cfg(debug_assertions)]
                                    println!("📤 Emitting HP potion update event: {}", hp_count);
                                    app.emit("ocr:hp-potion-update", HpPotionUpdate { hp_potion_count: hp_count }).ok();

                                    #[cfg(debug_assertions)]
                                    println!("📤 Emitting MP potion update event: {}", mp_count);
                                    app.emit("ocr:mp-potion-update", MpPotionUpdate { mp_potion_count: mp_count }).ok();
                                    
                                    #[cfg(debug_assertions)]
                                    {
                                        let elapsed = start.elapsed().as_millis();
                                        println!("✅ Inventory OCR (ONNX): HP={} MP={} - {}ms", hp_count, mp_count, elapsed);
                                    }

                                    (Ok(()), roi_coords)
                                } else {
                                    (Err("Failed to detect inventory".to_string()), None)
                                }
                            }).await;

                            let (_, new_roi) = match updated_roi {
                                Ok(result) => result,
                                Err(e) => (Err(format!("Spawn failed: {}", e)), None)
                            };

                            // Update memoized ROI if we got a new one
                            if new_roi.is_some() {
                                memoized_inventory_roi = new_roi;
                            }
                        }

                        // Update cache
                        last_image_bytes = Some(current_bytes);
                    }
                    Err(e) => {
                        #[cfg(debug_assertions)]
                        eprintln!("❌ Full screen capture failed: {}", e);
                    }
                }

                sleep(Duration::from_millis(500)).await;
            }

            #[cfg(debug_assertions)]
            println!("⏹️  Combined Level+Inventory OCR task stopped");
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
            #[cfg(debug_assertions)]
            println!("🚀 EXP OCR task started - using shared OCR service");

            // Image cache for duplicate detection
            let mut last_image_bytes: Option<Vec<u8>> = None;

            while !*stop_signal.lock().await {
                let start = std::time::Instant::now();

                match screen_capture.capture_region(&roi) {
                    Ok(image) => {
                        #[cfg(debug_assertions)]
                        println!("📸 EXP capture: ROI({},{},{}x{}), Image({}x{})", 
                            roi.x, roi.y, roi.width, roi.height, 
                            image.width(), image.height());
                        
                        // Convert image to raw bytes for comparison
                        let current_bytes = image.as_bytes().to_vec();

                        // Check if image is identical to last capture
                        if let Some(ref last_bytes) = last_image_bytes {
                            if current_bytes == *last_bytes {
                                #[cfg(debug_assertions)]
                                println!("⏭️  EXP: Skipped (identical image)");
                                sleep(Duration::from_millis(500)).await;
                                continue;
                            }
                        }

                        // Image changed - run OCR
                        let http_client = {
                            let service = ocr_service.lock();
                            service.http_client.clone()
                        };
                        
                        #[cfg(debug_assertions)]
                        println!("🔍 Sending EXP image to OCR server...");
                        
                        match http_client.recognize_exp(&image).await {
                            Ok(result) => {
                                let should_emit = {
                                    let mut state_guard = state.lock().await;
                                    state_guard.update_exp_data(result.absolute, result.percentage)
                                };

                                // Emit event to Frontend if EXP changed
                                if should_emit {
                                    #[cfg(debug_assertions)]
                                    println!("📤 Emitting EXP update event: {} [{}%]", result.absolute, result.percentage);
                                    if let Err(e) = app.emit("ocr:exp-update", ExpUpdate {
                                        exp: result.absolute,
                                        percentage: result.percentage
                                    }) {
                                        eprintln!("❌ Failed to emit EXP update event: {}", e);
                                    }
                                } else {
                                    #[cfg(debug_assertions)]
                                    println!("⏭️  EXP unchanged, skipping emit");
                                }

                                #[cfg(debug_assertions)]
                                {
                                    let elapsed = start.elapsed().as_millis();
                                    println!(
                                        "✅ EXP OCR: {} [{}%] ({}ms)",
                                        result.absolute, result.percentage, elapsed
                                    );
                                }
                            }
                            Err(e) => {
                                #[cfg(debug_assertions)]
                                eprintln!("❌ EXP OCR failed: {}", e);
                            }
                        }

                        // Update cache
                        last_image_bytes = Some(current_bytes);
                    }
                    Err(e) => {
                        #[cfg(debug_assertions)]
                        eprintln!("❌ EXP capture failed: {}", e);
                    }
                }

                sleep(Duration::from_millis(500)).await;
            }

            #[cfg(debug_assertions)]
            println!("⏹️  EXP OCR task stopped");
        });
    }

    // Unified Inventory OCR loop - Rust native with automatic ROI detection
    fn spawn_inventory_loop(&self, app: AppHandle) {
        let state = Arc::clone(&self.state);
        let stop_signal = Arc::clone(&self.stop_signal);
        let screen_capture = Arc::clone(&self.screen_capture);
        let ocr_service = Arc::clone(&self.ocr_service);

        tokio::spawn(async move {
            #[cfg(debug_assertions)]
            println!("🚀 Inventory OCR task started - Rust native with auto ROI");

            // Image cache for duplicate detection
            let mut last_image_bytes: Option<Vec<u8>> = None;

            while !*stop_signal.lock().await {
                let start = std::time::Instant::now();

                // Capture full screen for automatic inventory detection
                match screen_capture.capture_full() {
                    Ok(image) => {
                        // Convert image to raw bytes for comparison
                        let current_bytes = image.as_bytes().to_vec();

                        // Check if image is identical to last capture
                        if let Some(ref last_bytes) = last_image_bytes {
                            if current_bytes == *last_bytes {
                                #[cfg(debug_assertions)]
                                println!("⏭️  Inventory: Skipped (identical image)");
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

                                #[cfg(debug_assertions)]
                                {
                                    let elapsed = start.elapsed().as_millis();
                                    println!("✅ Inventory OCR: HP={} ({}), MP={} ({}) - {}ms",
                                        hp_potion_count, potion_config.hp_potion_slot,
                                        mp_potion_count, potion_config.mp_potion_slot,
                                        elapsed);
                                }
                            }
                            Err(e) => {
                                #[cfg(debug_assertions)]
                                eprintln!("❌ Inventory OCR failed: {}", e);
                            }
                        }

                        // Update cache
                        last_image_bytes = Some(current_bytes);
                    }
                    Err(e) => {
                        #[cfg(debug_assertions)]
                        eprintln!("❌ Full screen capture failed: {}", e);
                    }
                }

                sleep(Duration::from_millis(500)).await;
            }

            #[cfg(debug_assertions)]
            println!("⏹️  Inventory OCR task stopped");
        });
    }


    /// Spawn health check loop - monitors OCR server health
    fn spawn_health_check_loop(&self, _app: AppHandle) {
        let state = Arc::clone(&self.state);
        let stop_signal = Arc::clone(&self.stop_signal);
        let ocr_service = Arc::clone(&self.ocr_service);  // Use shared service

        tokio::spawn(async move {
            #[cfg(debug_assertions)]
            println!("🏥 Health check loop started - using shared OCR service");

            while !*stop_signal.lock().await {
                // Use shared OCR service for health check
                let http_client = {
                    let service = ocr_service.lock();
                    service.http_client.clone()
                };
                match http_client.health_check().await {
                            Ok(_) => {
                        let mut state = state.lock().await;
                        if !state.ocr_server_healthy {
                            #[cfg(debug_assertions)]
                            println!("✅ OCR server is now healthy");
                        }
                        state.ocr_server_healthy = true;
                        state.latest_stats.ocr_server_healthy = true;
                    }
                    Err(e) => {
                        let mut state = state.lock().await;
                        if state.ocr_server_healthy {
                            #[cfg(debug_assertions)]
                            println!("❌ OCR server health check failed: {}", e);
                        }
                        state.ocr_server_healthy = false;
                        state.latest_stats.ocr_server_healthy = false;
                    }
                }

                // Check every 2 seconds
                sleep(Duration::from_secs(2)).await;
            }

            #[cfg(debug_assertions)]
            println!("⏹️  Health check loop stopped");
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
