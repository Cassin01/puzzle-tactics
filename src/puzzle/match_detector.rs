use crate::prelude::*;
use super::{PuzzleBoard, Tile, TileType, GridPosition, Matched, IceMeltEvent, BombDefuseEvent};
use crate::bridge::{MatchEvent, CoreAbilityEvent};
use crate::audio::MatchSoundEvent;

pub fn detect_matches(
    mut commands: Commands,
    _board: Res<PuzzleBoard>,
    combo: Res<ComboCounter>,
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

    // Trigger sound event if there are matches
    if !match_groups.is_empty() {
        commands.trigger(MatchSoundEvent {
            combo_count: combo.current,
        });
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

/// Check if swapping two positions would result in a match (without modifying the board)
/// Used to prevent invalid moves that don't create any matches
pub fn would_match_after_swap(
    grid: &[[Option<TileType>; PUZZLE_BOARD_SIZE]; PUZZLE_BOARD_SIZE],
    pos1: (usize, usize),
    pos2: (usize, usize),
) -> bool {
    // Create a virtual copy of the grid
    let mut virtual_grid = *grid;

    // Perform virtual swap
    let temp = virtual_grid[pos1.1][pos1.0];
    virtual_grid[pos1.1][pos1.0] = virtual_grid[pos2.1][pos2.0];
    virtual_grid[pos2.1][pos2.0] = temp;

    // Check for matches at both swapped positions
    check_match_at_position(&virtual_grid, pos1.0, pos1.1)
        || check_match_at_position(&virtual_grid, pos2.0, pos2.1)
}

/// Check if there's a match (3+ in a row) at the given position
fn check_match_at_position(
    grid: &[[Option<TileType>; PUZZLE_BOARD_SIZE]; PUZZLE_BOARD_SIZE],
    x: usize,
    y: usize,
) -> bool {
    let Some(tile_type) = grid[y][x] else {
        return false;
    };

    // Check horizontal match
    let mut h_count = 1;
    // Count left
    let mut nx = x as i32 - 1;
    while nx >= 0 && grid[y][nx as usize] == Some(tile_type) {
        h_count += 1;
        nx -= 1;
    }
    // Count right
    let mut nx = x + 1;
    while nx < PUZZLE_BOARD_SIZE && grid[y][nx] == Some(tile_type) {
        h_count += 1;
        nx += 1;
    }
    if h_count >= 3 {
        return true;
    }

    // Check vertical match
    let mut v_count = 1;
    // Count down
    let mut ny = y as i32 - 1;
    while ny >= 0 && grid[ny as usize][x] == Some(tile_type) {
        v_count += 1;
        ny -= 1;
    }
    // Count up
    let mut ny = y + 1;
    while ny < PUZZLE_BOARD_SIZE && grid[ny][x] == Some(tile_type) {
        v_count += 1;
        ny += 1;
    }
    v_count >= 3
}

/// Build a TileType grid from Query for use with would_match_after_swap
pub fn build_tile_grid(
    tiles: &[(Entity, &GridPosition, &TileType)],
) -> [[Option<TileType>; PUZZLE_BOARD_SIZE]; PUZZLE_BOARD_SIZE] {
    let mut grid: [[Option<TileType>; PUZZLE_BOARD_SIZE]; PUZZLE_BOARD_SIZE] =
        [[None; PUZZLE_BOARD_SIZE]; PUZZLE_BOARD_SIZE];
    for (_, pos, tile_type) in tiles {
        grid[pos.y][pos.x] = Some(**tile_type);
    }
    grid
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

    // Defuse bombs adjacent to matched tiles (similar to ice melt)
    for (x, y) in &matched_positions {
        for (dx, dy) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
            let nx = *x as i32 + dx;
            let ny = *y as i32 + dy;
            if nx >= 0 && ny >= 0 && (nx as usize) < PUZZLE_BOARD_SIZE && (ny as usize) < PUZZLE_BOARD_SIZE {
                if board.has_bomb(nx as usize, ny as usize) {
                    commands.trigger(BombDefuseEvent { position: (nx as usize, ny as usize) });
                    board.clear_obstacle(nx as usize, ny as usize);
                }
            }
        }
    }

    // Despawn matched tiles (despawn_recursive removes child bombs too)
    for (entity, pos) in matched.iter() {
        board.set(pos.x, pos.y, None);
        commands.entity(entity).despawn_recursive();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_grid() -> [[Option<TileType>; PUZZLE_BOARD_SIZE]; PUZZLE_BOARD_SIZE] {
        [[None; PUZZLE_BOARD_SIZE]; PUZZLE_BOARD_SIZE]
    }

    #[test]
    fn test_would_match_horizontal_swap_creates_match() {
        // Setup: R R _ R G G G _
        //        Row 0: Red at 0, Red at 1, Red at 3
        //        Swapping (2,0) with (3,0) should create R R R
        let mut grid = empty_grid();
        grid[0][0] = Some(TileType::Red);
        grid[0][1] = Some(TileType::Red);
        grid[0][2] = Some(TileType::Green);
        grid[0][3] = Some(TileType::Red);

        // Swap (2,0) Green with (3,0) Red -> R R R G
        assert!(would_match_after_swap(&grid, (2, 0), (3, 0)));
    }

    #[test]
    fn test_would_match_vertical_swap_creates_match() {
        // Setup: Column 0 has Red at y=0, Red at y=1, Green at y=2, Red at y=3
        //        Swapping (0,2) with (0,3) should create vertical R R R
        let mut grid = empty_grid();
        grid[0][0] = Some(TileType::Red);
        grid[1][0] = Some(TileType::Red);
        grid[2][0] = Some(TileType::Green);
        grid[3][0] = Some(TileType::Red);

        // Swap (0,2) Green with (0,3) Red -> vertical R R R G
        assert!(would_match_after_swap(&grid, (0, 2), (0, 3)));
    }

    #[test]
    fn test_would_match_no_match_returns_false() {
        // Setup: All different colors, no possible match
        let mut grid = empty_grid();
        grid[0][0] = Some(TileType::Red);
        grid[0][1] = Some(TileType::Blue);
        grid[0][2] = Some(TileType::Green);
        grid[0][3] = Some(TileType::Yellow);

        // Swapping any two won't create a match
        assert!(!would_match_after_swap(&grid, (0, 0), (1, 0)));
    }

    #[test]
    fn test_would_match_swap_completes_match_at_destination() {
        // Setup: B B _ R R R
        //        Swapping Blue into position to complete Red match
        let mut grid = empty_grid();
        grid[0][0] = Some(TileType::Blue);
        grid[0][1] = Some(TileType::Blue);
        grid[0][2] = Some(TileType::Red);  // This will be swapped
        grid[0][3] = Some(TileType::Red);
        grid[0][4] = Some(TileType::Red);
        grid[0][5] = Some(TileType::Blue);

        // Swap (2,0) with (5,0) - but they're not adjacent, so this tests the logic
        // Let's use adjacent swap: swap (1,0) Blue with (2,0) Red
        // Result: B R B R R R - no match at (1,0), but check (2,0) which is now Blue
        // Actually, we need: R R _ R -> swap middle

        // Better test: R _ R R (swap pos 1 with something that makes R R R R)
        let mut grid2 = empty_grid();
        grid2[0][0] = Some(TileType::Red);
        grid2[0][1] = Some(TileType::Blue);
        grid2[0][2] = Some(TileType::Red);
        grid2[0][3] = Some(TileType::Red);
        grid2[1][1] = Some(TileType::Red);  // Red below position (1,0)

        // Swap (1,0) Blue with (1,1) Red -> R R R R at row 0
        assert!(would_match_after_swap(&grid2, (1, 0), (1, 1)));
    }

    #[test]
    fn test_would_match_with_longer_match() {
        // Setup: R R R R _ (4 in a row after swap)
        let mut grid = empty_grid();
        grid[0][0] = Some(TileType::Red);
        grid[0][1] = Some(TileType::Red);
        grid[0][2] = Some(TileType::Blue);
        grid[0][3] = Some(TileType::Red);
        grid[0][4] = Some(TileType::Red);
        grid[1][2] = Some(TileType::Red);

        // Swap (2,0) Blue with (2,1) Red -> 5 Reds in a row
        assert!(would_match_after_swap(&grid, (2, 0), (2, 1)));
    }

    #[test]
    fn test_would_match_empty_positions() {
        let grid = empty_grid();
        // Swapping empty positions should return false
        assert!(!would_match_after_swap(&grid, (0, 0), (1, 0)));
    }

    #[test]
    fn test_check_match_at_position_horizontal() {
        let mut grid = empty_grid();
        grid[0][0] = Some(TileType::Red);
        grid[0][1] = Some(TileType::Red);
        grid[0][2] = Some(TileType::Red);

        assert!(check_match_at_position(&grid, 0, 0));
        assert!(check_match_at_position(&grid, 1, 0));
        assert!(check_match_at_position(&grid, 2, 0));
    }

    #[test]
    fn test_check_match_at_position_vertical() {
        let mut grid = empty_grid();
        grid[0][0] = Some(TileType::Blue);
        grid[1][0] = Some(TileType::Blue);
        grid[2][0] = Some(TileType::Blue);

        assert!(check_match_at_position(&grid, 0, 0));
        assert!(check_match_at_position(&grid, 0, 1));
        assert!(check_match_at_position(&grid, 0, 2));
    }

    #[test]
    fn test_check_match_at_position_no_match() {
        let mut grid = empty_grid();
        grid[0][0] = Some(TileType::Red);
        grid[0][1] = Some(TileType::Blue);
        grid[0][2] = Some(TileType::Red);

        assert!(!check_match_at_position(&grid, 0, 0));
        assert!(!check_match_at_position(&grid, 1, 0));
        assert!(!check_match_at_position(&grid, 2, 0));
    }
}
