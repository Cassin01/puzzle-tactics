use crate::prelude::*;
use crate::battle::{BattleStats, GameResult};

#[derive(Component)]
pub struct GameOverSummary;

pub fn spawn_game_over_summary(
    mut commands: Commands,
    game_result: Res<GameResult>,
    battle_stats: Res<BattleStats>,
    existing_summary: Query<Entity, With<GameOverSummary>>,
) {
    // Only show summary when game ends and no summary exists yet
    if !game_result.game_ended || !existing_summary.is_empty() {
        return;
    }

    let enemy_name = BattleStats::unit_type_name(battle_stats.most_dangerous_enemy.unit_type);
    let enemy_damage = battle_stats.most_dangerous_enemy.total_damage as i32;

    let mvp_name = BattleStats::unit_type_name(battle_stats.mvp_ally.unit_type);
    let mvp_kills = battle_stats.mvp_ally.kills;
    let mvp_damage = battle_stats.mvp_ally.damage_dealt as i32;

    let total_matches = battle_stats.total_matches;
    let max_combo = battle_stats.max_combo;

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                bottom: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            },
            GameOverSummary,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("=== BATTLE SUMMARY ==="),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.8, 0.2)),
            ));

            // Most Dangerous Enemy
            parent.spawn((
                Text::new(format!(
                    "Most Dangerous Enemy: {} (dealt {} damage)",
                    enemy_name, enemy_damage
                )),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.3, 0.3)),
            ));

            // MVP Ally
            parent.spawn((
                Text::new(format!(
                    "MVP Ally: {} ({} kills, {} damage)",
                    mvp_name, mvp_kills, mvp_damage
                )),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.3, 0.9, 0.3)),
            ));

            // Total Matches
            parent.spawn((
                Text::new(format!("Total Matches: {}", total_matches)),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Max Combo
            parent.spawn((
                Text::new(format!("Max Combo: {}", max_combo)),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.8, 0.0)),
            ));
        });
}
