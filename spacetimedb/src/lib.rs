// =============================================================================
// Tower Defense MMO - SpacetimeDB Server Module
// =============================================================================
//
// This is the main entry point for the SpacetimeDB WebAssembly module.
// The server is organized into the following modules:
//
// - tables/    : Database table definitions
// - reducers/  : Client-callable actions
// - agents/    : Scheduled game loops (wave spawning, combat, etc.)
// - helpers/   : Shared utility functions
//
// =============================================================================

use spacetimedb::{ReducerContext, Table};

// Module declarations
pub mod tables;
pub mod reducers;
pub mod agents;
pub mod helpers;

// Re-export commonly used types for client bindings
pub use tables::user::{User, Color};
pub use tables::message::Message;

// Import table access traits
use tables::user::user as UserTable;
use tables::game_state::{GameState, game_state as GameStateTable};
use tables::wave::{WaveState, wave_state as WaveStateTable, TowerTypeDef, tower_type_def as TowerTypeDefTable, EnemyTypeDef, enemy_type_def as EnemyTypeDefTable};
use tables::components::{AttackType, DefenseType};

// =============================================================================
// Lifecycle Hooks
// =============================================================================

/// Called once when the module is first published
#[spacetimedb::reducer(init)]
pub fn init(ctx: &ReducerContext) {
    log::info!("Tower Defense server initialized!");

    // Initialize singleton tables
    init_game_state(ctx);
    init_wave_state(ctx);

    // Seed static data
    init_tower_types(ctx);
    init_enemy_types(ctx);

    // Initialize all agents (scheduled game loops)
    agents::init(ctx);
}

/// Initialize the game state singleton
fn init_game_state(ctx: &ReducerContext) {
    if ctx.db.game_state().id().find(0).is_none() {
        ctx.db.game_state().insert(GameState::default());
        log::info!("Game state initialized");
    }
}

/// Initialize the wave state singleton
fn init_wave_state(ctx: &ReducerContext) {
    if ctx.db.wave_state().id().find(0).is_none() {
        ctx.db.wave_state().insert(WaveState::default());
        log::info!("Wave state initialized");
    }
}

/// Initialize tower type definitions (seed data)
fn init_tower_types(ctx: &ReducerContext) {
    // Only seed if table is empty
    if ctx.db.tower_type_def().iter().next().is_some() {
        return;
    }

    let towers = vec![
        TowerTypeDef {
            tower_type: "archer".to_string(),
            name: "Archer".to_string(),
            cost: 50,
            base_range: 256.0,
            base_damage: 25.0,
            base_fire_rate: 0.5,
            projectile_speed: 300.0,
            attack_type: AttackType::Pierce,
        },
        TowerTypeDef {
            tower_type: "catapult".to_string(),
            name: "Catapult".to_string(),
            cost: 100,
            base_range: 384.0,
            base_damage: 60.0,
            base_fire_rate: 1.5,
            projectile_speed: 200.0,
            attack_type: AttackType::Blunt,
        },
        TowerTypeDef {
            tower_type: "holy".to_string(),
            name: "Holy".to_string(),
            cost: 300,
            base_range: 256.0,
            base_damage: 25.0,
            base_fire_rate: 5.0,
            projectile_speed: 300.0,
            attack_type: AttackType::Divine,
        },
        TowerTypeDef {
            tower_type: "tower".to_string(),
            name: "Tower".to_string(),
            cost: 50,
            base_range: 256.0,
            base_damage: 25.0,
            base_fire_rate: 0.5,
            projectile_speed: 300.0,
            attack_type: AttackType::Pierce,
        },
    ];

    for tower in towers {
        ctx.db.tower_type_def().insert(tower);
    }
    log::info!("Tower type definitions seeded");
}

/// Initialize enemy type definitions (seed data)
fn init_enemy_types(ctx: &ReducerContext) {
    // Only seed if table is empty
    if ctx.db.enemy_type_def().iter().next().is_some() {
        return;
    }

    let enemies = vec![
        EnemyTypeDef {
            enemy_type: "warrior".to_string(),
            name: "Red Warrior".to_string(),
            base_health: 50.0,
            base_speed: 50.0,
            gold_reward: 10,
            damage_to_base: 1,
            defense_type: DefenseType::Armor,
        },
        EnemyTypeDef {
            enemy_type: "archer".to_string(),
            name: "Red Archer".to_string(),
            base_health: 30.0,
            base_speed: 60.0,
            gold_reward: 8,
            damage_to_base: 1,
            defense_type: DefenseType::Agility,
        },
        EnemyTypeDef {
            enemy_type: "lancer".to_string(),
            name: "Red Lancer".to_string(),
            base_health: 80.0,
            base_speed: 40.0,
            gold_reward: 15,
            damage_to_base: 2,
            defense_type: DefenseType::Armor,
        },
        EnemyTypeDef {
            enemy_type: "monk".to_string(),
            name: "Red Monk".to_string(),
            base_health: 80.0,
            base_speed: 40.0,
            gold_reward: 25,
            damage_to_base: 3,
            defense_type: DefenseType::Mystical,
        },
        EnemyTypeDef {
            enemy_type: "skull".to_string(),
            name: "Skull".to_string(),
            base_health: 70.0,
            base_speed: 45.0,
            gold_reward: 12,
            damage_to_base: 2,
            defense_type: DefenseType::Mystical,
        },
        EnemyTypeDef {
            enemy_type: "turtle".to_string(),
            name: "Turtle".to_string(),
            base_health: 120.0,
            base_speed: 20.0,
            gold_reward: 15,
            damage_to_base: 1,
            defense_type: DefenseType::Armor,
        },
        EnemyTypeDef {
            enemy_type: "minotaur".to_string(),
            name: "Minotaur".to_string(),
            base_health: 150.0,
            base_speed: 35.0,
            gold_reward: 20,
            damage_to_base: 3,
            defense_type: DefenseType::Armor,
        },
        EnemyTypeDef {
            enemy_type: "ogre".to_string(),
            name: "Ogre".to_string(),
            base_health: 400.0,
            base_speed: 25.0,
            gold_reward: 50,
            damage_to_base: 5,
            defense_type: DefenseType::Armor,
        },
    ];

    for enemy in enemies {
        ctx.db.enemy_type_def().insert(enemy);
    }
    log::info!("Enemy type definitions seeded");
}

/// Called when a client connects
#[spacetimedb::reducer(client_connected)]
pub fn identity_connected(ctx: &ReducerContext) {
    let identity = ctx.sender;

    // Check if user already exists
    if let Some(mut user) = ctx.db.user().identity().find(identity) {
        // Returning user - update online status
        user.online = true;
        ctx.db.user().identity().update(user);
        log::info!("User reconnected: {:?}", identity);
    } else {
        // New user - create with default resources
        let new_user = User::new(identity);
        ctx.db.user().insert(new_user);
        log::info!("New user connected: {:?}", identity);
    }
}

/// Called when a client disconnects
#[spacetimedb::reducer(client_disconnected)]
pub fn identity_disconnected(ctx: &ReducerContext) {
    let identity = ctx.sender;

    if let Some(mut user) = ctx.db.user().identity().find(identity) {
        user.online = false;
        ctx.db.user().identity().update(user);
        log::info!("User disconnected: {:?}", identity);
    } else {
        log::warn!("Disconnect for unknown user: {:?}", identity);
    }
}
