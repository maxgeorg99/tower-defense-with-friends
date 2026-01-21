use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

/// Plugin that configures native Bevy plugins with project-specific settings
pub struct BevyPlugin;

impl Plugin for BevyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Tower Defense MMO".to_string(),
                resolution: (1024u32, 768u32).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(TiledPlugin::default());
    }
}
