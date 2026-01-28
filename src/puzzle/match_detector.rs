use crate::prelude::*;
use super::{PuzzleBoard, Tile, TileType, GridPosition, Matched};
use crate::bridge::MatchEvent;

pub fn detect_matches(
    mut commands: Commands,
    _board: Res<PuzzleBoard>,
    tiles: Query<(Entity, &GridPosition, &TileType), (With<Tile>, Without<Matched>)>,
) {
    let mut matched_positions: Vec<(usize, usize)> = Vec::new();
    let mut match_groups: Vec<(TileType, Vec<(usize, usize)>)> = Vec::new();

    let mut grid: [[Option<TileType>; PUZZLE_BOARD_SIZE]; PUZZLE_BOARD_SIZE] =
        [[None; PUZZLE_BOARD_SIZE]; PUZZLE_BOARD_SIZE];

    for (_, pos, tile_type) in tiles.iter() {
        grid[pos.y][pos.x] = Some(*tile_type);
    }

    for y in 0..PUZZLE_BOARD_SIZE {
        let mut x = 0;
        while x < PUZZLE_BOARD_SIZE {
            if let Some(tile_type) = grid[y][x] {
                let mut run = vec![(x, y)];
                let mut nx = x + 1;
                while nx < PUZZLE_BOARD_SIZE && grid[y][nx] == Some(tile_type) {
                    run.push((nx, y));
                    nx += 1;
                }
                if run.len() >= 3 {
                    matched_positions.extend(run.iter().copied());
                    match_groups.push((tile_type, run.clone()));
                }
                x = nx;
            } else {
                x += 1;
            }
        }
    }

    for x in 0..PUZZLE_BOARD_SIZE {
        let mut y = 0;
        while y < PUZZLE_BOARD_SIZE {
            if let Some(tile_type) = grid[y][x] {
                let mut run = vec![(x, y)];
                let mut ny = y + 1;
                while ny < PUZZLE_BOARD_SIZE && grid[ny][x] == Some(tile_type) {
                    run.push((x, ny));
                    ny += 1;
                }
                if run.len() >= 3 {
                    matched_positions.extend(run.iter().copied());
                    match_groups.push((tile_type, run.clone()));
                }
                y = ny;
            } else {
                y += 1;
            }
        }
    }

    matched_positions.sort();
    matched_positions.dedup();

    for (entity, pos, _) in tiles.iter() {
        if matched_positions.contains(&(pos.x, pos.y)) {
            commands.entity(entity).insert(Matched);
        }
    }

    for (tile_type, positions) in match_groups {
        commands.trigger(MatchEvent {
            tile_type,
            count: positions.len(),
            positions,
        });
    }
}

pub fn remove_matched_tiles(
    mut commands: Commands,
    mut board: ResMut<PuzzleBoard>,
    matched: Query<(Entity, &GridPosition), With<Matched>>,
) {
    for (entity, pos) in matched.iter() {
        board.set(pos.x, pos.y, None);
        commands.entity(entity).despawn();
    }
}
