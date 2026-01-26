use bevy::prelude::*;
use bevy::window::{CursorIcon, CustomCursor, CustomCursorImage};
use crate::map::world_to_tile;
use crate::resources::{AppState, BlockedTiles};

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorHandles>()
            .add_systems(Startup, setup_custom_cursor)
            .add_systems(Update, update_cursor_for_tile.run_if(in_state(AppState::InGame)));
    }
}

#[derive(Resource, Default)]
struct CursorHandles {
    normal: Handle<Image>,
    blocked: Handle<Image>,
}

fn setup_custom_cursor(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut cursor_handles: ResMut<CursorHandles>,
    windows: Query<Entity, With<Window>>,
) {
    cursor_handles.normal = asset_server.load("UI Elements/UI Elements/Cursors/Cursor_02.png");
    cursor_handles.blocked = asset_server.load("UI Elements/UI Elements/Cursors/Cursor_03.png");

    for window_entity in windows.iter() {
        commands.entity(window_entity).insert(
            CursorIcon::Custom(CustomCursor::Image(CustomCursorImage {
                handle: cursor_handles.normal.clone(),
                hotspot: (0, 0),
                ..default()
            }))
        );
    }
}

fn update_cursor_for_tile(
    mut commands: Commands,
    cursor_handles: Res<CursorHandles>,
    windows: Query<(Entity, &Window)>,
    camera: Query<(&Camera, &GlobalTransform)>,
    blocked_tiles: Res<BlockedTiles>,
) {
    let Ok((camera, camera_transform)) = camera.single() else { return };

    for (window_entity, window) in windows.iter() {
        let Some(cursor_pos) = window.cursor_position() else { continue };
        let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { continue };

        let (tile_x, tile_y) = world_to_tile(world_pos);

        // Use blocked cursor on road tiles (but not castle)
        let cursor_handle = if blocked_tiles.is_road(tile_x, tile_y) {
            cursor_handles.blocked.clone()
        } else {
            cursor_handles.normal.clone()
        };

        commands.entity(window_entity).insert(
            CursorIcon::Custom(CustomCursor::Image(CustomCursorImage {
                handle: cursor_handle,
                hotspot: (0, 0),
                ..default()
            }))
        );
    }
}