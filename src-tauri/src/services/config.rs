use crate::models::config::AppConfig;
use std::fs;
use std::path::PathBuf;

/// Configuration manager for app settings
pub struct ConfigManager {
    config_dir: PathBuf,
    config_path: PathBuf,
}

impl ConfigManager {
    /// Create a new ConfigManager instance
    ///
    /// This will create the config directory if it doesn't exist.
    /// Returns an error if directory creation fails.
    pub fn new() -> Result<Self, String> {
        // Get platform-specific config directory
        let config_dir = dirs::config_dir()
            .ok_or("Failed to determine config directory")?
            .join("exp-tracker");

        // Create directory if it doesn't exist
        fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;

        let config_path = config_dir.join("config.json");

        Ok(Self {
            config_dir,
            config_path,
        })
    }

    /// Save configuration to disk
    pub fn save(&self, config: &AppConfig) -> Result<(), String> {
        // Ensure config directory exists
        fs::create_dir_all(&self.config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;

        // Serialize config to JSON (pretty print for human readability)
        let json = serde_json::to_string_pretty(config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        // Write to file
        fs::write(&self.config_path, json)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        Ok(())
    }

    /// Load configuration from disk
    ///
    /// If config file doesn't exist, returns default configuration
    pub fn load(&self) -> Result<AppConfig, String> {
        // If file doesn't exist, return default
        if !self.config_exists() {
            return Ok(AppConfig::default());
        }

        // Read file
        let content = fs::read_to_string(&self.config_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        // Parse JSON
        let config: AppConfig = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        Ok(config)
    }

    /// Get the config file path
    pub fn config_file_path(&self) -> &PathBuf {
        &self.config_path
    }

    /// Check if config file exists
    pub fn config_exists(&self) -> bool {
        self.config_path.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::roi::Roi;
    use std::fs;

    /// Helper to create a temporary test config manager
    fn create_test_manager() -> ConfigManager {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);

        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let temp_dir = std::env::temp_dir().join(format!("exp-tracker-test-{}-{}", std::process::id(), id));
        // Clean up any existing test directory
        let _ = fs::remove_dir_all(&temp_dir);
        // Note: Don't create directory here - let save() handle it

        ConfigManager {
            config_dir: temp_dir.clone(),
            config_path: temp_dir.join("config.json"),
        }
    }

    /// Clean up test files
    fn cleanup_test_files(manager: &ConfigManager) {
        let _ = fs::remove_dir_all(&manager.config_dir);
    }

    #[test]
    fn test_config_manager_new() {
        // 🔴 RED: This test should FAIL initially
        let result = ConfigManager::new();

        // Should successfully create manager
        assert!(result.is_ok(), "ConfigManager::new() should succeed");

        let manager = result.unwrap();

        // Config directory should be created
        assert!(
            manager.config_dir.exists(),
            "Config directory should exist: {:?}",
            manager.config_dir
        );

        // Config path should be set
        assert!(
            manager.config_path.to_str().unwrap().ends_with("config.json"),
            "Config path should end with config.json"
        );

        // Clean up
        cleanup_test_files(&manager);
    }

    #[test]
    fn test_config_save() {
        // 🔴 RED: This test should FAIL initially
        let manager = create_test_manager();
        let config = AppConfig::default();

        let result = manager.save(&config);
        assert!(result.is_ok(), "save() should succeed");

        // Config file should exist
        assert!(
            manager.config_path.exists(),
            "Config file should exist after save"
        );

        // Should be valid JSON
        let file_content = fs::read_to_string(&manager.config_path).unwrap();
        let _parsed: AppConfig = serde_json::from_str(&file_content)
            .expect("Saved config should be valid JSON");

        cleanup_test_files(&manager);
    }

    #[test]
    fn test_config_load_default_when_not_exists() {
        // 🔴 RED: This test should FAIL initially
        let manager = create_test_manager();

        // Ensure config doesn't exist
        assert!(!manager.config_exists());

        let result = manager.load();
        assert!(result.is_ok(), "load() should return default when file doesn't exist");

        let config = result.unwrap();
        assert_eq!(config, AppConfig::default());

        cleanup_test_files(&manager);
    }

    #[test]
    fn test_config_save_and_load() {
        // 🔴 RED: This test should FAIL initially
        let manager = create_test_manager();

        // Create custom config
        let mut config = AppConfig::default();
        config.tracking.update_interval = 5;
        config.audio.volume = 0.8;
        config.roi.level = Some(Roi::new(100, 100, 200, 50));

        // Save
        manager.save(&config).expect("save should succeed");

        // Load
        let loaded = manager.load().expect("load should succeed");

        // Should match original
        assert_eq!(loaded, config);
        assert_eq!(loaded.tracking.update_interval, 5);
        assert_eq!(loaded.audio.volume, 0.8);
        assert!(loaded.roi.level.is_some());

        cleanup_test_files(&manager);
    }

    #[test]
    fn test_platform_config_paths() {
        // 🔴 RED: This test should FAIL initially
        let result = ConfigManager::new();
        assert!(result.is_ok());

        let manager = result.unwrap();
        let path_str = manager.config_dir.to_str().unwrap();

        // Platform-specific path checks
        #[cfg(target_os = "macos")]
        {
            assert!(
                path_str.contains("Library/Application Support")
                    || path_str.contains("exp-tracker"),
                "macOS config should be in Application Support or test directory"
            );
        }

        #[cfg(target_os = "windows")]
        {
            assert!(
                path_str.contains("AppData") || path_str.contains("exp-tracker"),
                "Windows config should be in AppData or test directory"
            );
        }

        #[cfg(target_os = "linux")]
        {
            assert!(
                path_str.contains(".config") || path_str.contains("exp-tracker"),
                "Linux config should be in .config or test directory"
            );
        }

        cleanup_test_files(&manager);
    }

    #[test]
    fn test_config_file_path() {
        let manager = create_test_manager();

        let path = manager.config_file_path();
        assert!(path.to_str().unwrap().ends_with("config.json"));

        cleanup_test_files(&manager);
    }

    #[test]
    fn test_config_exists() {
        let manager = create_test_manager();

        // Initially doesn't exist
        assert!(!manager.config_exists());

        // After save, exists
        manager.save(&AppConfig::default()).unwrap();
        assert!(manager.config_exists());

        cleanup_test_files(&manager);
    }

    #[test]
    fn test_config_overwrite() {
        let manager = create_test_manager();

        // Save first config
        let mut config1 = AppConfig::default();
        config1.audio.volume = 0.3;
        manager.save(&config1).unwrap();

        // Save second config (overwrite)
        let mut config2 = AppConfig::default();
        config2.audio.volume = 0.7;
        manager.save(&config2).unwrap();

        // Load should get latest
        let loaded = manager.load().unwrap();
        assert_eq!(loaded.audio.volume, 0.7);

        cleanup_test_files(&manager);
    }
}
