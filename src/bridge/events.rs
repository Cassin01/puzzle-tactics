use crate::prelude::*;
use crate::puzzle::TileType;
use crate::battle::{
    Unit, UnitStats, UnitType, StarRank, Team, BattleGrid, HexPosition,
    Target, AttackCooldown,
};

#[derive(Event)]
pub struct MatchEvent {
    pub tile_type: TileType,
    pub count: usize,
    pub positions: Vec<(usize, usize)>,
}

#[derive(Event)]
pub struct UnitSummonEvent {
    pub unit_type: TileType,
    pub star_rank: u8,
}

#[derive(Event)]
pub struct ManaSupplyEvent {
    pub amount: f32,
}

#[derive(Event)]
pub struct CoreAbilityEvent {
    pub tile_type: TileType,
    pub count: usize,
    pub positions: Vec<(usize, usize)>,
}

#[derive(Event)]
pub struct SkillOrbEvent {
    pub orb_type: SkillOrbType,
}

#[derive(Clone, Copy, Debug)]
pub enum SkillOrbType {
    Buff,
    Meteor,
    Heal,
}

pub fn match_to_summon(trigger: Trigger<MatchEvent>, mut commands: Commands) {
    let event = trigger.event();

    let star_rank = if event.count >= 5 {
        2
    } else {
        1
    };

    commands.trigger(UnitSummonEvent {
        unit_type: event.tile_type,
        star_rank,
    });

    if event.count >= 4 {
        let orb_type = match event.tile_type {
            TileType::Red => SkillOrbType::Meteor,
            TileType::Blue => SkillOrbType::Buff,
            TileType::Green => SkillOrbType::Heal,
            TileType::Yellow => SkillOrbType::Buff,
            TileType::Purple => SkillOrbType::Meteor,
        };
        commands.trigger(SkillOrbEvent { orb_type });
    }
}

pub fn summon_unit(
    trigger: Trigger<UnitSummonEvent>,
    mut commands: Commands,
    mut grid: ResMut<BattleGrid>,
    existing_units: Query<(Entity, &UnitType, &StarRank, &HexPosition), With<Unit>>,
) {
    let event = trigger.event();

    let mut same_type_units: Vec<(Entity, u8, HexPosition)> = existing_units
        .iter()
        .filter(|(_, ut, _, _)| ut.0 == event.unit_type)
        .map(|(e, _, sr, pos)| (e, sr.0, *pos))
        .collect();

    same_type_units.sort_by_key(|(_, sr, _)| *sr);

    if same_type_units.len() >= 2 && same_type_units[0].1 == same_type_units[1].1 {
        let (e1, star, pos1) = same_type_units[0];
        let (e2, _, pos2) = same_type_units[1];

        if star < 3 {
            grid.remove_unit(&pos1);
            grid.remove_unit(&pos2);
            commands.entity(e1).despawn_recursive();
            commands.entity(e2).despawn_recursive();

            if let Some(new_pos) = grid.find_empty_position() {
                spawn_unit_at(&mut commands, &mut grid, event.unit_type, star + 1, new_pos, Team::Player);
            }
            return;
        }
    }

    if let Some(pos) = grid.find_empty_position() {
        spawn_unit_at(&mut commands, &mut grid, event.unit_type, event.star_rank, pos, Team::Player);
    }
}

fn spawn_unit_at(
    commands: &mut Commands,
    grid: &mut ResMut<BattleGrid>,
    unit_type: TileType,
    star_rank: u8,
    pos: HexPosition,
    team: Team,
) {
    let stats = UnitStats::for_type(unit_type, star_rank);
    let world_pos = grid.axial_to_pixel(&pos);

    let size = 30.0 + (star_rank as f32 * 5.0);

    let entity = commands
        .spawn((
            Unit,
            UnitType(unit_type),
            StarRank(star_rank),
            stats,
            pos,
            team,
            Target(None),
            AttackCooldown(0.0),
            Sprite {
                color: unit_type.color(),
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_translation(world_pos.extend(1.0)),
        ))
        .id();

    grid.place_unit(pos, entity);
}

pub fn handle_skill_orb(
    trigger: Trigger<SkillOrbEvent>,
    mut units: Query<(&mut UnitStats, &Team), With<Unit>>,
) {
    let event = trigger.event();

    match event.orb_type {
        SkillOrbType::Buff => {
            for (mut stats, team) in units.iter_mut() {
                if *team == Team::Player {
                    stats.attack *= 1.2;
                }
            }
        }
        SkillOrbType::Heal => {
            for (mut stats, team) in units.iter_mut() {
                if *team == Team::Player {
                    let heal = stats.max_health * 0.3;
                    stats.health = (stats.health + heal).min(stats.max_health);
                }
            }
        }
        SkillOrbType::Meteor => {
            for (mut stats, team) in units.iter_mut() {
                if *team == Team::Enemy {
                    stats.take_damage(50.0);
                }
            }
        }
    }
}

pub fn handle_mana_supply(
    trigger: Trigger<ManaSupplyEvent>,
    mut units: Query<(&mut UnitStats, &Team), With<Unit>>,
) {
    let event = trigger.event();

    for (mut stats, team) in units.iter_mut() {
        if *team == Team::Player {
            stats.gain_mana(event.amount);
        }
    }
}
