use crate::prelude::*;
use crate::battle::{ActiveSynergies, SynergyLevel, WaveManager, GameResult};
use crate::puzzle::TileType;

#[derive(Resource, Default)]
pub struct Score(pub u32);

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct WaveText;

#[derive(Component)]
pub struct SynergyDisplay;

#[derive(Component)]
pub struct GameOverScreen;

#[derive(Component)]
pub struct ComboText;

pub fn setup_hud(mut commands: Commands) {
    commands.insert_resource(Score::default());

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                top: Val::Px(10.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(5.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Wave: 0"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.8, 0.2)),
                WaveText,
            ));
            parent.spawn((
                Text::new("Score: 0"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                ScoreText,
            ));
        });

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(10.0),
                top: Val::Px(10.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            SynergyDisplay,
        ));

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Px(80.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.8, 0.0)),
                ComboText,
            ));
        });
}

pub fn update_score_display(
    score: Res<Score>,
    mut query: Query<&mut Text, With<ScoreText>>,
) {
    if score.is_changed() {
        for mut text in query.iter_mut() {
            **text = format!("Score: {}", score.0);
        }
    }
}

pub fn update_wave_display(
    wave_manager: Res<WaveManager>,
    mut query: Query<&mut Text, With<WaveText>>,
) {
    if wave_manager.is_changed() {
        for mut text in query.iter_mut() {
            **text = format!("Wave: {}", wave_manager.current_wave);
        }
    }
}

pub fn update_combo_display(
    combo: Res<ComboCounter>,
    mut query: Query<(&mut Text, &mut Visibility), With<ComboText>>,
) {
    if combo.is_changed() {
        for (mut text, mut visibility) in query.iter_mut() {
            if combo.current > 1 {
                **text = format!("{}x COMBO!", combo.current);
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

pub fn update_synergy_display(
    synergies: Res<ActiveSynergies>,
    mut commands: Commands,
    display: Query<Entity, With<SynergyDisplay>>,
) {
    if !synergies.is_changed() {
        return;
    }

    let Ok(display_entity) = display.get_single() else { return };

    commands.entity(display_entity).despawn_descendants();

    commands.entity(display_entity).with_children(|parent| {
        for tile_type in [TileType::Red, TileType::Blue, TileType::Green, TileType::Yellow, TileType::Purple] {
            let level = synergies.get_level(tile_type);
            if level == SynergyLevel::None {
                continue;
            }

            let level_text = match level {
                SynergyLevel::Bronze => "Bronze",
                SynergyLevel::Silver => "Silver",
                SynergyLevel::Gold => "Gold",
                SynergyLevel::None => continue,
            };

            let type_name = match tile_type {
                TileType::Red => "Warrior",
                TileType::Blue => "Tank",
                TileType::Green => "Ranger",
                TileType::Yellow => "Assassin",
                TileType::Purple => "Mage",
            };

            parent.spawn((
                Text::new(format!("{}: {}", type_name, level_text)),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(tile_type.color()),
            ));
        }
    });
}

pub fn show_game_over_screen(
    mut commands: Commands,
    game_result: Res<GameResult>,
    existing_screen: Query<Entity, With<GameOverScreen>>,
) {
    if !game_result.game_ended || !existing_screen.is_empty() {
        return;
    }

    let (title, color) = if game_result.victory {
        ("VICTORY!", Color::srgb(0.2, 0.9, 0.3))
    } else {
        ("GAME OVER", Color::srgb(0.9, 0.2, 0.2))
    };

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            GameOverScreen,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(title),
                TextFont {
                    font_size: 64.0,
                    ..default()
                },
                TextColor(color),
            ));
            parent.spawn((
                Text::new(format!("Waves Completed: {}", game_result.waves_completed)),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}
