use std::time::Instant;

/// MP Potion consumption tracker - completely independent
pub struct MpPotionCalculator {
    start_time: Option<Instant>,
    last_count: Option<u32>,
    total_used: u32,
    // Pending increase validation (value, consecutive_count)
    pending_increase: Option<(u32, u8)>,
}

impl MpPotionCalculator {
    pub fn new() -> Self {
        Self {
            start_time: None,
            last_count: None,
            total_used: 0,
            pending_increase: None,
        }
    }

    /// Start tracking
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        self.last_count = None;
        self.total_used = 0;
        self.pending_increase = None;
    }

    /// Reset tracking
    pub fn reset(&mut self) {
        self.start_time = None;
        self.last_count = None;
        self.total_used = 0;
        self.pending_increase = None;
    }

    /// Update MP potion count and return (total_used, per_minute_rate)
    pub fn update(&mut self, current_count: u32) -> (u32, f64) {
        const MAX_USAGE_PER_UPDATE: u32 = 10;

        if let Some(last) = self.last_count {
            if current_count < last {
                // Potion count decreased = potions used
                let used = last - current_count;

                if used > MAX_USAGE_PER_UPDATE {
                    // OCR error - reject
                    #[cfg(debug_assertions)]
                    println!("💊 [MP Calculator] ⚠️ OCR ERROR: {} -> {} (-{}) exceeds threshold ({})",
                        last, current_count, used, MAX_USAGE_PER_UPDATE);
                } else {
                    // Normal usage
                    self.total_used += used;
                    self.last_count = Some(current_count);

                    #[cfg(debug_assertions)]
                    println!("💊 [MP Calculator] Used: {} -> {} (-{}), total: {}",
                        last, current_count, used, self.total_used);
                }
            } else if current_count > last {
                // Potion count increased - validate 5 times
                match self.pending_increase {
                    Some((pending_val, count)) if pending_val == current_count => {
                        if count + 1 >= 5 {
                            // Verified - accept increase
                            self.last_count = Some(current_count);
                            self.pending_increase = None;

                            #[cfg(debug_assertions)]
                            println!("💊 [MP Calculator] ✅ Increase verified: {} -> {} (+{})",
                                last, current_count, current_count - last);
                        } else {
                            // Continue verification
                            self.pending_increase = Some((current_count, count + 1));

                            #[cfg(debug_assertions)]
                            println!("💊 [MP Calculator] 🔍 Verifying increase: {}/{}", count + 1, 5);
                        }
                    }
                    _ => {
                        // New increase - start verification
                        self.pending_increase = Some((current_count, 1));

                        #[cfg(debug_assertions)]
                        println!("💊 [MP Calculator] 🔍 New increase detected: {} -> {}, verifying...",
                            last, current_count);
                    }
                }
            } else if let Some((_, _)) = self.pending_increase {
                // Value reverted during verification
                self.pending_increase = None;

                #[cfg(debug_assertions)]
                println!("💊 [MP Calculator] 🚫 Increase cancelled (value reverted)");
            }
        } else {
            // First reading
            self.last_count = Some(current_count);
            self.start_time.get_or_insert_with(Instant::now);

            #[cfg(debug_assertions)]
            println!("💊 [MP Calculator] Started tracking: {}", current_count);
        }

        // Calculate per-minute rate
        let per_minute = if let Some(start) = self.start_time {
            let elapsed_secs = start.elapsed().as_secs();
            if elapsed_secs > 0 {
                (self.total_used as f64 * 60.0) / elapsed_secs as f64
            } else {
                0.0
            }
        } else {
            0.0
        };

        (self.total_used, per_minute)
    }
}
