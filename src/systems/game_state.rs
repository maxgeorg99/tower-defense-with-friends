use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

use crate::components::GameOverScreen;
use crate::resources::{AppState, GameState};
use crate::systems::{BackgroundMusicHandle, SoundEffect};

pub fn check_game_over(game_state: Res<GameState>, mut next_state: ResMut<NextState<AppState>>) {
    if game_state.lives <= 0 {
        next_state.set(AppState::GameOver);
    }
}

pub fn setup_game_over_screen(
    mut commands: Commands,
    music_handle: Option<Res<BackgroundMusicHandle>>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    mut sound_events: MessageWriter<SoundEffect>,
) {
    // Stop background music
    if let Some(handle) = music_handle {
        if let Some(instance) = audio_instances.get_mut(&handle.0) {
            instance.stop(default());
        }
    }

    // Play game over sound
    sound_events.write(SoundEffect::GameOver);
    commands
        .spawn((
            Node {
                // Stretch over whole screen
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::linear_rgba(0.0, 0.0, 0.0, 0.7)),
            GameOverScreen,
        ))
        .with_children(|parent| {
            // GAME OVER text
            parent.spawn((
                Text::new("GAME OVER"),
                TextFont {
                    font_size: 80.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

pub fn cleanup_game_over_screen(mut commands: Commands, query: Query<Entity, With<GameOverScreen>>) {
    for e in &query {
        commands.entity(e).despawn();
    }
}
