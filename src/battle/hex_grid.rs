use crate::prelude::*;
use crate::prelude::WindowSize;
use std::collections::HashMap;

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct HexPosition {
    pub q: i32,
    pub r: i32,
}

impl HexPosition {
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    pub fn neighbors(&self) -> [HexPosition; 6] {
        [
            HexPosition::new(self.q + 1, self.r),
            HexPosition::new(self.q + 1, self.r - 1),
            HexPosition::new(self.q, self.r - 1),
            HexPosition::new(self.q - 1, self.r),
            HexPosition::new(self.q - 1, self.r + 1),
            HexPosition::new(self.q, self.r + 1),
        ]
    }

    pub fn distance(&self, other: &HexPosition) -> i32 {
        ((self.q - other.q).abs()
            + (self.q + self.r - other.q - other.r).abs()
            + (self.r - other.r).abs())
            / 2
    }
}

#[derive(Resource, Default)]
pub struct BattleGrid {
    pub units: HashMap<HexPosition, Entity>,
    pub hex_size: f32,
    pub origin: Vec2,
}

impl BattleGrid {
    pub fn new() -> Self {
        Self {
            units: HashMap::new(),
            hex_size: HEX_SIZE,
            origin: Vec2::new(-WINDOW_WIDTH / 4.0, 0.0),
        }
    }

    /// Calculate origin based on window width (battle grid on left side)
    pub fn calculate_origin(window_width: f32) -> Vec2 {
        Vec2::new(-window_width / 4.0, 0.0)
    }

    /// Update origin based on new window dimensions
    pub fn update_origin(&mut self, window_width: f32) {
        self.origin = Self::calculate_origin(window_width);
    }

    pub fn axial_to_pixel(&self, pos: &HexPosition) -> Vec2 {
        let x = self.hex_size * (3.0_f32.sqrt() * pos.q as f32 + 3.0_f32.sqrt() / 2.0 * pos.r as f32);
        let y = self.hex_size * (3.0 / 2.0 * pos.r as f32);
        self.origin + Vec2::new(x, y)
    }

    pub fn pixel_to_axial(&self, pixel: Vec2) -> HexPosition {
        let local = pixel - self.origin;
        let q = (3.0_f32.sqrt() / 3.0 * local.x - 1.0 / 3.0 * local.y) / self.hex_size;
        let r = (2.0 / 3.0 * local.y) / self.hex_size;
        axial_round(q, r)
    }

    pub fn is_valid_position(&self, pos: &HexPosition) -> bool {
        pos.q >= -BATTLE_GRID_COLS / 2
            && pos.q <= BATTLE_GRID_COLS / 2
            && pos.r >= -BATTLE_GRID_ROWS / 2
            && pos.r <= BATTLE_GRID_ROWS / 2
    }

    pub fn is_occupied(&self, pos: &HexPosition) -> bool {
        self.units.contains_key(pos)
    }

    pub fn place_unit(&mut self, pos: HexPosition, entity: Entity) -> bool {
        if self.is_valid_position(&pos) && !self.is_occupied(&pos) {
            self.units.insert(pos, entity);
            true
        } else {
            false
        }
    }

    pub fn remove_unit(&mut self, pos: &HexPosition) -> Option<Entity> {
        self.units.remove(pos)
    }

    pub fn move_unit(&mut self, from: &HexPosition, to: &HexPosition) -> bool {
        if let Some(entity) = self.units.remove(from) {
            if !self.is_occupied(to) && self.is_valid_position(to) {
                self.units.insert(*to, entity);
                return true;
            }
            self.units.insert(*from, entity);
        }
        false
    }

    pub fn find_empty_position(&self) -> Option<HexPosition> {
        for q in -BATTLE_GRID_COLS / 2..=BATTLE_GRID_COLS / 2 {
            for r in -BATTLE_GRID_ROWS / 2..=0 {
                let pos = HexPosition::new(q, r);
                if self.is_valid_position(&pos) && !self.is_occupied(&pos) {
                    return Some(pos);
                }
            }
        }
        None
    }
}

fn axial_round(q: f32, r: f32) -> HexPosition {
    let s = -q - r;
    let mut rq = q.round();
    let mut rr = r.round();
    let rs = s.round();

    let q_diff = (rq - q).abs();
    let r_diff = (rr - r).abs();
    let s_diff = (rs - s).abs();

    if q_diff > r_diff && q_diff > s_diff {
        rq = -rr - rs;
    } else if r_diff > s_diff {
        rr = -rq - rs;
    }

    HexPosition::new(rq as i32, rr as i32)
}

pub fn setup_battle_grid(mut commands: Commands) {
    commands.insert_resource(BattleGrid::new());
}

/// System to update battle grid origin when window size changes
pub fn update_battle_grid_on_resize(
    window_size: Res<WindowSize>,
    mut grid: ResMut<BattleGrid>,
) {
    if window_size.is_changed() {
        grid.update_origin(window_size.width);
    }
}

/// System to reposition units when grid origin changes
pub fn reposition_units_on_resize(
    grid: Res<BattleGrid>,
    mut units: Query<(&HexPosition, &mut Transform)>,
) {
    if grid.is_changed() {
        for (hex_pos, mut transform) in units.iter_mut() {
            let new_pos = grid.axial_to_pixel(hex_pos);
            transform.translation.x = new_pos.x;
            transform.translation.y = new_pos.y;
        }
    }
}
