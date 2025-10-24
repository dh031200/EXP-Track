mod commands;
mod models;
mod services;
mod utils;

use tauri::Manager;

use commands::config::{
    clear_roi, get_all_rois, get_config_path, init_config_manager, load_config, load_roi,
    open_roi_preview, save_config, save_roi, save_roi_preview, ConfigManagerState,
};
use commands::ocr::{
    init_ocr_service, recognize_all_parallel, recognize_exp, recognize_hp, recognize_level,
    check_ocr_health,
    recognize_map, recognize_mp, OcrServiceState,
};
use commands::screen_capture::{
    capture_full_screen, capture_region, get_screen_dimensions, init_screen_capture,
    ScreenCaptureState,
};
use commands::exp::{
    add_exp_data, reset_exp_session, start_exp_session, ExpCalculatorState,
};
use commands::tracking::{
    get_tracking_stats, reset_tracking, start_ocr_tracking, stop_ocr_tracking, TrackerState,
};
use services::exp_calculator::ExpCalculator;
use services::python_server::PythonServerManager;
use std::sync::Mutex;
use tokio::sync::Mutex as AsyncMutex;

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

    // Initialize OCR Tracker
    let tracker_state = TrackerState::new().expect("Failed to initialize OCR tracker");

    // Initialize Python server manager
    let python_server = AsyncMutex::new(PythonServerManager::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(ScreenCaptureState::default())
        .manage(config_manager)
        .manage(ocr_service)
        .manage(exp_calculator_state)
        .manage(tracker_state)
        .manage(python_server)
        .setup(|app| {
            // Start Python OCR server on app startup
            let handle = app.handle().clone();

            tauri::async_runtime::spawn(async move {
                let server_state = handle.state::<AsyncMutex<PythonServerManager>>();
                let mut server = server_state.lock().await;

                match server.start().await {
                    Ok(_) => {
                        #[cfg(debug_assertions)]
                        println!("✅ Python OCR server initialized successfully");
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to start Python OCR server: {}", e);
                        eprintln!("⚠️  OCR features will not be available");
                    }
                }
            });

            Ok(())
        })
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
            recognize_all_parallel,
            check_ocr_health,
            start_exp_session,
            add_exp_data,
            reset_exp_session,
            start_ocr_tracking,
            stop_ocr_tracking,
            get_tracking_stats,
            reset_tracking
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
