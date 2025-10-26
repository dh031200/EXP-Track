use crate::models::exp_data::{ExpData, ExpStats};
use crate::services::exp_calculator::ExpCalculator;
use std::sync::Mutex;
use tauri::State;

/// Global state for ExpCalculator
pub struct ExpCalculatorState(pub Mutex<ExpCalculator>);

/// Start a new EXP tracking session
#[tauri::command]
pub fn start_exp_session(
    state: State<ExpCalculatorState>,
    level: u32,
    exp: u64,
    percentage: f64,
    meso: Option<u64>,
) -> Result<String, String> {
    let mut calculator = state.0.lock().map_err(|e| format!("Failed to lock calculator: {}", e))?;

    let initial_data = ExpData {
        level,
        exp,
        percentage,
        meso,
    };

    calculator.start(initial_data);

    Ok(format!("Session started at level {} with {} EXP ({}%)", level, exp, percentage))
}

/// Add new EXP data and get updated statistics
#[tauri::command]
pub fn add_exp_data(
    state: State<ExpCalculatorState>,
    level: u32,
    exp: u64,
    percentage: f64,
    meso: Option<u64>,
) -> Result<ExpStats, String> {
    println!("🦀 [Rust] add_exp_data called: level={}, exp={}, percentage={}", level, exp, percentage);

    let mut calculator = state.0.lock().map_err(|e| format!("Failed to lock calculator: {}", e))?;

    let data = ExpData {
        level,
        exp,
        percentage,
        meso,
    };

    let result = calculator.update(data);

    match &result {
        Ok(stats) => println!("🦀 [Rust] Calculated stats: total_exp={}, total_percentage={}", stats.total_exp, stats.total_percentage),
        Err(e) => println!("🦀 [Rust] Error: {}", e),
    }

    result
}

/// Get current EXP statistics
/// Note: Currently requires calling add_exp_data to get stats
#[tauri::command]
pub fn get_exp_stats(state: State<ExpCalculatorState>) -> Result<ExpStats, String> {
    let _calculator = state.0.lock().map_err(|e| format!("Failed to lock calculator: {}", e))?;

    // For now, this requires calling add_exp_data
    // TODO: Add get_current_stats method to ExpCalculator
    Err("Use add_exp_data to get updated stats".to_string())
}

/// Reset the current EXP tracking session
#[tauri::command]
pub fn reset_exp_session(state: State<ExpCalculatorState>) -> Result<String, String> {
    let mut calculator = state.0.lock().map_err(|e| format!("Failed to lock calculator: {}", e))?;

    calculator.reset();

    Ok("Session reset successfully".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exp_calculator_state_creation() {
        let calculator = ExpCalculator::new().unwrap();
        let state = ExpCalculatorState(Mutex::new(calculator));

        // Verify state can be locked
        let calc = state.0.lock().unwrap();
        drop(calc); // Explicitly drop lock

        // State should be reusable
        let _calc2 = state.0.lock().unwrap();
    }

    #[test]
    fn test_exp_data_roundtrip() {
        let mut calculator = ExpCalculator::new().unwrap();

        let initial = ExpData {
            level: 126,
            exp: 5000,
            percentage: 50.0,
            meso: Some(100000),
        };

        calculator.start(initial.clone());

        let updated = ExpData {
            level: 126,
            exp: 6000,
            percentage: 60.0,
            meso: Some(150000),
        };

        let stats = calculator.update(updated).unwrap();

        assert_eq!(stats.total_exp, 1000);
        assert_eq!(stats.total_percentage, 10.0);
        assert_eq!(stats.total_meso, 50000);
        assert_eq!(stats.current_level, 126);
        assert_eq!(stats.start_level, 126);
        assert_eq!(stats.levels_gained, 0);
    }
}
