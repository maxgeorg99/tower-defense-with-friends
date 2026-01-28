// =============================================================================
// Wave Spawner Agent
// =============================================================================
//
// Handles spawning enemies during active waves.
// Runs at a slower rate than the main game tick.
//
// =============================================================================

use spacetimedb::{ReducerContext, ScheduleAt, Table, TimeDuration};
use crate::tables::game_entity::{GameEntity, game_entity as GameEntityTable};
use crate::tables::components::{EnemyComponent, DefenseType, enemy_component as EnemyComponentTable};
use crate::tables::wave::{
    WaveState, WaveConfig, EnemyTypeDef, PathWaypoint,
    wave_state as WaveStateTable,
    wave_config as WaveConfigTable,
    enemy_type_def as EnemyTypeDefTable,
    path_waypoint as PathWaypointTable,
};
use crate::tables::game_state::{GameState, GameStatus, game_state as GameStateTable};

/// Spawn check interval in microseconds (200ms = 5 checks per second)
const SPAWN_CHECK_INTERVAL_US: i64 = 200_000;

// =============================================================================
// Timer Table
// =============================================================================

#[spacetimedb::table(name = wave_spawner_timer, scheduled(wave_spawner_tick, at = scheduled_at))]
pub struct WaveSpawnerTimer {
    #[primary_key]
    #[auto_inc]
    pub scheduled_id: u64,
    pub scheduled_at: ScheduleAt,
}

// =============================================================================
// Agent Functions
// =============================================================================

pub fn init(ctx: &ReducerContext) {
    ctx.db.wave_spawner_timer().insert(WaveSpawnerTimer {
        scheduled_id: 0,
        scheduled_at: ScheduleAt::Time(ctx.timestamp),
    });
    log::info!("Wave spawner agent initialized");
}

#[spacetimedb::reducer]
pub fn wave_spawner_tick(ctx: &ReducerContext, timer: WaveSpawnerTimer) {
    // Delete triggering timer
    ctx.db.wave_spawner_timer().scheduled_id().delete(timer.scheduled_id);

    // Get wave state
    if let Some(mut wave_state) = ctx.db.wave_state().id().find(0) {
        if wave_state.wave_active {
            process_wave_spawning(ctx, &mut wave_state);
        }
    }

    // Reschedule
    let next_time = ctx.timestamp + TimeDuration::from_micros(SPAWN_CHECK_INTERVAL_US);
    ctx.db.wave_spawner_timer().insert(WaveSpawnerTimer {
        scheduled_id: 0,
        scheduled_at: ScheduleAt::Time(next_time),
    });
}

fn process_wave_spawning(ctx: &ReducerContext, wave_state: &mut WaveState) {
    // Check if we've spawned all enemies
    if wave_state.enemies_spawned >= wave_state.enemies_this_wave {
        return;
    }

    // Update spawn cooldown (200ms = 0.2s per tick)
    wave_state.spawn_cooldown -= 0.2;

    if wave_state.spawn_cooldown <= 0.0 {
        // Time to spawn!
        spawn_next_enemy(ctx, wave_state);
    }

    ctx.db.wave_state().id().update(wave_state.clone());
}

fn spawn_next_enemy(ctx: &ReducerContext, wave_state: &mut WaveState) {
    // Get wave configs for current wave
    let wave_configs: Vec<_> = ctx.db.wave_config()
        .iter()
        .filter(|c| c.wave_number == wave_state.current_wave)
        .collect();

    // Find which enemy type to spawn based on spawn count
    let mut cumulative = 0;
    let mut selected_config: Option<&WaveConfig> = None;

    for config in &wave_configs {
        if wave_state.enemies_spawned >= cumulative
            && wave_state.enemies_spawned < cumulative + config.count
        {
            selected_config = Some(config);
            break;
        }
        cumulative += config.count;
    }

    let Some(config) = selected_config else {
        return;
    };

    // Get enemy type definition
    let Some(enemy_def) = ctx.db.enemy_type_def().enemy_type().find(&config.enemy_type) else {
        log::warn!("Unknown enemy type: {}", config.enemy_type);
        return;
    };

    // Get spawn position (first waypoint)
    let spawn_pos = ctx.db.path_waypoint()
        .iter()
        .filter(|w| w.order_index == 0)
        .next()
        .map(|w| (w.x, w.y))
        .unwrap_or((0.0, 0.0));

    // Create entity
    let entity = GameEntity::new_enemy(spawn_pos.0, spawn_pos.1);
    let entity = ctx.db.game_entity().insert(entity);
    let entity_id = entity.entity_id;

    // Create enemy component
    let max_health = enemy_def.base_health * config.health_multiplier;
    let enemy_component = EnemyComponent {
        entity_id,
        enemy_type: config.enemy_type.clone(),
        health: max_health,
        max_health,
        speed: enemy_def.base_speed,
        current_waypoint: 0,
        gold_reward: enemy_def.gold_reward,
        damage_to_base: enemy_def.damage_to_base,
        defense_type: enemy_def.defense_type,
    };
    ctx.db.enemy_component().insert(enemy_component);

    // Update spawn state
    wave_state.enemies_spawned += 1;
    wave_state.spawn_cooldown = config.spawn_interval;

    log::debug!(
        "Spawned enemy {} ({}) at ({}, {})",
        entity_id, config.enemy_type, spawn_pos.0, spawn_pos.1
    );
}
