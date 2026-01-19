#[cfg(feature = "bevy-demo")]
mod auth;
#[cfg(feature = "bevy-demo")]
mod module_bindings;

mod components;
mod config;
mod constants;
mod map;
mod resources;
mod systems;

use config::{TowersConfig, UnitsConfig, WavesConfig};

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_spacetimedb::*;
use module_bindings::user_table::UserTableAccess;
use module_bindings::{DbConnection, RemoteModule, RemoteTables};

#[cfg(feature = "bevy-demo")]
use notify::{RecursiveMode, Watcher};
#[cfg(feature = "bevy-demo")]
use std::path::Path;
#[cfg(feature = "bevy-demo")]
use std::sync::mpsc::channel;
#[cfg(feature = "bevy-demo")]
use std::sync::{Arc, Mutex};
use auth::{
    check_auth_and_connect, cleanup_login_screen, handle_anonymous_button,
    handle_login_button, load_token_from_file, setup_login_screen, update_login_button_colors,
    AuthConfig, AuthState,
};
use map::create_path_waypoints;
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

    // SpacetimeDB connection config from environment
    let stdb_uri = std::env::var("SPACETIMEDB_URI")
        .unwrap_or_else(|_| "http://127.0.0.1:3000".to_string());
    let stdb_module = std::env::var("SPACETIMEDB_MODULE")
        .unwrap_or_else(|_| "td-mmo".to_string());
    // Try to load token from env var first, then from file
    let stdb_token = std::env::var("SPACETIMEDB_TOKEN")
        .ok()
        .or_else(load_token_from_file);
    let require_auth = std::env::var("SPACETIMEDB_REQUIRE_AUTH")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false);

    // Determine initial state based on auth
    let has_token = stdb_token.is_some();
    let initial_state = if require_auth && !has_token {
        AppState::Login
    } else {
        AppState::InGame
    };

    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Tower Defense MMO".to_string(),
            resolution: (1024, 768).into(),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(TiledPlugin::default());

    // Store connection config for deferred connection
    app.insert_resource(StdbConfig {
        uri: stdb_uri.clone(),
        module: stdb_module.clone(),
        token: stdb_token.clone(),
    });

    // Always add the SpacetimeDB plugin with delayed connect
    // Connection will be established via connect_with_token() after auth or on game start
    info!(
        "Setting up SpacetimeDB plugin with delayed connect (uri: {}, module: {})",
        stdb_uri, stdb_module
    );

    let stdb_plugin = StdbPlugin::<DbConnection, RemoteModule>::default()
        .with_uri(&stdb_uri)
        .with_module_name(&stdb_module)
        .with_run_fn(DbConnection::run_threaded)
        .with_delayed_connect(true)
        .add_table(|tables: &RemoteTables| tables.user());

    app.add_plugins(stdb_plugin);

    app
    // State management
    .insert_state(initial_state)
    .init_resource::<GameState>()
    // Auth resources
    .init_resource::<AuthConfig>()
    .init_resource::<AuthState>()
    // Game resources
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
    // Camera setup runs on startup (needed for all states)
    .add_systems(Startup, setup_camera)
    // Login screen systems
    .add_systems(OnEnter(AppState::Login), setup_login_screen)
    .add_systems(
        Update,
        (
            handle_login_button,
            handle_anonymous_button,
            update_login_button_colors,
            check_auth_and_connect,
        )
            .run_if(in_state(AppState::Login)),
    )
    .add_systems(OnExit(AppState::Login), cleanup_login_screen)
    // Game setup when entering InGame state
    .add_systems(OnEnter(AppState::InGame), (connect_to_spacetimedb, setup_game, setup_fog_of_war, setup_online_users_ui).chain())
    // SpacetimeDB connection handling systems (always running)
    .add_systems(
        Update,
        (
            on_connected,
            on_disconnected,
            on_connection_error,
            on_user_inserted,
            on_user_updated,
            on_user_deleted,
            update_online_users_ui,
        ),
    )
    // Game systems (only in InGame state)
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
        )
            .run_if(in_state(AppState::InGame)),
    )
    .add_systems(Update, (camera_zoom, camera_pan).run_if(in_state(AppState::InGame)))
    // Game over screen
    .add_systems(OnEnter(AppState::GameOver), setup_game_over_screen)
    .add_systems(OnExit(AppState::GameOver), cleanup_game_over_screen);

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
