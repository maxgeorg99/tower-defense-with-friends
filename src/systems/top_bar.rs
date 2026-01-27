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

#[derive(Component)]
pub struct WoodText;

#[derive(Component)]
pub struct MeatText;

#[derive(Component)]
pub struct EffectivenessHint;

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
                "Terrain/Resources/Wood/Wood Resource/Wood Resource.png",
                WoodText,
            );
            spawn_stat_display(
                parent,
                &asset_server,
                "Terrain/Resources/Meat/Meat Resource/Meat Resource.png",
                MeatText,
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
    mut lives_query: Query<&mut Text, (With<LivesText>, Without<GoldText>, Without<WoodText>, Without<MeatText>)>,
    mut gold_query: Query<&mut Text, (With<GoldText>, Without<LivesText>, Without<WoodText>, Without<MeatText>)>,
    mut wood_query: Query<&mut Text, (With<WoodText>, Without<LivesText>, Without<GoldText>, Without<MeatText>)>,
    mut meat_query: Query<&mut Text, (With<MeatText>, Without<LivesText>, Without<GoldText>, Without<WoodText>)>,
) {
    for mut text in lives_query.iter_mut() {
        text.0 = game_state.lives.to_string();
    }
    for mut text in gold_query.iter_mut() {
        text.0 = game_state.gold.to_string();
    }
    for mut text in wood_query.iter_mut() {
        text.0 = game_state.wood.to_string();
    }
    for mut text in meat_query.iter_mut() {
        text.0 = game_state.meat.to_string();
    }
}

/// Setup the effectiveness matrix hint in the bottom left
pub fn setup_effectiveness_hint(mut commands: Commands, asset_server: Res<AssetServer>) {
    let icon_size = 20.0;
    let cell_size = 28.0;
    let font_size = 11.0;

    // Colors for effectiveness values
    let strong_color = Color::srgb(0.3, 1.0, 0.3);   // Green for bonus
    let weak_color = Color::srgb(1.0, 0.4, 0.4);     // Red for penalty
    let neutral_color = Color::srgb(0.8, 0.8, 0.8); // Gray for neutral

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                left: Val::Px(10.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                row_gap: Val::Px(2.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            BorderRadius::all(Val::Px(6.0)),
            EffectivenessHint,
            GameUI,
        ))
        .with_children(|parent| {
            // Header row with defense type icons
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(5.0),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|row| {
                    // Empty corner cell
                    row.spawn(Node {
                        width: Val::Px(cell_size),
                        height: Val::Px(icon_size),
                        ..default()
                    });
                    // Defense icons: Armor, Agility, Mystical
                    spawn_icon(row, &asset_server, "UI Elements/UI Elements/Icons/Defense_Icon.png", icon_size);
                    spawn_icon(row, &asset_server, "UI Elements/UI Elements/Icons/Agility_Icon.png", icon_size);
                    spawn_icon(row, &asset_server, "UI Elements/UI Elements/Icons/Mystical_Icon.png", icon_size);
                });

            // Blunt row
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(2.0),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|row| {
                    spawn_icon(row, &asset_server, "UI Elements/UI Elements/Icons/Blunt_Icon.png", icon_size);
                    spawn_value_cell(row, "+25", strong_color, cell_size, font_size);
                    spawn_value_cell(row, "-15", weak_color, cell_size, font_size);
                    spawn_value_cell(row, "+10", strong_color, cell_size, font_size);
                });

            // Pierce row
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(2.0),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|row| {
                    spawn_icon(row, &asset_server, "UI Elements/UI Elements/Icons/Pierce_Icon.png", icon_size);
                    spawn_value_cell(row, "-20", weak_color, cell_size, font_size);
                    spawn_value_cell(row, "+25", strong_color, cell_size, font_size);
                    spawn_value_cell(row, "-10", weak_color, cell_size, font_size);
                });

            // Divine row
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(2.0),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|row| {
                    spawn_icon(row, &asset_server, "UI Elements/UI Elements/Icons/Divine_Icon.png", icon_size);
                    spawn_value_cell(row, "0", neutral_color, cell_size, font_size);
                    spawn_value_cell(row, "-10", weak_color, cell_size, font_size);
                    spawn_value_cell(row, "+30", strong_color, cell_size, font_size);
                });
        });
}

fn spawn_icon(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    asset_server: &AssetServer,
    path: impl Into<String>,
    size: f32,
) {
    let path_string: String = path.into();
    parent.spawn((
        Node {
            width: Val::Px(size),
            height: Val::Px(size),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        ImageNode::new(asset_server.load(path_string)),
    ));
}

fn spawn_value_cell(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    value: &str,
    color: Color,
    width: f32,
    font_size: f32,
) {
    parent
        .spawn(Node {
            width: Val::Px(width),
            height: Val::Px(width),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|cell| {
            cell.spawn((
                Text(value.to_string()),
                TextFont { font_size, ..default() },
                TextColor(color),
            ));
        });
}