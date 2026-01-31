//! Unit placement system for Wave Break phase
//!
//! During Wave Break, players can reposition their units by:
//! 1. Clicking a friendly unit to select it
//! 2. Clicking an empty hex to move the selected unit

use crate::prelude::*;
use super::{Unit, Team, BattleGrid, HexPosition};

// ============================================================
// Components
// ============================================================

/// Marker component for selected unit
#[derive(Component)]
pub struct Selected;

/// Marker component for units that can be selected (player units only)
#[derive(Component)]
pub struct SelectableUnit;

/// Marker component for movement highlight tiles
#[derive(Component)]
pub struct MovementHighlight;

// ============================================================
// Events
// ============================================================

/// Event for unit selection
#[derive(Event)]
pub struct UnitSelectEvent {
    pub entity: Entity,
}

/// Event for unit movement
#[derive(Event)]
pub struct UnitMoveEvent {
    pub entity: Entity,
    pub target_pos: HexPosition,
}

// ============================================================
// Systems
// ============================================================

/// System to handle click input during WaveBreak phase
pub fn placement_input_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    grid: Res<BattleGrid>,
    current_phase: Res<State<PhaseState>>,
    selected_query: Query<Entity, With<Selected>>,
    selectable_units: Query<(Entity, &HexPosition), (With<SelectableUnit>, With<Unit>)>,
    mut commands: Commands,
) {
    // Only active during WaveBreak phase
    if *current_phase.get() != PhaseState::WaveBreak {
        return;
    }

    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    // Get cursor position in world coordinates
    let Some(world_pos) = get_cursor_world_position(&windows, &camera) else {
        return;
    };

    // Convert to hex position
    let hex_pos = grid.pixel_to_axial(world_pos);

    // Check if clicking on a selectable unit
    for (entity, unit_pos) in selectable_units.iter() {
        if *unit_pos == hex_pos {
            // Toggle selection
            if selected_query.get(entity).is_ok() {
                // Already selected, deselect
                commands.entity(entity).remove::<Selected>();
            } else {
                // Deselect any currently selected unit
                for selected_entity in selected_query.iter() {
                    commands.entity(selected_entity).remove::<Selected>();
                }
                // Select this unit
                commands.entity(entity).insert(Selected);
                commands.trigger(UnitSelectEvent { entity });
            }
            return;
        }
    }

    // If we have a selected unit and clicked on empty space, try to move
    if let Ok(selected_entity) = selected_query.get_single() {
        if grid.is_valid_position(&hex_pos) && !grid.is_occupied(&hex_pos) {
            commands.trigger(UnitMoveEvent {
                entity: selected_entity,
                target_pos: hex_pos,
            });
        }
    }
}

/// System to handle unit movement events
pub fn handle_unit_move(
    trigger: Trigger<UnitMoveEvent>,
    mut grid: ResMut<BattleGrid>,
    mut unit_query: Query<(&mut HexPosition, &mut Transform), With<Unit>>,
    mut commands: Commands,
) {
    let event = trigger.event();

    if let Ok((mut hex_pos, mut transform)) = unit_query.get_mut(event.entity) {
        let old_pos = *hex_pos;

        // Update grid
        if grid.move_unit(&old_pos, &event.target_pos) {
            // Update unit position component
            *hex_pos = event.target_pos;

            // Update transform
            let world_pos = grid.axial_to_pixel(&event.target_pos);
            transform.translation = world_pos.extend(transform.translation.z);

            // Deselect after move
            commands.entity(event.entity).remove::<Selected>();
        }
    }
}

/// System to spawn movement highlights for selected unit
pub fn spawn_movement_highlights(
    selected_query: Query<&HexPosition, (With<Selected>, Added<Selected>)>,
    grid: Res<BattleGrid>,
    current_phase: Res<State<PhaseState>>,
    mut commands: Commands,
) {
    // Only during WaveBreak
    if *current_phase.get() != PhaseState::WaveBreak {
        return;
    }

    for selected_pos in selected_query.iter() {
        // Show highlights on empty adjacent hexes
        for neighbor in selected_pos.neighbors() {
            if grid.is_valid_position(&neighbor) && !grid.is_occupied(&neighbor) {
                let world_pos = grid.axial_to_pixel(&neighbor);
                commands.spawn((
                    MovementHighlight,
                    Sprite {
                        color: Color::srgba(0.2, 0.8, 0.2, 0.4),
                        custom_size: Some(Vec2::splat(50.0)),
                        ..default()
                    },
                    Transform::from_translation(world_pos.extend(0.5)),
                ));
            }
        }
    }
}

/// System to despawn movement highlights when selection changes
pub fn despawn_movement_highlights(
    highlights: Query<Entity, With<MovementHighlight>>,
    selected_query: Query<(), With<Selected>>,
    current_phase: Res<State<PhaseState>>,
    mut commands: Commands,
) {
    // Despawn if no selection or not in WaveBreak
    if selected_query.is_empty() || *current_phase.get() != PhaseState::WaveBreak {
        for entity in highlights.iter() {
            commands.entity(entity).despawn();
        }
    }
}

/// System to update selected unit visual (highlight effect)
pub fn update_selected_visual(
    mut selected_units: Query<&mut Sprite, Added<Selected>>,
) {
    for mut sprite in selected_units.iter_mut() {
        // Brighten the selected unit
        let current = sprite.color.to_srgba();
        sprite.color = Color::srgba(
            (current.red * 1.5).min(1.0),
            (current.green * 1.5).min(1.0),
            (current.blue * 1.5).min(1.0),
            current.alpha,
        );
    }
}

/// System to restore deselected unit visual
pub fn restore_deselected_visual(
    mut removed: RemovedComponents<Selected>,
    mut units: Query<&mut Sprite, With<Unit>>,
) {
    for entity in removed.read() {
        if let Ok(mut sprite) = units.get_mut(entity) {
            // Restore original brightness
            let current = sprite.color.to_srgba();
            sprite.color = Color::srgba(
                current.red / 1.5,
                current.green / 1.5,
                current.blue / 1.5,
                current.alpha,
            );
        }
    }
}

/// System to mark player units as selectable at wave break start
pub fn mark_units_selectable(
    player_units: Query<Entity, (With<Unit>, With<Team>)>,
    team_query: Query<&Team>,
    current_phase: Res<State<PhaseState>>,
    mut commands: Commands,
) {
    if *current_phase.get() != PhaseState::WaveBreak {
        return;
    }

    for entity in player_units.iter() {
        if let Ok(team) = team_query.get(entity) {
            if *team == Team::Player {
                commands.entity(entity).insert(SelectableUnit);
            }
        }
    }
}

/// System to remove selectable marker when leaving WaveBreak
pub fn unmark_units_selectable(
    selectable: Query<Entity, With<SelectableUnit>>,
    current_phase: Res<State<PhaseState>>,
    mut commands: Commands,
) {
    if *current_phase.get() == PhaseState::WaveBreak {
        return;
    }

    for entity in selectable.iter() {
        commands.entity(entity).remove::<SelectableUnit>();
        commands.entity(entity).remove::<Selected>();
    }
}

// ============================================================
// Helper Functions
// ============================================================

fn get_cursor_world_position(
    windows: &Query<&Window>,
    camera: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let window = windows.single();
    let (camera, camera_transform) = camera.single();

    window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selected_component_exists() {
        // Selected should be a valid component
        let _selected = Selected;
    }

    #[test]
    fn test_selectable_unit_component_exists() {
        // SelectableUnit should be a valid component
        let _selectable = SelectableUnit;
    }

    #[test]
    fn test_movement_highlight_component_exists() {
        // MovementHighlight should be a valid component
        let _highlight = MovementHighlight;
    }

    #[test]
    fn test_unit_select_event() {
        use bevy::ecs::entity::Entity;
        let event = UnitSelectEvent {
            entity: Entity::from_raw(1),
        };
        assert_eq!(event.entity.index(), 1);
    }

    #[test]
    fn test_unit_move_event() {
        use bevy::ecs::entity::Entity;
        let event = UnitMoveEvent {
            entity: Entity::from_raw(1),
            target_pos: HexPosition::new(2, -1),
        };
        assert_eq!(event.entity.index(), 1);
        assert_eq!(event.target_pos.q, 2);
        assert_eq!(event.target_pos.r, -1);
    }
}
