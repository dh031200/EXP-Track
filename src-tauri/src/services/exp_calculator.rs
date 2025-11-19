use crate::models::exp_data::{ExpData, ExpStats, LevelExpTable};
use std::time::{Duration, Instant};

pub struct ExpCalculator {
    level_table: LevelExpTable,
    initial_data: Option<ExpData>,
    last_data: Option<ExpData>,
    pub start_time: Option<Instant>,
    start_level: u32,  // Original starting level (never changes after session start)
    pub completed_levels_exp: u64,
    pub completed_levels_percentage: f64,
    paused_duration: Duration,
}

impl ExpCalculator {
    /// Create a new ExpCalculator with level table
    pub fn new() -> Result<Self, String> {
        let level_table = LevelExpTable::load()?;

        Ok(Self {
            level_table,
            initial_data: None,
            last_data: None,
            start_time: None,
            start_level: 0,
            completed_levels_exp: 0,
            completed_levels_percentage: 0.0,
            paused_duration: Duration::ZERO,
        })
    }

    /// Start tracking with initial data
    pub fn start(&mut self, data: ExpData) {
        self.start_level = data.level;
        self.initial_data = Some(data.clone());
        self.last_data = Some(data);
        self.start_time = Some(Instant::now());
        self.completed_levels_exp = 0;
        self.completed_levels_percentage = 0.0;
        self.paused_duration = Duration::ZERO;
    }

    /// Update with new data and calculate statistics
    pub fn update(&mut self, data: ExpData) -> Result<ExpStats, String> {
        let initial = self
            .initial_data
            .as_ref()
            .ok_or("Calculator not started")?;

        // Clone last_data early to avoid borrow conflicts
        let last = self.last_data.as_ref().ok_or("No previous data")?.clone();

        // Detect OCR errors: if exp change is unrealistic (>10x or <0.1x from last reading)
        // This handles cases where OCR misreads digits (e.g., bracket '[' becomes '1')
        if data.level == initial.level {
            // Check against LAST reading (not initial) for better accuracy
            if let Some(ref last) = self.last_data {
                if last.level == data.level {
                    // 1. Negative EXP Check: EXP should never decrease within the same level
                    // Allow small variance for potential minor OCR wobbles, but generally NO drops allowed
                    if data.exp < last.exp {
                         #[cfg(debug_assertions)]
                        {
                            println!("🦀 [Calculator] ⚠️ OCR ERROR: Negative EXP gain detected ({} -> {})", last.exp, data.exp);
                            println!("🦀 [Calculator] 🚫 Rejecting drop in EXP within same level");
                        }
                        return self.update(last.clone());
                    }

                    // 2. Ratio Check: Only apply for meaningful values (> 1000) to avoid division by zero or small number volatility
                    if last.exp > 1000 {
                        let ratio = data.exp as f64 / last.exp as f64;

                        // Detect both explosions (ratio > 10) and significant drops (ratio < 0.1)
                        // Also check for impossibly high gains in short time (e.g. > 200% gain in 1 second is suspicious unless low levels)
                        if ratio > 10.0 || ratio < 0.1 {
                            // Don't update last_data - keep the good value
                            // Return stats based on last good data
                            return self.update(last.clone());
                        }
                    }
                }
            }
        }

        // Re-fetch initial after potential reset
        let initial = self.initial_data.as_ref().unwrap();

        // Handle level up
        if data.level > last.level {

            let max_exp_result = self.level_table.get_exp_for_level(last.level);
            
            let exp_gained_from_prev_level = match max_exp_result {
                Some(max_exp) => max_exp.saturating_sub(initial.exp),
                None => {
                     #[cfg(debug_assertions)]
                     println!("🦀 [Calculator] ⚠️ Unknown Max EXP for level {}. Assuming 0 remaining gain from prev level.", last.level);
                     0 
                }
            };
            
            // Total gained = Remainder of old level + All of new level (current data.exp)
            // This ensures if we go 129 (99%) -> 130 (1%), we gain that 1% + the missing 1% of 129.
            // Note: We rely on data.exp being "fresh" (starting from 0 or low value).
            // If user connects late (130 | 50%), we treat that 50% as gained this session if we just leveled up.
            let total_transition_gain = exp_gained_from_prev_level + data.exp;

            self.completed_levels_exp += total_transition_gain;

            let percentage_gained = 100.0 - initial.percentage;
            self.completed_levels_percentage += percentage_gained;

            // Reset initial data for new level -> It effectively starts "now" with the current data
            // We set initial.exp to data.exp so that the "diff" calculation below works naturally (diff will be 0 initially)
            // But wait, if we set initial to data, the update logic below calculates `data.exp - initial.exp`.
            // If we just added `data.exp` to `completed_levels_exp`, we should set initial.exp to `data.exp`.
            self.initial_data = Some(ExpData {
                level: data.level,
                exp: data.exp, 
                percentage: data.percentage,
                meso: data.meso,
            });
            
            // Update 'initial' reference for the calculation below
            // We just replaced self.initial_data, so get the new one.
            // The 'initial' variable in the outer scope is now stale.
        }

        // Re-fetch initial (it might have changed due to level up)
        let initial = self.initial_data.as_ref().unwrap();
        
        // Calculate accumulated values
        // If we just leveled up, initial.exp == data.exp, so exp_diff is 0.
        // The gain was already added to `completed_levels_exp`.
        let exp_diff = data.exp.saturating_sub(initial.exp);
        let total_exp = exp_diff + self.completed_levels_exp;
        let percentage_diff = data.percentage - initial.percentage;
        let total_percentage = percentage_diff + self.completed_levels_percentage;

        let total_meso = data
            .meso
            .unwrap_or(0)
            .saturating_sub(initial.meso.unwrap_or(0));

        // Calculate elapsed time
        let elapsed = self
            .start_time
            .ok_or("Start time not set")?
            .elapsed()
            .saturating_sub(self.paused_duration);
        let elapsed_seconds = elapsed.as_secs();

        // Calculate hourly averages
        let exp_per_hour = if elapsed_seconds > 0 {
            (total_exp * 3600) / elapsed_seconds
        } else {
            0
        };

        let percentage_per_hour = if elapsed_seconds > 0 {
            (total_percentage * 3600.0) / elapsed_seconds as f64
        } else {
            0.0
        };

        let meso_per_hour = if elapsed_seconds > 0 {
            (total_meso * 3600) / elapsed_seconds
        } else {
            0
        };

        // Get current and start levels (before moving data)
        let current_level = data.level;
        let start_level = self.start_level;  // Use stored start level (never changes)
        let levels_gained = current_level.saturating_sub(start_level);

        // Calculate per-minute average
        let exp_per_minute = if elapsed_seconds > 0 {
            (total_exp * 60) / elapsed_seconds
        } else {
            0
        };

        self.last_data = Some(data);

        Ok(ExpStats {
            total_exp,
            total_percentage,
            total_meso,
            elapsed_seconds,
            exp_per_hour,
            percentage_per_hour,
            meso_per_hour,
            exp_per_minute,
            current_level,
            start_level,
            levels_gained,
            // HP/MP potion stats are now managed by separate calculators
            hp_potions_used: 0,
            mp_potions_used: 0,
            hp_potions_per_minute: 0.0,
            mp_potions_per_minute: 0.0,
        })
    }

    /// Reset calculator state
    pub fn reset(&mut self) {
        self.initial_data = None;
        self.last_data = None;
        self.start_time = None;
        self.start_level = 0;
        self.completed_levels_exp = 0;
        self.completed_levels_percentage = 0.0;
        self.paused_duration = Duration::ZERO;
    }

    #[cfg(test)]
    pub fn with_level_table(mut self, table: LevelExpTable) -> Self {
        self.level_table = table;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_calculator_creation() {
        let calculator = ExpCalculator::new();
        assert!(calculator.is_ok());
    }

    #[test]
    fn test_start_tracking() {
        let mut calculator = ExpCalculator::new().unwrap();

        let initial = ExpData {
            level: 50,
            exp: 1000,
            percentage: 10.0,
            meso: Some(5000),
        };

        calculator.start(initial.clone());

        assert_eq!(calculator.initial_data, Some(initial.clone()));
        assert_eq!(calculator.last_data, Some(initial));
        assert!(calculator.start_time.is_some());
    }

    #[test]
    fn test_basic_exp_calculation() {
        let mut calculator = ExpCalculator::new().unwrap();

        let initial = ExpData {
            level: 50,
            exp: 1000,
            percentage: 10.0,
            meso: Some(5000),
        };

        calculator.start(initial);

        thread::sleep(Duration::from_millis(100));

        let updated = ExpData {
            level: 50,
            exp: 1500,
            percentage: 15.0,
            meso: Some(6000),
        };

        let stats = calculator.update(updated).unwrap();

        assert_eq!(stats.total_exp, 500);
        assert_eq!(stats.total_percentage, 5.0);
        assert_eq!(stats.total_meso, 1000);
        assert!(stats.elapsed_seconds >= 0);
        assert_eq!(stats.current_level, 50);
        assert_eq!(stats.start_level, 50);
        assert_eq!(stats.levels_gained, 0);
    }

    #[test]
    fn test_level_up_calculation() {
        let level_table = LevelExpTable::load()
            .unwrap()
            .with_levels(vec![(50, 10000), (51, 12000)]);

        let mut calculator = ExpCalculator::new().unwrap().with_level_table(level_table);

        // Start at level 50 with 9500 EXP (95%)
        let initial = ExpData {
            level: 50,
            exp: 9500,
            percentage: 95.0,
            meso: None,
        };

        calculator.start(initial);

        thread::sleep(Duration::from_millis(100));

        // Level up to 51 with 200 EXP (2%)
        let level_up = ExpData {
            level: 51,
            exp: 200,
            percentage: 2.0,
            meso: None,
        };

        let stats = calculator.update(level_up).unwrap();

        // Should calculate: (10000 - 9500) from level 50 + 200 from level 51
        assert_eq!(stats.total_exp, 500 + 200);
        assert_eq!(stats.total_percentage, 5.0 + 2.0);
        assert_eq!(stats.current_level, 51);
        assert_eq!(stats.start_level, 50);
        assert_eq!(stats.levels_gained, 1);
    }

    #[test]
    fn test_hourly_average_calculation() {
        let mut calculator = ExpCalculator::new().unwrap();

        let initial = ExpData {
            level: 50,
            exp: 0,
            percentage: 0.0,
            meso: Some(0),
        };

        calculator.start(initial);

        // Manually set elapsed time to 600 seconds (10 minutes)
        calculator.start_time = Some(Instant::now() - Duration::from_secs(600));

        let updated = ExpData {
            level: 50,
            exp: 1000,
            percentage: 10.0,
            meso: Some(5000),
        };

        let stats = calculator.update(updated).unwrap();

        // 1000 EXP in 600 seconds = (1000 / 600) * 3600 = 6000 EXP/hour
        assert_eq!(stats.exp_per_hour, 6000);

        // 10% in 600 seconds = (10 / 600) * 3600 = 60% per hour
        assert_eq!(stats.percentage_per_hour, 60.0);

        // 5000 meso in 600 seconds = (5000 / 600) * 3600 = 30000 meso/hour
        assert_eq!(stats.meso_per_hour, 30000);

        // 1000 EXP in 600 seconds = (1000 / 600) * 60 = 100 EXP/minute
        assert_eq!(stats.exp_per_minute, 100);
    }

    #[test]
    fn test_exp_per_minute_calculation() {
        let mut calculator = ExpCalculator::new().unwrap();

        let initial = ExpData {
            level: 50,
            exp: 0,
            percentage: 0.0,
            meso: None,
        };

        calculator.start(initial);

        // Manually set elapsed time to 600 seconds (10 minutes)
        calculator.start_time = Some(Instant::now() - Duration::from_secs(600));

        let updated = ExpData {
            level: 50,
            exp: 6000,
            percentage: 60.0,
            meso: None,
        };

        let stats = calculator.update(updated).unwrap();

        // 6000 EXP in 600 seconds (10 minutes) = 600 EXP/minute
        assert_eq!(stats.exp_per_minute, 600);
    }

    #[test]
    fn test_reset_calculator() {
        let mut calculator = ExpCalculator::new().unwrap();

        let initial = ExpData {
            level: 50,
            exp: 1000,
            percentage: 10.0,
            meso: Some(5000),
        };

        calculator.start(initial);
        calculator.reset();

        assert_eq!(calculator.initial_data, None);
        assert_eq!(calculator.last_data, None);
        assert_eq!(calculator.start_time, None);
        assert_eq!(calculator.completed_levels_exp, 0);
        assert_eq!(calculator.completed_levels_percentage, 0.0);
    }

    #[test]
    fn test_update_before_start_fails() {
        let mut calculator = ExpCalculator::new().unwrap();

        let data = ExpData {
            level: 50,
            exp: 1000,
            percentage: 10.0,
            meso: None,
        };

        let result = calculator.update(data);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Calculator not started");
    }
}
