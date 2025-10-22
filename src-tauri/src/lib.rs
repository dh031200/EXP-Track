mod commands;
mod models;
mod services;
mod utils;

use commands::config::{
    clear_roi, get_all_rois, get_config_path, init_config_manager, load_config, load_roi,
    open_roi_preview, save_config, save_roi, save_roi_preview, ConfigManagerState,
};
use commands::ocr::{
    init_ocr_service, recognize_exp, recognize_hp, recognize_level, recognize_map, recognize_mp,
    OcrServiceState,
};
use commands::screen_capture::{
    capture_full_screen, capture_region, get_screen_dimensions, init_screen_capture,
    ScreenCaptureState,
};
use commands::exp::{
    add_exp_data, reset_exp_session, start_exp_session, ExpCalculatorState,
};
use services::exp_calculator::ExpCalculator;
use std::sync::Mutex;

// Placeholder command for initial setup
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize config manager
    let config_manager = init_config_manager().expect("Failed to initialize config manager");

    // Initialize OCR service
    let ocr_service = init_ocr_service().expect("Failed to initialize OCR service");

    // Initialize EXP calculator
    let exp_calculator = ExpCalculator::new().expect("Failed to initialize EXP calculator");
    let exp_calculator_state = ExpCalculatorState(Mutex::new(exp_calculator));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(ScreenCaptureState::default())
        .manage(config_manager)
        .manage(ocr_service)
        .manage(exp_calculator_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            init_screen_capture,
            get_screen_dimensions,
            capture_region,
            capture_full_screen,
            save_roi,
            load_roi,
            get_all_rois,
            clear_roi,
            save_config,
            load_config,
            get_config_path,
            save_roi_preview,
            open_roi_preview,
            recognize_level,
            recognize_exp,
            recognize_map,
            recognize_hp,
            recognize_mp,
            start_exp_session,
            add_exp_data,
            reset_exp_session
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
