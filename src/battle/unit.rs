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
        self.health = (self.health - amount).max(0.0);
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

#[derive(Component)]
pub struct HealthBar;

#[derive(Component)]
pub struct HealthBarBackground;

pub fn spawn_health_bars(
    mut commands: Commands,
    units: Query<Entity, Added<Unit>>,
) {
    for entity in units.iter() {
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
