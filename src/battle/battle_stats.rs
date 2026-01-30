use crate::prelude::*;
use super::UnitType;
use crate::puzzle::TileType;

/// Tracks enemy damage dealt during battle
#[derive(Clone, Default)]
pub struct EnemyDamageRecord {
    pub unit_type: Option<TileType>,
    pub total_damage: f32,
}

/// Tracks ally performance during battle
#[derive(Clone, Default)]
pub struct AllyPerformanceRecord {
    pub unit_type: Option<TileType>,
    pub kills: u32,
    pub damage_dealt: f32,
}

/// Resource for tracking battle statistics
#[derive(Resource, Default)]
pub struct BattleStats {
    /// Most dangerous enemy (dealt most damage to player units)
    pub most_dangerous_enemy: EnemyDamageRecord,
    /// MVP ally (most kills + damage)
    pub mvp_ally: AllyPerformanceRecord,
    /// Total puzzle matches made
    pub total_matches: u32,
    /// Maximum combo achieved
    pub max_combo: u32,
    /// Current combo (for tracking max)
    current_combo: u32,
    /// All enemy damage records for finding most dangerous
    enemy_damage_map: std::collections::HashMap<TileType, f32>,
    /// All ally performance records for finding MVP
    ally_performance_map: std::collections::HashMap<TileType, (u32, f32)>,
}

impl BattleStats {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record damage dealt by an enemy
    pub fn record_enemy_damage(&mut self, unit_type: TileType, damage: f32) {
        let entry = self.enemy_damage_map.entry(unit_type).or_insert(0.0);
        *entry += damage;

        // Update most dangerous enemy if this one has dealt more damage
        if *entry > self.most_dangerous_enemy.total_damage {
            self.most_dangerous_enemy = EnemyDamageRecord {
                unit_type: Some(unit_type),
                total_damage: *entry,
            };
        }
    }

    /// Record a kill by an ally
    pub fn record_ally_kill(&mut self, unit_type: TileType, damage: f32) {
        let entry = self.ally_performance_map.entry(unit_type).or_insert((0, 0.0));
        entry.0 += 1;
        entry.1 += damage;

        self.update_mvp();
    }

    /// Record damage dealt by an ally (without kill)
    pub fn record_ally_damage(&mut self, unit_type: TileType, damage: f32) {
        let entry = self.ally_performance_map.entry(unit_type).or_insert((0, 0.0));
        entry.1 += damage;

        self.update_mvp();
    }

    fn update_mvp(&mut self) {
        let mut best_score = 0.0f32;
        let mut best_type: Option<TileType> = None;
        let mut best_kills = 0u32;
        let mut best_damage = 0.0f32;

        for (unit_type, (kills, damage)) in &self.ally_performance_map {
            // MVP score: kills * 100 + damage
            let score = (*kills as f32) * 100.0 + damage;
            if score > best_score {
                best_score = score;
                best_type = Some(*unit_type);
                best_kills = *kills;
                best_damage = *damage;
            }
        }

        if best_type.is_some() {
            self.mvp_ally = AllyPerformanceRecord {
                unit_type: best_type,
                kills: best_kills,
                damage_dealt: best_damage,
            };
        }
    }

    /// Record a match (called when puzzle matches are made)
    pub fn record_match(&mut self) {
        self.total_matches += 1;
    }

    /// Update combo count and track maximum
    pub fn update_combo(&mut self, combo: u32) {
        self.current_combo = combo;
        if combo > self.max_combo {
            self.max_combo = combo;
        }
    }

    /// Reset stats for a new game
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Record a kill for the ally that dealt most damage
    /// Called when an enemy dies
    pub fn record_kill_for_top_ally(&mut self) {
        if let Some((&top_type, _)) = self.ally_performance_map.iter()
            .max_by(|a, b| a.1.1.partial_cmp(&b.1.1).unwrap_or(std::cmp::Ordering::Equal))
        {
            let entry = self.ally_performance_map.entry(top_type).or_insert((0, 0.0));
            entry.0 += 1;
            self.update_mvp();
        }
    }

    /// Get unit type name for display
    pub fn unit_type_name(unit_type: Option<TileType>) -> &'static str {
        match unit_type {
            Some(TileType::Red) => "Warrior",
            Some(TileType::Blue) => "Tank",
            Some(TileType::Green) => "Ranger",
            Some(TileType::Yellow) => "Assassin",
            Some(TileType::Purple) => "Mage",
            None => "None",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_battle_stats_tracks_damage() {
        let mut stats = BattleStats::new();

        stats.record_enemy_damage(TileType::Red, 50.0);
        stats.record_enemy_damage(TileType::Red, 30.0);

        assert_eq!(stats.most_dangerous_enemy.total_damage, 80.0);
        assert_eq!(stats.most_dangerous_enemy.unit_type, Some(TileType::Red));
    }

    #[test]
    fn test_battle_stats_tracks_max_combo() {
        let mut stats = BattleStats::new();

        stats.update_combo(3);
        stats.update_combo(5);
        stats.update_combo(2);

        assert_eq!(stats.max_combo, 5);
    }

    #[test]
    fn test_battle_stats_finds_mvp() {
        let mut stats = BattleStats::new();

        // Green ranger: 2 kills, 100 damage = score 300
        stats.record_ally_kill(TileType::Green, 50.0);
        stats.record_ally_kill(TileType::Green, 50.0);

        // Red warrior: 1 kill, 200 damage = score 300
        stats.record_ally_kill(TileType::Red, 200.0);

        // Purple mage: 0 kills, 500 damage = score 500
        stats.record_ally_damage(TileType::Purple, 500.0);

        // Purple mage should be MVP (highest score)
        assert_eq!(stats.mvp_ally.unit_type, Some(TileType::Purple));
        assert_eq!(stats.mvp_ally.damage_dealt, 500.0);
    }

    #[test]
    fn test_battle_stats_tracks_matches() {
        let mut stats = BattleStats::new();

        stats.record_match();
        stats.record_match();
        stats.record_match();

        assert_eq!(stats.total_matches, 3);
    }

    #[test]
    fn test_most_dangerous_enemy_updates_correctly() {
        let mut stats = BattleStats::new();

        stats.record_enemy_damage(TileType::Red, 30.0);
        assert_eq!(stats.most_dangerous_enemy.unit_type, Some(TileType::Red));

        stats.record_enemy_damage(TileType::Purple, 50.0);
        assert_eq!(stats.most_dangerous_enemy.unit_type, Some(TileType::Purple));

        // Red deals more damage total
        stats.record_enemy_damage(TileType::Red, 30.0);
        assert_eq!(stats.most_dangerous_enemy.unit_type, Some(TileType::Red));
        assert_eq!(stats.most_dangerous_enemy.total_damage, 60.0);
    }

    #[test]
    fn test_unit_type_name() {
        assert_eq!(BattleStats::unit_type_name(Some(TileType::Red)), "Warrior");
        assert_eq!(BattleStats::unit_type_name(Some(TileType::Blue)), "Tank");
        assert_eq!(BattleStats::unit_type_name(Some(TileType::Green)), "Ranger");
        assert_eq!(BattleStats::unit_type_name(None), "None");
    }
}
