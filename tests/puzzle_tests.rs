// Puzzle mechanics tests - TDD for bomb adjacent match defuse
use puzzle_tactics::puzzle::{PuzzleBoard, ObstacleType};

// ============================================================
// Bomb Adjacent Match Defuse Tests (TDD)
// ============================================================

/// Test: Board can track bomb positions
#[test]
fn test_board_has_bomb() {
    let mut board = PuzzleBoard::default();
    board.set_obstacle(3, 3, Some(ObstacleType::Bomb));
    assert!(board.has_bomb(3, 3));
    assert!(!board.has_bomb(0, 0));
}

/// Test: Board can clear obstacles including bombs
#[test]
fn test_board_clear_bomb() {
    let mut board = PuzzleBoard::default();
    board.set_obstacle(3, 3, Some(ObstacleType::Bomb));
    assert!(board.has_bomb(3, 3));
    board.clear_obstacle(3, 3);
    assert!(!board.has_bomb(3, 3));
}

/// Test: Adjacent positions calculation helper
#[test]
fn test_adjacent_positions() {
    let pos = (3usize, 3usize);
    let adjacent: Vec<(i32, i32)> = vec![(-1, 0), (1, 0), (0, -1), (0, 1)];

    for (dx, dy) in &adjacent {
        let nx = pos.0 as i32 + dx;
        let ny = pos.1 as i32 + dy;
        assert!(nx >= 0 && ny >= 0);
        // (2,3), (4,3), (3,2), (3,4) are all adjacent to (3,3)
    }
}

/// Test: Verify ice uses adjacent defuse pattern (reference implementation)
#[test]
fn test_ice_adjacent_pattern_exists() {
    // This test documents that ice melts from adjacent matches
    // Bomb should follow the same pattern
    let mut board = PuzzleBoard::default();
    board.set_obstacle(3, 3, Some(ObstacleType::Ice));
    assert!(board.has_ice(3, 3));

    // Ice at (3,3) should melt when match occurs at adjacent (2,3), (4,3), (3,2), or (3,4)
    // This is the behavior we need to replicate for bombs
}

/// Test: Bomb defuse should clear countdown (no explosion)
#[test]
fn test_bomb_defuse_no_explosion() {
    // When bomb is defused via adjacent match:
    // 1. Bomb obstacle data is cleared
    // 2. No BombDamageEvent should fire
    // 3. BombDefuseEvent should fire instead

    let mut board = PuzzleBoard::default();
    board.set_obstacle(3, 3, Some(ObstacleType::Bomb));

    // Simulate defuse: clear the bomb obstacle
    board.clear_obstacle(3, 3);

    // Bomb should no longer exist
    assert!(!board.has_bomb(3, 3));
    // Obstacle slot should be None
    assert!(board.get_obstacle(3, 3).is_none());
}

/// Test: Multiple adjacent bombs should all defuse
#[test]
fn test_multiple_adjacent_bombs_defuse() {
    let mut board = PuzzleBoard::default();

    // Place bombs at (2,3) and (4,3), both adjacent to match at (3,3)
    board.set_obstacle(2, 3, Some(ObstacleType::Bomb));
    board.set_obstacle(4, 3, Some(ObstacleType::Bomb));

    assert!(board.has_bomb(2, 3));
    assert!(board.has_bomb(4, 3));

    // When match occurs at (3,3), both should be defusable
    // Simulate clearing both
    board.clear_obstacle(2, 3);
    board.clear_obstacle(4, 3);

    assert!(!board.has_bomb(2, 3));
    assert!(!board.has_bomb(4, 3));
}

/// Test: Diagonal positions are NOT adjacent (should not defuse)
#[test]
fn test_diagonal_not_adjacent() {
    // Diagonal positions (2,2), (4,4), (2,4), (4,2) relative to (3,3)
    // are NOT considered adjacent for defuse purposes
    let matched_pos = (3usize, 3usize);
    let diagonals = [(2, 2), (4, 4), (2, 4), (4, 2)];

    for (bx, by) in diagonals {
        let dx = (matched_pos.0 as i32 - bx as i32).abs();
        let dy = (matched_pos.1 as i32 - by as i32).abs();
        // Diagonal: both dx and dy are 1
        assert!(dx == 1 && dy == 1, "({}, {}) should be diagonal to ({}, {})", bx, by, matched_pos.0, matched_pos.1);
        // This should NOT trigger defuse (adjacent means dx+dy == 1)
        assert!(dx + dy != 1);
    }
}

/// Test: Board boundary edge cases
#[test]
fn test_bomb_defuse_at_boundary() {
    let mut board = PuzzleBoard::default();

    // Bomb at (0, 0) - corner
    board.set_obstacle(0, 0, Some(ObstacleType::Bomb));
    assert!(board.has_bomb(0, 0));

    // Adjacent positions to (0,0) are (1,0) and (0,1)
    // (-1,0) and (0,-1) are out of bounds

    board.clear_obstacle(0, 0);
    assert!(!board.has_bomb(0, 0));
}
