use crate::prelude::*;
use crate::puzzle::TileType;
use super::{
    Unit, UnitStats, UnitType, StarRank, Team, BattleGrid, HexPosition,
    Target, AttackCooldown,
};

#[derive(Resource)]
pub struct WaveManager {
    pub current_wave: u32,
    pub enemies_remaining: u32,
    pub wave_timer: f32,
    pub spawn_delay: f32,
    pub wave_active: bool,
}

impl Default for WaveManager {
    fn default() -> Self {
        Self {
            current_wave: 0,
            enemies_remaining: 0,
            wave_timer: 3.0,
            spawn_delay: 0.0,
            wave_active: false,
        }
    }
}

impl WaveManager {
    pub fn start_wave(&mut self, wave_number: u32) {
        self.current_wave = wave_number;
        self.wave_active = true;
        self.spawn_delay = 0.5;
        self.enemies_remaining = self.enemies_for_wave(wave_number);
    }

    pub fn enemies_for_wave(&self, wave: u32) -> u32 {
        (3 + wave * 2).min(12)
    }

    pub fn enemy_star_rank(&self, wave: u32) -> u8 {
        match wave {
            0..=2 => 1,
            3..=5 => if rand::random::<f32>() < 0.3 { 2 } else { 1 },
            _ => if rand::random::<f32>() < 0.5 { 2 } else { 1 },
        }
    }

    pub fn random_enemy_type() -> TileType {
        use rand::Rng;
        match rand::thread_rng().gen_range(0..5) {
            0 => TileType::Red,
            1 => TileType::Blue,
            2 => TileType::Green,
            3 => TileType::Yellow,
            _ => TileType::Purple,
        }
    }
}

pub fn wave_spawner_system(
    time: Res<Time>,
    mut wave_manager: ResMut<WaveManager>,
    mut commands: Commands,
    mut grid: ResMut<BattleGrid>,
    enemy_units: Query<Entity, (With<Unit>, With<Team>)>,
) {
    let _enemy_count = enemy_units.iter().count();

    if !wave_manager.wave_active {
        wave_manager.wave_timer -= time.delta_secs();
        if wave_manager.wave_timer <= 0.0 {
            let next_wave = wave_manager.current_wave + 1;
            wave_manager.start_wave(next_wave);
            wave_manager.wave_timer = 10.0;
        }
        return;
    }

    if wave_manager.enemies_remaining == 0 {
        wave_manager.wave_active = false;
        return;
    }

    wave_manager.spawn_delay -= time.delta_secs();
    if wave_manager.spawn_delay > 0.0 {
        return;
    }

    if let Some(pos) = find_enemy_spawn_position(&grid) {
        let unit_type = WaveManager::random_enemy_type();
        let star_rank = wave_manager.enemy_star_rank(wave_manager.current_wave);
        spawn_enemy_unit(&mut commands, &mut grid, unit_type, star_rank, pos);
        wave_manager.enemies_remaining -= 1;
        wave_manager.spawn_delay = 0.8;
    }
}

fn find_enemy_spawn_position(grid: &BattleGrid) -> Option<HexPosition> {
    for r in 1..=BATTLE_GRID_ROWS / 2 {
        for q in -BATTLE_GRID_COLS / 2..=BATTLE_GRID_COLS / 2 {
            let pos = HexPosition::new(q, r);
            if grid.is_valid_position(&pos) && !grid.is_occupied(&pos) {
                return Some(pos);
            }
        }
    }
    None
}

fn spawn_enemy_unit(
    commands: &mut Commands,
    grid: &mut ResMut<BattleGrid>,
    unit_type: TileType,
    star_rank: u8,
    pos: HexPosition,
) {
    let stats = UnitStats::for_type(unit_type, star_rank);
    let world_pos = grid.axial_to_pixel(&pos);
    let size = 30.0 + (star_rank as f32 * 5.0);

    let mut color = unit_type.color();
    color = color.darker(0.3);

    let entity = commands
        .spawn((
            Unit,
            UnitType(unit_type),
            StarRank(star_rank),
            stats,
            pos,
            Team::Enemy,
            Target(None),
            AttackCooldown(0.0),
            Sprite {
                color,
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_translation(world_pos.extend(1.0)),
        ))
        .id();

    grid.place_unit(pos, entity);
}
