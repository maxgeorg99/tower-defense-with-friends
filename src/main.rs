#[cfg(feature = "bevy-demo")]
mod auth;
#[cfg(feature = "bevy-demo")]
mod debug;
#[cfg(feature = "bevy-demo")]
mod module_bindings;

mod bevy;
mod components;
mod config;
mod constants;
mod events;
mod map;
mod resources;
mod systems;
use config::{TowersConfig, UnitsConfig, WavesConfig};

use ::bevy::prelude::*;
use bevy_spacetimedb::*;
use module_bindings::user_table::UserTableAccess;
use module_bindings::{DbConnection, RemoteModule, RemoteTables};

use auth::{
    AuthConfig, AuthState, CallbackServerState, check_auth_and_connect, load_token_from_file,
    start_login,
};
use bevy::BevyPlugin;
#[cfg(feature = "bevy-demo")]
use debug::DebugPlugin;
use events::EventPlugin;
use map::create_path_waypoints;
use resources::*;
use systems::*;

fn main() {
    let units = UnitsConfig::load()
        .expect("Failed to load units.toml")
        .units;
    let waves = WavesConfig::load()
        .expect("Failed to load waves.toml")
        .waves;
    let towers = TowersConfig::load()
        .expect("Failed to load towers.toml")
        .towers;

    let spawner = if let Some(first_wave) = waves.first() {
        EnemySpawner::from_wave_config(first_wave)
    } else {
        panic!("No waves defined in waves.toml!");
    };

    let stdb_uri =
        std::env::var("SPACETIMEDB_URI").unwrap_or_else(|_| "http://127.0.0.1:3000".to_string());
    let stdb_module = std::env::var("SPACETIMEDB_MODULE").unwrap_or_else(|_| "td-mmo".to_string());
    let stdb_token = std::env::var("SPACETIMEDB_TOKEN")
        .ok()
        .or_else(load_token_from_file);

    let initial_state = AppState::MainMenu;

    let mut app = App::new();

    app.add_plugins(BevyPlugin)
        .add_plugins(EventPlugin)
        .add_plugins(MenuPlugin)
        .add_plugins(ColorSelectPlugin)
        .add_plugins(CursorPlugin)
        .add_plugins(WaveManagerPlugin);

    // Store connection config for deferred connection
    app.insert_resource(StdbConfig {
        uri: stdb_uri.clone(),
        module: stdb_module.clone(),
        token: stdb_token.clone(),
    });

    let stdb_plugin = StdbPlugin::<DbConnection, RemoteModule>::default()
        .with_uri(&stdb_uri)
        .with_module_name(&stdb_module)
        .with_run_fn(DbConnection::run_threaded)
        .with_delayed_connect(true)
        .add_table(|tables: &RemoteTables| tables.user());

    app.add_plugins(stdb_plugin);

    app.insert_state(initial_state)
        .init_resource::<GameState>()
        .init_resource::<AuthConfig>()
        .init_resource::<AuthState>()
        .insert_resource(WavesConfig::load().unwrap())
        .insert_resource(UnitsConfig::load().unwrap())
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
        .add_systems(Startup, setup_camera)
        .add_systems(Update, (handle_login_request, check_auth_and_connect))
        .add_systems(OnEnter(AppState::ColorSelect), connect_to_spacetimedb)
        .add_systems(
            OnEnter(AppState::InGame),
            (setup_game, setup_fog_of_war, setup_online_users_ui, setup_top_bar).chain(),
        )
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
                update_top_bar
            )
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            Update,
            (
                cleanup_dead_enemies,
                check_game_over,
                update_fog_visibility,
            )
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            Update,
            (camera_zoom, camera_pan).run_if(in_state(AppState::InGame)),
        )
        // Game over screen
        .add_systems(OnEnter(AppState::GameOver), setup_game_over_screen)
        .add_systems(OnExit(AppState::GameOver), cleanup_game_over_screen);

    // Add debug plugin for hot-reloading (only in bevy-demo feature)
    #[cfg(feature = "bevy-demo")]
    app.add_plugins(DebugPlugin);

    app.run();
}

/// Handle login button press from menu
fn handle_login_request(
    mut commands: Commands,
    mut events: EventReader<LoginRequestEvent>,
    config: Res<AuthConfig>,
    mut auth_state: ResMut<AuthState>,
) {
    for _ in events.read() {
        if !auth_state.pending {
            info!("Login button pressed, starting OAuth PKCE flow...");
            let callback_state = start_login(&config);
            commands.insert_resource(CallbackServerState(callback_state));
            auth_state.pending = true;
        }
    }
}
