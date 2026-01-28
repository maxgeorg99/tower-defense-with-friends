// =============================================================================
// Wave Management Tables
// =============================================================================
//
// Handles wave state, spawn queues, and wave configuration.
//
// =============================================================================

use spacetimedb::SpacetimeType;
use crate::tables::components::DefenseType;

// =============================================================================
// Wave State
// =============================================================================

/// Current wave state for the game
#[spacetimedb::table(name = wave_state, public)]
#[derive(Clone)]
pub struct WaveState {
    #[primary_key]
    pub id: u64,  // Always 0 (singleton)

    /// Current wave number (1-indexed)
    pub current_wave: i32,

    /// Whether a wave is currently active
    pub wave_active: bool,

    /// Countdown until next wave starts (seconds)
    pub time_until_next_wave: f32,

    /// Number of enemies spawned this wave
    pub enemies_spawned: i32,

    /// Total enemies to spawn this wave
    pub enemies_this_wave: i32,

    /// Time until next enemy spawn (seconds)
    pub spawn_cooldown: f32,
}

impl Default for WaveState {
    fn default() -> Self {
        Self {
            id: 0,
            current_wave: 1,
            wave_active: false,
            time_until_next_wave: 30.0, // 30 seconds before first wave
            enemies_spawned: 0,
            enemies_this_wave: 0,
            spawn_cooldown: 0.0,
        }
    }
}

// =============================================================================
// Wave Configuration (static data)
// =============================================================================

/// Defines a wave's spawn configuration
#[spacetimedb::table(name = wave_config, public)]
pub struct WaveConfig {
    #[primary_key]
    #[auto_inc]
    pub config_id: u64,

    /// Wave number this config applies to
    pub wave_number: i32,

    /// Enemy type to spawn
    pub enemy_type: String,

    /// Number of this enemy type to spawn
    pub count: i32,

    /// Health multiplier for this wave
    pub health_multiplier: f32,

    /// Spawn interval between enemies (seconds)
    pub spawn_interval: f32,
}

// =============================================================================
// Enemy Type Definitions (static data)
// =============================================================================

/// Defines an enemy type's base stats
#[spacetimedb::table(name = enemy_type_def, public)]
pub struct EnemyTypeDef {
    #[primary_key]
    pub enemy_type: String,

    /// Display name
    pub name: String,

    /// Base health
    pub base_health: f32,

    /// Base movement speed
    pub base_speed: f32,

    /// Gold reward when killed
    pub gold_reward: i32,

    /// Damage to base when reaching end
    pub damage_to_base: i32,

    /// Defense type
    pub defense_type: DefenseType,
}

// =============================================================================
// Tower Type Definitions (static data)
// =============================================================================

/// Defines a tower type's base stats
#[spacetimedb::table(name = tower_type_def, public)]
pub struct TowerTypeDef {
    #[primary_key]
    pub tower_type: String,

    /// Display name
    pub name: String,

    /// Gold cost to build
    pub cost: i32,

    /// Base attack range
    pub base_range: f32,

    /// Base damage
    pub base_damage: f32,

    /// Base fire rate (seconds between attacks)
    pub base_fire_rate: f32,

    /// Projectile speed (0 = instant)
    pub projectile_speed: f32,

    /// Attack type
    pub attack_type: crate::tables::components::AttackType,
}

// =============================================================================
// Unit Type Definitions (for recruiting)
// =============================================================================

/// Defines a recruitable unit type's stats
#[spacetimedb::table(name = unit_type_def, public)]
pub struct UnitTypeDef {
    #[primary_key]
    pub unit_type: String,

    /// Display name
    pub name: String,

    /// Meat cost to recruit
    pub meat_cost: i32,

    /// Base health
    pub base_health: f32,

    /// Movement speed
    pub base_speed: f32,

    /// Attack damage
    pub base_damage: f32,

    /// Attack range
    pub attack_range: f32,

    /// Attack cooldown (seconds)
    pub attack_cooldown: f32,

    /// Attack type
    pub attack_type: crate::tables::components::AttackType,
}

// =============================================================================
// Path Waypoints
// =============================================================================

/// Defines a waypoint on the enemy path
#[spacetimedb::table(name = path_waypoint, public)]
pub struct PathWaypoint {
    #[primary_key]
    #[auto_inc]
    pub waypoint_id: u64,

    /// Order in the path (0 = start, higher = closer to base)
    pub order_index: i32,

    /// Position
    pub x: f32,
    pub y: f32,
}
