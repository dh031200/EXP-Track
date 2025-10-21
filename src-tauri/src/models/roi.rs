use serde::{Deserialize, Serialize};

/// Region of Interest for screen capture
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Roi {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Roi {
    /// Create a new ROI from coordinates
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Create ROI from bounds (x1, y1, x2, y2)
    pub fn from_bounds(x1: i32, y1: i32, x2: i32, y2: i32) -> Result<Self, String> {
        if x2 <= x1 {
            return Err("x2 must be greater than x1".to_string());
        }
        if y2 <= y1 {
            return Err("y2 must be greater than y1".to_string());
        }

        Ok(Self {
            x: x1,
            y: y1,
            width: (x2 - x1) as u32,
            height: (y2 - y1) as u32,
        })
    }

    /// Validate ROI dimensions
    pub fn is_valid(&self) -> bool {
        self.width > 0 && self.height > 0
    }

    /// Get the end coordinates
    pub fn x2(&self) -> i32 {
        self.x + self.width as i32
    }

    pub fn y2(&self) -> i32 {
        self.y + self.height as i32
    }

    /// Calculate area
    pub fn area(&self) -> u64 {
        self.width as u64 * self.height as u64
    }

    /// Check if ROI contains a point
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x2() && y >= self.y && y < self.y2()
    }

    /// Check if ROI intersects with another ROI
    pub fn intersects(&self, other: &Roi) -> bool {
        self.x < other.x2()
            && self.x2() > other.x
            && self.y < other.y2()
            && self.y2() > other.y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roi_creation() {
        let roi = Roi::new(100, 100, 200, 150);
        assert_eq!(roi.x, 100);
        assert_eq!(roi.y, 100);
        assert_eq!(roi.width, 200);
        assert_eq!(roi.height, 150);
    }

    #[test]
    fn test_roi_from_bounds_valid() {
        let roi = Roi::from_bounds(100, 100, 300, 250).unwrap();
        assert_eq!(roi.x, 100);
        assert_eq!(roi.y, 100);
        assert_eq!(roi.width, 200);
        assert_eq!(roi.height, 150);
    }

    #[test]
    fn test_roi_from_bounds_invalid_x() {
        let result = Roi::from_bounds(300, 100, 100, 250);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "x2 must be greater than x1");
    }

    #[test]
    fn test_roi_from_bounds_invalid_y() {
        let result = Roi::from_bounds(100, 250, 300, 100);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "y2 must be greater than y1");
    }

    #[test]
    fn test_roi_validation() {
        let valid = Roi::new(0, 0, 100, 100);
        assert!(valid.is_valid());

        let zero_width = Roi::new(0, 0, 0, 100);
        assert!(!zero_width.is_valid());

        let zero_height = Roi::new(0, 0, 100, 0);
        assert!(!zero_height.is_valid());
    }

    #[test]
    fn test_roi_bounds() {
        let roi = Roi::new(100, 200, 300, 400);
        assert_eq!(roi.x2(), 400); // 100 + 300
        assert_eq!(roi.y2(), 600); // 200 + 400
    }

    #[test]
    fn test_roi_area() {
        let roi = Roi::new(0, 0, 100, 50);
        assert_eq!(roi.area(), 5000); // 100 * 50
    }

    #[test]
    fn test_roi_contains_point() {
        let roi = Roi::new(100, 100, 200, 200);

        // Inside
        assert!(roi.contains(150, 150));
        assert!(roi.contains(100, 100)); // Top-left corner

        // Outside
        assert!(!roi.contains(50, 150));
        assert!(!roi.contains(150, 50));
        assert!(!roi.contains(300, 300));

        // Edge (exclusive)
        assert!(!roi.contains(300, 150)); // Right edge
        assert!(!roi.contains(150, 300)); // Bottom edge
    }

    #[test]
    fn test_roi_intersection() {
        let roi1 = Roi::new(100, 100, 200, 200);

        // Overlapping
        let roi2 = Roi::new(150, 150, 200, 200);
        assert!(roi1.intersects(&roi2));
        assert!(roi2.intersects(&roi1));

        // Non-overlapping
        let roi3 = Roi::new(400, 400, 100, 100);
        assert!(!roi1.intersects(&roi3));
        assert!(!roi3.intersects(&roi1));

        // Adjacent (no overlap)
        let roi4 = Roi::new(300, 100, 100, 200);
        assert!(!roi1.intersects(&roi4));
    }

    #[test]
    fn test_roi_serialization() {
        let roi = Roi::new(100, 200, 300, 400);
        let json = serde_json::to_string(&roi).unwrap();
        let deserialized: Roi = serde_json::from_str(&json).unwrap();
        assert_eq!(roi, deserialized);
    }
}
