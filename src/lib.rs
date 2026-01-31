pub mod camera;
mod prelude;
mod state;

pub mod puzzle;
pub mod battle;
pub mod bridge;
pub mod ui;
pub mod audio;

use bevy::window::WindowMode;
use prelude::*;
use camera::setup_cameras;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .init_state::<PhaseState>()
            .init_resource::<TimeScale>()
            .init_resource::<WindowSize>()
            .add_systems(Startup, (setup_cameras, start_game, maximize_window))
            .add_systems(Update, (update_timescale, track_window_size))
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

/// Maximize the window on startup
fn maximize_window(mut windows: Query<&mut Window>) {
    for mut window in windows.iter_mut() {
        window.mode = WindowMode::BorderlessFullscreen(MonitorSelection::Current);
    }
}

/// Track window size changes and update WindowSize resource
fn track_window_size(windows: Query<&Window>, mut window_size: ResMut<WindowSize>) {
    if let Ok(window) = windows.get_single() {
        let new_width = window.width();
        let new_height = window.height();
        if (window_size.width - new_width).abs() > 1.0
            || (window_size.height - new_height).abs() > 1.0
        {
            window_size.width = new_width;
            window_size.height = new_height;
        }
    }
}
