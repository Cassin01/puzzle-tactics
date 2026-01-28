mod hud;
mod pause_menu;

use crate::prelude::*;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, hud::setup_hud)
            .add_systems(
                Update,
                (
                    hud::update_score_display,
                    hud::update_wave_display,
                    hud::update_synergy_display,
                    hud::update_combo_display,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                hud::show_game_over_screen.run_if(in_state(GameState::GameOver)),
            )
            .add_systems(
                Update,
                pause_menu::handle_pause_input
                    .run_if(in_state(GameState::Playing).or(in_state(GameState::Paused))),
            )
            .add_systems(OnEnter(GameState::Paused), pause_menu::setup_pause_menu)
            .add_systems(OnExit(GameState::Paused), pause_menu::cleanup_pause_menu)
            .add_systems(
                Update,
                (
                    pause_menu::handle_resume_button,
                    pause_menu::handle_quit_button,
                )
                    .run_if(in_state(GameState::Paused)),
            );
    }
}
