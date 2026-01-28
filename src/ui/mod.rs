mod hud;

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
            );
    }
}
