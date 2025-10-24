use crate::commands::ocr::OcrService;
use crate::models::exp_data::ExpData;
use crate::models::roi::Roi;
use crate::services::exp_calculator::ExpCalculator;
use crate::services::screen_capture::ScreenCapture;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;

/// Current tracking statistics
#[derive(Debug, Clone, Serialize)]
pub struct TrackingStats {
    pub level: Option<i32>,
    pub exp: Option<i64>,
    pub percentage: Option<f64>,
    pub hp: Option<i32>,
    pub mp: Option<i32>,
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
    hp: Option<u32>,
    mp: Option<u32>,
    exp_calculator: ExpCalculator,
    is_tracking: bool,
    error: Option<String>,
    // Level stability tracking
    prev_level: Option<u32>,
    level_match_count: u32,
    // Session started flag
    session_started: bool,
    // Latest stats from calculator
    latest_stats: TrackingStats,
}

impl TrackerState {
    fn new() -> Result<Self, String> {
        Ok(Self {
            level: None,
            exp: None,
            percentage: None,
            hp: None,
            mp: None,
            exp_calculator: ExpCalculator::new()?,
            is_tracking: false,
            error: None,
            prev_level: None,
            level_match_count: 0,
            session_started: false,
            latest_stats: TrackingStats {
                level: None,
                exp: None,
                percentage: None,
                hp: None,
                mp: None,
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
                // Update session with HP/MP tracking
                let result = if let (Some(hp), Some(mp)) = (self.hp, self.mp) {
                    self.exp_calculator.update_with_hp_mp(data, Some(hp), Some(mp))
                } else {
                    self.exp_calculator.update(data)
                };

                match result {
                    Ok(stats) => {
                        self.latest_stats.total_exp = stats.total_exp as i64;
                        self.latest_stats.total_percentage = stats.total_percentage;
                        self.latest_stats.elapsed_seconds = stats.elapsed_seconds as i64;
                        self.latest_stats.exp_per_hour = stats.exp_per_hour as i64;
                        self.latest_stats.percentage_per_hour = stats.percentage_per_hour;
                        self.latest_stats.hp_potions_used = stats.hp_potions_used as i32;
                        self.latest_stats.mp_potions_used = stats.mp_potions_used as i32;
                        self.latest_stats.hp_potions_per_minute = stats.hp_potions_per_minute;
                        self.latest_stats.mp_potions_per_minute = stats.mp_potions_per_minute;
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
        TrackingStats {
            level: self.level.map(|l| l as i32),
            exp: self.exp.map(|e| e as i64),
            percentage: self.percentage,
            hp: self.hp.map(|h| h as i32),
            mp: self.mp.map(|m| m as i32),
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

/// Global OCR Tracker instance
pub struct OcrTracker {
    state: Arc<Mutex<TrackerState>>,
    stop_signal: Arc<Mutex<bool>>,
    screen_capture: Arc<ScreenCapture>,
}

impl OcrTracker {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            state: Arc::new(Mutex::new(TrackerState::new()?)),
            stop_signal: Arc::new(Mutex::new(false)),
            screen_capture: Arc::new(ScreenCapture::new()?),
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
        // Reset stop signal
        *self.stop_signal.lock().await = false;

        // Reset state
        let mut state = self.state.lock().await;
        *state = TrackerState::new()?;
        state.is_tracking = true;
        drop(state);

        // Spawn 4 independent OCR tasks
        self.spawn_level_loop(level_roi);
        self.spawn_exp_loop(exp_roi);
        self.spawn_hp_loop(hp_roi);
        self.spawn_mp_loop(mp_roi);

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

    // Independent Level OCR loop with dedicated model instance
    fn spawn_level_loop(&self, roi: Roi) {
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

            while !*stop_signal.lock().await {
                let start = std::time::Instant::now();

                match screen_capture.capture_region(&roi) {
                    Ok(image) => match ocr_service.recognize_level(&image).await {
                        Ok(result) => {
                            let mut state = state.lock().await;
                            state.update_level(result.level);
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
                    },
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

    // Independent EXP OCR loop with dedicated model instance
    fn spawn_exp_loop(&self, roi: Roi) {
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

            while !*stop_signal.lock().await {
                let start = std::time::Instant::now();

                match screen_capture.capture_region(&roi) {
                    Ok(image) => match ocr_service.recognize_exp(&image).await {
                        Ok(result) => {
                            let mut state_guard = state.lock().await;
                            state_guard.update_exp_data(result.absolute, result.percentage);

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
                    },
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

    // Independent HP OCR loop with dedicated model instance
    fn spawn_hp_loop(&self, roi: Roi) {
        let state = Arc::clone(&self.state);
        let stop_signal = Arc::clone(&self.stop_signal);
        let screen_capture = Arc::clone(&self.screen_capture);

        tokio::spawn(async move {
            #[cfg(debug_assertions)]
            println!("🚀 HP OCR task started - loading dedicated model instance");

            // Create dedicated OCR service instance for this task
            let ocr_service = match OcrService::new() {
                Ok(service) => service,
                Err(e) => {
                    eprintln!("❌ HP OCR: Failed to initialize OCR service: {}", e);
                    return;
                }
            };

            #[cfg(debug_assertions)]
            println!("✅ HP OCR: Model loaded successfully");

            while !*stop_signal.lock().await {
                let start = std::time::Instant::now();

                match screen_capture.capture_region(&roi) {
                    Ok(image) => match ocr_service.recognize_hp(&image).await {
                        Ok(hp) => {
                            let mut state = state.lock().await;
                            state.hp = Some(hp);
                            #[cfg(debug_assertions)]
                            {
                                let elapsed = start.elapsed().as_millis();
                                println!("✅ HP OCR: {} ({}ms)", hp, elapsed);
                            }
                        }
                        Err(e) => {
                            #[cfg(debug_assertions)]
                            eprintln!("❌ HP OCR failed: {}", e);
                        }
                    },
                    Err(e) => {
                        #[cfg(debug_assertions)]
                        eprintln!("❌ HP capture failed: {}", e);
                    }
                }

                sleep(Duration::from_millis(500)).await;
            }

            #[cfg(debug_assertions)]
            println!("⏹️  HP OCR task stopped");
        });
    }

    // Independent MP OCR loop with dedicated model instance
    fn spawn_mp_loop(&self, roi: Roi) {
        let state = Arc::clone(&self.state);
        let stop_signal = Arc::clone(&self.stop_signal);
        let screen_capture = Arc::clone(&self.screen_capture);

        tokio::spawn(async move {
            #[cfg(debug_assertions)]
            println!("🚀 MP OCR task started - loading dedicated model instance");

            // Create dedicated OCR service instance for this task
            let ocr_service = match OcrService::new() {
                Ok(service) => service,
                Err(e) => {
                    eprintln!("❌ MP OCR: Failed to initialize OCR service: {}", e);
                    return;
                }
            };

            #[cfg(debug_assertions)]
            println!("✅ MP OCR: Model loaded successfully");

            while !*stop_signal.lock().await {
                let start = std::time::Instant::now();

                match screen_capture.capture_region(&roi) {
                    Ok(image) => match ocr_service.recognize_mp(&image).await {
                        Ok(mp) => {
                            let mut state = state.lock().await;
                            state.mp = Some(mp);
                            #[cfg(debug_assertions)]
                            {
                                let elapsed = start.elapsed().as_millis();
                                println!("✅ MP OCR: {} ({}ms)", mp, elapsed);
                            }
                        }
                        Err(e) => {
                            #[cfg(debug_assertions)]
                            eprintln!("❌ MP OCR failed: {}", e);
                        }
                    },
                    Err(e) => {
                        #[cfg(debug_assertions)]
                        eprintln!("❌ MP capture failed: {}", e);
                    }
                }

                sleep(Duration::from_millis(500)).await;
            }

            #[cfg(debug_assertions)]
            println!("⏹️  MP OCR task stopped");
        });
    }
}
