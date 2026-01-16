mod components;
mod config;
mod constants;
mod helpers;
mod resources;
mod systems;

use config::{TowersConfig, UnitsConfig, WavesConfig};

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

#[cfg(feature = "bevy-demo")]
use notify::{RecursiveMode, Watcher};
#[cfg(feature = "bevy-demo")]
use std::path::Path;
#[cfg(feature = "bevy-demo")]
use std::sync::mpsc::channel;
#[cfg(feature = "bevy-demo")]
use std::sync::{Arc, Mutex};

use helpers::create_path_waypoints;
use resources::*;
use systems::*;

fn main() {
    // Connect to devtools for hot-reloading
    #[cfg(feature = "bevy-demo")]
    {
        std::thread::spawn(|| {
            dioxus_devtools::connect_subsecond();
        });
    }

    // Load initial configs
    let units = UnitsConfig::load()
        .expect("Failed to load units.toml")
        .units;
    let waves = WavesConfig::load()
        .expect("Failed to load waves.toml")
        .waves;
    let towers = TowersConfig::load()
        .expect("Failed to load towers.toml")
        .towers;

    // Initialize spawner with first wave config
    let spawner = if let Some(first_wave) = waves.first() {
        EnemySpawner::from_wave_config(first_wave)
    } else {
        panic!("No waves defined in waves.toml!");
    };

    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Tower Defense Game".to_string(),
            resolution: (1024, 768).into(),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(TiledPlugin::default())
    .init_state::<AppState>()
    .init_resource::<GameState>()
    .insert_resource(spawner)
    .insert_resource(PathWaypoints {
        points: create_path_waypoints(),
    })
    .insert_resource(WaveConfigs { units, waves })
    .insert_resource(TowerConfigs { towers })
    .insert_resource(TowerWheelState {
        active: false,
        position: Vec2::ZERO,
    })
    .insert_resource(FogOfWar::new())
    .add_systems(Startup, (setup, setup_fog_of_war).chain())
    .add_systems(
        Update,
        (
            spawn_enemies,
            move_enemies,
            animate_sprites,
            show_tower_wheel_menu,
            hide_tower_wheel_menu,
            handle_tower_selection,
            tower_shooting,
            move_projectiles,
            handle_projectile_hits,
            update_health_bars,
        )
            .run_if(in_state(AppState::InGame)),
    )
    .add_systems(
        Update,
        (
            cleanup_dead_enemies,
            update_ui,
            check_game_over,
            update_fog_visibility,
        ),
    )
    .add_systems(Update, camera_zoom)
    .add_systems(OnEnter(AppState::GameOver), setup_game_over_screen)
    .add_systems(OnExit(AppState::GameOver), cleanup_game_over_screen)
    .add_systems(Update, camera_pan);

    // Setup file watching for hot-reloading
    #[cfg(feature = "bevy-demo")]
    {
        let (tx, rx) = channel();
        let mut watcher =
            notify::recommended_watcher(tx).expect("Failed to create file watcher");

        watcher
            .watch(Path::new("units.toml"), RecursiveMode::NonRecursive)
            .expect("Failed to watch units.toml");
        watcher
            .watch(Path::new("waves.toml"), RecursiveMode::NonRecursive)
            .expect("Failed to watch waves.toml");
        watcher
            .watch(Path::new("towers.toml"), RecursiveMode::NonRecursive)
            .expect("Failed to watch towers.toml");

        app.insert_resource(FileWatcher {
            receiver: Arc::new(Mutex::new(rx)),
            _watcher: Arc::new(Mutex::new(Box::new(watcher))),
        });
        app.add_systems(Update, watch_config_files);
    }

    app.run();
}
