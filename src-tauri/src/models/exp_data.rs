use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExpData {
    pub level: u32,
    pub exp: u64,
    pub percentage: f64,
    pub meso: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExpStats {
    pub total_exp: u64,
    pub total_percentage: f64,
    pub total_meso: u64,
    pub elapsed_seconds: u64,
    pub exp_per_hour: u64,
    pub percentage_per_hour: f64,
    pub meso_per_hour: u64,
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
        };

        assert_eq!(stats.total_exp, 1000);
        assert_eq!(stats.elapsed_seconds, 600);
        assert_eq!(stats.exp_per_hour, 6000);
    }
}
