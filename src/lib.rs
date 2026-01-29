pub mod camera;
mod prelude;
mod state;

pub mod puzzle;
pub mod battle;
pub mod bridge;
pub mod ui;
pub mod audio;

use prelude::*;
use camera::setup_cameras;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .init_state::<PhaseState>()
            .add_systems(Startup, (setup_cameras, start_game))
            .add_plugins((
                puzzle::PuzzlePlugin,
                puzzle::ObstaclePlugin,
                battle::BattlePlugin,
                bridge::BridgePlugin,
                ui::UIPlugin,
                audio::AudioPlugin,
            ));
    }
}

fn start_game(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Playing);
}
