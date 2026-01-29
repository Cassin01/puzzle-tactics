use crate::prelude::*;
use crate::audio::{VictorySoundEvent, DefeatSoundEvent};
use super::{Unit, Team, WaveManager, HexPosition};

#[derive(Event)]
pub struct WaveCompleteEvent {
    pub wave_number: u32,
}

#[derive(Event)]
pub struct GameOverEvent {
    pub victory: bool,
    pub waves_survived: u32,
}

#[derive(Resource)]
pub struct GameResult {
    pub game_ended: bool,
    pub victory: bool,
    pub waves_completed: u32,
    pub player_had_units: bool,
    pub defenseless_timer: f32,
}

impl Default for GameResult {
    fn default() -> Self {
        Self {
            game_ended: false,
            victory: false,
            waves_completed: 0,
            player_had_units: false,
            defenseless_timer: 0.0,
        }
    }
}

const DEFENSELESS_TIMEOUT: f32 = 5.0;

pub fn check_game_result(
    time: Res<Time>,
    mut commands: Commands,
    units: Query<(&Team, &HexPosition), With<Unit>>,
    wave_manager: Res<WaveManager>,
    mut game_result: ResMut<GameResult>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if game_result.game_ended {
        return;
    }

    let mut player_count = 0;
    let mut enemy_count = 0;
    let mut enemy_reached_base = false;

    for (team, pos) in units.iter() {
        match team {
            Team::Player => player_count += 1,
            Team::Enemy => {
                enemy_count += 1;
                if pos.r <= -2 {
                    enemy_reached_base = true;
                }
            }
        }
    }

    if player_count > 0 {
        game_result.player_had_units = true;
        game_result.defenseless_timer = 0.0;
    }

    if wave_manager.wave_active && enemy_count == 0 && wave_manager.enemies_remaining == 0 {
        game_result.waves_completed = wave_manager.current_wave;
        commands.trigger(WaveCompleteEvent {
            wave_number: wave_manager.current_wave,
        });
    }

    let should_lose =
        (game_result.player_had_units && player_count == 0 && wave_manager.current_wave > 0) ||
        enemy_reached_base ||
        (player_count == 0 && enemy_count > 0 && {
            game_result.defenseless_timer += time.delta_secs();
            game_result.defenseless_timer >= DEFENSELESS_TIMEOUT
        });

    if should_lose {
        game_result.game_ended = true;
        game_result.victory = false;
        commands.trigger(GameOverEvent {
            victory: false,
            waves_survived: wave_manager.current_wave.saturating_sub(1),
        });
        commands.trigger(DefeatSoundEvent);
        next_state.set(GameState::GameOver);
    }

    if wave_manager.current_wave >= 10 && enemy_count == 0 && !wave_manager.wave_active {
        game_result.game_ended = true;
        game_result.victory = true;
        game_result.waves_completed = wave_manager.current_wave;
        commands.trigger(GameOverEvent {
            victory: true,
            waves_survived: wave_manager.current_wave,
        });
        commands.trigger(VictorySoundEvent);
        next_state.set(GameState::GameOver);
    }
}

pub fn handle_wave_complete(
    trigger: Trigger<WaveCompleteEvent>,
) {
    let event = trigger.event();
    info!("Wave {} complete!", event.wave_number);
}

pub fn handle_game_over(
    trigger: Trigger<GameOverEvent>,
) {
    let event = trigger.event();
    if event.victory {
        info!("Victory! You survived {} waves!", event.waves_survived);
    } else {
        info!("Game Over! You survived {} waves.", event.waves_survived);
    }
}
