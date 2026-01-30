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
            .init_resource::<TimeScale>()
            .add_systems(Startup, (setup_cameras, start_game))
            .add_systems(Update, update_timescale)
            .add_observer(handle_slowmo_event)
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

/// System to update the time scale each frame
fn update_timescale(time: Res<Time>, mut timescale: ResMut<TimeScale>) {
    timescale.update(time.delta_secs());
}

/// Observer to handle slow motion events
fn handle_slowmo_event(trigger: Trigger<SlowMoEvent>, mut timescale: ResMut<TimeScale>) {
    let event = trigger.event();
    timescale.trigger_slowmo(event.scale, event.duration);
}

fn start_game(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Playing);
}
