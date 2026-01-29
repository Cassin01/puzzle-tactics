use crate::prelude::*;
use super::{PuzzleBoard, Tile, TileType, GridPosition, Matched, IceMeltEvent, BombDefuseEvent};
use crate::bridge::{MatchEvent, CoreAbilityEvent};

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
        let is_core_adjacent = positions
            .iter()
            .any(|(x, y)| PuzzleBoard::is_adjacent_to_core(*x, *y) || PuzzleBoard::is_core_position(*x, *y));

        commands.trigger(MatchEvent {
            tile_type,
            count: positions.len(),
            positions: positions.clone(),
        });

        if is_core_adjacent {
            commands.trigger(CoreAbilityEvent {
                tile_type,
                count: positions.len(),
                positions,
            });
        }
    }
}

pub fn remove_matched_tiles(
    mut commands: Commands,
    mut board: ResMut<PuzzleBoard>,
    matched: Query<(Entity, &GridPosition), With<Matched>>,
) {
    let matched_positions: Vec<(usize, usize)> = matched
        .iter()
        .map(|(_, pos)| (pos.x, pos.y))
        .collect();

    // Clear ice obstacles adjacent to matched tiles
    for (x, y) in &matched_positions {
        for (dx, dy) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
            let nx = *x as i32 + dx;
            let ny = *y as i32 + dy;
            if nx >= 0 && ny >= 0 && (nx as usize) < PUZZLE_BOARD_SIZE && (ny as usize) < PUZZLE_BOARD_SIZE {
                if board.has_ice(nx as usize, ny as usize) {
                    board.clear_obstacle(nx as usize, ny as usize);
                    commands.trigger(IceMeltEvent { position: (nx as usize, ny as usize) });
                }
            }
        }
    }

    // Defuse bombs on matched tiles (bomb is child, will be despawned with tile)
    for &(x, y) in &matched_positions {
        if board.has_bomb(x, y) {
            commands.trigger(BombDefuseEvent { position: (x, y) });
            board.clear_obstacle(x, y);
        }
    }

    // Despawn matched tiles (despawn_recursive removes child bombs too)
    for (entity, pos) in matched.iter() {
        board.set(pos.x, pos.y, None);
        commands.entity(entity).despawn_recursive();
    }
}
