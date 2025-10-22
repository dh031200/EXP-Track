use regex::Regex;

/// Parsed EXP data containing both absolute and percentage values
#[derive(Debug, Clone, PartialEq)]
pub struct ExpData {
    pub absolute: u64,
    pub percentage: f64,
}

/// Parse level from OCR text
/// Expected format: "LV. 126" or "LV.126" or "LV 126"
/// Returns the level number (1-300)
pub fn parse_level(text: &str) -> Result<u32, String> {
    // Pattern: "LV" + optional dot + optional space + digits
    let re = Regex::new(r"LV\.?\s*(\d{1,3})").unwrap();

    let captures = re
        .captures(text.trim())
        .ok_or_else(|| format!("Could not parse level from: {}", text))?;

    let level_str = captures
        .get(1)
        .ok_or("No level number found")?
        .as_str();

    let level: u32 = level_str
        .parse()
        .map_err(|e| format!("Failed to parse level number: {}", e))?;

    // Validate range
    if !validate_level(level) {
        return Err(format!("Level {} out of valid range (1-300)", level));
    }

    Ok(level)
}

/// Parse EXP from OCR text
/// Expected format: "5509611[12.76%]" or "1000000[50%]"
/// Returns ExpData with absolute value and percentage
pub fn parse_exp(text: &str) -> Result<ExpData, String> {
    // Pattern 1: Extract absolute value (digits before bracket)
    let abs_re = Regex::new(r"(\d+)\s*\[").unwrap();
    // Pattern 2: Extract percentage (number inside brackets with %)
    let pct_re = Regex::new(r"\[\s*(\d+\.?\d*)\s*%\s*\]").unwrap();

    // Extract absolute value
    let abs_captures = abs_re
        .captures(text)
        .ok_or_else(|| format!("Could not parse absolute EXP from: {}", text))?;

    let absolute: u64 = abs_captures
        .get(1)
        .ok_or("No absolute value found")?
        .as_str()
        .parse()
        .map_err(|e| format!("Failed to parse absolute EXP: {}", e))?;

    // Extract percentage
    let pct_captures = pct_re
        .captures(text)
        .ok_or_else(|| format!("Could not parse percentage from: {}", text))?;

    let percentage: f64 = pct_captures
        .get(1)
        .ok_or("No percentage found")?
        .as_str()
        .parse()
        .map_err(|e| format!("Failed to parse percentage: {}", e))?;

    // Validate percentage range
    if !validate_exp_percentage(percentage) {
        return Err(format!("Percentage {} out of valid range (0.0-100.0)", percentage));
    }

    Ok(ExpData {
        absolute,
        percentage,
    })
}

/// Parse map name from OCR text
/// Expected format: Korean text like "히든스트리트 작은 난파선"
/// Returns the map name (trimmed, non-empty)
pub fn parse_map(text: &str) -> Result<String, String> {
    let trimmed = text.trim().to_string();

    if !validate_map(&trimmed) {
        return Err("Map name is empty".to_string());
    }

    Ok(trimmed)
}

/// Validate level is within acceptable range (1-300)
pub fn validate_level(level: u32) -> bool {
    level >= 1 && level <= 300
}

/// Validate EXP percentage is within range (0.0-100.0)
pub fn validate_exp_percentage(percentage: f64) -> bool {
    percentage >= 0.0 && percentage < 100.0
}

/// Validate map name is non-empty
pub fn validate_map(map: &str) -> bool {
    !map.trim().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // Level Parser Tests (🔴 RED Phase)
    // ============================================================

    #[test]
    fn test_parse_level_valid_with_space() {
        let result = parse_level("LV. 126");
        assert!(result.is_ok(), "Should parse 'LV. 126'");
        assert_eq!(result.unwrap(), 126);
    }

    #[test]
    fn test_parse_level_valid_no_space() {
        let result = parse_level("LV.45");
        assert!(result.is_ok(), "Should parse 'LV.45'");
        assert_eq!(result.unwrap(), 45);
    }

    #[test]
    fn test_parse_level_valid_no_dot() {
        let result = parse_level("LV 1");
        assert!(result.is_ok(), "Should parse 'LV 1'");
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_parse_level_valid_max() {
        let result = parse_level("LV. 300");
        assert!(result.is_ok(), "Should parse level 300");
        assert_eq!(result.unwrap(), 300);
    }

    #[test]
    fn test_parse_level_invalid_no_prefix() {
        let result = parse_level("126");
        assert!(result.is_err(), "Should fail without 'LV' prefix");
    }

    #[test]
    fn test_parse_level_invalid_out_of_range_zero() {
        let result = parse_level("LV. 0");
        assert!(result.is_err(), "Should fail for level 0");
    }

    #[test]
    fn test_parse_level_invalid_out_of_range_high() {
        let result = parse_level("LV. 301");
        assert!(result.is_err(), "Should fail for level > 300");
    }

    #[test]
    fn test_parse_level_with_trailing_whitespace() {
        let result = parse_level("LV. 126 ");
        assert!(result.is_ok(), "Should handle trailing whitespace");
        assert_eq!(result.unwrap(), 126);
    }

    #[test]
    fn test_parse_level_with_leading_whitespace() {
        let result = parse_level(" LV. 126");
        assert!(result.is_ok(), "Should handle leading whitespace");
        assert_eq!(result.unwrap(), 126);
    }

    // ============================================================
    // EXP Parser Tests (🔴 RED Phase)
    // ============================================================

    #[test]
    fn test_parse_exp_valid_decimal_percentage() {
        let result = parse_exp("5509611[12.76%]");
        assert!(result.is_ok(), "Should parse decimal percentage");

        let exp_data = result.unwrap();
        assert_eq!(exp_data.absolute, 5509611);
        assert!((exp_data.percentage - 12.76).abs() < 0.01);
    }

    #[test]
    fn test_parse_exp_valid_integer_percentage() {
        let result = parse_exp("1000000[50%]");
        assert!(result.is_ok(), "Should parse integer percentage");

        let exp_data = result.unwrap();
        assert_eq!(exp_data.absolute, 1000000);
        assert_eq!(exp_data.percentage, 50.0);
    }

    #[test]
    fn test_parse_exp_valid_zero_percent() {
        let result = parse_exp("100000[0%]");
        assert!(result.is_ok(), "Should parse 0%");

        let exp_data = result.unwrap();
        assert_eq!(exp_data.absolute, 100000);
        assert_eq!(exp_data.percentage, 0.0);
    }

    #[test]
    fn test_parse_exp_valid_high_percent() {
        let result = parse_exp("999999[99.99%]");
        assert!(result.is_ok(), "Should parse 99.99%");

        let exp_data = result.unwrap();
        assert_eq!(exp_data.absolute, 999999);
        assert!((exp_data.percentage - 99.99).abs() < 0.01);
    }

    #[test]
    fn test_parse_exp_invalid_no_brackets() {
        let result = parse_exp("5509611");
        assert!(result.is_err(), "Should fail without brackets");
    }

    #[test]
    fn test_parse_exp_invalid_no_absolute() {
        let result = parse_exp("[12.76%]");
        assert!(result.is_err(), "Should fail without absolute value");
    }

    #[test]
    fn test_parse_exp_invalid_percentage_out_of_range() {
        let result = parse_exp("100000[100%]");
        assert!(result.is_err(), "Should fail for 100% (out of range)");
    }

    #[test]
    fn test_parse_exp_invalid_percentage_over_100() {
        let result = parse_exp("100000[150%]");
        assert!(result.is_err(), "Should fail for >100%");
    }

    #[test]
    fn test_parse_exp_with_spaces() {
        let result = parse_exp("5509611[ 12.76 %]");
        assert!(result.is_ok(), "Should handle spaces in brackets");

        let exp_data = result.unwrap();
        assert_eq!(exp_data.absolute, 5509611);
        assert!((exp_data.percentage - 12.76).abs() < 0.01);
    }

    // ============================================================
    // Map Parser Tests (🔴 RED Phase)
    // ============================================================

    #[test]
    fn test_parse_map_valid_korean() {
        let result = parse_map("히든스트리트 작은 난파선");
        assert!(result.is_ok(), "Should parse Korean text");
        assert_eq!(result.unwrap(), "히든스트리트 작은 난파선");
    }

    #[test]
    fn test_parse_map_valid_simple() {
        let result = parse_map("리스항구");
        assert!(result.is_ok(), "Should parse simple Korean text");
        assert_eq!(result.unwrap(), "리스항구");
    }

    #[test]
    fn test_parse_map_with_leading_whitespace() {
        let result = parse_map("  히든스트리트 작은 난파선");
        assert!(result.is_ok(), "Should trim leading whitespace");
        assert_eq!(result.unwrap(), "히든스트리트 작은 난파선");
    }

    #[test]
    fn test_parse_map_with_trailing_whitespace() {
        let result = parse_map("히든스트리트 작은 난파선  ");
        assert!(result.is_ok(), "Should trim trailing whitespace");
        assert_eq!(result.unwrap(), "히든스트리트 작은 난파선");
    }

    #[test]
    fn test_parse_map_invalid_empty() {
        let result = parse_map("");
        assert!(result.is_err(), "Should fail on empty string");
    }

    #[test]
    fn test_parse_map_invalid_whitespace_only() {
        let result = parse_map("   ");
        assert!(result.is_err(), "Should fail on whitespace-only string");
    }

    // ============================================================
    // Validation Tests
    // ============================================================

    #[test]
    fn test_validate_level_valid_range() {
        assert!(validate_level(1), "Level 1 should be valid");
        assert!(validate_level(150), "Level 150 should be valid");
        assert!(validate_level(300), "Level 300 should be valid");
    }

    #[test]
    fn test_validate_level_invalid_range() {
        assert!(!validate_level(0), "Level 0 should be invalid");
        assert!(!validate_level(301), "Level 301 should be invalid");
        assert!(!validate_level(999), "Level 999 should be invalid");
    }

    #[test]
    fn test_validate_exp_percentage_valid_range() {
        assert!(validate_exp_percentage(0.0), "0% should be valid");
        assert!(validate_exp_percentage(50.5), "50.5% should be valid");
        assert!(validate_exp_percentage(99.99), "99.99% should be valid");
    }

    #[test]
    fn test_validate_exp_percentage_invalid_range() {
        assert!(!validate_exp_percentage(100.0), "100% should be invalid");
        assert!(!validate_exp_percentage(150.0), "150% should be invalid");
        assert!(!validate_exp_percentage(-1.0), "Negative should be invalid");
    }

    #[test]
    fn test_validate_map_valid() {
        assert!(validate_map("히든스트리트"), "Korean text should be valid");
        assert!(validate_map("  Text  "), "Text with spaces should be valid after trim");
    }

    #[test]
    fn test_validate_map_invalid() {
        assert!(!validate_map(""), "Empty should be invalid");
        assert!(!validate_map("   "), "Whitespace-only should be invalid");
    }
}
