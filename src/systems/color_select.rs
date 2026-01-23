use bevy::ecs::prelude::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy_spacetimedb::*;

use crate::auth::AuthState;
use crate::module_bindings::set_color_reducer::set_color;
use crate::module_bindings::set_name_reducer::set_name;
use crate::module_bindings::{Color as PlayerColor, DbConnection};
use crate::resources::AppState;
use crate::systems::menu::{ButtonStyle, spawn_nine_slice_button};

/// Type alias for cleaner SpacetimeDB resource access
pub type SpacetimeDB<'a> = Res<'a, StdbConnection<DbConnection>>;

/// Marker component for the color select screen root
#[derive(Component)]
pub struct ColorSelectScreen;

/// Marker component for the color selector panel
#[derive(Component)]
pub struct ColorSelectorPanel;

/// Marker component for color selection buttons
#[derive(Component)]
pub struct ColorButton(pub PlayerColor);

/// Marker for the username text input
#[derive(Component)]
pub struct UsernameInput;

/// Marker for the continue button
#[derive(Component)]
pub struct ContinueButton;

/// Resource to store the current username being edited
#[derive(Resource, Default)]
pub struct UsernameInputState {
    pub value: String,
}

/// Plugin for the color select screen
pub struct ColorSelectPlugin;

impl Plugin for ColorSelectPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<UsernameInputState>()
            .add_systems(OnEnter(AppState::ColorSelect), setup_color_select_screen)
            .add_systems(
                Update,
                (
                    handle_color_button_click,
                    update_color_button_visuals,
                    handle_username_input,
                    handle_continue_button,
                ).run_if(in_state(AppState::ColorSelect)),
            )
            .add_systems(OnExit(AppState::ColorSelect), cleanup_color_select_screen);
    }
}

/// Setup the color select screen UI
fn setup_color_select_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut username_state: ResMut<UsernameInputState>,
    auth_state: Res<AuthState>,
) {
    // Initialize username from auth profile or use default
    if username_state.value.is_empty() {
        username_state.value = auth_state
            .user_profile
            .as_ref()
            .and_then(|p| p.preferred_username.clone().or_else(|| Some(p.name.clone())))
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "Player".to_string());
    }

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(30.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.28, 0.32)),
            ColorSelectScreen,
        ))
        .with_children(|parent| {

            // Username section
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(10.0),
                        ..default()
                    },
                ))
                .with_children(|section| {
                    section.spawn((
                        Text::new("Username"),
                        TextFont { font_size: 18.0, ..default() },
                        TextColor(Color::srgb(0.7, 0.78, 0.78)),
                    ));

                    // Username input field
                    section
                        .spawn((
                            Node {
                                width: Val::Px(380.0),
                                height: Val::Px(50.0),
                                padding: UiRect::axes(Val::Px(15.0), Val::Px(10.0)),
                                justify_content: JustifyContent::FlexStart,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.1, 0.18, 0.22)),
                            BorderColor::all(Color::srgb(0.3, 0.45, 0.5)),
                            BorderRadius::all(Val::Px(8.0)),
                            UsernameInput,
                            Button,
                        ))
                        .with_children(|input| {
                            input.spawn((
                                Text::new(username_state.value.clone()),
                                TextFont { font_size: 22.0, ..default() },
                                TextColor(Color::WHITE),
                            ));
                        });
                });

            // Color selection section
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(15.0),
                        ..default()
                    },
                ))
                .with_children(|section| {
                    section.spawn((
                        Text::new("Choose Your Color"),
                        TextFont { font_size: 18.0, ..default() },
                        TextColor(Color::srgb(0.7, 0.78, 0.78)),
                    ));

                    // Color buttons row
                    section
                        .spawn((
                            Node {
                                width: Val::Px(380.0),
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceEvenly,
                                align_items: AlignItems::Center,
                                padding: UiRect::axes(Val::Px(10.0), Val::Px(20.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                            BorderRadius::all(Val::Px(16.0)),
                            ColorSelectorPanel,
                        ))
                        .with_children(|row| {
                            spawn_color_button(row, &asset_server, PlayerColor::Blue);
                            spawn_color_button(row, &asset_server, PlayerColor::Yellow);
                            spawn_color_button(row, &asset_server, PlayerColor::Purple);
                            spawn_color_button(row, &asset_server, PlayerColor::Black);
                        });
                });

            // Continue button
            spawn_nine_slice_button(parent, &asset_server, ButtonStyle::SmallBlueRound, "CONTINUE", ContinueButton);
        });
}

/// Spawns a single color selection button with sword icon
fn spawn_color_button(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    color: PlayerColor,
) {
    let sword_path = match color {
        PlayerColor::Blue => "UI Elements/UI Elements/Swords/Sword_Blue.png",
        PlayerColor::Yellow => "UI Elements/UI Elements/Swords/Sword_Yellow.png",
        PlayerColor::Purple => "UI Elements/UI Elements/Swords/Sword_Purple.png",
        PlayerColor::Black => "UI Elements/UI Elements/Swords/Sword_Black.png",
    };

    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(75.0),
                height: Val::Px(75.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
            BorderRadius::all(Val::Px(12.0)),
            ColorButton(color),
        ))
        .with_children(|button| {
            button.spawn((
                ImageNode::new(asset_server.load(sword_path)),
                Node {
                    width: Val::Px(55.0),
                    height: Val::Px(55.0),
                    ..default()
                },
            ));
        });
}

/// System to handle color button interactions
pub fn handle_color_button_click(
    stdb: Option<SpacetimeDB>,
    interaction_query: Query<(&Interaction, &ColorButton), (Changed<Interaction>, With<Button>)>,
) {
    let Some(stdb) = stdb else {
        return;
    };

    for (interaction, color_button) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            if let Err(e) = stdb.reducers().set_color(color_button.0) {
                eprintln!("Failed to set color: {}", e);
            }
        }
    }
}

/// System to update button visual feedback on hover/press
pub fn update_color_button_visuals(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ColorButton>),
    >,
) {
    for (interaction, mut bg_color) in button_query.iter_mut() {
        *bg_color = match *interaction {
            Interaction::Pressed => BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
            Interaction::Hovered => BackgroundColor(Color::srgba(0.35, 0.35, 0.35, 0.9)),
            Interaction::None => BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
        };
    }
}

/// Handle keyboard input for username
fn handle_username_input(
    mut username_state: ResMut<UsernameInputState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut text_query: Query<&mut Text>,
    input_query: Query<(&Interaction, &Children), With<UsernameInput>>,
    mut focused: Local<bool>,
) {
    // Check if the input field was clicked
    for (interaction, _) in &input_query {
        if *interaction == Interaction::Pressed {
            *focused = true;
        }
    }

    if !*focused {
        return;
    }

    // Handle backspace
    if keyboard.just_pressed(KeyCode::Backspace) {
        username_state.value.pop();
    }

    // Handle escape to unfocus
    if keyboard.just_pressed(KeyCode::Escape) || keyboard.just_pressed(KeyCode::Enter) {
        *focused = false;
    }

    // Handle character input via key codes
    let key_char_pairs = [
        (KeyCode::KeyA, 'a'), (KeyCode::KeyB, 'b'), (KeyCode::KeyC, 'c'),
        (KeyCode::KeyD, 'd'), (KeyCode::KeyE, 'e'), (KeyCode::KeyF, 'f'),
        (KeyCode::KeyG, 'g'), (KeyCode::KeyH, 'h'), (KeyCode::KeyI, 'i'),
        (KeyCode::KeyJ, 'j'), (KeyCode::KeyK, 'k'), (KeyCode::KeyL, 'l'),
        (KeyCode::KeyM, 'm'), (KeyCode::KeyN, 'n'), (KeyCode::KeyO, 'o'),
        (KeyCode::KeyP, 'p'), (KeyCode::KeyQ, 'q'), (KeyCode::KeyR, 'r'),
        (KeyCode::KeyS, 's'), (KeyCode::KeyT, 't'), (KeyCode::KeyU, 'u'),
        (KeyCode::KeyV, 'v'), (KeyCode::KeyW, 'w'), (KeyCode::KeyX, 'x'),
        (KeyCode::KeyY, 'y'), (KeyCode::KeyZ, 'z'),
        (KeyCode::Digit0, '0'), (KeyCode::Digit1, '1'), (KeyCode::Digit2, '2'),
        (KeyCode::Digit3, '3'), (KeyCode::Digit4, '4'), (KeyCode::Digit5, '5'),
        (KeyCode::Digit6, '6'), (KeyCode::Digit7, '7'), (KeyCode::Digit8, '8'),
        (KeyCode::Digit9, '9'),
        (KeyCode::Minus, '-'),
    ];

    let shift_pressed = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    for (key, c) in key_char_pairs {
        if keyboard.just_pressed(key) && username_state.value.len() < 20 {
            let final_char = if shift_pressed && c.is_alphabetic() {
                c.to_ascii_uppercase()
            } else {
                c
            };
            username_state.value.push(final_char);
        }
    }

    // Update the displayed text
    for (_, children) in &input_query {
        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                if *focused {
                    text.0 = format!("{}|", username_state.value);
                } else {
                    text.0 = username_state.value.clone();
                }
            }
        }
    }
}

/// Handle continue button click
fn handle_continue_button(
    stdb: Option<SpacetimeDB>,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ContinueButton>)>,
    username_state: Res<UsernameInputState>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            // Set username on server if connected
            if let Some(stdb) = &stdb {
                if !username_state.value.is_empty() {
                    if let Err(e) = stdb.reducers().set_name(username_state.value.clone()) {
                        eprintln!("Failed to set name: {}", e);
                    }
                }
            }
            next_state.set(AppState::InGame);
        }
    }
}

/// Cleanup the color select screen
fn cleanup_color_select_screen(
    mut commands: Commands,
    query: Query<Entity, With<ColorSelectScreen>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
