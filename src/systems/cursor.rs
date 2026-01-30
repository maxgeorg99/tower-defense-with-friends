use bevy::prelude::*;
use bevy::window::{CursorIcon, CustomCursor, CustomCursorImage};
use crate::map::world_to_tile;
use crate::resources::{AppState, BlockedTiles};

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorHandles>()
            .init_resource::<CurrentCursorState>()
            .add_systems(Startup, setup_custom_cursor)
            .add_systems(Update, update_cursor_for_tile.run_if(in_state(AppState::InGame)));
    }
}

#[derive(Resource, Default)]
struct CursorHandles {
    normal: Handle<Image>,
    blocked: Handle<Image>,
}

#[derive(Resource, Default, PartialEq)]
enum CurrentCursorState {
    #[default]
    Normal,
    Blocked,
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
                hotspot: (24, 20),
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
    mut current_state: ResMut<CurrentCursorState>,
    interaction_query: Query<&Interaction>,
) {
    let Ok((camera, camera_transform)) = camera.single() else { return };

    // Check if cursor is over any UI element
    let is_over_ui = interaction_query.iter().any(|interaction| {
        matches!(interaction, Interaction::Hovered | Interaction::Pressed)
    });

    // If over UI, always use normal cursor
    if is_over_ui {
        if *current_state != CurrentCursorState::Normal {
            for (window_entity, _) in windows.iter() {
                commands.entity(window_entity).insert(
                    CursorIcon::Custom(CustomCursor::Image(CustomCursorImage {
                        handle: cursor_handles.normal.clone(),
                        hotspot: (24, 20),
                        ..default()
                    }))
                );
            }
            *current_state = CurrentCursorState::Normal;
        }
        return;
    }

    for (window_entity, window) in windows.iter() {
        let Some(cursor_pos) = window.cursor_position() else { continue };
        let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { continue };

        let (tile_x, tile_y) = world_to_tile(world_pos);

        // Determine desired cursor state
        let desired_state = if blocked_tiles.is_road(tile_x, tile_y) {
            CurrentCursorState::Blocked
        } else {
            CurrentCursorState::Normal
        };

        // Only update if state changed
        if *current_state != desired_state {
            let (cursor_handle, hotspot) = match desired_state {
                CurrentCursorState::Blocked => (cursor_handles.blocked.clone(), (32, 29)),
                CurrentCursorState::Normal => (cursor_handles.normal.clone(), (24, 20)),
            };

            commands.entity(window_entity).insert(
                CursorIcon::Custom(CustomCursor::Image(CustomCursorImage {
                    handle: cursor_handle,
                    hotspot,
                    ..default()
                }))
            );

            *current_state = desired_state;
        }
    }
}