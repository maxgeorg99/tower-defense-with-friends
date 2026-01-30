use bevy::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use bevy_ecs_tiled::prelude::*;

/// Plugin that configures native Bevy plugins with project-specific settings
pub struct BevyPlugin;

impl Plugin for BevyPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(not(target_arch = "wasm32"))]
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Tower Defense MMO".to_string(),
                resolution: (1024u32, 768u32).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(TiledPlugin::default());

        #[cfg(target_arch = "wasm32")]
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Tower Defense MMO".to_string(),
                        resolution: (1024u32, 768u32).into(),
                        canvas: Some("#bevy".to_string()),
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: bevy::asset::AssetMetaCheck::Never,
                    ..default()
                }),
        );
    }
}
