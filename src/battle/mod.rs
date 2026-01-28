mod hex_grid;
mod unit;
mod combat;
mod synergy;
mod wave;
mod game_result;

use crate::prelude::*;

pub use hex_grid::{BattleGrid, HexPosition};
pub use unit::{Unit, UnitStats, UnitType, StarRank, Team, Target, AttackCooldown, HealthBar, HealthBarBackground};
pub use synergy::{ActiveSynergies, SynergyLevel};
pub use wave::{WaveManager, BombDamageEvent};
pub use game_result::{GameResult, WaveCompleteEvent, GameOverEvent};

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BattleGrid>()
            .init_resource::<ActiveSynergies>()
            .init_resource::<WaveManager>()
            .init_resource::<GameResult>()
            .add_observer(game_result::handle_wave_complete)
            .add_observer(game_result::handle_game_over)
            .add_observer(wave::handle_bomb_damage)
            .add_systems(Startup, hex_grid::setup_battle_grid)
            .add_systems(
                Update,
                (
                    wave::wave_spawner_system,
                    wave::bomb_countdown_system,
                    unit::spawn_health_bars,
                    combat::targeting_system,
                    combat::movement_system,
                    combat::attack_system,
                    combat::ability_system,
                    combat::death_system,
                    unit::update_health_bars,
                    synergy::update_synergies,
                    synergy::apply_synergy_bonuses,
                    game_result::check_game_result,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}
