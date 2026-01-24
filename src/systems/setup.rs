use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_spacetimedb::*;
use crate::components::{Castle, FogTile, GameUI};
use crate::constants::{CASTLE_SIZE, MAP_HEIGHT, MAP_SCALE, MAP_WIDTH, SCALED_TILE_SIZE};
use crate::map::tile_to_world;
use crate::module_bindings::{DbConnection, RemoteModule};
use crate::resources::{FogOfWar, StdbConfig};

/// Setup camera (runs on startup, needed for all states)
pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Connect to SpacetimeDB using the delayed connection feature
/// This runs when entering InGame state and establishes the connection with optional token
pub fn connect_to_spacetimedb(world: &mut World) {
    let token = world
        .get_resource::<StdbConfig>()
        .and_then(|config| config.token.clone());

    if token.is_some() {
        info!("Connecting to SpacetimeDB with auth token...");
        connect_with_token::<DbConnection, RemoteModule>(world, token);
    } else {
        info!("Connecting to SpacetimeDB anonymously...");
    }

}

/// Setup game elements (runs when entering InGame state)
pub fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load and spawn the tilemap
    commands.spawn((
        TiledMap(asset_server.load("map.tmx")),
        Transform::from_xyz(-480.0, -320.0, 0.0).with_scale(Vec3::splat(MAP_SCALE)),
    ));

    // Spawn castle at the end of the path (right side)
    let castle_scale = (SCALED_TILE_SIZE * 4.0) / CASTLE_SIZE.x.max(CASTLE_SIZE.y); // 4 tiles tall
    commands.spawn((
        Sprite::from_image(asset_server.load("Decorations/Buildings/Blue Buildings/Castle.png")),
        Transform::from_xyz(400.0, 0.0, 1.0).with_scale(Vec3::splat(castle_scale)),
        Castle,
    ));
}

/// Legacy alias for setup_game (deprecated)
pub fn setup(commands: Commands, asset_server: Res<AssetServer>) {
    setup_game(commands, asset_server);
}

pub fn setup_fog_of_war(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut fog: ResMut<FogOfWar>,
) {
    // Castle is at world position (400.0, 0.0), which is approximately tile (27, 10)
    // Explore a larger area (radius 8) around the castle
    const CASTLE_TILE_X: i32 = 27;
    const CASTLE_TILE_Y: i32 = 10;
    const INITIAL_EXPLORE_RADIUS: i32 = 8;

    fog.explore_rect(CASTLE_TILE_X, CASTLE_TILE_Y, INITIAL_EXPLORE_RADIUS);

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
