pub use bevy::prelude::*;
pub use bevy::math::primitives::Triangle2d;
pub use bevy::sprite::ColorMaterial;
pub use crate::state::{GameState, PhaseState, ComboCounter, TimeScale, SlowMoEvent, WaveBreakTimer};

// Shared types from puzzle module (re-exported for battle module to avoid direct dependency)
pub use crate::puzzle::{TileType, ObstacleType, GridPosition, Obstacle, PuzzleBoard};

// Default window size (used as fallback before dynamic sizing takes effect)
pub const WINDOW_WIDTH: f32 = 1400.0;
pub const WINDOW_HEIGHT: f32 = 800.0;

/// Resource to track dynamic window size
#[derive(Resource)]
pub struct WindowSize {
    pub width: f32,
    pub height: f32,
}

impl Default for WindowSize {
    fn default() -> Self {
        Self {
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
        }
    }
}

pub const PUZZLE_BOARD_SIZE: usize = 8;
pub const TILE_SIZE: f32 = 64.0;
pub const TILE_GAP: f32 = 4.0;

pub const HEX_SIZE: f32 = 55.0;
pub const BATTLE_GRID_ROWS: i32 = 4;
pub const BATTLE_GRID_COLS: i32 = 7;
