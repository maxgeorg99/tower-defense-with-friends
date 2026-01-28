// =============================================================================
// Input Validation Helpers
// =============================================================================

/// Maximum allowed name length
const MAX_NAME_LENGTH: usize = 32;

/// Maximum allowed message length
const MAX_MESSAGE_LENGTH: usize = 500;

/// Validate and sanitize a player name
pub fn validate_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();

    if trimmed.is_empty() {
        return Err("Name cannot be empty".to_string());
    }

    if trimmed.len() > MAX_NAME_LENGTH {
        return Err(format!("Name cannot exceed {} characters", MAX_NAME_LENGTH));
    }

    // Basic sanitization - remove control characters
    let sanitized: String = trimmed
        .chars()
        .filter(|c| !c.is_control())
        .collect();

    if sanitized.is_empty() {
        return Err("Name cannot be empty after sanitization".to_string());
    }

    Ok(sanitized)
}

/// Validate and sanitize a chat message
pub fn validate_message(text: &str) -> Result<String, String> {
    let trimmed = text.trim();

    if trimmed.is_empty() {
        return Err("Message cannot be empty".to_string());
    }

    if trimmed.len() > MAX_MESSAGE_LENGTH {
        return Err(format!("Message cannot exceed {} characters", MAX_MESSAGE_LENGTH));
    }

    // Basic sanitization - remove control characters except newlines
    let sanitized: String = trimmed
        .chars()
        .filter(|c| !c.is_control() || *c == '\n')
        .collect();

    if sanitized.is_empty() {
        return Err("Message cannot be empty after sanitization".to_string());
    }

    Ok(sanitized)
}

/// Validate a positive integer value (e.g., for costs, amounts)
pub fn validate_positive_i32(value: i32, field_name: &str) -> Result<i32, String> {
    if value <= 0 {
        return Err(format!("{} must be positive", field_name));
    }
    Ok(value)
}

/// Validate a coordinate is within bounds
pub fn validate_coordinate(x: i32, y: i32, max_x: i32, max_y: i32) -> Result<(i32, i32), String> {
    if x < 0 || x >= max_x || y < 0 || y >= max_y {
        return Err(format!(
            "Coordinates ({}, {}) out of bounds (0-{}, 0-{})",
            x, y, max_x - 1, max_y - 1
        ));
    }
    Ok((x, y))
}
