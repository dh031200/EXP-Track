use crate::commands::ocr::OcrService;
use crate::models::exp_data::ExpData;
use crate::models::roi::Roi;
use crate::services::exp_calculator::ExpCalculator;
use crate::services::hp_potion_calculator::HpPotionCalculator;
use crate::services::mp_potion_calculator::MpPotionCalculator;
use crate::services::screen_capture::ScreenCapture;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
use tokio::time::sleep;

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
            },
        })
    }

    /// Update level with stability check (needs 2 consecutive matches)
    fn update_level(&mut self, new_level: u32) {
        if let Some(prev) = self.prev_level {
            if prev == new_level {
                self.level_match_count += 1;
                if self.level_match_count >= 2 {
                    // Stable - update confirmed level
                    self.level = Some(new_level);
                }
            } else {
                // Different value - reset counter
                self.prev_level = Some(new_level);
                self.level_match_count = 1;
            }
        } else {
            // First reading
            self.prev_level = Some(new_level);
            self.level_match_count = 1;
        }
    }

    /// Update EXP and trigger calculator update
    fn update_exp_data(&mut self, exp: u64, percentage: f64) {
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
}

impl OcrTracker {
    pub fn new(app: AppHandle) -> Result<Self, String> {
        Ok(Self {
            state: Arc::new(Mutex::new(TrackerState::new()?)),
            stop_signal: Arc::new(Mutex::new(false)),
            screen_capture: Arc::new(ScreenCapture::new()?),
            app,
        })
    }

    /// Start OCR tracking with 4 independent parallel tasks
    pub async fn start_tracking(
        &self,
        level_roi: Roi,
        exp_roi: Roi,
        hp_roi: Roi,
        mp_roi: Roi,
    ) -> Result<(), String> {
        // Check if already tracking - prevent reinitialization
        let state = self.state.lock().await;
        if state.is_tracking {
            #[cfg(debug_assertions)]
            println!("⚠️  Already tracking, ignoring restart request");
            return Ok(());
        }
        drop(state);

        // Reset stop signal
        *self.stop_signal.lock().await = false;

        // Reset state (only if not tracking)
        let mut state = self.state.lock().await;
        *state = TrackerState::new()?;
        state.is_tracking = true;
        drop(state);

        // Spawn 4 independent OCR tasks
        self.spawn_level_loop(level_roi, self.app.clone());
        self.spawn_exp_loop(exp_roi, self.app.clone());
        self.spawn_hp_potion_loop(hp_roi, self.app.clone());
        self.spawn_mp_potion_loop(mp_roi, self.app.clone());

        #[cfg(debug_assertions)]
        println!("🚀 OCR Tracker started with 4 parallel tasks");
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

    // Independent Level OCR loop with dedicated model instance + image caching
    fn spawn_level_loop(&self, roi: Roi, app: AppHandle) {
        let state = Arc::clone(&self.state);
        let stop_signal = Arc::clone(&self.stop_signal);
        let screen_capture = Arc::clone(&self.screen_capture);

        tokio::spawn(async move {
            #[cfg(debug_assertions)]
            println!("🚀 LEVEL OCR task started - loading dedicated model instance");

            // Create dedicated OCR service instance for this task
            let ocr_service = match OcrService::new() {
                Ok(service) => service,
                Err(e) => {
                    eprintln!("❌ LEVEL OCR: Failed to initialize OCR service: {}", e);
                    return;
                }
            };

            #[cfg(debug_assertions)]
            println!("✅ LEVEL OCR: Model loaded successfully");

            // Image cache for duplicate detection
            let mut last_image_bytes: Option<Vec<u8>> = None;

            while !*stop_signal.lock().await {
                let start = std::time::Instant::now();

                match screen_capture.capture_region(&roi) {
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

                        // Image changed - run OCR
                        match ocr_service.recognize_level(&image).await {
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
                        eprintln!("❌ LEVEL capture failed: {}", e);
                    }
                }

                sleep(Duration::from_millis(500)).await;
            }

            #[cfg(debug_assertions)]
            println!("⏹️  LEVEL OCR task stopped");
        });
    }

    // Independent EXP OCR loop with dedicated model instance + image caching
    fn spawn_exp_loop(&self, roi: Roi, app: AppHandle) {
        let state = Arc::clone(&self.state);
        let stop_signal = Arc::clone(&self.stop_signal);
        let screen_capture = Arc::clone(&self.screen_capture);

        tokio::spawn(async move {
            #[cfg(debug_assertions)]
            println!("🚀 EXP OCR task started - loading dedicated model instance");

            // Create dedicated OCR service instance for this task
            let ocr_service = match OcrService::new() {
                Ok(service) => service,
                Err(e) => {
                    eprintln!("❌ EXP OCR: Failed to initialize OCR service: {}", e);
                    return;
                }
            };

            #[cfg(debug_assertions)]
            println!("✅ EXP OCR: Model loaded successfully");

            // Image cache for duplicate detection
            let mut last_image_bytes: Option<Vec<u8>> = None;

            while !*stop_signal.lock().await {
                let start = std::time::Instant::now();

                match screen_capture.capture_region(&roi) {
                    Ok(image) => {
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
                        match ocr_service.recognize_exp(&image).await {
                            Ok(result) => {
                                let mut state_guard = state.lock().await;
                                state_guard.update_exp_data(result.absolute, result.percentage);

                                // Emit event to Frontend immediately
                                app.emit("ocr:exp-update", ExpUpdate {
                                    exp: result.absolute,
                                    percentage: result.percentage
                                }).ok();

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

    // Independent HP Potion OCR loop with dedicated model instance + image caching
    fn spawn_hp_potion_loop(&self, roi: Roi, app: AppHandle) {
        let state = Arc::clone(&self.state);
        let stop_signal = Arc::clone(&self.stop_signal);
        let screen_capture = Arc::clone(&self.screen_capture);

        tokio::spawn(async move {
            #[cfg(debug_assertions)]
            println!("🚀 HP Potion OCR task started - loading dedicated model instance");

            // Create dedicated OCR service instance for this task
            let ocr_service = match OcrService::new() {
                Ok(service) => service,
                Err(e) => {
                    eprintln!("❌ HP Potion OCR: Failed to initialize OCR service: {}", e);
                    return;
                }
            };

            #[cfg(debug_assertions)]
            println!("✅ HP Potion OCR: Model loaded successfully");

            // Image cache for duplicate detection
            let mut last_image_bytes: Option<Vec<u8>> = None;

            while !*stop_signal.lock().await {
                let start = std::time::Instant::now();

                match screen_capture.capture_region(&roi) {
                    Ok(image) => {
                        // Convert image to raw bytes for comparison
                        let current_bytes = image.as_bytes().to_vec();

                        // Check if image is identical to last capture
                        if let Some(ref last_bytes) = last_image_bytes {
                            if current_bytes == *last_bytes {
                                #[cfg(debug_assertions)]
                                println!("⏭️  HP Potion: Skipped (identical image)");
                                sleep(Duration::from_millis(500)).await;
                                continue;
                            }
                        }

                        // Image changed - run OCR
                        match ocr_service.recognize_hp_potion_count(&image).await {
                            Ok(hp_potion_count) => {
                                let mut state = state.lock().await;
                                state.hp_potion_count = Some(hp_potion_count);

                                // Update HP potion count - INDEPENDENT CALCULATOR
                                let (hp_used, hp_per_min) = state.hp_calculator.update(hp_potion_count);

                                // Cache HP potion stats
                                state.latest_stats.hp_potions_used = hp_used as i32;
                                state.latest_stats.hp_potions_per_minute = hp_per_min;

                                #[cfg(debug_assertions)]
                                println!("🧪 HP Potion: used={}, per_min={:.2}",
                                    hp_used, hp_per_min);

                                // Emit event to Frontend immediately
                                app.emit("ocr:hp-potion-update", HpPotionUpdate { hp_potion_count }).ok();

                                #[cfg(debug_assertions)]
                                {
                                    let elapsed = start.elapsed().as_millis();
                                    println!("✅ HP Potion OCR: {} ({}ms)", hp_potion_count, elapsed);
                                }
                            }
                            Err(e) => {
                                #[cfg(debug_assertions)]
                                eprintln!("❌ HP Potion OCR failed: {}", e);
                            }
                        }

                        // Update cache
                        last_image_bytes = Some(current_bytes);
                    }
                    Err(e) => {
                        #[cfg(debug_assertions)]
                        eprintln!("❌ HP Potion capture failed: {}", e);
                    }
                }

                sleep(Duration::from_millis(500)).await;
            }

            #[cfg(debug_assertions)]
            println!("⏹️  HP Potion OCR task stopped");
        });
    }

    // Independent MP Potion OCR loop with dedicated model instance + image caching
    fn spawn_mp_potion_loop(&self, roi: Roi, app: AppHandle) {
        let state = Arc::clone(&self.state);
        let stop_signal = Arc::clone(&self.stop_signal);
        let screen_capture = Arc::clone(&self.screen_capture);

        tokio::spawn(async move {
            #[cfg(debug_assertions)]
            println!("🚀 MP Potion OCR task started - loading dedicated model instance");

            // Create dedicated OCR service instance for this task
            let ocr_service = match OcrService::new() {
                Ok(service) => service,
                Err(e) => {
                    eprintln!("❌ MP Potion OCR: Failed to initialize OCR service: {}", e);
                    return;
                }
            };

            #[cfg(debug_assertions)]
            println!("✅ MP Potion OCR: Model loaded successfully");

            // Image cache for duplicate detection
            let mut last_image_bytes: Option<Vec<u8>> = None;

            while !*stop_signal.lock().await {
                let start = std::time::Instant::now();

                match screen_capture.capture_region(&roi) {
                    Ok(image) => {
                        // Convert image to raw bytes for comparison
                        let current_bytes = image.as_bytes().to_vec();

                        // Check if image is identical to last capture
                        if let Some(ref last_bytes) = last_image_bytes {
                            if current_bytes == *last_bytes {
                                #[cfg(debug_assertions)]
                                println!("⏭️  MP Potion: Skipped (identical image)");
                                sleep(Duration::from_millis(500)).await;
                                continue;
                            }
                        }

                        // Image changed - run OCR
                        match ocr_service.recognize_mp_potion_count(&image).await {
                            Ok(mp_potion_count) => {
                                let mut state = state.lock().await;
                                state.mp_potion_count = Some(mp_potion_count);

                                // Update MP potion count - INDEPENDENT CALCULATOR
                                let (mp_used, mp_per_min) = state.mp_calculator.update(mp_potion_count);

                                // Cache MP potion stats
                                state.latest_stats.mp_potions_used = mp_used as i32;
                                state.latest_stats.mp_potions_per_minute = mp_per_min;

                                #[cfg(debug_assertions)]
                                println!("💊 MP Potion: used={}, per_min={:.2}",
                                    mp_used, mp_per_min);

                                // Emit event to Frontend immediately
                                app.emit("ocr:mp-potion-update", MpPotionUpdate { mp_potion_count }).ok();

                                #[cfg(debug_assertions)]
                                {
                                    let elapsed = start.elapsed().as_millis();
                                    println!("✅ MP Potion OCR: {} ({}ms)", mp_potion_count, elapsed);
                                }
                            }
                            Err(e) => {
                                #[cfg(debug_assertions)]
                                eprintln!("❌ MP Potion OCR failed: {}", e);
                            }
                        }

                        // Update cache
                        last_image_bytes = Some(current_bytes);
                    }
                    Err(e) => {
                        #[cfg(debug_assertions)]
                        eprintln!("❌ MP Potion capture failed: {}", e);
                    }
                }

                sleep(Duration::from_millis(500)).await;
            }

            #[cfg(debug_assertions)]
            println!("⏹️  MP Potion OCR task stopped");
        });
    }
}
