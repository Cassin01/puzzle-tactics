use crate::prelude::*;

#[derive(Component)]
pub struct Tile;

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum TileType {
    Red,
    Blue,
    Green,
    Yellow,
    Purple,
}

impl TileType {
    pub fn random() -> Self {
        use rand::Rng;
        match rand::thread_rng().gen_range(0..5) {
            0 => TileType::Red,
            1 => TileType::Blue,
            2 => TileType::Green,
            3 => TileType::Yellow,
            _ => TileType::Purple,
        }
    }

    pub fn color(&self) -> Color {
        match self {
            TileType::Red => Color::srgb(0.9, 0.2, 0.2),
            TileType::Blue => Color::srgb(0.2, 0.4, 0.9),
            TileType::Green => Color::srgb(0.2, 0.8, 0.3),
            TileType::Yellow => Color::srgb(0.9, 0.8, 0.2),
            TileType::Purple => Color::srgb(0.7, 0.3, 0.8),
        }
    }
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub struct GridPosition {
    pub x: usize,
    pub y: usize,
}

impl GridPosition {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

#[derive(Component)]
pub struct Matched;

#[derive(Component)]
pub struct Falling {
    pub target_y: f32,
}

#[derive(Component)]
pub struct Selected;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ObstacleType {
    Ice,
    Bomb,
}

#[derive(Component)]
pub struct Obstacle {
    pub obstacle_type: ObstacleType,
    pub countdown: Option<u8>,
}

impl Obstacle {
    pub fn ice() -> Self {
        Self {
            obstacle_type: ObstacleType::Ice,
            countdown: None,
        }
    }

    pub fn bomb(countdown: u8) -> Self {
        Self {
            obstacle_type: ObstacleType::Bomb,
            countdown: Some(countdown),
        }
    }

    pub fn is_ice(&self) -> bool {
        self.obstacle_type == ObstacleType::Ice
    }

    pub fn is_bomb(&self) -> bool {
        self.obstacle_type == ObstacleType::Bomb
    }
}
