// menu.rs - Medieval/fantasy themed menu with 9-slice buttons
// Updated for Bevy 0.15+

use bevy::prelude::*;
use crate::resources::AppState;
// ============================================================================
// MENU COMPONENTS
// ============================================================================

#[derive(Component)]
struct MenuUI;

#[derive(Component)]
struct PlayButton;

#[derive(Component)]
struct SettingsButton;

#[derive(Component)]
struct QuitButton;

// ============================================================================
// MENU PLUGIN
// ============================================================================

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::MainMenu), setup_main_menu)
            .add_systems(
                Update,
                (
                    handle_play_button,
                    handle_settings_button,
                    handle_quit_button,
                    update_button_images,
                )
                    .run_if(in_state(AppState::MainMenu)),
            )
            .add_systems(OnExit(AppState::MainMenu), cleanup_menu);
    }
}

// ============================================================================
// MENU SETUP SYSTEM
// ============================================================================

fn setup_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Root container with background
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(40.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.28, 0.42, 0.45)), // Teal background like your image
            MenuUI,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("TOWER DEFENSE MMO"),
                TextFont {
                    font_size: 72.0,
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.9, 0.9)),
            ));

            // Subtitle
            parent.spawn((
                Text::new("Defend Together!"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.8, 0.8)),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));

            // Play Button (Big Blue Square - stretched with 9-slice)
            parent
                .spawn((
                    Button,
                    ImageNode {
                        image: asset_server.load("UI Elements/UI Elements/Buttons/BigBlueButton_Regular.png"),
                        image_mode: NodeImageMode::Sliced(TextureSlicer {
                            border: BorderRect::all(8.0), // Small border for thin frames
                            center_scale_mode: SliceScaleMode::Stretch,
                            sides_scale_mode: SliceScaleMode::Stretch,
                            max_corner_scale: 1.0,
                        }),
                        ..default()
                    },
                    Node {
                        width: Val::Px(350.0),
                        height: Val::Px(100.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    PlayButton,
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("PLAY"),
                        TextFont {
                            font_size: 48.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Settings Button (Small Blue Round)
            parent
                .spawn((
                    Button,
                    ImageNode {
                        image: asset_server.load("UI Elements/UI Elements/Buttons/SmallBlueRoundButton_Regular.png"),
                        image_mode: NodeImageMode::Sliced(TextureSlicer {
                            border: BorderRect::all(10.0), // Adjust for round buttons
                            center_scale_mode: SliceScaleMode::Stretch,
                            sides_scale_mode: SliceScaleMode::Stretch,
                            max_corner_scale: 1.0,
                        }),
                        ..default()
                    },
                    Node {
                        width: Val::Px(280.0),
                        height: Val::Px(75.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    SettingsButton,
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("SETTINGS"),
                        TextFont {
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Quit Button (Small Red Round)
            parent
                .spawn((
                    Button,
                    ImageNode {
                        image: asset_server.load("UI Elements/UI Elements/Buttons/SmallRedRoundButton_Regular.png"),
                        image_mode: NodeImageMode::Sliced(TextureSlicer {
                            border: BorderRect::all(10.0),
                            center_scale_mode: SliceScaleMode::Stretch,
                            sides_scale_mode: SliceScaleMode::Stretch,
                            max_corner_scale: 1.0,
                        }),
                        ..default()
                    },
                    Node {
                        width: Val::Px(280.0),
                        height: Val::Px(75.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    QuitButton,
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("QUIT"),
                        TextFont {
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

// ============================================================================
// BUTTON INTERACTION SYSTEMS
// ============================================================================

fn handle_play_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<PlayButton>)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            info!("Play button pressed!");
            next_state.set(AppState::InGame);
        }
    }
}

fn handle_settings_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<SettingsButton>)>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            info!("Settings button pressed!");
        }
    }
}

fn handle_quit_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<QuitButton>)>,
    mut exit: MessageWriter<AppExit>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            info!("Quit button pressed!");
            exit.write(AppExit::Success);
        }
    }
}

// Change button images based on interaction state
fn update_button_images(
    asset_server: Res<AssetServer>,
    mut play_query: Query<(&Interaction, &mut ImageNode), (Changed<Interaction>, With<PlayButton>)>,
    mut settings_query: Query<(&Interaction, &mut ImageNode), (Changed<Interaction>, With<SettingsButton>, Without<PlayButton>)>,
    mut quit_query: Query<(&Interaction, &mut ImageNode), (Changed<Interaction>, With<QuitButton>, Without<PlayButton>, Without<SettingsButton>)>,
) {
    // Update Play button
    for (interaction, mut image_node) in &mut play_query {
        let new_image = match *interaction {
            Interaction::Pressed => asset_server.load("UI Elements/UI Elements/Buttons/BigBlueButton_Pressed.png"),
            _ => asset_server.load("UI Elements/UI Elements/Buttons/BigBlueButton_Regular.png"),
        };

        image_node.image = new_image;
        // Keep the 9-slice settings
        image_node.image_mode = NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(8.0),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 1.0,
        });
    }

    // Update Settings button
    for (interaction, mut image_node) in &mut settings_query {
        let new_image = match *interaction {
            Interaction::Pressed => asset_server.load("UI Elements/UI Elements/Buttons/SmallBlueRoundButton_Pressed.png"),
            _ => asset_server.load("UI Elements/UI Elements/Buttons/SmallBlueRoundButton_Regular.png"),
        };

        image_node.image = new_image;
        image_node.image_mode = NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(10.0),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 1.0,
        });
    }

    // Update Quit button
    for (interaction, mut image_node) in &mut quit_query {
        let new_image = match *interaction {
            Interaction::Pressed => asset_server.load("UI Elements/UI Elements/Buttons/SmallRedRoundButton_Pressed.png"),
            _ => asset_server.load("UI Elements/UI Elements/Buttons/SmallRedRoundButton_Regular.png"),
        };

        image_node.image = new_image;
        image_node.image_mode = NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(10.0),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 1.0,
        });
    }
}

fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MenuUI>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

// ============================================================================
// USAGE INSTRUCTIONS
// ============================================================================

// Place button assets in: assets/UI Elements/UI Elements/Buttons/
//
// IMPORTANT: Adjust the `border` values in TextureSlicer based on your images!
// The border value defines how many pixels from each edge should NOT be stretched.
//
// For example, if your button has 20px decorative corners, use:
// BorderRect::all(20.0)
//
// If corners are different sizes, use:
// BorderRect { left: 20.0, right: 20.0, top: 15.0, bottom: 15.0 }