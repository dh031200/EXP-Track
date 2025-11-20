use serde::{Deserialize, Serialize};
use crate::models::roi::Roi;

/// Window dimensions and position
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WindowDimensions {
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
}

impl Default for WindowDimensions {
    fn default() -> Self {
        Self {
            width: 400,
            height: 150,
            x: 100,
            y: 100,
        }
    }
}

/// Window mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WindowMode {
    Compact,
    Dashboard,
}

impl Default for WindowMode {
    fn default() -> Self {
        Self::Compact
    }
}

/// Window configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WindowConfig {
    pub compact: WindowDimensions,
    pub dashboard: WindowDimensions,
    pub current_mode: WindowMode,
    pub always_on_top: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            compact: WindowDimensions::default(),
            dashboard: WindowDimensions {
                width: 1000,
                height: 700,
                x: 100,
                y: 100,
            },
            current_mode: WindowMode::Compact,
            always_on_top: true,
        }
    }
}

/// ROI configuration for all capture regions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RoiConfig {
    pub level: Option<Roi>,
    pub exp: Option<Roi>,
    pub hp: Option<Roi>,
    pub mp: Option<Roi>,
    #[serde(default)]
    pub inventory: Option<Roi>,
    // pub meso: Option<Roi>, // Commented out temporarily
    // pub map_location: Option<Roi>, // Commented out temporarily
}

/// Tracking configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrackingConfig {
    pub update_interval: u64,
    pub track_meso: bool,
    pub auto_start: bool,
    pub auto_pause_threshold: u64,
}

impl Default for TrackingConfig {
    fn default() -> Self {
        Self {
            update_interval: 1,
            track_meso: false,
            auto_start: false,
            auto_pause_threshold: 300,
        }
    }
}

/// Time format preference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TimeFormat {
    #[serde(rename = "12h")]
    TwelveHour,
    #[serde(rename = "24h")]
    TwentyFourHour,
}

impl Default for TimeFormat {
    fn default() -> Self {
        Self::TwentyFourHour
    }
}

/// Display configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DisplayConfig {
    pub time_format: TimeFormat,
    pub number_format: String,
    pub show_expected_time: bool,
    pub graph_time_window: u64,
    pub show_trend_line: bool,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            time_format: TimeFormat::TwentyFourHour,
            number_format: "en-US".to_string(),
            show_expected_time: true,
            graph_time_window: 600,
            show_trend_line: true,
        }
    }
}

/// Audio configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioConfig {
    pub volume: f32,
    pub enable_sounds: bool,
    pub level_up_sound: bool,
    pub milestone_sound: bool,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            volume: 0.5,
            enable_sounds: true,
            level_up_sound: true,
            milestone_sound: true,
        }
    }
}

/// OCR engine choice (Python FastAPI server only)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OcrEngine {
    Native,
}

impl Default for OcrEngine {
    fn default() -> Self {
        Self::Native
    }
}

/// Image preprocessing configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PreprocessingConfig {
    pub scale_factor: f64,
    pub apply_blur: bool,
    pub blur_radius: u32,
}

impl Default for PreprocessingConfig {
    fn default() -> Self {
        Self {
            scale_factor: 2.0,
            apply_blur: true,
            blur_radius: 3,
        }
    }
}

/// Advanced configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdvancedConfig {
    pub ocr_engine: OcrEngine,
    pub preprocessing: PreprocessingConfig,
    pub spike_threshold: f64,
    pub data_retention_days: u32,
}

impl Default for AdvancedConfig {
    fn default() -> Self {
        Self {
            ocr_engine: OcrEngine::Native,
            preprocessing: PreprocessingConfig::default(),
            spike_threshold: 2.0,
            data_retention_days: 30,
        }
    }
}

/// Potion slot configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PotionConfig {
    pub hp_potion_slot: String,
    pub mp_potion_slot: String,
}

impl Default for PotionConfig {
    fn default() -> Self {
        Self {
            hp_potion_slot: "shift".to_string(),
            mp_potion_slot: "ins".to_string(),
        }
    }
}

impl PotionConfig {
    /// Validate that slots are different and valid
    pub fn validate(&self) -> Result<(), String> {
        const VALID_SLOTS: &[&str] = &["shift", "ins", "home", "pup", "ctrl", "del", "end", "pdn"];

        if !VALID_SLOTS.contains(&self.hp_potion_slot.as_str()) {
            return Err(format!("Invalid HP potion slot: {}", self.hp_potion_slot));
        }

        if !VALID_SLOTS.contains(&self.mp_potion_slot.as_str()) {
            return Err(format!("Invalid MP potion slot: {}", self.mp_potion_slot));
        }

        if self.hp_potion_slot == self.mp_potion_slot {
            return Err("HP and MP potion slots must be different".to_string());
        }

        Ok(())
    }
}

/// Complete application configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AppConfig {
    pub window: WindowConfig,
    pub roi: RoiConfig,
    pub tracking: TrackingConfig,
    pub display: DisplayConfig,
    pub audio: AudioConfig,
    pub advanced: AdvancedConfig,
    #[serde(default)]
    pub potion: PotionConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();

        // Window config
        assert_eq!(config.window.compact.width, 400);
        assert_eq!(config.window.compact.height, 150);
        assert_eq!(config.window.dashboard.width, 1000);
        assert_eq!(config.window.dashboard.height, 700);
        assert_eq!(config.window.current_mode, WindowMode::Compact);
        assert!(config.window.always_on_top);

        // ROI config
        assert!(config.roi.level.is_none());
        assert!(config.roi.exp.is_none());
        assert!(config.roi.hp.is_none());
        assert!(config.roi.mp.is_none());

        // Tracking config
        assert_eq!(config.tracking.update_interval, 1);
        assert!(!config.tracking.track_meso);

        // Display config
        assert_eq!(config.display.time_format, TimeFormat::TwentyFourHour);
        assert!(config.display.show_expected_time);

        // Audio config
        assert_eq!(config.audio.volume, 0.5);
        assert!(config.audio.enable_sounds);

        // Advanced config
        assert_eq!(config.advanced.ocr_engine, OcrEngine::Native);
        assert_eq!(config.advanced.spike_threshold, 2.0);
    }

    #[test]
    fn test_app_config_serialization() {
        let config = AppConfig::default();
        let json = serde_json::to_string_pretty(&config).unwrap();

        // Should be able to deserialize
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_app_config_with_roi() {
        let mut config = AppConfig::default();
        config.roi.level = Some(Roi::new(100, 100, 200, 50));
        config.roi.exp = Some(Roi::new(300, 100, 300, 50));

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config, deserialized);
        assert!(deserialized.roi.level.is_some());
        assert!(deserialized.roi.exp.is_some());
        assert!(deserialized.roi.hp.is_none());
        assert!(deserialized.roi.mp.is_none());
    }

    #[test]
    fn test_window_mode_serialization() {
        let compact = WindowMode::Compact;
        let dashboard = WindowMode::Dashboard;

        assert_eq!(
            serde_json::to_string(&compact).unwrap(),
            "\"compact\""
        );
        assert_eq!(
            serde_json::to_string(&dashboard).unwrap(),
            "\"dashboard\""
        );
    }

    #[test]
    fn test_time_format_serialization() {
        let twelve = TimeFormat::TwelveHour;
        let twenty_four = TimeFormat::TwentyFourHour;

        assert_eq!(
            serde_json::to_string(&twelve).unwrap(),
            "\"12h\""
        );
        assert_eq!(
            serde_json::to_string(&twenty_four).unwrap(),
            "\"24h\""
        );
    }
}
