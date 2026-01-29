//! Simple tilemap renderer for WASM builds
//! Uses the tiled crate to parse TMX files and renders with Bevy sprites

use bevy::prelude::*;

/// Component to mark tilemap entities
#[derive(Component)]
pub struct TilemapTile;

/// Resource to track tilemap loading state
#[derive(Resource, Default)]
pub struct TilemapState {
    pub loaded: bool,
}

/// Plugin for WASM tilemap loading
pub struct WasmTilemapPlugin;

impl Plugin for WasmTilemapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TilemapState>()
            .add_systems(Startup, setup_tilemap);
    }
}

/// Tilemap configuration
const MAP_WIDTH: usize = 30;
const MAP_HEIGHT: usize = 20;
const TILE_SIZE: f32 = 16.0;
const MAP_SCALE: f32 = 2.0;
const TILESET_COLUMNS: usize = 36;

/// The tile data from map.tmx (CSV layer data)
const TILE_DATA: &[u32] = &[
    1,2,3,4,5,6,7,8,9,10,11,2,3,4,5,6,7,8,9,2,3,4,5,6,7,8,9,10,11,12,
    73,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,48,
    109,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,84,
    145,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,120,
    181,257,257,257,257,1806,1806,1806,1806,1806,1806,1806,257,257,257,257,257,1795,1795,1795,1795,1795,1795,1795,257,257,257,257,257,156,
    217,257,257,257,257,1806,257,257,257,257,257,1806,257,257,257,257,257,1795,257,257,257,257,257,1795,257,257,257,257,257,192,
    253,257,257,257,257,1806,257,257,257,257,257,1806,257,257,257,257,257,1795,257,257,257,257,257,1795,257,257,257,257,257,228,
    289,257,257,257,257,1806,257,257,257,257,257,1806,257,257,257,257,257,1795,257,257,257,257,257,1795,257,257,257,257,257,264,
    325,257,257,257,257,1806,257,257,257,257,257,1806,257,257,257,257,257,1795,257,257,257,257,257,1795,257,257,257,257,257,300,
    361,257,257,257,257,1806,257,257,257,257,257,1806,257,257,257,257,257,1795,257,257,257,257,257,1795,257,257,257,257,257,48,
    1806,1806,1806,1806,1806,1806,257,257,257,257,257,1806,257,257,257,257,257,1795,257,257,257,257,257,1795,257,257,257,257,257,84,
    109,257,257,257,257,257,257,257,257,257,257,1806,257,257,257,257,257,1795,257,257,257,257,257,1795,1795,1795,1795,257,257,120,
    145,257,257,257,257,257,257,257,257,257,257,1806,257,257,257,257,257,1795,257,257,257,257,257,257,257,257,257,257,257,156,
    181,257,257,257,257,257,257,257,257,257,257,1806,257,257,257,257,257,1795,257,257,257,257,257,257,257,257,257,257,257,192,
    217,257,257,257,257,257,257,257,257,257,257,1806,257,257,257,257,257,1795,257,257,257,257,257,257,257,257,257,257,257,228,
    253,257,257,257,257,257,257,257,257,257,257,1806,257,257,257,257,257,1795,257,257,257,257,257,257,257,257,257,257,257,264,
    289,257,257,257,257,257,257,257,257,257,257,1806,1806,1806,1806,1795,1795,1795,257,257,257,257,257,257,257,257,257,257,257,300,
    325,257,257,257,257,257,257,257,257,257,257,258,258,258,258,258,258,258,257,257,257,257,257,257,257,257,257,257,257,336,
    361,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,257,372,
    397,398,399,400,401,402,403,404,405,406,398,399,400,401,402,403,404,405,406,407,399,400,401,402,403,404,405,406,407,408,
];

fn setup_tilemap(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut tilemap_state: ResMut<TilemapState>,
) {
    // Load the tileset image
    let tileset_handle: Handle<Image> = asset_server.load("Terrain/Tilemap_color1.png");
    let tileset2_handle: Handle<Image> = asset_server.load("Terrain/Tilemap_color2.png");

    // Create texture atlas layout for tileset 1 (36 columns, 24 rows = 864 tiles)
    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(TILE_SIZE as u32, TILE_SIZE as u32),
        TILESET_COLUMNS as u32,
        24, // rows
        None,
        None,
    );
    let atlas_layout = texture_atlases.add(layout);

    // Create texture atlas layout for tileset 2
    let layout2 = TextureAtlasLayout::from_grid(
        UVec2::new(TILE_SIZE as u32, TILE_SIZE as u32),
        TILESET_COLUMNS as u32,
        24,
        None,
        None,
    );
    let atlas_layout2 = texture_atlases.add(layout2);

    // Map offset to match native rendering
    let map_offset_x = -480.0;
    let map_offset_y = -320.0;

    // Spawn tiles
    for row in 0..MAP_HEIGHT {
        for col in 0..MAP_WIDTH {
            let tile_index = row * MAP_WIDTH + col;
            let tile_id = TILE_DATA[tile_index];

            if tile_id == 0 {
                continue; // Skip empty tiles
            }

            // Calculate world position (Bevy Y is up, Tiled Y is down)
            let world_x = map_offset_x + (col as f32 * TILE_SIZE * MAP_SCALE) + (TILE_SIZE * MAP_SCALE / 2.0);
            let world_y = map_offset_y + ((MAP_HEIGHT - 1 - row) as f32 * TILE_SIZE * MAP_SCALE) + (TILE_SIZE * MAP_SCALE / 2.0);

            // Determine which tileset and calculate atlas index
            let (tileset_handle, atlas_layout, atlas_index) = if tile_id < 865 {
                // Tileset 1 (firstgid=1)
                (tileset_handle.clone(), atlas_layout.clone(), (tile_id - 1) as usize)
            } else if tile_id < 1729 {
                // Tileset 2 (firstgid=865)
                (tileset2_handle.clone(), atlas_layout2.clone(), (tile_id - 865) as usize)
            } else {
                // Shadow tileset - skip for now or use tileset 1 as fallback
                continue;
            };

            commands.spawn((
                Sprite {
                    image: tileset_handle,
                    texture_atlas: Some(TextureAtlas {
                        layout: atlas_layout,
                        index: atlas_index,
                    }),
                    ..default()
                },
                Transform::from_xyz(world_x, world_y, 0.0)
                    .with_scale(Vec3::splat(MAP_SCALE)),
                TilemapTile,
            ));
        }
    }

    tilemap_state.loaded = true;
    info!("WASM tilemap loaded: {}x{} tiles", MAP_WIDTH, MAP_HEIGHT);
}
