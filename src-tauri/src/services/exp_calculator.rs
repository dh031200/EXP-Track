use crate::models::exp_data::{ExpData, ExpStats, LevelExpTable};
use std::time::{Duration, Instant};

pub struct ExpCalculator {
    level_table: LevelExpTable,
    initial_data: Option<ExpData>,
    last_data: Option<ExpData>,
    start_time: Option<Instant>,
    start_level: u32,  // Original starting level (never changes after session start)
    completed_levels_exp: u64,
    completed_levels_percentage: f64,
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
        println!("🦀 [Calculator] Session started: level={}, exp={}, percentage={}", data.level, data.exp, data.percentage);
        self.start_level = data.level;  // Save original starting level
        self.initial_data = Some(data.clone());
        self.last_data = Some(data);
        self.start_time = Some(Instant::now());
        self.completed_levels_exp = 0;
        self.completed_levels_percentage = 0.0;
        self.paused_duration = Duration::ZERO;
    }

    /// Update with new data and calculate statistics
    pub fn update(&mut self, data: ExpData) -> Result<ExpStats, String> {
        println!("🦀 [Calculator] update() called: level={}, exp={}, percentage={}", data.level, data.exp, data.percentage);

        let initial = self
            .initial_data
            .as_ref()
            .ok_or("Calculator not started")?;

        // Clone last_data early to avoid borrow conflicts
        let last = self.last_data.as_ref().ok_or("No previous data")?.clone();

        println!("🦀 [Calculator] initial_data: level={}, exp={}, percentage={}", initial.level, initial.exp, initial.percentage);
        println!("🦀 [Calculator] last_data: level={}, exp={}, percentage={}", last.level, last.exp, last.percentage);

        // Detect OCR errors: if new exp is significantly smaller than initial, reset baseline
        // This handles cases where the first OCR read was wrong (e.g., 4618571 instead of 461857)
        if data.level == initial.level && data.exp < initial.exp {
            let ratio = initial.exp as f64 / data.exp.max(1) as f64;
            println!("🦀 [Calculator] 🔍 OCR Check: ratio={:.2}x (threshold=10.0x)", ratio);
            // Use 10x threshold to avoid false positives (OCR digit errors typically add/remove a digit)
            if ratio >= 10.0 {
                println!("🦀 [Calculator] ⚠️ OCR ERROR DETECTED: initial_exp={} is {:.1}x larger than current_exp={}", initial.exp, ratio, data.exp);
                println!("🦀 [Calculator] 🔄 Resetting baseline to current value (likely first read was corrupted)");
                self.initial_data = Some(data.clone());
                // Reset last_data too to avoid confusing the next update
                self.last_data = Some(data.clone());
            }
        }

        // Re-fetch initial after potential reset
        let initial = self.initial_data.as_ref().unwrap();

        // Handle level up
        if data.level > last.level {
            let max_exp = self
                .level_table
                .get_exp_for_level(last.level)
                .ok_or_else(|| format!("Invalid level: {}", last.level))?;

            let exp_gained = max_exp.saturating_sub(initial.exp);
            self.completed_levels_exp += exp_gained;

            let percentage_gained = 100.0 - initial.percentage;
            self.completed_levels_percentage += percentage_gained;

            // Reset initial data for new level
            self.initial_data = Some(ExpData {
                level: data.level,
                exp: 0,
                percentage: 0.0,
                meso: data.meso,
            });
        }

        // Calculate accumulated values
        let initial = self.initial_data.as_ref().unwrap();
        let exp_diff = data.exp.saturating_sub(initial.exp);
        let total_exp = exp_diff + self.completed_levels_exp;
        let percentage_diff = data.percentage - initial.percentage;
        let total_percentage = percentage_diff + self.completed_levels_percentage;

        println!("🦀 [Calculator] Calculation: data.exp={} - initial.exp={} = exp_diff={}", data.exp, initial.exp, exp_diff);
        println!("🦀 [Calculator] Calculation: total_exp = {} + {} = {}", exp_diff, self.completed_levels_exp, total_exp);
        println!("🦀 [Calculator] Calculation: percentage_diff={}, total_percentage={}", percentage_diff, total_percentage);
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
