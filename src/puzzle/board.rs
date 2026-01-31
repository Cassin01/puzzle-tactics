use crate::prelude::*;
use crate::prelude::WindowSize;
use super::tile::{Tile, TileType, GridPosition, ObstacleType};

#[derive(Resource)]
pub struct PuzzleBoard {
    pub grid: [[Option<Entity>; PUZZLE_BOARD_SIZE]; PUZZLE_BOARD_SIZE],
    pub obstacles: [[Option<ObstacleType>; PUZZLE_BOARD_SIZE]; PUZZLE_BOARD_SIZE],
    pub tile_size: f32,
    pub origin: Vec2,
}

impl Default for PuzzleBoard {
    fn default() -> Self {
        Self {
            grid: [[None; PUZZLE_BOARD_SIZE]; PUZZLE_BOARD_SIZE],
            obstacles: [[None; PUZZLE_BOARD_SIZE]; PUZZLE_BOARD_SIZE],
            tile_size: TILE_SIZE,
            origin: Vec2::new(
                WINDOW_WIDTH / 4.0 - ((PUZZLE_BOARD_SIZE as f32 * (TILE_SIZE + TILE_GAP)) / 2.0) + (TILE_SIZE / 2.0),
                -((PUZZLE_BOARD_SIZE as f32 * (TILE_SIZE + TILE_GAP)) / 2.0) + (TILE_SIZE / 2.0),
            ),
        }
    }
}

pub const CORE_POSITIONS: [(usize, usize); 4] = [(3, 3), (3, 4), (4, 3), (4, 4)];

impl PuzzleBoard {
    /// Calculate origin based on window width (puzzle board on right side)
    pub fn calculate_origin(window_width: f32) -> Vec2 {
        Vec2::new(
            window_width / 4.0 - ((PUZZLE_BOARD_SIZE as f32 * (TILE_SIZE + TILE_GAP)) / 2.0) + (TILE_SIZE / 2.0),
            -((PUZZLE_BOARD_SIZE as f32 * (TILE_SIZE + TILE_GAP)) / 2.0) + (TILE_SIZE / 2.0),
        )
    }

    /// Update origin based on new window dimensions
    pub fn update_origin(&mut self, window_width: f32, _window_height: f32) {
        self.origin = Self::calculate_origin(window_width);
    }

    pub fn is_core_position(x: usize, y: usize) -> bool {
        CORE_POSITIONS.contains(&(x, y))
    }

    pub fn is_adjacent_to_core(x: usize, y: usize) -> bool {
        for (cx, cy) in CORE_POSITIONS {
            let dx = (x as i32 - cx as i32).abs();
            let dy = (y as i32 - cy as i32).abs();
            if (dx == 1 && dy == 0) || (dx == 0 && dy == 1) {
                return true;
            }
        }
        false
    }

    pub fn grid_to_world(&self, x: usize, y: usize) -> Vec2 {
        Vec2::new(
            self.origin.x + x as f32 * (self.tile_size + TILE_GAP),
            self.origin.y + y as f32 * (self.tile_size + TILE_GAP),
        )
    }

    pub fn world_to_grid(&self, pos: Vec2) -> Option<(usize, usize)> {
        let local = pos - self.origin + Vec2::splat((self.tile_size + TILE_GAP) / 2.0);
        let x = (local.x / (self.tile_size + TILE_GAP)).floor() as i32;
        let y = (local.y / (self.tile_size + TILE_GAP)).floor() as i32;

        if x >= 0 && x < PUZZLE_BOARD_SIZE as i32 && y >= 0 && y < PUZZLE_BOARD_SIZE as i32 {
            Some((x as usize, y as usize))
        } else {
            None
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<Entity> {
        self.grid.get(y).and_then(|row| row.get(x)).copied().flatten()
    }

    pub fn set(&mut self, x: usize, y: usize, entity: Option<Entity>) {
        if y < PUZZLE_BOARD_SIZE && x < PUZZLE_BOARD_SIZE {
            self.grid[y][x] = entity;
        }
    }

    pub fn swap(&mut self, a: (usize, usize), b: (usize, usize)) {
        // Swap tile entities
        let temp = self.grid[a.1][a.0];
        self.grid[a.1][a.0] = self.grid[b.1][b.0];
        self.grid[b.1][b.0] = temp;

        // Swap obstacles (bombs move with tiles)
        let temp_obstacle = self.obstacles[a.1][a.0];
        self.obstacles[a.1][a.0] = self.obstacles[b.1][b.0];
        self.obstacles[b.1][b.0] = temp_obstacle;
    }

    pub fn get_obstacle(&self, x: usize, y: usize) -> Option<ObstacleType> {
        self.obstacles.get(y).and_then(|row| row.get(x)).copied().flatten()
    }

    pub fn set_obstacle(&mut self, x: usize, y: usize, obstacle: Option<ObstacleType>) {
        if y < PUZZLE_BOARD_SIZE && x < PUZZLE_BOARD_SIZE {
            self.obstacles[y][x] = obstacle;
        }
    }

    pub fn has_ice(&self, x: usize, y: usize) -> bool {
        self.get_obstacle(x, y) == Some(ObstacleType::Ice)
    }

    pub fn has_bomb(&self, x: usize, y: usize) -> bool {
        self.get_obstacle(x, y) == Some(ObstacleType::Bomb)
    }

    pub fn clear_obstacle(&mut self, x: usize, y: usize) {
        self.set_obstacle(x, y, None);
    }
}

/// System to update puzzle board origin when window size changes
pub fn update_puzzle_board_on_resize(
    window_size: Res<WindowSize>,
    mut board: ResMut<PuzzleBoard>,
) {
    if window_size.is_changed() {
        board.update_origin(window_size.width, window_size.height);
    }
}

/// System to reposition tiles when board origin changes
pub fn reposition_tiles_on_resize(
    board: Res<PuzzleBoard>,
    mut tiles: Query<(&GridPosition, &mut Transform), With<Tile>>,
) {
    if board.is_changed() {
        for (grid_pos, mut transform) in tiles.iter_mut() {
            let new_pos = board.grid_to_world(grid_pos.x, grid_pos.y);
            transform.translation.x = new_pos.x;
            transform.translation.y = new_pos.y;
        }
    }
}

pub fn setup_puzzle_board(mut commands: Commands) {
    let mut board = PuzzleBoard::default();

    for y in 0..PUZZLE_BOARD_SIZE {
        for x in 0..PUZZLE_BOARD_SIZE {
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
                    Transform::from_translation(pos.extend(0.1)),
                    Visibility::default(),
                ))
                .id();

            board.set(x, y, Some(entity));
        }
    }

    commands.insert_resource(board);
}
