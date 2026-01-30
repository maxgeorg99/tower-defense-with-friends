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

/// Marker for volume decrease button
#[derive(Component)]
pub struct VolumeDownButton;

/// Marker for volume increase button
#[derive(Component)]
pub struct VolumeUpButton;

/// Marker for the volume bar (visual fill)
#[derive(Component)]
pub struct VolumeBar;

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
                    handle_settings_buttons,
                    update_volume_display,
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
                    row_gap: Val::Px(20.0),
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
                                    font_size: 28.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                            ));
                            row.spawn((
                                Text::new(format!("{}%", (volume.master * 100.0) as i32)),
                                TextFont {
                                    font_size: 28.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.4, 0.8, 0.4)),
                                VolumeText,
                            ));
                        });

                    // Volume bar (visual only)
                    section
                        .spawn((
                            Node {
                                width: Val::Px(300.0),
                                height: Val::Px(20.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.25, 0.3)),
                            BorderRadius::all(Val::Px(10.0)),
                        ))
                        .with_children(|bar_container| {
                            bar_container.spawn((
                                Node {
                                    width: Val::Percent(volume.master * 100.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.3, 0.6, 0.8)),
                                BorderRadius::all(Val::Px(10.0)),
                                VolumeBar,
                            ));
                        });

                    // Volume control buttons row
                    section
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(20.0),
                            ..default()
                        })
                        .with_children(|row| {
                            spawn_nine_slice_button(
                                row,
                                &asset_server,
                                ButtonStyle::SmallRedSquare,
                                "- VOL",
                                VolumeDownButton,
                            );
                            spawn_nine_slice_button(
                                row,
                                &asset_server,
                                ButtonStyle::SmallBlueSquare,
                                "+ VOL",
                                VolumeUpButton,
                            );
                        });
                });

            // Back button
            spawn_nine_slice_button(
                parent,
                &asset_server,
                ButtonStyle::SmallBlueRound,
                "BACK",
                SettingsBackButton,
            );
        });
}

fn handle_settings_buttons(
    back_query: Query<&Interaction, (Changed<Interaction>, With<SettingsBackButton>)>,
    down_query: Query<&Interaction, (Changed<Interaction>, With<VolumeDownButton>)>,
    up_query: Query<&Interaction, (Changed<Interaction>, With<VolumeUpButton>)>,
    mut next_state: ResMut<NextState<AppState>>,
    mut volume: ResMut<AudioVolume>,
    mut sound_events: EventWriter<SoundEffect>,
) {
    // Handle back button
    for interaction in &back_query {
        if *interaction == Interaction::Pressed {
            sound_events.write(SoundEffect::ButtonClick);
            next_state.set(AppState::MainMenu);
        }
    }

    // Handle volume down
    for interaction in &down_query {
        if *interaction == Interaction::Pressed {
            volume.master = (volume.master - 0.1).max(0.0);
            sound_events.write(SoundEffect::ButtonClick);
        }
    }

    // Handle volume up
    for interaction in &up_query {
        if *interaction == Interaction::Pressed {
            volume.master = (volume.master + 0.1).min(1.0);
            sound_events.write(SoundEffect::ButtonClick);
        }
    }
}

fn update_volume_display(
    volume: Res<AudioVolume>,
    mut text_query: Query<&mut Text, With<VolumeText>>,
    mut bar_query: Query<&mut Node, With<VolumeBar>>,
) {
    if volume.is_changed() {
        for mut text in &mut text_query {
            text.0 = format!("{}%", (volume.master * 100.0) as i32);
        }
        for mut bar in &mut bar_query {
            bar.width = Val::Percent(volume.master * 100.0);
        }
    }
}

fn cleanup_settings_screen(mut commands: Commands, query: Query<Entity, With<SettingsScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
