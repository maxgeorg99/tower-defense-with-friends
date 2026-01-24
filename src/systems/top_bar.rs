use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::prelude::*;
use crate::components::GameUI;
use crate::resources::GameState;

#[derive(Component)]
pub struct TopBar;

#[derive(Component)]
pub struct LivesText;

#[derive(Component)]
pub struct GoldText;

// Setup top bar
pub fn setup_top_bar(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(20.0),
                ..default()
            },
            TopBar,
            GameUI,
        ))
        .with_children(|parent| {
            spawn_stat_display(
                parent,
                &asset_server,
                "UI Elements/UI Elements/Icons/Gold_Icon.png",
                GoldText,
            );
            spawn_stat_display(
                parent,
                &asset_server,
                "UI Elements/UI Elements/Icons/Defense_Icon.png",
                LivesText,
            );
        });
}

// Helper function to spawn a stat display (icon + text)
fn spawn_stat_display(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    asset_server: &AssetServer,
    icon_path: impl Into<String>,
    text_marker: impl Component,
) {
    let icon_path = icon_path.into();
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|stat_parent| {
            // Icon
            stat_parent.spawn((
                Node {
                    width: Val::Px(32.0),
                    height: Val::Px(32.0),
                    ..default()
                },
                ImageNode::new(asset_server.load(icon_path)),
            ));
            // Text
            stat_parent.spawn((
                Text(String::from("0")),
                TextFont { font_size: 24.0, ..default() },
                TextColor(Color::WHITE),
                text_marker,
            ));
        });
}

// Update function
pub fn update_top_bar(
    game_state: Res<GameState>,
    mut lives_query: Query<&mut Text, (With<LivesText>, Without<GoldText>)>,
    mut gold_query: Query<&mut Text, (With<GoldText>, Without<LivesText>)>,
) {
    for mut text in lives_query.iter_mut() {
        text.0 = game_state.lives.to_string();
    }
    for mut text in gold_query.iter_mut() {
        text.0 = game_state.gold.to_string();
    }
}