#[cfg(all(feature = "bevy-demo", not(target_arch = "wasm32")))]
mod auth;
#[cfg(all(feature = "bevy-demo", not(target_arch = "wasm32")))]
mod debug;
#[cfg(any(feature = "bevy-demo", feature = "bevy-wasm"))]
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

#[cfg(not(target_arch = "wasm32"))]
use auth::{
    AuthConfig, AuthState, CallbackServerState, check_auth_and_connect, load_token_from_file,
    start_login,
};
use bevy::BevyPlugin;
#[cfg(all(feature = "bevy-demo", not(target_arch = "wasm32")))]
use debug::DebugPlugin;
use events::EventPlugin;
use map::{create_path_waypoints, create_blocked_tiles};
use resources::*;
use systems::*;

// WASM stubs for auth types
#[cfg(target_arch = "wasm32")]
mod wasm_auth {
    use ::bevy::prelude::*;

    #[derive(Resource, Clone, Default)]
    pub struct AuthConfig {
        pub client_id: String,
        pub callback_port: u16,
    }

    #[derive(Resource, Default)]
    pub struct AuthState {
        pub access_token: Option<String>,
        pub id_token: Option<String>,
        pub refresh_token: Option<String>,
        pub pending: bool,
    }

    pub fn check_auth_and_connect() {}
}

#[cfg(target_arch = "wasm32")]
use wasm_auth::{AuthConfig, AuthState};

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

    #[cfg(not(target_arch = "wasm32"))]
    let stdb_uri =
        std::env::var("SPACETIMEDB_URI").unwrap_or_else(|_| "http://127.0.0.1:3000".to_string());
    #[cfg(target_arch = "wasm32")]
    let stdb_uri = "https://maincloud.spacetimedb.com".to_string();

    #[cfg(not(target_arch = "wasm32"))]
    let stdb_module = std::env::var("SPACETIMEDB_MODULE").unwrap_or_else(|_| "td-mmo".to_string());
    #[cfg(target_arch = "wasm32")]
    let stdb_module = "td-mmo".to_string();

    #[cfg(not(target_arch = "wasm32"))]
    let stdb_token = std::env::var("SPACETIMEDB_TOKEN")
        .ok()
        .or_else(load_token_from_file);
    #[cfg(target_arch = "wasm32")]
    let stdb_token: Option<String> = None;

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

    #[cfg(not(target_arch = "wasm32"))]
    let stdb_plugin = StdbPlugin::<DbConnection, RemoteModule>::default()
        .with_uri(&stdb_uri)
        .with_module_name(&stdb_module)
        .with_run_fn(DbConnection::run_threaded)
        .with_delayed_connect(true)
        .add_table(|tables: &RemoteTables| tables.user());

    #[cfg(target_arch = "wasm32")]
    let stdb_plugin = StdbPlugin::<DbConnection, RemoteModule>::default()
        .with_uri(&stdb_uri)
        .with_module_name(&stdb_module)
        .with_run_fn(DbConnection::run)
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
        .init_resource::<RecruitMenuState>()
        .init_resource::<HouseMenuState>()
        .init_resource::<TowerUpgradeMenuState>()
        .insert_resource({
            let (blocked, castle) = create_blocked_tiles();
            BlockedTiles { tiles: blocked, castle_tiles: castle }
        })
        .add_systems(Startup, setup_camera);

    // Native-only auth systems
    #[cfg(not(target_arch = "wasm32"))]
    app.add_systems(Update, (handle_login_request, check_auth_and_connect));

    app.add_systems(OnEnter(AppState::ColorSelect), connect_to_spacetimedb)
        .add_systems(
            OnEnter(AppState::InGame),
            (setup_game, setup_fog_of_war, setup_online_users_ui, setup_top_bar, setup_effectiveness_hint, setup_resource_gathering).chain(),
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
                update_holy_tower_effects,
                move_projectiles,
                handle_projectile_hits,
                update_health_bars,
                update_top_bar,
                show_recruit_menu,
                hide_recruit_menu,
                handle_recruit_selection,
                show_tower_upgrade_menu,
                hide_tower_upgrade_menu,
                handle_tower_upgrade,
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
            (
                spawn_workers,
                worker_find_resource,
                worker_movement,
                worker_arrive_check,
                worker_harvest,
                worker_sprite_update,
                animate_worker_sprites,
                show_house_menu,
                hide_house_menu,
                handle_build_worker,
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

    // Add debug plugin for hot-reloading (only in bevy-demo feature, native only)
    #[cfg(all(feature = "bevy-demo", not(target_arch = "wasm32")))]
    app.add_plugins(DebugPlugin);

    app.run();
}

/// Handle login button press from menu (native only)
#[cfg(not(target_arch = "wasm32"))]
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
