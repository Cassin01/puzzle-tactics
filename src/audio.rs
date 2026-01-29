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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_settings_default() {
        let settings = AudioSettings::default();
        assert!(settings.enabled);
        assert!((settings.volume - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_match_sound_event_creation() {
        let event = MatchSoundEvent { combo_count: 5 };
        assert_eq!(event.combo_count, 5);
    }

    #[test]
    fn test_match_sound_event_combo_threshold() {
        // Combo > 3 should use different sound
        let low_combo = MatchSoundEvent { combo_count: 2 };
        let high_combo = MatchSoundEvent { combo_count: 4 };

        assert!(low_combo.combo_count <= 3);
        assert!(high_combo.combo_count > 3);
    }

    #[test]
    fn test_attack_sound_event_creation() {
        let normal = AttackSoundEvent { is_critical: false };
        let critical = AttackSoundEvent { is_critical: true };

        assert!(!normal.is_critical);
        assert!(critical.is_critical);
    }

    #[test]
    fn test_victory_sound_event_creation() {
        let _event = VictorySoundEvent;
        // Unit struct, just verify it compiles
    }

    #[test]
    fn test_defeat_sound_event_creation() {
        let _event = DefeatSoundEvent;
        // Unit struct, just verify it compiles
    }

    #[test]
    fn test_audio_settings_disabled() {
        let mut settings = AudioSettings::default();
        settings.enabled = false;
        assert!(!settings.enabled);
    }

    #[test]
    fn test_audio_settings_volume_range() {
        let mut settings = AudioSettings::default();
        settings.volume = 0.5;
        assert!((settings.volume - 0.5).abs() < f32::EPSILON);

        settings.volume = 0.0;
        assert!((settings.volume - 0.0).abs() < f32::EPSILON);
    }
}
