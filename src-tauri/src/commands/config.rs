use crate::models::config::AppConfig;
use crate::models::roi::Roi;
use crate::services::config::ConfigManager;
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::Mutex;
use tauri::State;

/// ROI type identifier
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum RoiType {
    Level,
    Exp,
    Meso,
    MapLocation,
}

/// State wrapper for configuration manager
pub type ConfigManagerState = Mutex<ConfigManager>;

/// Initialize config manager state
pub fn init_config_manager() -> Result<ConfigManagerState, String> {
    let manager = ConfigManager::new()?;
    Ok(Mutex::new(manager))
}

/// Save ROI to configuration
#[tauri::command]
pub fn save_roi(
    state: State<ConfigManagerState>,
    roi_type: RoiType,
    roi: Roi,
) -> Result<(), String> {
    let manager = state
        .lock()
        .map_err(|e| format!("Failed to lock config manager: {}", e))?;

    // Load current config
    let mut config = manager.load()?;

    // Update the specific ROI
    match roi_type {
        RoiType::Level => config.roi.level = Some(roi),
        RoiType::Exp => config.roi.exp = Some(roi),
        RoiType::Meso => config.roi.meso = Some(roi),
        RoiType::MapLocation => config.roi.map_location = Some(roi),
    }

    // Save updated config
    manager.save(&config)?;

    Ok(())
}

/// Load ROI from configuration
#[tauri::command]
pub fn load_roi(state: State<ConfigManagerState>, roi_type: RoiType) -> Result<Option<Roi>, String> {
    let manager = state
        .lock()
        .map_err(|e| format!("Failed to lock config manager: {}", e))?;

    let config = manager.load()?;

    let roi = match roi_type {
        RoiType::Level => config.roi.level,
        RoiType::Exp => config.roi.exp,
        RoiType::Meso => config.roi.meso,
        RoiType::MapLocation => config.roi.map_location,
    };

    Ok(roi)
}

/// Get all ROIs from configuration
#[tauri::command]
pub fn get_all_rois(state: State<ConfigManagerState>) -> Result<serde_json::Value, String> {
    let manager = state
        .lock()
        .map_err(|e| format!("Failed to lock config manager: {}", e))?;

    let config = manager.load()?;

    // Return ROI config as JSON
    serde_json::to_value(&config.roi)
        .map_err(|e| format!("Failed to serialize ROI config: {}", e))
}

/// Clear ROI from configuration
#[tauri::command]
pub fn clear_roi(state: State<ConfigManagerState>, roi_type: RoiType) -> Result<(), String> {
    let manager = state
        .lock()
        .map_err(|e| format!("Failed to lock config manager: {}", e))?;

    let mut config = manager.load()?;

    match roi_type {
        RoiType::Level => config.roi.level = None,
        RoiType::Exp => config.roi.exp = None,
        RoiType::Meso => config.roi.meso = None,
        RoiType::MapLocation => config.roi.map_location = None,
    }

    manager.save(&config)?;

    Ok(())
}

/// Save entire application configuration
#[tauri::command]
pub fn save_config(state: State<ConfigManagerState>, config: AppConfig) -> Result<(), String> {
    let manager = state
        .lock()
        .map_err(|e| format!("Failed to lock config manager: {}", e))?;

    manager.save(&config)
}

/// Load entire application configuration
#[tauri::command]
pub fn load_config(state: State<ConfigManagerState>) -> Result<AppConfig, String> {
    let manager = state
        .lock()
        .map_err(|e| format!("Failed to lock config manager: {}", e))?;

    manager.load()
}

/// Get config file path
#[tauri::command]
pub fn get_config_path(state: State<ConfigManagerState>) -> Result<String, String> {
    let manager = state
        .lock()
        .map_err(|e| format!("Failed to lock config manager: {}", e))?;

    Ok(manager
        .config_file_path()
        .to_str()
        .unwrap_or("")
        .to_string())
}

/// Save ROI preview image to temp directory
#[tauri::command]
pub fn save_roi_preview(roi_type: RoiType, image_data: String) -> Result<String, String> {
    // Get temp directory
    let temp_dir = std::env::temp_dir().join("exp-tracker-previews");
    fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create preview directory: {}", e))?;

    // Decode base64
    let image_bytes = base64::engine::general_purpose::STANDARD
        .decode(&image_data)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;

    // Save to file
    let filename = format!("{}_preview.png", match roi_type {
        RoiType::Level => "level",
        RoiType::Exp => "exp",
        RoiType::Meso => "meso",
        RoiType::MapLocation => "map_location",
    });
    let file_path = temp_dir.join(&filename);

    fs::write(&file_path, image_bytes)
        .map_err(|e| format!("Failed to write preview file: {}", e))?;

    Ok(file_path.to_str().unwrap_or("").to_string())
}

/// Open ROI preview in system viewer
#[tauri::command]
pub fn open_roi_preview(roi_type: RoiType) -> Result<(), String> {
    let temp_dir = std::env::temp_dir().join("exp-tracker-previews");
    let filename = format!("{}_preview.png", match roi_type {
        RoiType::Level => "level",
        RoiType::Exp => "exp",
        RoiType::Meso => "meso",
        RoiType::MapLocation => "map_location",
    });
    let file_path = temp_dir.join(&filename);

    if !file_path.exists() {
        return Err("Preview file not found".to_string());
    }

    // Open with system default viewer
    #[cfg(target_os = "macos")]
    std::process::Command::new("open")
        .arg(&file_path)
        .spawn()
        .map_err(|e| format!("Failed to open preview: {}", e))?;

    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd")
        .args(&["/C", "start", "", file_path.to_str().unwrap_or("")])
        .spawn()
        .map_err(|e| format!("Failed to open preview: {}", e))?;

    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open")
        .arg(&file_path)
        .spawn()
        .map_err(|e| format!("Failed to open preview: {}", e))?;

    Ok(())
}

// Note: Integration tests for these commands will be in tests/ directory
// Unit tests for the underlying ConfigManager are in services/config.rs
