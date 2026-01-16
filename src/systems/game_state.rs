use bevy::prelude::*;

use crate::components::GameOverScreen;
use crate::resources::{AppState, GameState};

#[cfg(feature = "bevy-demo")]
use crate::config::{TowersConfig, UnitsConfig, WavesConfig};
#[cfg(feature = "bevy-demo")]
use crate::resources::{FileWatcher, TowerConfigs, WaveConfigs};

pub fn check_game_over(game_state: Res<GameState>, mut next_state: ResMut<NextState<AppState>>) {
    if game_state.lives <= 0 {
        next_state.set(AppState::GameOver);
    }
}

pub fn setup_game_over_screen(mut commands: Commands) {
    commands
        .spawn((
            Node {
                // Stretch over whole screen
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::linear_rgba(0.0, 0.0, 0.0, 0.7)),
            GameOverScreen,
        ))
        .with_children(|parent| {
            // GAME OVER text
            parent.spawn((
                Text::new("GAME OVER"),
                TextFont {
                    font_size: 80.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

pub fn cleanup_game_over_screen(mut commands: Commands, query: Query<Entity, With<GameOverScreen>>) {
    for e in &query {
        commands.entity(e).despawn();
    }
}

#[cfg(feature = "bevy-demo")]
pub fn watch_config_files(
    file_watcher: Res<FileWatcher>,
    mut wave_configs: ResMut<WaveConfigs>,
    mut tower_configs: ResMut<TowerConfigs>,
) {
    // Check for file change events (non-blocking)
    let receiver = file_watcher.receiver.lock().unwrap();
    while let Ok(Ok(event)) = receiver.try_recv() {
        use notify::EventKind;

        match event.kind {
            EventKind::Modify(_) | EventKind::Create(_) => {
                // Check which file was modified
                for path in event.paths {
                    if path.ends_with("units.toml") {
                        match UnitsConfig::load() {
                            Ok(config) => {
                                wave_configs.units = config.units;
                                info!(
                                    "Hot-reloaded units.toml - {} units loaded",
                                    wave_configs.units.len()
                                );
                            }
                            Err(e) => error!("Failed to reload units.toml: {}", e),
                        }
                    } else if path.ends_with("waves.toml") {
                        match WavesConfig::load() {
                            Ok(config) => {
                                wave_configs.waves = config.waves;
                                info!(
                                    "Hot-reloaded waves.toml - {} waves loaded",
                                    wave_configs.waves.len()
                                );
                            }
                            Err(e) => error!("Failed to reload waves.toml: {}", e),
                        }
                    } else if path.ends_with("towers.toml") {
                        match TowersConfig::load() {
                            Ok(config) => {
                                tower_configs.towers = config.towers;
                                info!(
                                    "Hot-reloaded towers.toml - {} towers loaded",
                                    tower_configs.towers.len()
                                );
                            }
                            Err(e) => error!("Failed to reload towers.toml: {}", e),
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
