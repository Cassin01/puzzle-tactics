use crate::prelude::*;
use crate::puzzle::{TileType, PuzzleBoard, GridPosition, Obstacle};
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

#[derive(Event)]
pub struct BombDamageEvent {
    pub position: (usize, usize),
    pub damage: u32,
}

/// Component for bomb explosion visual effect
#[derive(Component)]
pub struct BombExplosionEffect {
    pub timer: f32,
    pub duration: f32,
}

/// Interval between bomb countdown decrements (in seconds)
/// Increase this value to slow down bomb countdown
pub const BOMB_COUNTDOWN_INTERVAL: f32 = 1.5;

/// Resource to track bomb countdown timing
#[derive(Resource)]
pub struct BombCountdownTimer {
    pub timer: f32,
}

impl Default for BombCountdownTimer {
    fn default() -> Self {
        Self { timer: 0.0 }
    }
}

pub fn bomb_countdown_system(
    mut commands: Commands,
    time: Res<Time>,
    mut countdown_timer: ResMut<BombCountdownTimer>,
    mut board: ResMut<PuzzleBoard>,
    mut obstacles: Query<(Entity, &GridPosition, &mut Obstacle)>,
) {
    // Timer-based countdown: only tick when interval elapsed
    countdown_timer.timer += time.delta_secs();
    if countdown_timer.timer < BOMB_COUNTDOWN_INTERVAL {
        return;
    }
    countdown_timer.timer = 0.0;

    for (entity, pos, mut obstacle) in obstacles.iter_mut() {
        if obstacle.is_bomb() {
            if let Some(ref mut countdown) = obstacle.countdown {
                if *countdown > 0 {
                    *countdown -= 1;
                } else {
                    // Spawn explosion effect at bomb position
                    let world_pos = board.grid_to_world(pos.x, pos.y);
                    commands.spawn((
                        BombExplosionEffect {
                            timer: 0.0,
                            duration: 0.5,
                        },
                        Sprite {
                            color: Color::srgba(1.0, 0.5, 0.0, 1.0),
                            custom_size: Some(Vec2::splat(TILE_SIZE)),
                            ..default()
                        },
                        Transform::from_translation(world_pos.extend(1.0)),
                    ));

                    // Trigger damage event
                    commands.trigger(BombDamageEvent {
                        position: (pos.x, pos.y),
                        damage: 10,
                    });
                    // Clear the obstacle from the board
                    board.clear_obstacle(pos.x, pos.y);
                    // Despawn the bomb entity entirely
                    commands.entity(entity).despawn_recursive();
                }
            }
        }
    }
}

/// Animates and removes bomb explosion effects
pub fn animate_bomb_explosion(
    mut commands: Commands,
    time: Res<Time>,
    mut effects: Query<(Entity, &mut BombExplosionEffect, &mut Transform, &mut Sprite)>,
) {
    for (entity, mut effect, mut transform, mut sprite) in effects.iter_mut() {
        effect.timer += time.delta_secs();
        let progress = (effect.timer / effect.duration).min(1.0);

        // Scale up and fade out
        let scale = 1.0 + progress * 1.5;
        transform.scale = Vec3::splat(scale);
        sprite.color = Color::srgba(1.0, 0.5 - progress * 0.3, 0.0, 1.0 - progress);

        if effect.timer >= effect.duration {
            commands.entity(entity).despawn();
        }
    }
}

pub fn handle_bomb_damage(
    trigger: Trigger<BombDamageEvent>,
    mut player_units: Query<&mut UnitStats, (With<Unit>, With<Team>)>,
) {
    let event = trigger.event();
    let damage = event.damage as f32;

    // Apply damage to player units (simplified: damage all friendly units)
    for mut stats in player_units.iter_mut() {
        stats.health = (stats.health - damage).max(0.0);
    }
}
