use crate::models::config::{AppConfig, PotionConfig};
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
    Hp,
    Mp,
    Inventory,  // Auto-detected inventory region (read-only preview)
    // Meso, // Commented out temporarily
    // MapLocation, // Commented out temporarily
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
        RoiType::Hp => config.roi.hp = Some(roi),
        RoiType::Mp => config.roi.mp = Some(roi),
        RoiType::Inventory => config.roi.inventory = Some(roi),
        // RoiType::Meso => config.roi.meso = Some(roi), // Commented out temporarily
        // RoiType::MapLocation => config.roi.map_location = Some(roi), // Commented out temporarily
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
        RoiType::Hp => config.roi.hp,
        RoiType::Mp => config.roi.mp,
        RoiType::Inventory => config.roi.inventory,
        // RoiType::Meso => config.roi.meso, // Commented out temporarily
        // RoiType::MapLocation => config.roi.map_location, // Commented out temporarily
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
        RoiType::Hp => config.roi.hp = None,
        RoiType::Mp => config.roi.mp = None,
        RoiType::Inventory => config.roi.inventory = None,
        // RoiType::Meso => config.roi.meso = None, // Commented out temporarily
        // RoiType::MapLocation => config.roi.map_location = None, // Commented out temporarily
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
        RoiType::Hp => "hp",
        RoiType::Mp => "mp",
        RoiType::Inventory => "inventory",
        // RoiType::Meso => "meso", // Commented out temporarily
        // RoiType::MapLocation => "map_location", // Commented out temporarily
    });
    let file_path = temp_dir.join(&filename);

    fs::write(&file_path, image_bytes)
        .map_err(|e| format!("Failed to write preview file: {}", e))?;

    Ok(file_path.to_str().unwrap_or("").to_string())
}

/// Get ROI preview as base64 encoded string
#[tauri::command]
pub fn get_roi_preview(roi_type: RoiType) -> Result<String, String> {
    let temp_dir = std::env::temp_dir().join("exp-tracker-previews");
    let filename = format!("{}_preview.png", match roi_type {
        RoiType::Level => "level",
        RoiType::Exp => "exp",
        RoiType::Hp => "hp",
        RoiType::Mp => "mp",
        RoiType::Inventory => "inventory",
    });
    let file_path = temp_dir.join(&filename);

    if !file_path.exists() {
        return Err("Preview file not found".to_string());
    }

    let image_bytes = fs::read(&file_path)
        .map_err(|e| format!("Failed to read preview file: {}", e))?;

    let base64_str = base64::engine::general_purpose::STANDARD.encode(&image_bytes);
    Ok(format!("data:image/png;base64,{}", base64_str))
}

/// Open ROI preview in system viewer
#[tauri::command]
pub fn open_roi_preview(roi_type: RoiType) -> Result<(), String> {
    let temp_dir = std::env::temp_dir().join("exp-tracker-previews");
    let filename = format!("{}_preview.png", match roi_type {
        RoiType::Level => "level",
        RoiType::Exp => "exp",
        RoiType::Hp => "hp",
        RoiType::Mp => "mp",
        RoiType::Inventory => "inventory",
        // RoiType::Meso => "meso", // Commented out temporarily
        // RoiType::MapLocation => "map_location", // Commented out temporarily
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

/// Get potion slot configuration
#[tauri::command]
pub fn get_potion_slot_config(state: State<ConfigManagerState>) -> Result<PotionConfig, String> {
    let manager = state
        .lock()
        .map_err(|e| format!("Failed to lock config manager: {}", e))?;

    // Return default config if load fails
    match manager.load() {
        Ok(config) => Ok(config.potion),
        Err(_) => {
            // Config file doesn't exist or is corrupted, return default
            Ok(PotionConfig::default())
        }
    }
}

/// Set potion slot configuration
#[tauri::command]
pub fn set_potion_slot_config(
    state: State<ConfigManagerState>,
    potion_config: PotionConfig,
) -> Result<(), String> {
    // Validate configuration
    potion_config.validate()?;

    let manager = state
        .lock()
        .map_err(|e| format!("Failed to lock config manager: {}", e))?;

    // Load existing config or create new one if it doesn't exist
    let mut config = match manager.load() {
        Ok(cfg) => cfg,
        Err(_) => {
            // Config doesn't exist, create new one with defaults
            AppConfig::default()
        }
    };
    
    config.potion = potion_config;
    manager.save(&config)?;

    Ok(())
}

// Note: Integration tests for these commands will be in tests/ directory
// Unit tests for the underlying ConfigManager are in services/config.rs
