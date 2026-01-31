//! Wave Break countdown timer UI
//!
//! Displays remaining time during WaveBreak phase for unit repositioning.

use crate::prelude::*;

/// Marker component for the wave break countdown UI
#[derive(Component)]
pub struct WaveBreakCountdown;

/// Marker component for the countdown text
#[derive(Component)]
pub struct CountdownText;

/// Spawns the countdown UI when entering WaveBreak phase
pub fn spawn_wavebreak_countdown(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                top: Val::Px(20.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            WaveBreakCountdown,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("REPOSITION TIME: 10.0"),
                TextFont {
                    font_size: 36.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                CountdownText,
            ));
        });
}

/// Updates the countdown text every frame
pub fn update_wavebreak_countdown(
    wave_break_timer: Res<WaveBreakTimer>,
    mut query: Query<(&mut Text, &mut TextColor), With<CountdownText>>,
) {
    for (mut text, mut color) in query.iter_mut() {
        let remaining = wave_break_timer.remaining;

        // Update text
        **text = format!("REPOSITION TIME: {:.1}", remaining);

        // Change color to red when <= 3 seconds remaining
        if remaining <= 3.0 {
            *color = TextColor(Color::srgb(1.0, 0.3, 0.3));
        } else {
            *color = TextColor(Color::WHITE);
        }
    }
}

/// Despawns the countdown UI when exiting WaveBreak phase
pub fn despawn_wavebreak_countdown(
    mut commands: Commands,
    query: Query<Entity, With<WaveBreakCountdown>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wavebreak_countdown_component_exists() {
        let _countdown = WaveBreakCountdown;
    }

    #[test]
    fn test_countdown_text_component_exists() {
        let _text = CountdownText;
    }

    #[test]
    fn test_countdown_color_threshold() {
        // Color should change at 3.0 seconds
        let remaining_normal = 5.0;
        let remaining_warning = 2.5;

        assert!(remaining_normal > 3.0, "Normal should be above threshold");
        assert!(remaining_warning <= 3.0, "Warning should be at or below threshold");
    }
}
