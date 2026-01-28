// =============================================================================
// Authentication Helpers
// =============================================================================
//
// Note: In SpacetimeDB 1.11+, JWT claims are not directly accessible
// from ReducerContext. User profile information should be set via
// explicit reducers (set_name, etc.) after connection.
//
// This module is kept as a placeholder for future auth-related helpers.
// =============================================================================

/// User profile data (can be populated from external auth systems)
#[derive(Debug, Default, Clone)]
pub struct UserProfile {
    pub name: Option<String>,
    pub email: Option<String>,
    pub picture: Option<String>,
}

/// Parse user profile from a JSON string (e.g., from external auth callback)
pub fn parse_user_profile(json_str: &str) -> Option<UserProfile> {
    match serde_json::from_str::<serde_json::Value>(json_str) {
        Ok(payload) => {
            Some(UserProfile {
                name: payload.get("name").and_then(|v| v.as_str()).map(String::from),
                email: payload.get("email").and_then(|v| v.as_str()).map(String::from),
                picture: payload.get("picture").and_then(|v| v.as_str()).map(String::from),
            })
        }
        Err(e) => {
            log::warn!("Failed to parse user profile JSON: {}", e);
            None
        }
    }
}
