// =============================================================================
// Agents (Scheduled Game Loops)
// =============================================================================
//
// Agents are self-rescheduling reducers that run game logic on a timer.
// They use SpacetimeDB's scheduled tables to trigger at specific times.
//
// Pattern (from BitCraft):
// 1. Define a timer table with `scheduled(reducer_name, at = scheduled_at)`
// 2. In init(), insert the first timer entry
// 3. The reducer runs, does work, then reschedules itself
//
// Agents:
// - wave_spawner_agent.rs    : Spawns enemies during active waves (5Hz)
// - enemy_movement_agent.rs  : Moves enemies along path (20Hz)
// - tower_attack_agent.rs    : Towers target and attack enemies (20Hz)
// - projectile_agent.rs      : Move projectiles, check collisions (20Hz)
// - cleanup_agent.rs         : Remove inactive entities, check wave end (1Hz)
//
// =============================================================================

pub mod wave_spawner_agent;
pub mod enemy_movement_agent;
pub mod tower_attack_agent;
pub mod projectile_agent;
pub mod cleanup_agent;
pub mod worker_agent;

use spacetimedb::ReducerContext;

/// Initialize all agents - called from lib.rs init()
pub fn init(ctx: &ReducerContext) {
    log::info!("Initializing agents...");

    wave_spawner_agent::init(ctx);
    enemy_movement_agent::init(ctx);
    tower_attack_agent::init(ctx);
    projectile_agent::init(ctx);
    cleanup_agent::init(ctx);
    worker_agent::init(ctx);

    log::info!("All agents initialized.");
}
