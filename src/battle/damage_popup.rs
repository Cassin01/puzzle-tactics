use crate::prelude::*;

#[derive(Component)]
pub struct DamagePopup {
    pub timer: Timer,
    pub start_pos: Vec3,
}

#[derive(Event)]
pub struct DamagePopupEvent {
    pub position: Vec3,
    pub damage: i32,
    pub is_critical: bool,
}

pub fn spawn_damage_popup(
    trigger: Trigger<DamagePopupEvent>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let spawn_pos = event.position + Vec3::new(0.0, 20.0, 10.0);

    let (color, font_size) = if event.is_critical {
        (Color::srgb(1.0, 0.2, 0.2), 32.0)
    } else {
        (Color::srgb(1.0, 1.0, 1.0), 24.0)
    };

    commands.spawn((
        Text2d::new(format!("{}", event.damage)),
        TextFont {
            font_size,
            ..default()
        },
        TextColor(color),
        Transform::from_translation(spawn_pos),
        DamagePopup {
            timer: Timer::from_seconds(0.5, TimerMode::Once),
            start_pos: spawn_pos,
        },
    ));
}

pub fn animate_damage_popup(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut TextColor, &mut DamagePopup)>,
) {
    for (entity, mut transform, mut text_color, mut popup) in &mut query {
        popup.timer.tick(time.delta());

        let progress = popup.timer.fraction();

        transform.translation.y = popup.start_pos.y + progress * 50.0;

        let alpha = 1.0 - progress;
        text_color.0 = text_color.0.with_alpha(alpha);

        if popup.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub const POPUP_DURATION: f32 = 0.5;
pub const POPUP_FLOAT_DISTANCE: f32 = 50.0;
pub const CRITICAL_FONT_SIZE: f32 = 32.0;
pub const NORMAL_FONT_SIZE: f32 = 24.0;

/// Calculate the Y offset for damage popup based on animation progress
pub fn calculate_popup_y_offset(progress: f32) -> f32 {
    progress * POPUP_FLOAT_DISTANCE
}

/// Calculate the alpha (opacity) for damage popup based on animation progress
pub fn calculate_popup_alpha(progress: f32) -> f32 {
    1.0 - progress
}

/// Determine font size based on critical hit status
pub fn get_popup_font_size(is_critical: bool) -> f32 {
    if is_critical {
        CRITICAL_FONT_SIZE
    } else {
        NORMAL_FONT_SIZE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_popup_y_offset_at_start() {
        assert_eq!(calculate_popup_y_offset(0.0), 0.0);
    }

    #[test]
    fn test_popup_y_offset_at_middle() {
        assert_eq!(calculate_popup_y_offset(0.5), 25.0);
    }

    #[test]
    fn test_popup_y_offset_at_end() {
        assert_eq!(calculate_popup_y_offset(1.0), 50.0);
    }

    #[test]
    fn test_popup_alpha_at_start() {
        assert_eq!(calculate_popup_alpha(0.0), 1.0);
    }

    #[test]
    fn test_popup_alpha_at_middle() {
        assert_eq!(calculate_popup_alpha(0.5), 0.5);
    }

    #[test]
    fn test_popup_alpha_at_end() {
        assert_eq!(calculate_popup_alpha(1.0), 0.0);
    }

    #[test]
    fn test_critical_font_size() {
        assert_eq!(get_popup_font_size(true), 32.0);
    }

    #[test]
    fn test_normal_font_size() {
        assert_eq!(get_popup_font_size(false), 24.0);
    }

    #[test]
    fn test_popup_duration_is_half_second() {
        assert_eq!(POPUP_DURATION, 0.5);
    }
}
