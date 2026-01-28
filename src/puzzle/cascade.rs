use crate::prelude::*;
use super::{PuzzleBoard, Tile, TileType, GridPosition};

pub fn apply_gravity(
    _commands: Commands,
    mut board: ResMut<PuzzleBoard>,
    mut tiles: Query<(Entity, &mut GridPosition, &mut Transform), With<Tile>>,
) {
    for x in 0..PUZZLE_BOARD_SIZE {
        let mut write_y = 0;

        for read_y in 0..PUZZLE_BOARD_SIZE {
            if let Some(entity) = board.get(x, read_y) {
                if read_y != write_y {
                    board.set(x, write_y, Some(entity));
                    board.set(x, read_y, None);

                    if let Ok((_, mut pos, mut transform)) = tiles.get_mut(entity) {
                        pos.y = write_y;
                        let target = board.grid_to_world(x, write_y);
                        transform.translation = target.extend(0.0);
                    }
                }
                write_y += 1;
            }
        }
    }
}

pub fn spawn_new_tiles(
    mut commands: Commands,
    mut board: ResMut<PuzzleBoard>,
) {
    for x in 0..PUZZLE_BOARD_SIZE {
        for y in 0..PUZZLE_BOARD_SIZE {
            if board.get(x, y).is_none() {
                let tile_type = TileType::random();
                let pos = board.grid_to_world(x, y);

                let entity = commands
                    .spawn((
                        Tile,
                        tile_type,
                        GridPosition::new(x, y),
                        Sprite {
                            color: tile_type.color(),
                            custom_size: Some(Vec2::splat(TILE_SIZE)),
                            ..default()
                        },
                        Transform::from_translation(pos.extend(0.0)),
                    ))
                    .id();

                board.set(x, y, Some(entity));
            }
        }
    }
}
