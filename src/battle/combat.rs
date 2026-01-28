use crate::prelude::*;
use crate::puzzle::TileType;
use super::{Unit, UnitStats, UnitType, HexPosition, BattleGrid, Team, Target, AttackCooldown};

#[derive(Component)]
pub struct AttackLine {
    pub timer: Timer,
}

pub fn targeting_system(
    units: Query<(Entity, &HexPosition, &Team), With<Unit>>,
    mut targets: Query<&mut Target, With<Unit>>,
) {
    let unit_data: Vec<(Entity, HexPosition, Team)> = units
        .iter()
        .map(|(e, p, t)| (e, *p, *t))
        .collect();

    for (entity, pos, team) in &unit_data {
        let mut closest: Option<(Entity, i32)> = None;

        for (other_entity, other_pos, other_team) in &unit_data {
            if entity == other_entity || team == other_team {
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
    positions: Query<&HexPosition, With<Unit>>,
    mut param_set: ParamSet<(
        Query<(&HexPosition, &UnitStats, &Target, &mut AttackCooldown), With<Unit>>,
        Query<&mut UnitStats, With<Unit>>,
    )>,
) {
    let attacks: Vec<(HexPosition, Entity, f32)> = {
        let attackers = param_set.p0();
        attackers
            .iter()
            .filter_map(|(pos, stats, target, cooldown)| {
                if cooldown.0 <= 0.0 {
                    target.0.map(|t| {
                        let is_crit = rand::random::<f32>() < stats.crit_chance;
                        let damage = if is_crit { stats.attack * 1.5 } else { stats.attack };
                        (*pos, t, damage)
                    })
                } else {
                    None
                }
            })
            .collect()
    };

    {
        let mut targets = param_set.p1();
        for (attacker_pos, target_entity, damage) in &attacks {
            if let Ok(mut target_stats) = targets.get_mut(*target_entity) {
                target_stats.take_damage(*damage);
            }
            if let Ok(target_pos) = positions.get(*target_entity) {
                let from = grid.axial_to_pixel(attacker_pos);
                let to = grid.axial_to_pixel(target_pos);
                spawn_attack_line(&mut commands, from, to);
            }
        }
    }

    {
        let mut attackers = param_set.p0();
        for (_pos, stats, _target, mut cooldown) in attackers.iter_mut() {
            cooldown.0 -= time.delta_secs();
            if cooldown.0 <= 0.0 {
                cooldown.0 = 1.0 / stats.attack_speed;
            }
        }
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

    // Collect potential targets for offensive abilities
    let all_units: Vec<(Entity, HexPosition, f32, Team)> = {
        let units = param_set.p1();
        units
            .iter()
            .map(|(e, pos, stats, team)| (e, *pos, stats.health, *team))
            .collect()
    };

    // Determine damage to apply
    let mut damage_list: Vec<(Entity, f32)> = Vec::new();

    for (caster_entity, caster_pos, attack, ability_power, max_health, tile_type, caster_team) in &casters {
        match tile_type {
            TileType::Red => {
                // Warrior: AoE damage to enemies within 1 hex
                for (target_entity, target_pos, _, target_team) in &all_units {
                    if target_team == caster_team || target_entity == caster_entity {
                        continue;
                    }
                    if caster_pos.distance(target_pos) <= 1 {
                        damage_list.push((*target_entity, attack * 0.5));
                    }
                }
            }
            TileType::Yellow => {
                // Assassin: Attack lowest HP enemy with 2x damage
                let lowest_hp_enemy = all_units
                    .iter()
                    .filter(|(e, _, _, team)| team != caster_team && e != caster_entity)
                    .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
                if let Some((target_entity, _, _, _)) = lowest_hp_enemy {
                    damage_list.push((*target_entity, attack * 2.0));
                }
            }
            TileType::Purple => {
                // Mage: Magic attack on random enemy
                let enemies: Vec<_> = all_units
                    .iter()
                    .filter(|(e, _, _, team)| team != caster_team && e != caster_entity)
                    .collect();
                if !enemies.is_empty() {
                    use rand::Rng;
                    let idx = rand::thread_rng().gen_range(0..enemies.len());
                    let (target_entity, _, _, _) = enemies[idx];
                    damage_list.push((*target_entity, ability_power * 1.5));
                }
            }
            _ => {}
        }
    }

    // Apply damage and self-effects
    {
        let mut units = param_set.p1();
        for (target_entity, damage) in damage_list {
            if let Ok((_, _, mut stats, _)) = units.get_mut(target_entity) {
                stats.take_damage(damage);
            }
        }
    }

    // Apply self-buffs and reset mana
    {
        let mut units = param_set.p0();
        for (caster_entity, _, _, _, max_health, tile_type, _) in casters {
            if let Ok((_, _, mut stats, _, _)) = units.get_mut(caster_entity) {
                match tile_type {
                    TileType::Blue => {
                        // Tank: Heal 20% max HP
                        let heal = max_health * 0.2;
                        stats.health = (stats.health + heal).min(stats.max_health);
                    }
                    TileType::Green => {
                        // Ranger: Attack speed boost (instant buff)
                        stats.attack_speed *= 1.3;
                    }
                    _ => {}
                }
                stats.mana = 0.0;
            }
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
