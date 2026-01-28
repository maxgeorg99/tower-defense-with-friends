// =============================================================================
// Wave Actions
// =============================================================================

use spacetimedb::{ReducerContext, Table};
use crate::tables::wave::{WaveState, WaveConfig, wave_state as WaveStateTable, wave_config as WaveConfigTable};
use crate::tables::game_state::{GameState, GameStatus, game_state as GameStateTable};

/// Manually start the next wave (skip countdown)
#[spacetimedb::reducer]
pub fn start_wave(ctx: &ReducerContext) -> Result<(), String> {
    let mut wave_state = ctx.db.wave_state().id().find(0)
        .ok_or("Wave state not found")?;

    let mut game_state = ctx.db.game_state().id().find(0)
        .ok_or("Game state not found")?;

    if wave_state.wave_active {
        return Err("Wave already active".to_string());
    }

    // Calculate total enemies for this wave
    let wave_configs: Vec<_> = ctx.db.wave_config()
        .iter()
        .filter(|c| c.wave_number == wave_state.current_wave)
        .collect();

    if wave_configs.is_empty() {
        return Err(format!("No configuration for wave {}", wave_state.current_wave));
    }

    let total_enemies: i32 = wave_configs.iter().map(|c| c.count).sum();
    let spawn_interval = wave_configs.first()
        .map(|c| c.spawn_interval)
        .unwrap_or(1.0);

    let current_wave = wave_state.current_wave;

    // Update wave state
    wave_state.wave_active = true;
    wave_state.enemies_spawned = 0;
    wave_state.enemies_this_wave = total_enemies;
    wave_state.spawn_cooldown = spawn_interval;
    wave_state.time_until_next_wave = 0.0;
    ctx.db.wave_state().id().update(wave_state);

    // Update game status
    game_state.status = GameStatus::WaveActive;
    ctx.db.game_state().id().update(game_state);

    log::info!("Wave {} started with {} enemies", current_wave, total_enemies);
    Ok(())
}

/// Skip to a specific wave (admin/debug)
#[spacetimedb::reducer]
pub fn skip_to_wave(ctx: &ReducerContext, wave_number: i32) -> Result<(), String> {
    if wave_number < 1 {
        return Err("Wave number must be positive".to_string());
    }

    let mut wave_state = ctx.db.wave_state().id().find(0)
        .ok_or("Wave state not found")?;

    wave_state.current_wave = wave_number;
    wave_state.wave_active = false;
    wave_state.enemies_spawned = 0;
    wave_state.time_until_next_wave = 5.0; // Short countdown
    ctx.db.wave_state().id().update(wave_state);

    log::info!("Skipped to wave {}", wave_number);
    Ok(())
}
