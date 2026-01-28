use crate::prelude::*;
use bevy::audio::AudioSource;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AudioSettings>()
            .add_observer(handle_match_sound)
            .add_observer(handle_attack_sound)
            .add_observer(handle_victory_sound)
            .add_observer(handle_defeat_sound);
    }
}

#[derive(Resource)]
pub struct AudioSettings {
    pub enabled: bool,
    pub volume: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            volume: 1.0,
        }
    }
}

#[derive(Event)]
pub struct MatchSoundEvent {
    pub combo_count: u32,
}

#[derive(Event)]
pub struct AttackSoundEvent {
    pub is_critical: bool,
}

#[derive(Event)]
pub struct VictorySoundEvent;

#[derive(Event)]
pub struct DefeatSoundEvent;

fn handle_match_sound(
    trigger: Trigger<MatchSoundEvent>,
    asset_server: Res<AssetServer>,
    settings: Res<AudioSettings>,
    mut commands: Commands,
) {
    if !settings.enabled {
        return;
    }

    let event = trigger.event();
    let sound_path = if event.combo_count > 3 {
        "audio/match_combo.ogg"
    } else {
        "audio/match.ogg"
    };

    let handle: Handle<AudioSource> = asset_server.load(sound_path);
    commands.spawn((
        AudioPlayer::new(handle),
        PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::new(settings.volume)),
    ));
}

fn handle_attack_sound(
    trigger: Trigger<AttackSoundEvent>,
    asset_server: Res<AssetServer>,
    settings: Res<AudioSettings>,
    mut commands: Commands,
) {
    if !settings.enabled {
        return;
    }

    let event = trigger.event();
    let sound_path = if event.is_critical {
        "audio/attack_critical.ogg"
    } else {
        "audio/attack.ogg"
    };

    let handle: Handle<AudioSource> = asset_server.load(sound_path);
    commands.spawn((
        AudioPlayer::new(handle),
        PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::new(settings.volume)),
    ));
}

fn handle_victory_sound(
    _trigger: Trigger<VictorySoundEvent>,
    asset_server: Res<AssetServer>,
    settings: Res<AudioSettings>,
    mut commands: Commands,
) {
    if !settings.enabled {
        return;
    }

    let handle: Handle<AudioSource> = asset_server.load("audio/victory.ogg");
    commands.spawn((
        AudioPlayer::new(handle),
        PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::new(settings.volume)),
    ));
}

fn handle_defeat_sound(
    _trigger: Trigger<DefeatSoundEvent>,
    asset_server: Res<AssetServer>,
    settings: Res<AudioSettings>,
    mut commands: Commands,
) {
    if !settings.enabled {
        return;
    }

    let handle: Handle<AudioSource> = asset_server.load("audio/defeat.ogg");
    commands.spawn((
        AudioPlayer::new(handle),
        PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::new(settings.volume)),
    ));
}
