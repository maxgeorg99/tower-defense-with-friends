use bevy::prelude::*;

use crate::resources::{AppState, AudioVolume};
use crate::systems::menu::{ButtonStyle, spawn_nine_slice_button};
use crate::systems::SoundEffect;

/// Marker component for the settings screen
#[derive(Component)]
pub struct SettingsScreen;

/// Marker for the back button
#[derive(Component)]
pub struct SettingsBackButton;

/// Marker for the volume slider track
#[derive(Component)]
pub struct VolumeSliderTrack;

/// Marker for the volume slider handle
#[derive(Component)]
pub struct VolumeSliderHandle;

/// Marker for the volume percentage text
#[derive(Component)]
pub struct VolumeText;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AudioVolume>()
            .add_systems(OnEnter(AppState::Settings), setup_settings_screen)
            .add_systems(
                Update,
                (
                    handle_back_button,
                    handle_volume_slider,
                    update_volume_text,
                )
                    .run_if(in_state(AppState::Settings)),
            )
            .add_systems(OnExit(AppState::Settings), cleanup_settings_screen);
    }
}

fn setup_settings_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    volume: Res<AudioVolume>,
) {
    // Main container
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
            BackgroundColor(Color::srgb(0.15, 0.28, 0.32)),
            SettingsScreen,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Settings"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Volume section
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(15.0),
                    ..default()
                })
                .with_children(|section| {
                    // Volume label with percentage
                    section
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(10.0),
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn((
                                Text::new("Master Volume:"),
                                TextFont {
                                    font_size: 24.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.8, 0.85, 0.85)),
                            ));
                            row.spawn((
                                Text::new(format!("{}%", (volume.master * 100.0) as i32)),
                                TextFont {
                                    font_size: 24.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.4, 0.8, 0.4)),
                                VolumeText,
                            ));
                        });

                    // Volume slider
                    section
                        .spawn((
                            Node {
                                width: Val::Px(300.0),
                                height: Val::Px(30.0),
                                justify_content: JustifyContent::FlexStart,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.3, 0.35)),
                            BorderRadius::all(Val::Px(15.0)),
                            VolumeSliderTrack,
                            Button,
                        ))
                        .with_children(|track| {
                            // Filled portion
                            track.spawn((
                                Node {
                                    width: Val::Percent(volume.master * 100.0),
                                    height: Val::Percent(100.0),
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(0.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.3, 0.6, 0.8)),
                                BorderRadius::all(Val::Px(15.0)),
                                VolumeSliderHandle,
                            ));
                        });
                });
        });

    // Back button in bottom-right corner (like login button)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(50.0),
                right: Val::Px(50.0),
                ..default()
            },
            GlobalZIndex(10),
            SettingsScreen,
        ))
        .with_children(|parent| {
            spawn_nine_slice_button(
                parent,
                &asset_server,
                ButtonStyle::SmallBlueRound,
                "BACK",
                SettingsBackButton,
            );
        });
}

fn handle_back_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<SettingsBackButton>)>,
    mut next_state: ResMut<NextState<AppState>>,
    mut sound_events: EventWriter<SoundEffect>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            sound_events.write(SoundEffect::ButtonClick);
            next_state.set(AppState::MainMenu);
        }
    }
}

fn handle_volume_slider(
    interaction_query: Query<(&Interaction, &ComputedNode, &GlobalTransform), With<VolumeSliderTrack>>,
    mut volume: ResMut<AudioVolume>,
    windows: Query<&Window>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut handle_query: Query<&mut Node, (With<VolumeSliderHandle>, Without<VolumeSliderTrack>)>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    for (interaction, computed_node, transform) in &interaction_query {
        // Only process when clicking/dragging on the slider
        if *interaction == Interaction::Pressed
            || (*interaction == Interaction::Hovered && mouse_button.pressed(MouseButton::Left))
        {
            // Get the slider's actual computed size
            let slider_size = computed_node.size();
            let slider_width = slider_size.x;

            // GlobalTransform gives the center of the node in screen coordinates
            let slider_center_x = transform.translation().x;
            let slider_left = slider_center_x - slider_width / 2.0;

            // Calculate relative position within the slider (0.0 to 1.0)
            let relative_x = cursor_pos.x - slider_left;
            let new_volume = (relative_x / slider_width).clamp(0.0, 1.0);

            volume.master = new_volume;

            // Update the handle width
            for mut handle_node in &mut handle_query {
                handle_node.width = Val::Percent(new_volume * 100.0);
            }
        }
    }
}

fn update_volume_text(volume: Res<AudioVolume>, mut text_query: Query<&mut Text, With<VolumeText>>) {
    if volume.is_changed() {
        for mut text in &mut text_query {
            text.0 = format!("{}%", (volume.master * 100.0) as i32);
        }
    }
}

fn cleanup_settings_screen(mut commands: Commands, query: Query<Entity, With<SettingsScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
