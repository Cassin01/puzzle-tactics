use crate::prelude::*;
use crate::puzzle::{TileType, ObstacleType};
use crate::bridge::ObstacleSpawnEvent;
use crate::audio::AttackSoundEvent;
use super::{Unit, UnitStats, UnitType, HexPosition, BattleGrid, Team, Target, AttackCooldown, WaveManager, RageBuff, SnipeBuff, StealthBuff, MeteorAbility, DamagePopupEvent};

// ============================================================
// Damage Calculator
// ============================================================

/// Damage calculation utility for critical hits and defense reduction
pub struct DamageCalculator;

impl DamageCalculator {
    /// Critical hit multiplier (1.5x damage)
    pub const CRIT_MULTIPLIER: f32 = 1.5;
    /// Maximum defense reduction (80%)
    pub const MAX_DEFENSE_REDUCTION: f32 = 0.8;
    /// Minimum damage dealt
    pub const MIN_DAMAGE: f32 = 1.0;

    /// Returns the critical hit multiplier
    pub fn crit_multiplier() -> f32 {
        Self::CRIT_MULTIPLIER
    }

    /// Applies critical hit modifier to damage
    pub fn apply_crit(base_damage: f32, is_crit: bool) -> f32 {
        if is_crit {
            base_damage * Self::CRIT_MULTIPLIER
        } else {
            base_damage
        }
    }

    /// Applies defense reduction to damage
    /// Defense is percentage-based: defense 50 = 50% reduction
    /// Capped at 80% reduction maximum
    pub fn apply_defense(damage: f32, defense: f32) -> f32 {
        let reduction = (defense / 100.0).min(Self::MAX_DEFENSE_REDUCTION);
        damage * (1.0 - reduction)
    }

    /// Full damage calculation: base * crit_multiplier * defense_reduction
    /// Ensures minimum damage of 1.0
    pub fn calculate(base_damage: f32, is_crit: bool, defense: f32) -> f32 {
        let after_crit = Self::apply_crit(base_damage, is_crit);
        let final_damage = Self::apply_defense(after_crit, defense);
        final_damage.max(Self::MIN_DAMAGE)
    }
}

#[derive(Component)]
pub struct AttackLine {
    pub timer: Timer,
}

pub fn targeting_system(
    units: Query<(Entity, &HexPosition, &Team), With<Unit>>,
    stealth_units: Query<Entity, With<StealthBuff>>,
    mut targets: Query<&mut Target, With<Unit>>,
) {
    let unit_data: Vec<(Entity, HexPosition, Team)> = units
        .iter()
        .map(|(e, p, t)| (e, *p, *t))
        .collect();

    // Collect stealthed entities
    let stealthed: std::collections::HashSet<Entity> = stealth_units.iter().collect();

    for (entity, pos, team) in &unit_data {
        let mut closest: Option<(Entity, i32)> = None;

        for (other_entity, other_pos, other_team) in &unit_data {
            if entity == other_entity || team == other_team {
                continue;
            }

            // Skip stealthed units - they cannot be targeted
            if stealthed.contains(other_entity) {
                continue;
            }

            let dist = pos.distance(other_pos);
            if closest.is_none() || dist < closest.unwrap().1 {
                closest = Some((*other_entity, dist));
            }
        }

        if let Ok(mut target) = targets.get_mut(*entity) {
            target.0 = closest.map(|(e, _)| e);
        }
    }
}

pub fn movement_system(
    mut grid: ResMut<BattleGrid>,
    mut units: Query<(Entity, &mut HexPosition, &UnitStats, &Target, &mut Transform), With<Unit>>,
) {
    let unit_positions: std::collections::HashMap<Entity, HexPosition> = units
        .iter()
        .map(|(e, pos, _, _, _)| (e, *pos))
        .collect();

    let mut movements: Vec<(Entity, HexPosition, HexPosition)> = Vec::new();

    for (entity, pos, stats, target, _) in units.iter() {
        let Some(target_entity) = target.0 else { continue };
        let Some(target_pos) = unit_positions.get(&target_entity) else { continue };

        let distance = pos.distance(target_pos);
        if distance <= stats.attack_range {
            continue;
        }

        if let Some(next_pos) = find_best_move(&grid, &pos, target_pos) {
            if !grid.is_occupied(&next_pos) {
                movements.push((entity, *pos, next_pos));
            }
        }
    }

    for (entity, from, to) in movements {
        if grid.move_unit(&from, &to) {
            if let Ok((_, mut pos, _, _, mut transform)) = units.get_mut(entity) {
                *pos = to;
                let world_pos = grid.axial_to_pixel(&to);
                transform.translation = world_pos.extend(1.0);
            }
        }
    }
}

fn find_best_move(
    grid: &BattleGrid,
    from: &HexPosition,
    target: &HexPosition,
) -> Option<HexPosition> {
    let neighbors = from.neighbors();
    let mut best: Option<(HexPosition, i32)> = None;

    for neighbor in neighbors {
        if !grid.is_valid_position(&neighbor) || grid.is_occupied(&neighbor) {
            continue;
        }

        let dist = neighbor.distance(target);
        if best.is_none() || dist < best.unwrap().1 {
            best = Some((neighbor, dist));
        }
    }

    best.map(|(pos, _)| pos)
}

pub fn attack_system(
    mut commands: Commands,
    grid: Res<BattleGrid>,
    time: Res<Time>,
    wave_manager: Res<WaveManager>,
    positions: Query<&HexPosition, With<Unit>>,
    rage_buffs: Query<(Entity, &RageBuff), With<Unit>>,
    mut snipe_buffs: Query<(Entity, &mut SnipeBuff), With<Unit>>,
    mut param_set: ParamSet<(
        Query<(Entity, &HexPosition, &UnitStats, &Target, &mut AttackCooldown, &Team), With<Unit>>,
        Query<&mut UnitStats, With<Unit>>,
    )>,
) {
    let current_wave = wave_manager.current_wave;

    // Collect rage buff entities for damage calculation
    let rage_entities: std::collections::HashSet<Entity> = rage_buffs.iter().map(|(e, _)| e).collect();

    // Collect attacks with team info for obstacle spawning
    // Tuple: (attacker_entity, attacker_pos, target_entity, damage, team, is_critical)
    let attacks: Vec<(Entity, HexPosition, Entity, f32, Team, bool)> = {
        let attackers = param_set.p0();
        attackers
            .iter()
            .filter_map(|(entity, pos, stats, target, cooldown, team)| {
                if cooldown.0 <= 0.0 {
                    target.0.map(|t| {
                        let is_crit = rand::random::<f32>() < stats.crit_chance;
                        let mut damage = if is_crit { stats.attack * 1.5 } else { stats.attack };

                        // Apply Rage buff (ATK +20%)
                        if rage_entities.contains(&entity) {
                            damage *= RageBuff::ATTACK_MULTIPLIER;
                        }

                        (entity, *pos, t, damage, *team, is_crit)
                    })
                } else {
                    None
                }
            })
            .collect()
    };

    // Apply Snipe buff (2x damage on next attack) and consume it
    // Tuple: (attacker_pos, target_entity, damage, team, is_critical)
    let mut final_attacks: Vec<(HexPosition, Entity, f32, Team, bool)> = Vec::new();
    for (attacker_entity, attacker_pos, target_entity, mut damage, team, is_crit) in attacks {
        if let Ok((_, mut snipe)) = snipe_buffs.get_mut(attacker_entity) {
            if !snipe.is_consumed() {
                damage = snipe.apply_damage_modifier(damage / if rage_entities.contains(&attacker_entity) { RageBuff::ATTACK_MULTIPLIER } else { 1.0 });
                // Re-apply rage if present
                if rage_entities.contains(&attacker_entity) {
                    damage *= RageBuff::ATTACK_MULTIPLIER;
                }
                snipe.consume();
                // Remove the consumed snipe buff
                commands.entity(attacker_entity).remove::<SnipeBuff>();
            }
        }
        final_attacks.push((attacker_pos, target_entity, damage, team, is_crit));
    }

    // Trigger attack sound if there are attacks
    if !final_attacks.is_empty() {
        // Check if any attack was critical
        let has_critical = final_attacks.iter().any(|(_, _, _, _, is_crit)| *is_crit);
        commands.trigger(AttackSoundEvent { is_critical: has_critical });
    }

    {
        let mut targets = param_set.p1();
        for (attacker_pos, target_entity, damage, team, is_crit) in &final_attacks {
            if let Ok(mut target_stats) = targets.get_mut(*target_entity) {
                target_stats.take_damage(*damage);
            }
            if let Ok(target_pos) = positions.get(*target_entity) {
                let from = grid.axial_to_pixel(attacker_pos);
                let to = grid.axial_to_pixel(target_pos);
                spawn_attack_line(&mut commands, from, to);

                // Spawn damage popup at target position
                commands.trigger(DamagePopupEvent {
                    position: to.extend(0.0),
                    damage: *damage as i32,
                    is_critical: *is_crit,
                });
            }

            // Enemy attack triggers obstacle spawn based on wave
            if *team == Team::Enemy {
                maybe_spawn_obstacle_on_attack(&mut commands, current_wave);
            }
        }
    }

    {
        let mut attackers = param_set.p0();
        for (_entity, _pos, stats, _target, mut cooldown, _team) in attackers.iter_mut() {
            cooldown.0 -= time.delta_secs();
            if cooldown.0 <= 0.0 {
                cooldown.0 = 1.0 / stats.attack_speed;
            }
        }
    }
}

/// Spawns obstacles on the puzzle board when enemies attack
fn maybe_spawn_obstacle_on_attack(commands: &mut Commands, current_wave: u32) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    // Wave 5+: 15% chance to spawn bomb
    if current_wave >= 5 && rng.gen::<f32>() < 0.15 {
        let x = rng.gen_range(0..PUZZLE_BOARD_SIZE);
        let y = rng.gen_range(0..PUZZLE_BOARD_SIZE);
        commands.trigger(ObstacleSpawnEvent {
            position: (x, y),
            obstacle_type: ObstacleType::Bomb,
            countdown: Some(3),
        });
        return;
    }

    // Wave 3+: 10% chance to spawn ice
    if current_wave >= 3 && rng.gen::<f32>() < 0.10 {
        let x = rng.gen_range(0..PUZZLE_BOARD_SIZE);
        let y = rng.gen_range(0..PUZZLE_BOARD_SIZE);
        commands.trigger(ObstacleSpawnEvent {
            position: (x, y),
            obstacle_type: ObstacleType::Ice,
            countdown: None,
        });
    }
}

fn spawn_attack_line(commands: &mut Commands, from: Vec2, to: Vec2) {
    let diff = to - from;
    let length = diff.length();
    let angle = diff.y.atan2(diff.x);
    let mid = (from + to) / 2.0;

    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 0.3, 0.3),
            custom_size: Some(Vec2::new(length, 2.0)),
            ..default()
        },
        Transform::from_translation(mid.extend(10.0))
            .with_rotation(Quat::from_rotation_z(angle)),
        AttackLine {
            timer: Timer::from_seconds(0.1, TimerMode::Once),
        },
    ));
}

pub fn ability_system(
    mut commands: Commands,
    mut param_set: ParamSet<(
        Query<(Entity, &HexPosition, &mut UnitStats, &UnitType, &Team), With<Unit>>,
        Query<(Entity, &HexPosition, &mut UnitStats, &Team), With<Unit>>,
    )>,
) {
    // Collect caster data first
    let casters: Vec<(Entity, HexPosition, f32, f32, f32, TileType, Team)> = {
        let units = param_set.p0();
        units
            .iter()
            .filter(|(_, _, stats, _, _)| stats.can_cast())
            .map(|(e, pos, stats, ut, team)| {
                (e, *pos, stats.attack, stats.ability_power, stats.max_health, ut.0, *team)
            })
            .collect()
    };

    // Collect potential targets for offensive abilities (enemies only for Meteor)
    let all_units: Vec<(Entity, HexPosition, f32, Team)> = {
        let units = param_set.p1();
        units
            .iter()
            .map(|(e, pos, stats, team)| (e, *pos, stats.health, *team))
            .collect()
    };

    // Determine damage to apply (for Purple/Meteor only now)
    let mut damage_list: Vec<(Entity, f32)> = Vec::new();

    // Track which buffs to apply
    let mut rage_buffs_to_add: Vec<Entity> = Vec::new();
    let mut snipe_buffs_to_add: Vec<Entity> = Vec::new();
    let mut stealth_buffs_to_add: Vec<Entity> = Vec::new();

    for (caster_entity, _caster_pos, _attack, _ability_power, _max_health, tile_type, caster_team) in &casters {
        match tile_type {
            TileType::Red => {
                // Warrior: Rage - ATK +20% for 5 seconds
                rage_buffs_to_add.push(*caster_entity);
            }
            TileType::Green => {
                // Ranger: Snipe - next attack deals 2x damage
                snipe_buffs_to_add.push(*caster_entity);
            }
            TileType::Yellow => {
                // Assassin: Stealth - untargetable for 3 seconds
                stealth_buffs_to_add.push(*caster_entity);
            }
            TileType::Purple => {
                // Mage: Meteor - 15 damage to ALL enemies
                for (target_entity, _, _, target_team) in &all_units {
                    if target_team != caster_team {
                        damage_list.push((*target_entity, MeteorAbility::damage()));
                    }
                }
            }
            _ => {}
        }
    }

    // Apply Meteor damage
    {
        let mut units = param_set.p1();
        for (target_entity, damage) in damage_list {
            if let Ok((_, _, mut stats, _)) = units.get_mut(target_entity) {
                stats.take_damage(damage);
            }
        }
    }

    // Apply Blue heal and reset mana for all casters
    {
        let mut units = param_set.p0();
        for (caster_entity, _, _, _, max_health, tile_type, _) in casters {
            if let Ok((_, _, mut stats, _, _)) = units.get_mut(caster_entity) {
                if tile_type == TileType::Blue {
                    // Tank: Heal 20% max HP
                    let heal = max_health * 0.2;
                    stats.health = (stats.health + heal).min(stats.max_health);
                }
                stats.mana = 0.0;
            }
        }
    }

    // Add buff components
    for entity in rage_buffs_to_add {
        commands.entity(entity).insert(RageBuff::new());
    }
    for entity in snipe_buffs_to_add {
        commands.entity(entity).insert(SnipeBuff::new());
    }
    for entity in stealth_buffs_to_add {
        commands.entity(entity).insert(StealthBuff::new());
    }
}

/// System to tick and expire buff timers
pub fn buff_timer_system(
    mut commands: Commands,
    time: Res<Time>,
    mut rage_buffs: Query<(Entity, &mut RageBuff)>,
    mut stealth_buffs: Query<(Entity, &mut StealthBuff)>,
) {
    let delta = time.delta_secs();

    // Tick Rage buffs
    for (entity, mut buff) in rage_buffs.iter_mut() {
        buff.tick(delta);
        if buff.is_expired() {
            commands.entity(entity).remove::<RageBuff>();
        }
    }

    // Tick Stealth buffs
    for (entity, mut buff) in stealth_buffs.iter_mut() {
        buff.tick(delta);
        if buff.is_expired() {
            commands.entity(entity).remove::<StealthBuff>();
        }
    }
}

pub fn death_system(
    mut commands: Commands,
    mut grid: ResMut<BattleGrid>,
    units: Query<(Entity, &HexPosition, &UnitStats), With<Unit>>,
) {
    for (entity, pos, stats) in units.iter() {
        if stats.is_dead() {
            grid.remove_unit(pos);
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn despawn_attack_lines(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut AttackLine)>,
) {
    for (entity, mut attack_line) in query.iter_mut() {
        attack_line.timer.tick(time.delta());
        if attack_line.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}
