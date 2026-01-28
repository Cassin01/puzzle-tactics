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
