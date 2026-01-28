use crate::prelude::*;
use crate::puzzle::TileType;
use super::{Unit, UnitStats, UnitType, HexPosition, BattleGrid, Team, Target, AttackCooldown};

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
    time: Res<Time>,
    mut param_set: ParamSet<(
        Query<(&HexPosition, &UnitStats, &Target, &mut AttackCooldown), With<Unit>>,
        Query<&mut UnitStats, With<Unit>>,
    )>,
) {
    let attacks: Vec<(Entity, f32)> = {
        let attackers = param_set.p0();
        attackers
            .iter()
            .filter_map(|(_pos, stats, target, cooldown)| {
                if cooldown.0 <= 0.0 {
                    target.0.map(|t| (t, stats.attack))
                } else {
                    None
                }
            })
            .collect()
    };

    {
        let mut targets = param_set.p1();
        for (target_entity, damage) in attacks {
            if let Ok(mut target_stats) = targets.get_mut(target_entity) {
                target_stats.take_damage(damage);
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

pub fn ability_system(
    mut units: Query<(&mut UnitStats, &UnitType), With<Unit>>,
) {
    for (mut stats, unit_type) in units.iter_mut() {
        if !stats.can_cast() {
            continue;
        }

        match unit_type.0 {
            TileType::Red => {}
            TileType::Blue => {
                let heal = stats.max_health * 0.2;
                stats.health = (stats.health + heal).min(stats.max_health);
            }
            TileType::Purple => {}
            _ => {}
        }

        stats.mana = 0.0;
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
            commands.entity(entity).despawn();
        }
    }
}
