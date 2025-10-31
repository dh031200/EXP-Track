mod commands;
mod models;
mod services;
mod utils;

use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use commands::config::{
    clear_roi, get_all_rois, get_config_path, init_config_manager, load_config, load_roi,
    get_roi_preview, open_roi_preview, save_config, save_roi, save_roi_preview,
    get_potion_slot_config, set_potion_slot_config,
};
use commands::ocr::{
    init_ocr_service, recognize_all_parallel, recognize_exp, recognize_hp_potion_count, recognize_level,
    check_ocr_health,
    recognize_map, recognize_mp_potion_count,
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
use commands::session::{
    get_session_records, save_session_record, delete_session_record, update_session_title,
    init_session_records, SessionRecordsState,
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

    // Initialize Python server manager
    let python_server = AsyncMutex::new(PythonServerManager::new());

    // Initialize session records
    let session_records = init_session_records();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(ScreenCaptureState::default())
        .manage(config_manager)
        .manage(ocr_service.clone())  // Clone for .manage()
        .manage(exp_calculator_state)
        .manage(python_server)
        .manage(session_records)
        .setup(move |app| {  // Move closure to capture ocr_service
            // Initialize OCR Tracker with AppHandle
            let tracker_state = TrackerState::new(app.handle().clone(), ocr_service.clone())
                .expect("Failed to initialize OCR tracker");
            app.manage(tracker_state);

            // Register global shortcut for ` (backtick/tilde) key
            let handle = app.handle().clone();
            app.global_shortcut().on_shortcut("`", move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    #[cfg(debug_assertions)]
                    println!("🎹 Global shortcut triggered: `");
                    
                    // Emit event to frontend
                    let _ = handle.emit("global-shortcut-toggle-timer", ());
                }
            }).expect("Failed to register global shortcut");

            #[cfg(debug_assertions)]
            println!("✅ Global shortcut registered: `");

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
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Prevent immediate close - we need to cleanup first
                api.prevent_close();
                
                let app = window.app_handle().clone();
                
                // Spawn async cleanup task to avoid blocking the event loop
                tauri::async_runtime::spawn(async move {
                    // Stop OCR tracking
                    let tracker_state = app.state::<TrackerState>();
                    {
                        let tracker = tracker_state.inner().0.lock().await;
                        tracker.stop_tracking().await;

                        #[cfg(debug_assertions)]
                        println!("🛑 OCR tracking stopped");
                    }

                    // Shutdown Python OCR server
                    let server_state = app.state::<AsyncMutex<PythonServerManager>>();
                    {
                        let mut server = server_state.lock().await;
                        server.stop_async().await;

                        #[cfg(debug_assertions)]
                        println!("🛑 Python server shutdown signal sent");
                    }

                    #[cfg(debug_assertions)]
                    println!("👋 Application closing");
                    
                    // Now that cleanup is complete, exit the app
                    app.exit(0);
                });
            }
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
            get_potion_slot_config,
            set_potion_slot_config,
            save_roi_preview,
            get_roi_preview,
            open_roi_preview,
            recognize_level,
            recognize_exp,
            recognize_map,
            recognize_hp_potion_count,
            recognize_mp_potion_count,
            recognize_all_parallel,
            check_ocr_health,
            start_exp_session,
            add_exp_data,
            reset_exp_session,
            start_ocr_tracking,
            stop_ocr_tracking,
            get_tracking_stats,
            reset_tracking,
            get_session_records,
            save_session_record,
            delete_session_record,
            update_session_title
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
