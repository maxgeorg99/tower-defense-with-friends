use bevy::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use bevy_ecs_tiled::prelude::*;
use bevy_spacetimedb::*;
use spacetimedb_sdk::Table;
use crate::components::{Castle, FogTile, GameUI};
use crate::constants::{CASTLE_SIZE, MAP_HEIGHT, MAP_SCALE, MAP_WIDTH, SCALED_TILE_SIZE};
use crate::map::tile_to_world;
use crate::module_bindings::{Color as PlayerColor, DbConnection, MyUserTableAccess, RemoteModule};
use crate::resources::{BlockedTiles, FogOfWar, StdbConfig};

/// Type alias for cleaner SpacetimeDB resource access
pub type SpacetimeDB<'a> = Res<'a, StdbConnection<DbConnection>>;

/// Get the color directory name for asset paths
fn get_color_dir(color: PlayerColor) -> &'static str {
    match color {
        PlayerColor::Blue => "Blue",
        PlayerColor::Yellow => "Yellow",
        PlayerColor::Purple => "Purple",
        PlayerColor::Black => "Black",
    }
}

/// Get the current player's color from SpacetimeDB, defaulting to Blue
fn get_player_color(stdb: &Option<SpacetimeDB>) -> PlayerColor {
    stdb.as_ref()
        .and_then(|db| db.db().my_user().iter().next())
        .map(|user| user.color)
        .unwrap_or(PlayerColor::Blue)
}

/// Setup camera (runs on startup, needed for all states)
pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Connect to SpacetimeDB using the delayed connection feature
/// This runs when entering ColorSelect state and establishes the connection with optional token
pub fn connect_to_spacetimedb(world: &mut World) {
    let token = world
        .get_resource::<StdbConfig>()
        .and_then(|config| config.token.clone());

    if token.is_some() {
        info!("Connecting to SpacetimeDB with auth token...");
    } else {
        info!("Connecting to SpacetimeDB anonymously...");
    }

    // Always call connect_with_token - it works with None for anonymous connection
    connect_with_token::<DbConnection, RemoteModule>(world, token);
}

/// Setup game elements (runs when entering InGame state)
pub fn setup_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    stdb: Option<SpacetimeDB>,
) {
    let color = get_player_color(&stdb);
    let color_dir = get_color_dir(color);

    // Load and spawn the tilemap (native only - bevy_ecs_tiled doesn't compile to WASM)
    #[cfg(not(target_arch = "wasm32"))]
    commands.spawn((
        TiledMap(asset_server.load("map.tmx")),
        Transform::from_xyz(-480.0, -320.0, 0.0).with_scale(Vec3::splat(MAP_SCALE)),
    ));

    // Spawn castle at the end of the path (right side) with dynamic player color
    let castle_scale = (SCALED_TILE_SIZE * 4.0) / CASTLE_SIZE.x.max(CASTLE_SIZE.y); // 4 tiles tall
    let castle_path = format!("Decorations/Buildings/{} Buildings/Castle.png", color_dir);
    commands.spawn((
        Sprite::from_image(asset_server.load(&castle_path)),
        Transform::from_xyz(400.0, 0.0, 1.0).with_scale(Vec3::splat(castle_scale)),
        Castle,
    ));
}

pub fn setup_fog_of_war(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut fog: ResMut<FogOfWar>,
    blocked_tiles: Res<BlockedTiles>,
) {
    // Castle is at world position (400.0, 0.0), which is approximately tile (27, 10)
    // Explore a larger area (radius 8) around the castle
    const CASTLE_TILE_X: i32 = 27;
    const CASTLE_TILE_Y: i32 = 10;
    const INITIAL_EXPLORE_RADIUS: i32 = 8;

    fog.explore_rect(CASTLE_TILE_X, CASTLE_TILE_Y, INITIAL_EXPLORE_RADIUS);

    // Always reveal the road/path tiles so enemies are visible
    for (tile_x, tile_y) in blocked_tiles.tiles.iter() {
        fog.set_explored(*tile_x, *tile_y, true);
    }

    // Load the shadow texture for fog
    let fog_texture = asset_server.load("Terrain/Shadow.png");

    // Shadow.png is 16x16, scale it to match tile size (32x32)
    let fog_scale = SCALED_TILE_SIZE / 16.0;

    // Spawn fog tiles for all map tiles using the shadow texture
    for tile_y in 0..MAP_HEIGHT {
        for tile_x in 0..MAP_WIDTH {
            let world_pos = tile_to_world(tile_x, tile_y);
            let is_explored = fog.is_explored(tile_x, tile_y);

            commands.spawn((
                Sprite::from_image(fog_texture.clone()),
                Transform::from_xyz(world_pos.x, world_pos.y, 5.0).with_scale(Vec3::splat(fog_scale)),
                Visibility::from(if is_explored {
                    Visibility::Hidden
                } else {
                    Visibility::Visible
                }),
                FogTile { tile_x, tile_y },
            ));
        }
    }
}
