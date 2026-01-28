use crate::prelude::*;
use crate::camera::MainCamera;
use super::{PuzzleBoard, Tile, GridPosition, Selected};

#[derive(Resource, Default)]
pub struct SelectedTile(pub Option<(usize, usize)>);

pub fn handle_tile_click(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    board: Res<PuzzleBoard>,
    mut selected: Local<Option<(usize, usize)>>,
    tiles: Query<(Entity, &GridPosition), With<Tile>>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.get_single() else { return };
    let Ok((camera, camera_transform)) = camera_q.get_single() else { return };

    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { return };

    let Some((x, y)) = board.world_to_grid(world_pos) else { return };

    // Ice tiles cannot be moved
    if board.has_ice(x, y) {
        return;
    }

    for (entity, _) in tiles.iter() {
        commands.entity(entity).remove::<Selected>();
    }

    if let Some(prev) = *selected {
        // Cannot swap if either tile has ice
        if board.has_ice(prev.0, prev.1) {
            *selected = None;
            return;
        }
        if is_adjacent(prev, (x, y)) {
            commands.trigger(SwapTilesEvent { from: prev, to: (x, y) });
        }
        *selected = None;
    } else {
        *selected = Some((x, y));
        if let Some(entity) = board.get(x, y) {
            commands.entity(entity).insert(Selected);
        }
    }
}

fn is_adjacent(a: (usize, usize), b: (usize, usize)) -> bool {
    let dx = (a.0 as i32 - b.0 as i32).abs();
    let dy = (a.1 as i32 - b.1 as i32).abs();
    (dx == 1 && dy == 0) || (dx == 0 && dy == 1)
}

#[derive(Event)]
pub struct SwapTilesEvent {
    pub from: (usize, usize),
    pub to: (usize, usize),
}

pub fn handle_tile_swap(
    trigger: Trigger<SwapTilesEvent>,
    mut board: ResMut<PuzzleBoard>,
    mut tiles: Query<(&mut GridPosition, &mut Transform), With<Tile>>,
) {
    let event = trigger.event();
    let from = event.from;
    let to = event.to;

    let from_entity = board.get(from.0, from.1);
    let to_entity = board.get(to.0, to.1);

    board.swap(from, to);

    if let Some(entity) = from_entity {
        if let Ok((mut pos, mut transform)) = tiles.get_mut(entity) {
            pos.x = to.0;
            pos.y = to.1;
            let world = board.grid_to_world(to.0, to.1);
            transform.translation = world.extend(0.0);
        }
    }

    if let Some(entity) = to_entity {
        if let Ok((mut pos, mut transform)) = tiles.get_mut(entity) {
            pos.x = from.0;
            pos.y = from.1;
            let world = board.grid_to_world(from.0, from.1);
            transform.translation = world.extend(0.0);
        }
    }
}
