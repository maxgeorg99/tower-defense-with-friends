// =============================================================================
// Cleanup Agent
// =============================================================================
//
// Periodically removes inactive entities and checks for wave completion.
// Runs at a slower rate than game tick.
//
// =============================================================================

use spacetimedb::{ReducerContext, ScheduleAt, Table, TimeDuration};
use crate::tables::game_entity::{GameEntity, EntityType, game_entity as GameEntityTable};
use crate::tables::components::{
    enemy_component as EnemyComponentTable,
};
use crate::tables::wave::{WaveState, WaveConfig, wave_state as WaveStateTable, wave_config as WaveConfigTable};
use crate::tables::game_state::{GameState, GameStatus, game_state as GameStateTable};

/// Cleanup interval in microseconds (1 second)
const CLEANUP_INTERVAL_US: i64 = 1_000_000;

// =============================================================================
// Timer Table
// =============================================================================

#[spacetimedb::table(name = cleanup_timer, scheduled(cleanup_tick, at = scheduled_at))]
pub struct CleanupTimer {
    #[primary_key]
    #[auto_inc]
    pub scheduled_id: u64,
    pub scheduled_at: ScheduleAt,
}

// =============================================================================
// Agent Functions
// =============================================================================

pub fn init(ctx: &ReducerContext) {
    ctx.db.cleanup_timer().insert(CleanupTimer {
        scheduled_id: 0,
        scheduled_at: ScheduleAt::Time(ctx.timestamp),
    });
    log::info!("Cleanup agent initialized");
}

#[spacetimedb::reducer]
pub fn cleanup_tick(ctx: &ReducerContext, timer: CleanupTimer) {
    // Delete triggering timer
    ctx.db.cleanup_timer().scheduled_id().delete(timer.scheduled_id);

    // Cleanup inactive entities
    cleanup_inactive_entities(ctx);

    // Check for wave completion
    check_wave_completion(ctx);

    // Reschedule
    let next_time = ctx.timestamp + TimeDuration::from_micros(CLEANUP_INTERVAL_US);
    ctx.db.cleanup_timer().insert(CleanupTimer {
        scheduled_id: 0,
        scheduled_at: ScheduleAt::Time(next_time),
    });
}

fn cleanup_inactive_entities(ctx: &ReducerContext) {
    // Find and delete all inactive entities
    let inactive: Vec<_> = ctx.db.game_entity()
        .iter()
        .filter(|e| !e.active)
        .collect();

    let count = inactive.len();

    for entity in inactive {
        ctx.db.game_entity().entity_id().delete(entity.entity_id);
    }

    if count > 0 {
        log::debug!("Cleaned up {} inactive entities", count);
    }
}

fn check_wave_completion(ctx: &ReducerContext) {
    let Some(mut wave_state) = ctx.db.wave_state().id().find(0) else {
        return;
    };

    if !wave_state.wave_active {
        return;
    }

    // Check if all enemies have been spawned and killed
    let active_enemies = ctx.db.game_entity()
        .iter()
        .filter(|e| e.entity_type == EntityType::Enemy && e.active)
        .count();

    let all_spawned = wave_state.enemies_spawned >= wave_state.enemies_this_wave;

    if all_spawned && active_enemies == 0 {
        // Wave complete!
        wave_state.wave_active = false;
        wave_state.current_wave += 1;
        wave_state.time_until_next_wave = 30.0; // 30 seconds until next wave
        ctx.db.wave_state().id().update(wave_state.clone());

        // Update game state
        if let Some(mut game_state) = ctx.db.game_state().id().find(0) {
            game_state.waves_completed += 1;
            game_state.status = GameStatus::PreWave;

            // Check for victory (no more waves configured)
            let next_wave_configs: Vec<_> = ctx.db.wave_config()
                .iter()
                .filter(|c| c.wave_number == wave_state.current_wave)
                .collect();

            if next_wave_configs.is_empty() {
                game_state.status = GameStatus::Victory;
                log::info!("Victory! All waves completed.");
            }

            ctx.db.game_state().id().update(game_state);
        }

        log::info!("Wave {} complete!", wave_state.current_wave - 1);
    }
}
