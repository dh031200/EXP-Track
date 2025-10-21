use crate::models::exp_data::{ExpData, ExpStats, LevelExpTable};
use std::time::{Duration, Instant};

pub struct ExpCalculator {
    level_table: LevelExpTable,
    initial_data: Option<ExpData>,
    last_data: Option<ExpData>,
    start_time: Option<Instant>,
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
            completed_levels_exp: 0,
            completed_levels_percentage: 0.0,
            paused_duration: Duration::ZERO,
        })
    }

    /// Start tracking with initial data
    pub fn start(&mut self, data: ExpData) {
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
        let last = self.last_data.as_ref().ok_or("No previous data")?;

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
        let total_exp = (data.exp.saturating_sub(initial.exp)) + self.completed_levels_exp;
        let total_percentage =
            (data.percentage - initial.percentage) + self.completed_levels_percentage;
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

        self.last_data = Some(data);

        Ok(ExpStats {
            total_exp,
            total_percentage,
            total_meso,
            elapsed_seconds,
            exp_per_hour,
            percentage_per_hour,
            meso_per_hour,
        })
    }

    /// Reset calculator state
    pub fn reset(&mut self) {
        self.initial_data = None;
        self.last_data = None;
        self.start_time = None;
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
