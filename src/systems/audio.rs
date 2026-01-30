use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

use crate::resources::AudioVolume;

/// Convert linear amplitude (0.0-1.0) to decibels for kira audio
fn amplitude_to_db(amplitude: f32) -> f32 {
    if amplitude <= 0.0 {
        -60.0 // kira's silence threshold
    } else {
        20.0 * amplitude.log10()
    }
}

/// Sound effect events - sent by various systems, played by play_sound_effects
#[derive(Message)]
pub enum SoundEffect {
    /// Tower shoots a projectile
    ArrowShoot,
    /// Worker harvests wood (axe)
    AxeHit,
    /// Worker harvests gold (pickaxe)
    PickaxeHit,
    /// Worker harvests sheep (knife)
    SheepHarvest,
    /// Castle takes damage (enemy reached end)
    CastleDamage,
    /// Resource collected (worker returns home)
    Reward,
    /// Menu button clicked
    ButtonClick,
}

/// Resource holding preloaded sound effect handles
#[derive(Resource)]
pub struct SoundAssets {
    pub arrow: Handle<bevy_kira_audio::AudioSource>,
    pub axe: Handle<bevy_kira_audio::AudioSource>,
    pub pickaxe: Handle<bevy_kira_audio::AudioSource>,
    pub sheep: Handle<bevy_kira_audio::AudioSource>,
    pub castle_damage: Handle<bevy_kira_audio::AudioSource>,
    pub reward: Handle<bevy_kira_audio::AudioSource>,
    pub button_click: Handle<bevy_kira_audio::AudioSource>,
    pub background_music: Handle<bevy_kira_audio::AudioSource>,
}

/// Load all sound assets at startup
pub fn load_sound_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(SoundAssets {
        arrow: asset_server.load("Sound Effects/attack_arrow.mp3"),
        axe: asset_server.load("Sound Effects/axe.mp3"),
        pickaxe: asset_server.load("Sound Effects/pickaxe.mp3"),
        sheep: asset_server.load("Sound Effects/sheep.mp3"),
        castle_damage: asset_server.load("Sound Effects/castle_demage.mp3"),
        reward: asset_server.load("Sound Effects/reward.mp3"),
        button_click: asset_server.load("Sound Effects/choose.mp3"),
        background_music: asset_server.load("Sound Effects/medieval_soundtrack.mp3"),
    });
}

/// Marker to track if background music has started
#[derive(Resource, Default)]
pub struct BackgroundMusicStarted;

/// Start playing background music once assets are loaded
pub fn start_background_music(
    mut commands: Commands,
    sounds: Option<Res<SoundAssets>>,
    audio: Res<Audio>,
    volume: Res<AudioVolume>,
    started: Option<Res<BackgroundMusicStarted>>,
) {
    if started.is_some() {
        return;
    }
    let Some(sounds) = sounds else { return };

    audio
        .play(sounds.background_music.clone())
        .looped()
        .with_volume(amplitude_to_db(volume.master));

    commands.insert_resource(BackgroundMusicStarted);
}

/// System that plays sound effects when events are received
pub fn play_sound_effects(
    mut events: EventReader<SoundEffect>,
    sounds: Option<Res<SoundAssets>>,
    audio: Res<Audio>,
    volume: Res<AudioVolume>,
) {
    let Some(sounds) = sounds else { return };

    for event in events.read() {
        let source = match event {
            SoundEffect::ArrowShoot => sounds.arrow.clone(),
            SoundEffect::AxeHit => sounds.axe.clone(),
            SoundEffect::PickaxeHit => sounds.pickaxe.clone(),
            SoundEffect::SheepHarvest => sounds.sheep.clone(),
            SoundEffect::CastleDamage => sounds.castle_damage.clone(),
            SoundEffect::Reward => sounds.reward.clone(),
            SoundEffect::ButtonClick => sounds.button_click.clone(),
        };

        audio.play(source).with_volume(amplitude_to_db(volume.master));
    }
}
