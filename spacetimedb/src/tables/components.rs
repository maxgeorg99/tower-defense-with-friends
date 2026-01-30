// =============================================================================
// Entity Components - Type-Specific Data
// =============================================================================
//
// Components store data specific to each entity type.
// They reference the base GameEntity via entity_id.
//
// Pattern: Query GameEntity for position, then join with component table
// for type-specific data.
//
// =============================================================================

use spacetimedb::SpacetimeType;

// =============================================================================
// Combat Types (shared between towers and enemies)
// =============================================================================

/// Attack types for towers/projectiles
#[derive(SpacetimeType, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AttackType {
    #[default]
    Blunt,   // Catapults, rocks
    Pierce,  // Arrows, bolts
    Divine,  // Holy damage
}

/// Defense types for enemies
#[derive(SpacetimeType, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DefenseType {
    #[default]
    Armor,    // Heavy armor - weak to blunt
    Agility,  // Fast/dodgy - weak to pierce
    Mystical, // Magical - weak to divine
}

/// Calculate damage multiplier based on attack vs defense type
pub fn get_damage_multiplier(attack: AttackType, defense: DefenseType) -> f32 {
    match (attack, defense) {
        // Blunt attacks
        (AttackType::Blunt, DefenseType::Armor) => 1.25,
        (AttackType::Blunt, DefenseType::Agility) => 0.85,
        (AttackType::Blunt, DefenseType::Mystical) => 1.10,
        // Pierce attacks
        (AttackType::Pierce, DefenseType::Armor) => 0.80,
        (AttackType::Pierce, DefenseType::Agility) => 1.25,
        (AttackType::Pierce, DefenseType::Mystical) => 0.90,
        // Divine attacks
        (AttackType::Divine, DefenseType::Armor) => 1.00,
        (AttackType::Divine, DefenseType::Agility) => 0.90,
        (AttackType::Divine, DefenseType::Mystical) => 1.30,
    }
}

// =============================================================================
// Tower Component
// =============================================================================

#[spacetimedb::table(name = tower_component, public)]
pub struct TowerComponent {
    #[primary_key]
    pub entity_id: u64,

    /// Tower type identifier (e.g., "archer", "catapult", "holy")
    pub tower_type: String,

    /// Attack range in world units
    pub range: f32,

    /// Base damage per hit
    pub damage: f32,

    /// Seconds between attacks
    pub fire_rate: f32,

    /// Current cooldown (decremented each tick)
    pub cooldown: f32,

    /// Projectile speed (0 = instant damage like holy tower)
    pub projectile_speed: f32,

    /// Attack type for damage calculation
    pub attack_type: AttackType,

    // Upgrade levels
    pub damage_level: i32,
    pub range_level: i32,
    pub fire_rate_level: i32,
}

// =============================================================================
// Enemy Component
// =============================================================================

#[spacetimedb::table(name = enemy_component, public)]
pub struct EnemyComponent {
    #[primary_key]
    pub entity_id: u64,

    /// Enemy type identifier (e.g., "goblin", "skeleton")
    pub enemy_type: String,

    /// Current health
    pub health: f32,

    /// Maximum health (for health bar calculations)
    pub max_health: f32,

    /// Movement speed in world units per second
    pub speed: f32,

    /// Current waypoint index on the path
    pub current_waypoint: i32,

    /// Gold reward when killed
    pub gold_reward: i32,

    /// Damage dealt to base when reaching end
    pub damage_to_base: i32,

    /// Defense type for damage calculation
    pub defense_type: DefenseType,
}

// =============================================================================
// Projectile Component
// =============================================================================

#[spacetimedb::table(name = projectile_component, public)]
pub struct ProjectileComponent {
    #[primary_key]
    pub entity_id: u64,

    /// Target entity ID
    pub target_id: u64,

    /// Damage to deal on hit
    pub damage: f32,

    /// Movement speed
    pub speed: f32,

    /// Attack type for damage calculation
    pub attack_type: AttackType,
}

// =============================================================================
// Unit Component (player-recruited units)
// =============================================================================

#[spacetimedb::table(name = unit_component, public)]
pub struct UnitComponent {
    #[primary_key]
    pub entity_id: u64,

    /// Unit type identifier
    pub unit_type: String,

    /// Current health
    pub health: f32,

    /// Maximum health
    pub max_health: f32,

    /// Movement speed
    pub speed: f32,

    /// Attack damage
    pub damage: f32,

    /// Attack range
    pub attack_range: f32,

    /// Attack cooldown
    pub attack_cooldown: f32,

    /// Current target (enemy entity_id)
    pub target_id: Option<u64>,
}
