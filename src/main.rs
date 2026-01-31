use bevy::prelude::*;
use puzzle_tactics::GamePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Puzzle Tactics".into(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(GamePlugin)
        .run();
}
