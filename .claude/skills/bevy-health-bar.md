# Bevy Health Bar

## Overview

A child-entity based health bar system for Bevy game units. The health bar is rendered as a sprite that scales with current HP and changes color based on health percentage.

## Use Cases

- Display HP above game units (player/enemy)
- Visual feedback for damage taken
- Real-time health tracking during combat

## Implementation Pattern

### Components

```rust
#[derive(Component)]
pub struct HealthBar;

#[derive(Component)]
pub struct HealthBarBackground;
```

### Constants

```rust
const HEALTH_BAR_WIDTH: f32 = 30.0;
const HEALTH_BAR_HEIGHT: f32 = 4.0;
const HEALTH_BAR_OFFSET_Y: f32 = 25.0;  // Distance above unit center
```

### Spawn System

Use `Added<Unit>` filter to automatically attach health bars to new units:

```rust
pub fn spawn_health_bars(
    mut commands: Commands,
    units: Query<Entity, Added<Unit>>,
) {
    for entity in units.iter() {
        commands.entity(entity).with_children(|parent| {
            // Background (dark gray, behind)
            parent.spawn((
                HealthBarBackground,
                Sprite {
                    color: Color::srgb(0.2, 0.2, 0.2),
                    custom_size: Some(Vec2::new(HEALTH_BAR_WIDTH, HEALTH_BAR_HEIGHT)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, HEALTH_BAR_OFFSET_Y, 0.1)),
            ));
            // Foreground (green, in front)
            parent.spawn((
                HealthBar,
                Sprite {
                    color: Color::srgb(0.2, 0.9, 0.2),
                    custom_size: Some(Vec2::new(HEALTH_BAR_WIDTH, HEALTH_BAR_HEIGHT)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, HEALTH_BAR_OFFSET_Y, 0.2)),
            ));
        });
    }
}
```

### Update System

```rust
pub fn update_health_bars(
    units: Query<(&Children, &UnitStats), With<Unit>>,
    mut health_bars: Query<&mut Sprite, With<HealthBar>>,
) {
    for (children, stats) in units.iter() {
        // Skip dead units to avoid zero-width sprite
        if stats.is_dead() {
            continue;
        }
        let health_ratio = stats.health / stats.max_health;
        for &child in children.iter() {
            if let Ok(mut sprite) = health_bars.get_mut(child) {
                sprite.custom_size = Some(Vec2::new(
                    HEALTH_BAR_WIDTH * health_ratio,
                    HEALTH_BAR_HEIGHT
                ));
                sprite.color = health_ratio_to_color(health_ratio);
            }
        }
    }
}
```

### Color Gradient

```rust
fn health_ratio_to_color(ratio: f32) -> Color {
    if ratio > 0.5 {
        Color::srgb(0.2, 0.9, 0.2)  // Green (healthy)
    } else if ratio > 0.25 {
        Color::srgb(0.9, 0.9, 0.2)  // Yellow (warning)
    } else {
        Color::srgb(0.9, 0.2, 0.2)  // Red (critical)
    }
}
```

### System Registration

```rust
.add_systems(
    Update,
    (
        spawn_health_bars,
        // ... other systems ...
        update_health_bars,
    )
        .chain()
        .run_if(in_state(GameState::Playing)),
)
```

## Pitfalls

### 1. Zero-Width Sprite Panic

**Problem**: When HP reaches 0, `health_ratio = 0` causes sprite width to be 0, triggering Bevy's `sprite_picking` panic.

**Solution**: Skip dead units in update system:
```rust
if stats.is_dead() {
    continue;
}
```

### 2. Orphaned Health Bars

**Problem**: Using `despawn()` on parent unit leaves child health bars as orphans.

**Solution**: Always use `despawn_recursive()` when removing units:
```rust
commands.entity(entity).despawn_recursive();
```

### 3. Z-Index Layering

**Problem**: Health bar background and foreground overlap incorrectly.

**Solution**: Use different z-values in Transform:
- Background: `z = 0.1`
- Foreground: `z = 0.2`

### 4. Health Bar Not Moving with Unit

**Problem**: Health bar stays in place when unit moves.

**Solution**: Health bars must be **children** of the unit entity (using `with_children`), not separate entities. Child transforms are relative to parent.
