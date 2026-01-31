mod board;
mod tile;
mod input;
mod match_detector;
mod cascade;
mod obstacle;
mod preview;

use crate::prelude::*;

pub use board::PuzzleBoard;
pub use tile::{Tile, TileType, GridPosition, Matched, Falling, Selected, Obstacle, ObstacleType};
pub use cascade::CascadeState;
pub use obstacle::{ObstaclePlugin, BombCountdownText, IceOverlay, IceMeltEvent, BombDefuseEvent};
pub use preview::TilePreview;

const HIGHLIGHT_INTENSITY: f32 = 0.4;

pub struct PuzzlePlugin;

impl Plugin for PuzzlePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CascadeState>()
            .init_resource::<ComboCounter>()
            .init_resource::<preview::TilePreview>()
            .add_systems(Startup, board::setup_puzzle_board)
            .add_observer(input::handle_tile_swap)
            .add_observer(input::handle_invalid_swap)
            .add_systems(
                Update,
                (
                    board::update_puzzle_board_on_resize,
                    board::reposition_tiles_on_resize,
                ),
            )
            .add_systems(
                Update,
                (
                    input::handle_tile_click,
                    input::animate_swap,
                    input::animate_ice_shake,
                    input::animate_invalid_swap_shake,
                    highlight_selected_tile,
                    match_detector::detect_matches,
                    cascade::start_cascade,
                    match_detector::remove_matched_tiles,
                    cascade::apply_gravity,
                    cascade::spawn_new_tiles,
                    cascade::check_cascade_complete,
                    cascade::reset_combo_on_idle,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn highlight_selected_tile(
    mut tiles: Query<(&mut Sprite, &TileType, Option<&Selected>), With<Tile>>,
) {
    for (mut sprite, tile_type, selected) in tiles.iter_mut() {
        let base_color = tile_type.color();
        if selected.is_some() {
            sprite.color = base_color.lighter(HIGHLIGHT_INTENSITY);
        } else {
            sprite.color = base_color;
        }
    }
}
