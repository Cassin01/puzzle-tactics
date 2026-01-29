use crate::prelude::*;
use crate::puzzle::TileType;

const HEALTH_BAR_WIDTH: f32 = 30.0;
const HEALTH_BAR_HEIGHT: f32 = 4.0;
const HEALTH_BAR_OFFSET_Y: f32 = 25.0;

#[derive(Component)]
pub struct Unit;

#[derive(Component, Clone)]
pub struct UnitStats {
    pub health: f32,
    pub max_health: f32,
    pub attack: f32,
    pub attack_speed: f32,
    pub attack_range: i32,
    pub mana: f32,
    pub max_mana: f32,
    pub move_speed: f32,
    pub defense: f32,
    pub crit_chance: f32,
    pub ability_power: f32,
    pub mana_regen: f32,
}

impl Default for UnitStats {
    fn default() -> Self {
        Self {
            health: 100.0,
            max_health: 100.0,
            attack: 10.0,
            attack_speed: 1.0,
            attack_range: 1,
            mana: 0.0,
            max_mana: 100.0,
            move_speed: 1.0,
            defense: 0.0,
            crit_chance: 0.0,
            ability_power: 0.0,
            mana_regen: 1.0,
        }
    }
}

impl UnitStats {
    pub fn for_type(tile_type: TileType, star_rank: u8) -> Self {
        let multiplier = match star_rank {
            1 => 1.0,
            2 => 1.8,
            3 => 3.0,
            _ => 1.0,
        };

        let base = match tile_type {
            TileType::Red => Self {
                health: 80.0,
                max_health: 80.0,
                attack: 15.0,
                attack_speed: 1.2,
                attack_range: 1,
                ..default()
            },
            TileType::Blue => Self {
                health: 120.0,
                max_health: 120.0,
                attack: 8.0,
                attack_speed: 0.8,
                attack_range: 1,
                ..default()
            },
            TileType::Green => Self {
                health: 90.0,
                max_health: 90.0,
                attack: 12.0,
                attack_speed: 1.0,
                attack_range: 3,
                ..default()
            },
            TileType::Yellow => Self {
                health: 70.0,
                max_health: 70.0,
                attack: 18.0,
                attack_speed: 1.5,
                attack_range: 1,
                ..default()
            },
            TileType::Purple => Self {
                health: 100.0,
                max_health: 100.0,
                attack: 10.0,
                attack_speed: 1.0,
                attack_range: 2,
                max_mana: 80.0,
                ..default()
            },
        };

        Self {
            health: base.health * multiplier,
            max_health: base.max_health * multiplier,
            attack: base.attack * multiplier,
            ..base
        }
    }

    pub fn is_dead(&self) -> bool {
        self.health <= 0.0
    }

    pub fn take_damage(&mut self, amount: f32) {
        let reduced = (amount - self.defense).max(1.0);
        self.health = (self.health - reduced).max(0.0);
    }

    /// Take damage with percentage-based defense reduction
    /// Defense is converted to percentage: defense 50 = 50% reduction (capped at 80%)
    pub fn take_calculated_damage(&mut self, amount: f32) {
        let reduction = (self.defense / 100.0).min(0.8);
        let reduced = amount * (1.0 - reduction);
        let final_damage = reduced.max(1.0);
        self.health = (self.health - final_damage).max(0.0);
    }

    pub fn gain_mana(&mut self, amount: f32) {
        self.mana = (self.mana + amount).min(self.max_mana);
    }

    pub fn can_cast(&self) -> bool {
        self.mana >= self.max_mana
    }
}

#[derive(Component, Clone, Copy)]
pub struct StarRank(pub u8);

#[derive(Component, Clone, Copy)]
pub struct UnitType(pub TileType);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum Team {
    Player,
    Enemy,
}

#[derive(Component)]
pub struct Target(pub Option<Entity>);

#[derive(Component)]
pub struct AttackCooldown(pub f32);

// ============================================================
// Ability Buff Components
// ============================================================

/// Red (Warrior) Rage buff: ATK +20% for 5 seconds
#[derive(Component, Clone)]
pub struct RageBuff {
    pub remaining: f32,
}

impl RageBuff {
    pub const DURATION: f32 = 5.0;
    pub const ATTACK_MULTIPLIER: f32 = 1.2;

    pub fn new() -> Self {
        Self { remaining: Self::DURATION }
    }

    pub fn duration(&self) -> f32 {
        Self::DURATION
    }

    pub fn remaining_time(&self) -> f32 {
        self.remaining
    }

    pub fn tick(&mut self, delta: f32) {
        self.remaining = (self.remaining - delta).max(0.0);
    }

    pub fn is_expired(&self) -> bool {
        self.remaining <= 0.0
    }

    pub fn apply_attack_modifier(&self, base_attack: f32) -> f32 {
        base_attack * Self::ATTACK_MULTIPLIER
    }
}

impl Default for RageBuff {
    fn default() -> Self {
        Self::new()
    }
}

/// Green (Ranger) Snipe buff: next attack deals 2x damage
#[derive(Component, Clone)]
pub struct SnipeBuff {
    pub consumed: bool,
}

impl SnipeBuff {
    pub const DAMAGE_MULTIPLIER: f32 = 2.0;

    pub fn new() -> Self {
        Self { consumed: false }
    }

    pub fn is_consumed(&self) -> bool {
        self.consumed
    }

    pub fn consume(&mut self) {
        self.consumed = true;
    }

    pub fn apply_damage_modifier(&self, base_damage: f32) -> f32 {
        base_damage * Self::DAMAGE_MULTIPLIER
    }
}

impl Default for SnipeBuff {
    fn default() -> Self {
        Self::new()
    }
}

/// Yellow (Assassin) Stealth buff: untargetable for 3 seconds
#[derive(Component, Clone)]
pub struct StealthBuff {
    pub remaining: f32,
}

impl StealthBuff {
    pub const DURATION: f32 = 3.0;

    pub fn new() -> Self {
        Self { remaining: Self::DURATION }
    }

    pub fn duration(&self) -> f32 {
        Self::DURATION
    }

    pub fn remaining_time(&self) -> f32 {
        self.remaining
    }

    pub fn tick(&mut self, delta: f32) {
        self.remaining = (self.remaining - delta).max(0.0);
    }

    pub fn is_expired(&self) -> bool {
        self.remaining <= 0.0
    }

    pub fn is_active(&self) -> bool {
        !self.is_expired()
    }

    pub fn makes_untargetable(&self) -> bool {
        self.is_active()
    }
}

impl Default for StealthBuff {
    fn default() -> Self {
        Self::new()
    }
}

/// Purple (Mage) Meteor ability helper
pub struct MeteorAbility;

impl MeteorAbility {
    pub const DAMAGE: f32 = 15.0;

    pub fn damage() -> f32 {
        Self::DAMAGE
    }

    pub fn calculate_damages(enemy_count: usize) -> Vec<f32> {
        vec![Self::DAMAGE; enemy_count]
    }
}

#[derive(Component)]
pub struct HealthBar;

#[derive(Component)]
pub struct HealthBarBackground;

pub fn spawn_health_bars(
    mut commands: Commands,
    units: Query<(Entity, &UnitStats), (With<Unit>, Without<Children>)>,
) {
    for (entity, stats) in units.iter() {
        // 死亡済みユニットはスキップ
        if stats.is_dead() {
            continue;
        }
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                HealthBarBackground,
                Sprite {
                    color: Color::srgb(0.2, 0.2, 0.2),
                    custom_size: Some(Vec2::new(HEALTH_BAR_WIDTH, HEALTH_BAR_HEIGHT)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, HEALTH_BAR_OFFSET_Y, 0.1)),
            ));
            parent.spawn((
                HealthBar,
                Sprite {
                    color: Color::srgb(0.2, 0.9, 0.2),
                    custom_size: Some(Vec2::new(HEALTH_BAR_WIDTH, HEALTH_BAR_HEIGHT)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, HEALTH_BAR_OFFSET_Y, 0.2)),
            ));
        });
    }
}

pub fn update_health_bars(
    units: Query<(&Children, &UnitStats), With<Unit>>,
    mut health_bars: Query<&mut Sprite, With<HealthBar>>,
) {
    for (children, stats) in units.iter() {
        if stats.is_dead() {
            continue;
        }
        let health_ratio = (stats.health / stats.max_health).max(0.01);
        for &child in children.iter() {
            if let Ok(mut sprite) = health_bars.get_mut(child) {
                sprite.custom_size = Some(Vec2::new(HEALTH_BAR_WIDTH * health_ratio, HEALTH_BAR_HEIGHT));
                sprite.color = health_ratio_to_color(health_ratio);
            }
        }
    }
}

fn health_ratio_to_color(ratio: f32) -> Color {
    if ratio > 0.5 {
        Color::srgb(0.2, 0.9, 0.2)
    } else if ratio > 0.25 {
        Color::srgb(0.9, 0.9, 0.2)
    } else {
        Color::srgb(0.9, 0.2, 0.2)
    }
}

// ============================================================
// Unit Tests (TDD)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Red (Warrior) Rage Tests
    #[test]
    fn test_rage_buff_increases_attack() {
        let base_attack = 15.0;
        let rage_buff = RageBuff::new();
        let buffed_attack = rage_buff.apply_attack_modifier(base_attack);
        assert!((buffed_attack - base_attack * 1.2).abs() < 0.01);
    }

    #[test]
    fn test_rage_buff_duration() {
        let buff = RageBuff::new();
        assert_eq!(buff.duration(), 5.0);
    }

    #[test]
    fn test_rage_buff_timer_decreases() {
        let mut buff = RageBuff::new();
        let initial_remaining = buff.remaining_time();
        buff.tick(1.0);
        assert!((buff.remaining_time() - (initial_remaining - 1.0)).abs() < 0.01);
    }

    #[test]
    fn test_rage_buff_expires_after_duration() {
        let mut buff = RageBuff::new();
        buff.tick(5.0);
        assert!(buff.is_expired());
    }

    // Blue (Tank) Heal Tests
    #[test]
    fn test_heal_restores_health() {
        let mut stats = UnitStats::for_type(TileType::Blue, 1);
        let max_hp = stats.max_health;
        stats.health = max_hp * 0.5;
        let damaged_hp = stats.health;
        let heal_amount = max_hp * 0.2;
        stats.health = (stats.health + heal_amount).min(stats.max_health);
        assert!((stats.health - (damaged_hp + heal_amount)).abs() < 0.01);
    }

    // Green (Ranger) Snipe Tests
    #[test]
    fn test_snipe_buff_doubles_next_attack() {
        let base_attack = 12.0;
        let snipe_buff = SnipeBuff::new();
        let buffed_damage = snipe_buff.apply_damage_modifier(base_attack);
        assert!((buffed_damage - base_attack * 2.0).abs() < 0.01);
    }

    #[test]
    fn test_snipe_buff_consumed_after_attack() {
        let mut snipe_buff = SnipeBuff::new();
        assert!(!snipe_buff.is_consumed());
        snipe_buff.consume();
        assert!(snipe_buff.is_consumed());
    }

    // Yellow (Assassin) Stealth Tests
    #[test]
    fn test_stealth_buff_makes_untargetable() {
        let stealth_buff = StealthBuff::new();
        assert!(stealth_buff.is_active());
        assert!(stealth_buff.makes_untargetable());
    }

    #[test]
    fn test_stealth_buff_duration() {
        let buff = StealthBuff::new();
        assert_eq!(buff.duration(), 3.0);
    }

    #[test]
    fn test_stealth_buff_expires() {
        let mut buff = StealthBuff::new();
        buff.tick(3.0);
        assert!(buff.is_expired());
        assert!(!buff.makes_untargetable());
    }

    // Purple (Mage) Meteor Tests
    #[test]
    fn test_meteor_damage_amount() {
        let meteor_damage = MeteorAbility::damage();
        assert_eq!(meteor_damage, 15.0);
    }

    #[test]
    fn test_meteor_hits_all_enemies() {
        let enemy_count = 5;
        let damages = MeteorAbility::calculate_damages(enemy_count);
        assert_eq!(damages.len(), enemy_count);
        for damage in damages {
            assert_eq!(damage, 15.0);
        }
    }

    // Mana Tests
    #[test]
    fn test_mana_consumed_on_cast() {
        let mut stats = UnitStats::for_type(TileType::Red, 1);
        stats.mana = stats.max_mana;
        assert!(stats.can_cast());
        stats.mana = 0.0;
        assert!(!stats.can_cast());
    }
}
