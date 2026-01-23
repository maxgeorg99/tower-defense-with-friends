use bevy::prelude::*;
use bevy::window::{CursorIcon, CustomCursor, CustomCursorImage};

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_custom_cursor);
    }
}

fn setup_custom_cursor(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    windows: Query<Entity, With<Window>>,
) {
    let cursor_image: Handle<Image> = asset_server.load("UI Elements/UI Elements/Cursors/Cursor_02.png");

    for window_entity in windows.iter() {
        commands.entity(window_entity).insert(
            CursorIcon::Custom(CustomCursor::Image(CustomCursorImage {
                handle: cursor_image.clone(),
                hotspot: (0, 0), //TODO can we make it more on the finger tip?
                ..default()
            }))
        );
    }
}