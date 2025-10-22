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
/// Expected format: "5509611[12.76%]" or "1000000[50%]" or "46185718.57%"
/// Brackets are optional - matches legacy Python parser behavior
/// Returns ExpData with absolute value and percentage
pub fn parse_exp(text: &str) -> Result<ExpData, String> {
    // First, clean the text: remove all characters except digits, ., %, [, ]
    // Matches legacy: re.sub(r"[^0-9\.\%\[\]]+", "", raw)
    let clean = text.chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '%' || *c == '[' || *c == ']')
        .collect::<String>();

    // Find percentage pattern - look for bracket first, then last decimal point
    // Strategy: In "46185718.57%]", find the LAST decimal point before %
    // This gives us "8.57%" instead of "18.57%" or "57%"

    // First try: look for bracket + percentage (most reliable)
    let bracketed_pct = Regex::new(r"\[(\d{1,2}\.?\d*)%").unwrap();

    if let Some(m) = bracketed_pct.find(&clean) {
        // Found bracketed percentage - use it
        let pct_str = m.as_str().trim_start_matches('[').trim_end_matches('%');
        let percentage: f64 = pct_str
            .parse()
            .map_err(|e| format!("Failed to parse percentage '{}': {}", pct_str, e))?;

        let exp_end = m.start();
        let exp_part = &clean[..exp_end];
        let mut exp_str: String = exp_part.chars().filter(|c| c.is_ascii_digit()).collect();

        if exp_str.is_empty() {
            return Err(format!("No absolute value found before percentage in: {}", text));
        }

        // Note: Don't restrict EXP length - it can vary by level
        // Validation should happen in calculator by comparing with previous values

        let absolute: u64 = exp_str
            .parse()
            .map_err(|e| format!("Failed to parse absolute EXP '{}': {}", exp_str, e))?;

        if !validate_exp_percentage(percentage) {
            return Err(format!("Percentage {} out of valid range (0.0-100.0)", percentage));
        }

        return Ok(ExpData { absolute, percentage });
    }

    // Fallback: no bracket found, find last decimal point before %
    // Example: "46185718.57%]" → last decimal at position 7 → "8.57%"
    if let Some(pct_pos) = clean.rfind('%') {
        // Work backwards from % to find decimal point
        let before_pct = &clean[..pct_pos];
        if let Some(dot_pos) = before_pct.rfind('.') {
            // Found decimal point - extract 1 digit before it (single-digit percentage)
            // Real data shows: 8.56%, 8.57%, not 18.57% or 98.23%
            // When bracket "[" becomes "1", we get "...18.57%" but want "8.57%"
            let mut start = dot_pos;
            if start > 0 && clean.chars().nth(start - 1).map_or(false, |c| c.is_ascii_digit()) {
                start -= 1; // Take exactly 1 digit before decimal
            }

            let pct_str = &clean[start..pct_pos];
            let percentage: f64 = pct_str
                .parse()
                .map_err(|e| format!("Failed to parse percentage '{}': {}", pct_str, e))?;

            // EXP is everything before the percentage
            // BUT: if there's a '1' immediately before (likely misread '['), skip it
            let mut exp_end = start;
            if exp_end > 0 && clean.chars().nth(exp_end - 1) == Some('1') {
                exp_end -= 1; // Skip the '1' that's likely a misread '['
            }

            let exp_part = &clean[..exp_end];
            let mut exp_str: String = exp_part.chars().filter(|c| c.is_ascii_digit()).collect();

            if exp_str.is_empty() {
                return Err(format!("No absolute value found before percentage in: {}", text));
            }

            // Note: Don't restrict EXP length - it can vary by level
            // Validation should happen in calculator by comparing with previous values

            let absolute: u64 = exp_str
                .parse()
                .map_err(|e| format!("Failed to parse absolute EXP '{}': {}", exp_str, e))?;

            if !validate_exp_percentage(percentage) {
                return Err(format!("Percentage {} out of valid range (0.0-100.0)", percentage));
            }

            return Ok(ExpData { absolute, percentage });
        }
    }

    Err(format!("No valid percentage pattern found in: {} (cleaned: {})", text, clean))
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
    fn test_parse_exp_valid_problematic_case() {
        // Test the problematic case: "46185718.57%]" (bracket became "1")
        // Should extract: exp=461857, pct=8.57 (not exp=4618571)
        let result = parse_exp("46185718.57%]");
        assert!(result.is_ok(), "Should parse problematic case correctly");

        let exp_data = result.unwrap();
        println!("DEBUG: absolute={}, percentage={}", exp_data.absolute, exp_data.percentage);
        assert_eq!(exp_data.absolute, 461857, "EXP should be 461857");
        assert!((exp_data.percentage - 8.57).abs() < 0.01, "Percentage should be ~8.57, got {}", exp_data.percentage);
    }

    #[test]
    fn test_parse_exp_valid_user_format() {
        // Test the actual user format: "461857[8.57%]"
        // This should parse as: absolute=461857, percentage=8.57
        let result = parse_exp("461857[8.57%]");
        assert!(result.is_ok(), "Should parse user's actual format");

        let exp_data = result.unwrap();
        assert_eq!(exp_data.absolute, 461857);
        assert!((exp_data.percentage - 8.57).abs() < 0.01);
    }

    #[test]
    fn test_parse_exp_valid_multiple_percent_signs() {
        // Test edge case: "461693%8.57%]" (% appeared twice)
        // Should extract: exp=461693, pct=8 (first percentage match)
        let result = parse_exp("461693%8.57%]");
        assert!(result.is_ok(), "Should parse multiple % signs");

        let exp_data = result.unwrap();
        // Note: This will match the FIRST percentage pattern (8.57%)
        // If we want to match differently, we'd need to adjust the logic
        assert_eq!(exp_data.absolute, 461693);
        assert!((exp_data.percentage - 8.57).abs() < 0.01);
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
