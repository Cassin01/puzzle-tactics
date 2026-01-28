use crate::prelude::*;
use crate::bridge::ManaSupplyEvent;
use super::{PuzzleBoard, Tile, TileType, GridPosition, Matched};
use super::input::SwapAnimation;

#[derive(Resource, Default)]
pub struct CascadeState {
    pub has_matches: bool,
    pub pending_gravity: bool,
    pub pending_spawn: bool,
}

pub fn start_cascade(
    mut cascade_state: ResMut<CascadeState>,
    mut next_phase: ResMut<NextState<PhaseState>>,
    matched: Query<Entity, With<Matched>>,
    mut combo: ResMut<ComboCounter>,
) {
    if !matched.is_empty() {
        cascade_state.has_matches = true;
        cascade_state.pending_gravity = true;
        combo.increment();
        next_phase.set(PhaseState::Cascading);
    }
}

pub fn apply_gravity(
    mut cascade_state: ResMut<CascadeState>,
    mut board: ResMut<PuzzleBoard>,
    mut tiles: Query<(Entity, &mut GridPosition, &mut Transform), With<Tile>>,
    swap_anims: Query<Entity, With<SwapAnimation>>,
) {
    if !cascade_state.pending_gravity {
        return;
    }

    // スワップアニメーション中は待機（pending_gravity は維持）
    if !swap_anims.is_empty() {
        return;
    }

    for x in 0..PUZZLE_BOARD_SIZE {
        let mut write_y = 0;

        for read_y in 0..PUZZLE_BOARD_SIZE {
            if let Some(entity) = board.get(x, read_y) {
                if read_y != write_y {
                    board.set(x, write_y, Some(entity));
                    board.set(x, read_y, None);

                    if let Ok((_, mut pos, mut transform)) = tiles.get_mut(entity) {
                        pos.y = write_y;
                        let target = board.grid_to_world(x, write_y);
                        transform.translation = target.extend(0.1);
                    }
                }
                write_y += 1;
            }
        }
    }

    cascade_state.pending_gravity = false;
    cascade_state.pending_spawn = true;
}

pub fn spawn_new_tiles(
    mut commands: Commands,
    mut board: ResMut<PuzzleBoard>,
    mut cascade_state: ResMut<CascadeState>,
) {
    if !cascade_state.pending_spawn {
        return;
    }

    for x in 0..PUZZLE_BOARD_SIZE {
        for y in 0..PUZZLE_BOARD_SIZE {
            if board.get(x, y).is_none() {
                let tile_type = TileType::random();
                let pos = board.grid_to_world(x, y);

                let entity = commands
                    .spawn((
                        Tile,
                        tile_type,
                        GridPosition::new(x, y),
                        Sprite {
                            color: tile_type.color(),
                            custom_size: Some(Vec2::splat(TILE_SIZE)),
                            ..default()
                        },
                        Transform::from_translation(pos.extend(0.1)),
                        Visibility::default(),
                    ))
                    .id();

                board.set(x, y, Some(entity));
            }
        }
    }

    cascade_state.pending_spawn = false;
}

pub fn check_cascade_complete(
    mut commands: Commands,
    mut cascade_state: ResMut<CascadeState>,
    mut next_phase: ResMut<NextState<PhaseState>>,
    combo: Res<ComboCounter>,
    matched: Query<Entity, With<Matched>>,
    phase: Res<State<PhaseState>>,
) {
    if *phase.get() != PhaseState::Cascading {
        return;
    }

    if cascade_state.pending_gravity || cascade_state.pending_spawn {
        return;
    }

    if matched.is_empty() && !cascade_state.has_matches {
        let mana_amount = calculate_mana_from_combo(combo.current);
        if mana_amount > 0.0 {
            commands.trigger(ManaSupplyEvent { amount: mana_amount });
        }

        next_phase.set(PhaseState::Idle);
        cascade_state.has_matches = false;
    } else {
        cascade_state.has_matches = false;
    }
}

fn calculate_mana_from_combo(combo_count: u32) -> f32 {
    if combo_count == 0 {
        return 0.0;
    }

    let base_mana = 10.0;
    let combo_bonus = match combo_count {
        1 => 1.0,
        2 => 1.5,
        3 => 2.0,
        4 => 3.0,
        _ => 3.0 + (combo_count - 4) as f32 * 0.5,
    };

    base_mana * combo_bonus
}

pub fn reset_combo_on_idle(
    mut combo: ResMut<ComboCounter>,
    phase: Res<State<PhaseState>>,
) {
    if *phase.get() == PhaseState::Idle && combo.current > 0 {
        combo.reset();
    }
}
