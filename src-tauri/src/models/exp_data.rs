use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Single snapshot of player's experience at a specific time
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExpSnapshot {
    pub timestamp: u64,  // Unix timestamp in seconds
    pub level: u32,
    pub exp: u64,        // Current EXP within level
    pub percentage: f64, // Percentage to next level
    pub meso: Option<u64>,
    pub hp: Option<u32>, // Current HP
    pub mp: Option<u32>, // Current MP
}

impl ExpSnapshot {
    /// Create a new snapshot with current timestamp
    pub fn new(level: u32, exp: u64, percentage: f64, meso: Option<u64>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            timestamp,
            level,
            exp,
            percentage,
            meso,
            hp: None,
            mp: None,
        }
    }

    /// Create a new snapshot with HP/MP
    pub fn with_hp_mp(level: u32, exp: u64, percentage: f64, meso: Option<u64>, hp: Option<u32>, mp: Option<u32>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            timestamp,
            level,
            exp,
            percentage,
            meso,
            hp,
            mp,
        }
    }

    /// Create snapshot with custom timestamp (for testing)
    pub fn with_timestamp(timestamp: u64, level: u32, exp: u64, percentage: f64, meso: Option<u64>) -> Self {
        Self {
            timestamp,
            level,
            exp,
            percentage,
            meso,
            hp: None,
            mp: None,
        }
    }
}

/// Legacy alias for backwards compatibility
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExpData {
    pub level: u32,
    pub exp: u64,
    pub percentage: f64,
    pub meso: Option<u64>,
}

/// Tracking session data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExpSession {
    pub start_snapshot: ExpSnapshot,
    pub current_snapshot: Option<ExpSnapshot>,
    pub snapshots: Vec<ExpSnapshot>,
}

impl ExpSession {
    /// Create a new session with starting snapshot
    pub fn new(start_snapshot: ExpSnapshot) -> Self {
        Self {
            start_snapshot: start_snapshot.clone(),
            current_snapshot: Some(start_snapshot.clone()),
            snapshots: vec![start_snapshot],
        }
    }

    /// Add a new snapshot to the session
    pub fn add_snapshot(&mut self, snapshot: ExpSnapshot) {
        self.current_snapshot = Some(snapshot.clone());
        self.snapshots.push(snapshot);
    }

    /// Get elapsed time in seconds
    pub fn elapsed_seconds(&self) -> u64 {
        if let Some(current) = &self.current_snapshot {
            current.timestamp.saturating_sub(self.start_snapshot.timestamp)
        } else {
            0
        }
    }

    /// Get total snapshots count
    pub fn snapshot_count(&self) -> usize {
        self.snapshots.len()
    }
}

/// Statistics calculated from session data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExpStats {
    pub total_exp: u64,
    pub total_percentage: f64,
    pub total_meso: u64,
    pub elapsed_seconds: u64,
    pub exp_per_hour: u64,
    pub percentage_per_hour: f64,
    pub meso_per_hour: u64,
    pub exp_per_minute: u64,
    pub current_level: u32,
    pub start_level: u32,
    pub levels_gained: u32,
    // Potion consumption tracking
    pub hp_potions_used: u32,     // Total HP potions consumed
    pub mp_potions_used: u32,     // Total MP potions consumed
    pub hp_potions_per_minute: f64, // HP potions consumed per minute
    pub mp_potions_per_minute: f64, // MP potions consumed per minute
}

pub struct LevelExpTable {
    data: HashMap<u32, u64>,
}

impl LevelExpTable {
    /// Load level experience data from embedded JSON
    pub fn load() -> Result<Self, String> {
        // For now, return an empty table - will be populated in future commits
        Ok(Self {
            data: HashMap::new(),
        })
    }

    /// Get required experience for a given level
    pub fn get_exp_for_level(&self, level: u32) -> Option<u64> {
        self.data.get(&level).copied()
    }

    /// Add level experience data (for testing)
    #[cfg(test)]
    pub fn with_levels(mut self, levels: Vec<(u32, u64)>) -> Self {
        for (level, exp) in levels {
            self.data.insert(level, exp);
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_exp_table_creation() {
        let table = LevelExpTable::load().unwrap();
        assert_eq!(table.data.len(), 0);
    }

    #[test]
    fn test_level_exp_table_with_data() {
        let table = LevelExpTable::load()
            .unwrap()
            .with_levels(vec![(50, 10000), (51, 12000)]);

        assert_eq!(table.get_exp_for_level(50), Some(10000));
        assert_eq!(table.get_exp_for_level(51), Some(12000));
        assert_eq!(table.get_exp_for_level(52), None);
    }

    #[test]
    fn test_exp_data_creation() {
        let data = ExpData {
            level: 50,
            exp: 5000,
            percentage: 50.0,
            meso: Some(100000),
        };

        assert_eq!(data.level, 50);
        assert_eq!(data.exp, 5000);
        assert_eq!(data.percentage, 50.0);
        assert_eq!(data.meso, Some(100000));
    }

    #[test]
    fn test_exp_stats_creation() {
        let stats = ExpStats {
            total_exp: 1000,
            total_percentage: 10.5,
            total_meso: 50000,
            elapsed_seconds: 600,
            exp_per_hour: 6000,
            percentage_per_hour: 63.0,
            meso_per_hour: 300000,
            exp_per_minute: 100,
            current_level: 126,
            start_level: 126,
            levels_gained: 0,
            hp_potions_used: 5,
            mp_potions_used: 3,
            hp_potions_per_minute: 0.5,
            mp_potions_per_minute: 0.3,
        };

        assert_eq!(stats.total_exp, 1000);
        assert_eq!(stats.elapsed_seconds, 600);
        assert_eq!(stats.exp_per_hour, 6000);
        assert_eq!(stats.exp_per_minute, 100);
        assert_eq!(stats.current_level, 126);
        assert_eq!(stats.start_level, 126);
        assert_eq!(stats.levels_gained, 0);
        assert_eq!(stats.hp_potions_used, 5);
        assert_eq!(stats.mp_potions_used, 3);
    }
}
