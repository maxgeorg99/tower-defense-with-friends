use bevy::prelude::*;

#[derive(Component)]
pub struct Enemy {
    pub health: f32,
    pub speed: f32,
    pub current_waypoint: usize,
    pub gold_reward: i32,
    pub damage_to_base: i32,
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
}

#[derive(Component)]
pub struct Projectile {
    pub damage: f32,
    pub speed: f32,
    pub target: Entity,
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
    pub max_workers: i32,
    pub spawned_workers: i32,
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
    pub wood_cost: i32,
}
