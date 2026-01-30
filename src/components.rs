use bevy::prelude::*;

// ==================== Combat Type System ====================

/// Attack types for towers/projectiles
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AttackType {
    #[default]
    Blunt,   // Hammers, catapults, rocks
    Pierce,  // Arrows, spears, bolts
    Divine,  // Holy/magical damage
}

/// Defense types for enemies
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DefenseType {
    #[default]
    Armor,    // Heavy armor - weak to blunt, strong vs pierce
    Agility,  // Dodgy/fast - weak to pierce, strong vs blunt
    Mystical, // Magical creatures - weak to divine
}

impl AttackType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "blunt" => AttackType::Blunt,
            "pierce" => AttackType::Pierce,
            "divine" => AttackType::Divine,
            _ => AttackType::Blunt,
        }
    }
}

impl DefenseType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "armor" => DefenseType::Armor,
            "agility" => DefenseType::Agility,
            "mystical" => DefenseType::Mystical,
            _ => DefenseType::Armor,
        }
    }
}

/// Calculate damage multiplier based on attack vs defense type
/// Returns a multiplier (e.g., 1.25 for +25%, 0.85 for -15%)
pub fn get_damage_multiplier(attack: AttackType, defense: DefenseType) -> f32 {
    match (attack, defense) {
        // Blunt attacks
        (AttackType::Blunt, DefenseType::Armor) => 1.25,    // +25%
        (AttackType::Blunt, DefenseType::Agility) => 0.85,  // -15%
        (AttackType::Blunt, DefenseType::Mystical) => 1.10, // +10%

        // Pierce attacks
        (AttackType::Pierce, DefenseType::Armor) => 0.80,   // -20%
        (AttackType::Pierce, DefenseType::Agility) => 1.25, // +25%
        (AttackType::Pierce, DefenseType::Mystical) => 0.90, // -10%

        // Divine attacks
        (AttackType::Divine, DefenseType::Armor) => 1.00,   // 0%
        (AttackType::Divine, DefenseType::Agility) => 0.90, // -10%
        (AttackType::Divine, DefenseType::Mystical) => 1.30, // +30%
    }
}

// ==================== Core Components ====================

#[derive(Component)]
pub struct Enemy {
    pub health: f32,
    pub speed: f32,
    pub current_waypoint: usize,
    pub gold_reward: i32,
    pub damage_to_base: i32,
    pub defense_type: DefenseType,
}

#[derive(Component)]
pub struct AnimationTimer {
    pub timer: Timer,
}

#[derive(Component)]
pub struct Tower {
    pub tower_type_id: String,
    pub range: f32,
    pub damage: f32,
    pub fire_rate: f32,
    pub cooldown: f32,
    pub projectile_sprite: String,
    pub projectile_speed: f32,
    pub attack_type: AttackType,
}

#[derive(Component)]
pub struct Projectile {
    pub damage: f32,
    pub speed: f32,
    pub target: Entity,
    pub attack_type: AttackType,
}

#[derive(Component)]
pub struct HealthBar {
    pub max_health: f32,
}

#[derive(Component)]
pub struct HealthBarFill {
    pub max_width: f32,
}

#[derive(Component)]
pub struct Castle;

#[derive(Component)]
pub struct GameUI;

#[derive(Component)]
pub struct TowerWheelMenu;

#[derive(Component)]
pub struct TowerWheelOption {
    pub tower_type_id: String,
}

#[derive(Component)]
pub struct FogTile {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Component)]
pub struct GameOverScreen;

// Resource types
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResourceType {
    Wood,
    Gold,
    Meat,
}

// Worker state machine
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum WorkerState {
    #[default]
    Idle,
    MovingToResource,
    Harvesting,
    ReturningWithResource,
}

// Worker component
#[derive(Component)]
pub struct Worker {
    pub speed: f32,
    pub home_building: Entity,
    pub current_resource: Option<ResourceType>,
}

// Target for workers
#[derive(Component)]
pub struct WorkerTarget {
    pub target_entity: Option<Entity>,
    pub target_position: Vec2,
}

// Resource node (tree)
#[derive(Component)]
pub struct ResourceNode {
    pub resource_type: ResourceType,
    pub remaining: i32,
}

// Marker for depleted resource
#[derive(Component)]
pub struct Depleted;

// Building that spawns workers
#[derive(Component)]
pub struct WorkerBuilding {
    pub spawn_timer: Timer,
    /// How many workers this building can have (increases when buying workers)
    pub worker_capacity: i32,
    /// How many workers have been spawned so far
    pub current_workers: i32,
}

// Harvest progress timer
#[derive(Component)]
pub struct HarvestTimer(pub Timer);

// Recruit menu components
#[derive(Component)]
pub struct RecruitMenu;

#[derive(Component)]
pub struct RecruitOption {
    pub unit_id: String,
    pub meat_cost: i32,
}

// House menu components
#[derive(Component)]
pub struct HouseMenu;

#[derive(Component)]
pub struct BuildWorkerOption {
    pub gold_cost: i32,
}

// Tower upgrade menu components
#[derive(Component)]
pub struct TowerUpgradeMenu;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UpgradeType {
    Damage,
    Range,
    FireRate,
}

#[derive(Component)]
pub struct TowerUpgradeOption {
    pub upgrade_type: UpgradeType,
    pub wood_cost: i32,
}

#[derive(Component)]
pub struct TowerSellButton {
    pub gold_refund: i32,
}

#[derive(Component, Default)]
pub struct TowerLevel {
    pub damage_level: i32,
    pub range_level: i32,
    pub fire_rate_level: i32,
}

/// Temporary visual effect component for holy tower heal animation
#[derive(Component)]
pub struct HolyTowerEffect {
    pub lifetime: Timer,
}

/// Temporary visual effect component for explosion animation (tower sell)
#[derive(Component)]
pub struct ExplosionEffect {
    pub frame_count: usize,
    pub timer: Timer,
}

/// Range indicator circle shown when selecting a tower to place
#[derive(Component)]
pub struct RangeIndicator;

/// Helper to get attack type icon path
pub fn get_attack_type_icon(attack_type: AttackType) -> &'static str {
    match attack_type {
        AttackType::Blunt => "Decorations/Rocks/Rock2.png",
        AttackType::Pierce => "Units/Blue Units/Archer/Arrow.png",
        AttackType::Divine => "UI Elements/UI Elements/Icons/Divine_Icon.png",
    }
}

/// Helper to get defense type icon path
pub fn get_defense_type_icon(defense_type: DefenseType) -> &'static str {
    match defense_type {
        DefenseType::Armor => "UI Elements/UI Elements/Icons/Defense_Icon.png",
        DefenseType::Agility => "UI Elements/UI Elements/Icons/Agility_Icon.png",
        DefenseType::Mystical => "UI Elements/UI Elements/Icons/Mystical_Icon.png",
    }
}
