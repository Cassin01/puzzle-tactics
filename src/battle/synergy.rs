use crate::prelude::*;
use crate::puzzle::TileType;
use super::{Unit, UnitType, UnitStats, Team};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SynergyLevel {
    None,
    Bronze,
    Silver,
    Gold,
}

impl SynergyLevel {
    pub fn from_count(count: usize) -> Self {
        match count {
            0..=1 => SynergyLevel::None,
            2..=3 => SynergyLevel::Bronze,
            4..=5 => SynergyLevel::Silver,
            _ => SynergyLevel::Gold,
        }
    }

    pub fn bonus_multiplier(&self) -> f32 {
        match self {
            SynergyLevel::None => 1.0,
            SynergyLevel::Bronze => 1.15,
            SynergyLevel::Silver => 1.30,
            SynergyLevel::Gold => 1.50,
        }
    }
}

#[derive(Resource, Default)]
pub struct ActiveSynergies {
    pub bonuses: HashMap<TileType, SynergyLevel>,
}

impl ActiveSynergies {
    pub fn get_level(&self, tile_type: TileType) -> SynergyLevel {
        self.bonuses.get(&tile_type).copied().unwrap_or(SynergyLevel::None)
    }
}

pub fn update_synergies(
    mut synergies: ResMut<ActiveSynergies>,
    units: Query<(&UnitType, &Team), With<Unit>>,
) {
    let mut counts: HashMap<TileType, usize> = HashMap::new();

    for (unit_type, team) in units.iter() {
        if *team == Team::Player {
            *counts.entry(unit_type.0).or_insert(0) += 1;
        }
    }

    synergies.bonuses.clear();
    for (tile_type, count) in counts {
        let level = SynergyLevel::from_count(count);
        if level != SynergyLevel::None {
            synergies.bonuses.insert(tile_type, level);
        }
    }
}

pub fn apply_synergy_bonuses(
    synergies: Res<ActiveSynergies>,
    mut units: Query<(&UnitType, &mut UnitStats, &Team), With<Unit>>,
) {
    for (unit_type, mut stats, team) in units.iter_mut() {
        if *team != Team::Player {
            continue;
        }

        let level = synergies.get_level(unit_type.0);
        if level == SynergyLevel::None {
            continue;
        }

        let multiplier = level.bonus_multiplier();

        match unit_type.0 {
            TileType::Red => {
                // Warrior: attack +20%, health +10%
                let attack_bonus = 1.0 + 0.20 * (multiplier - 1.0) / 0.15;
                let health_bonus = 1.0 + 0.10 * (multiplier - 1.0) / 0.15;
                stats.attack *= attack_bonus;
                stats.health *= health_bonus;
                stats.max_health *= health_bonus;
            }
            TileType::Blue => {
                // Tank: health +30%, defense +15%
                let health_bonus = 1.0 + 0.30 * (multiplier - 1.0) / 0.15;
                let defense_bonus = 0.15 * (multiplier - 1.0) / 0.15;
                stats.health *= health_bonus;
                stats.max_health *= health_bonus;
                stats.defense += defense_bonus * 10.0;
            }
            TileType::Green => {
                // Ranger: attack_range +1, attack_speed +15%
                let range_bonus = match level {
                    SynergyLevel::Bronze => 1,
                    SynergyLevel::Silver => 2,
                    SynergyLevel::Gold => 3,
                    _ => 0,
                };
                let speed_bonus = 1.0 + 0.15 * (multiplier - 1.0) / 0.15;
                stats.attack_range += range_bonus;
                stats.attack_speed *= speed_bonus;
            }
            TileType::Yellow => {
                // Assassin: attack +30%, crit chance
                let attack_bonus = 1.0 + 0.30 * (multiplier - 1.0) / 0.15;
                let crit_bonus = match level {
                    SynergyLevel::Bronze => 0.10,
                    SynergyLevel::Silver => 0.20,
                    SynergyLevel::Gold => 0.30,
                    _ => 0.0,
                };
                stats.attack *= attack_bonus;
                stats.crit_chance += crit_bonus;
            }
            TileType::Purple => {
                // Mage: ability_power +25%, mana_regen
                let ap_bonus = 1.0 + 0.25 * (multiplier - 1.0) / 0.15;
                let mana_regen_bonus = match level {
                    SynergyLevel::Bronze => 1.2,
                    SynergyLevel::Silver => 1.4,
                    SynergyLevel::Gold => 1.6,
                    _ => 1.0,
                };
                stats.ability_power *= ap_bonus;
                stats.mana_regen *= mana_regen_bonus;
            }
        }
    }
}
