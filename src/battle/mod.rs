mod hex_grid;
mod unit;
mod combat;
mod synergy;
mod wave;
mod game_result;
mod damage_popup;
mod battle_stats;
mod placement;

use crate::prelude::*;

pub use hex_grid::{BattleGrid, HexPosition};
pub use unit::{Unit, UnitStats, UnitType, StarRank, Team, Target, AttackCooldown, HealthBar, HealthBarBackground, RageBuff, SnipeBuff, StealthBuff, MeteorAbility};
pub use synergy::{ActiveSynergies, SynergyLevel};
pub use wave::{WaveManager, BombDamageEvent, BombExplosionEffect, BombCountdownTimer, BOMB_COUNTDOWN_INTERVAL, WaveBreakStartEvent, WaveBreakEndEvent};
pub use game_result::{GameResult, WaveCompleteEvent, GameOverEvent};
pub use damage_popup::{DamagePopup, DamagePopupEvent};
pub use combat::DamageCalculator;
pub use battle_stats::BattleStats;
pub use placement::{Selected, SelectableUnit, MovementHighlight, UnitSelectEvent, UnitMoveEvent};

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BattleGrid>()
            .init_resource::<ActiveSynergies>()
            .init_resource::<WaveManager>()
            .init_resource::<GameResult>()
            .init_resource::<BattleStats>()
            .init_resource::<wave::BombCountdownTimer>()
            .init_resource::<WaveBreakTimer>()
            .add_observer(game_result::handle_wave_complete)
            .add_observer(game_result::handle_game_over)
            .add_observer(wave::handle_bomb_damage)
            .add_observer(damage_popup::spawn_damage_popup)
            .add_observer(placement::handle_unit_move)
            .add_systems(Startup, hex_grid::setup_battle_grid)
            .add_systems(
                Update,
                (
                    hex_grid::update_battle_grid_on_resize,
                    hex_grid::reposition_units_on_resize,
                ),
            )
            .add_systems(
                Update,
                (
                    wave::wave_spawner_system,
                    wave::bomb_countdown_system,
                    wave::check_wave_complete_system,
                    combat::targeting_system,
                    combat::movement_system,
                    combat::attack_system,
                    combat::ability_system,
                    combat::death_system,
                    combat::despawn_attack_lines,
                    unit::spawn_health_bars,
                    unit::update_health_bars,
                    synergy::update_synergies,
                    synergy::apply_synergy_bonuses,
                    game_result::check_game_result,
                    damage_popup::animate_damage_popup,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                wave::animate_bomb_explosion
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                combat::buff_timer_system
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                wave::wave_break_timer_system
                    .run_if(in_state(GameState::Playing)),
            )
            // WaveBreak placement systems
            .add_systems(
                Update,
                (
                    placement::mark_units_selectable,
                    placement::placement_input_system,
                    placement::spawn_movement_highlights,
                    placement::update_selected_visual,
                    placement::restore_deselected_visual,
                    placement::despawn_movement_highlights,
                    placement::unmark_units_selectable,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}
