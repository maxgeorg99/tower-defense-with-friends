// =============================================================================
// Enemy Movement Agent
// =============================================================================
//
// Moves enemies along the path waypoints.
// Runs at game tick rate (20Hz) for smooth movement.
//
// =============================================================================

use spacetimedb::{ReducerContext, ScheduleAt, Table, TimeDuration};
use crate::tables::game_entity::{GameEntity, EntityType, game_entity as GameEntityTable};
use crate::tables::components::{EnemyComponent, enemy_component as EnemyComponentTable};
use crate::tables::wave::{PathWaypoint, WaveState, path_waypoint as PathWaypointTable, wave_state as WaveStateTable};
use crate::tables::game_state::{GameState, GameStatus, game_state as GameStateTable};

/// Movement tick interval in microseconds (50ms = 20Hz)
const MOVEMENT_TICK_US: i64 = 50_000;
/// Delta time in seconds for movement calculations
const DELTA_TIME: f32 = 0.05;
/// Distance threshold to consider waypoint reached
const WAYPOINT_THRESHOLD: f32 = 5.0;

// =============================================================================
// Timer Table
// =============================================================================

#[spacetimedb::table(name = enemy_movement_timer, scheduled(enemy_movement_tick, at = scheduled_at))]
pub struct EnemyMovementTimer {
    #[primary_key]
    #[auto_inc]
    pub scheduled_id: u64,
    pub scheduled_at: ScheduleAt,
}

// =============================================================================
// Agent Functions
// =============================================================================

pub fn init(ctx: &ReducerContext) {
    ctx.db.enemy_movement_timer().insert(EnemyMovementTimer {
        scheduled_id: 0,
        scheduled_at: ScheduleAt::Time(ctx.timestamp),
    });
    log::info!("Enemy movement agent initialized");
}

#[spacetimedb::reducer]
pub fn enemy_movement_tick(ctx: &ReducerContext, timer: EnemyMovementTimer) {
    // Delete triggering timer
    ctx.db.enemy_movement_timer().scheduled_id().delete(timer.scheduled_id);

    // Process all active enemies
    process_enemy_movement(ctx);

    // Reschedule
    let next_time = ctx.timestamp + TimeDuration::from_micros(MOVEMENT_TICK_US);
    ctx.db.enemy_movement_timer().insert(EnemyMovementTimer {
        scheduled_id: 0,
        scheduled_at: ScheduleAt::Time(next_time),
    });
}

fn process_enemy_movement(ctx: &ReducerContext) {
    // Get all waypoints sorted by order
    let mut waypoints: Vec<_> = ctx.db.path_waypoint().iter().collect();
    waypoints.sort_by_key(|w| w.order_index);

    if waypoints.is_empty() {
        return;
    }

    let max_waypoint = waypoints.len() as i32;

    // Get all active enemy entities
    let enemies: Vec<_> = ctx.db.game_entity()
        .iter()
        .filter(|e| e.entity_type == EntityType::Enemy && e.active)
        .collect();

    for entity in enemies {
        // Get enemy component
        let Some(mut enemy) = ctx.db.enemy_component().entity_id().find(entity.entity_id) else {
            continue;
        };

        // Check if reached end of path
        if enemy.current_waypoint >= max_waypoint {
            handle_enemy_reached_base(ctx, entity.entity_id, &enemy);
            continue;
        }

        // Get target waypoint
        let waypoint = &waypoints[enemy.current_waypoint as usize];
        let target_x = waypoint.x;
        let target_y = waypoint.y;

        // Calculate direction to waypoint
        let dx = target_x - entity.x;
        let dy = target_y - entity.y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance < WAYPOINT_THRESHOLD {
            // Reached waypoint, move to next
            enemy.current_waypoint += 1;
            ctx.db.enemy_component().entity_id().update(enemy);
        } else {
            // Move towards waypoint
            let dir_x = dx / distance;
            let dir_y = dy / distance;

            let move_dist = enemy.speed * DELTA_TIME;
            let new_x = entity.x + dir_x * move_dist;
            let new_y = entity.y + dir_y * move_dist;

            // Update entity position
            let mut updated_entity = entity.clone();
            updated_entity.x = new_x;
            updated_entity.y = new_y;
            ctx.db.game_entity().entity_id().update(updated_entity);
        }
    }
}

fn handle_enemy_reached_base(ctx: &ReducerContext, entity_id: u64, enemy: &EnemyComponent) {
    // Deal damage to base
    if let Some(mut game_state) = ctx.db.game_state().id().find(0) {
        game_state.lives -= enemy.damage_to_base;

        // Check for game over
        if game_state.lives <= 0 {
            game_state.lives = 0;
            game_state.status = GameStatus::GameOver;
            log::info!("Game Over! Base destroyed.");
        }

        ctx.db.game_state().id().update(game_state);
    }

    // Mark entity as inactive
    if let Some(mut entity) = ctx.db.game_entity().entity_id().find(entity_id) {
        entity.active = false;
        ctx.db.game_entity().entity_id().update(entity);
    }

    // Delete enemy component
    ctx.db.enemy_component().entity_id().delete(entity_id);

    log::debug!("Enemy {} reached base, dealt {} damage", entity_id, enemy.damage_to_base);
}
