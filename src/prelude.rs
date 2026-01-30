pub use bevy::prelude::*;
pub use crate::state::{GameState, PhaseState, ComboCounter, TimeScale, SlowMoEvent};

// Shared types from puzzle module (re-exported for battle module to avoid direct dependency)
pub use crate::puzzle::{TileType, ObstacleType, GridPosition, Obstacle, PuzzleBoard};

pub const WINDOW_WIDTH: f32 = 800.0;
pub const WINDOW_HEIGHT: f32 = 900.0;

pub const PUZZLE_BOARD_SIZE: usize = 8;
pub const TILE_SIZE: f32 = 64.0;
pub const TILE_GAP: f32 = 4.0;

pub const HEX_SIZE: f32 = 40.0;
pub const BATTLE_GRID_ROWS: i32 = 4;
pub const BATTLE_GRID_COLS: i32 = 7;
