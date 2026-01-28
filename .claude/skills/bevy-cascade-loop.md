# Bevy Cascade Loop Pattern

## Overview

Bevy ECS pattern for implementing cascade/chain mechanics in puzzle games. Uses state resources to manage gravity → spawn → match detection loops, with combo tracking and event emission.

## Use Cases

- Match-3 puzzle games with chain reactions
- Tile-based games requiring gravity and refill mechanics
- Any game needing iterative board state resolution

## Implementation Pattern

### 1. State Resources

```rust
// src/state.rs
#[derive(States, Default, Clone, Eq, PartialEq, Hash, Debug)]
pub enum PhaseState {
    #[default]
    Idle,
    Cascading,
}

#[derive(Resource, Default)]
pub struct ComboCounter {
    pub current: u32,
    pub max_this_turn: u32,
}

impl ComboCounter {
    pub fn increment(&mut self) {
        self.current += 1;
        if self.current > self.max_this_turn {
            self.max_this_turn = self.current;
        }
    }

    pub fn reset(&mut self) {
        self.current = 0;
        self.max_this_turn = 0;
    }
}
```

### 2. Cascade State Resource

```rust
// src/puzzle/cascade.rs
#[derive(Resource, Default)]
pub struct CascadeState {
    pub has_matches: bool,
    pub pending_gravity: bool,
    pub pending_spawn: bool,
}
```

### 3. Cascade Systems

```rust
// Start cascade when matches are detected
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

// Apply gravity (move tiles down)
pub fn apply_gravity(
    mut cascade_state: ResMut<CascadeState>,
    mut board: ResMut<PuzzleBoard>,
    mut tiles: Query<(Entity, &mut GridPosition, &mut Transform), With<Tile>>,
) {
    if !cascade_state.pending_gravity {
        return;
    }

    for x in 0..BOARD_SIZE {
        let mut write_y = 0;
        for read_y in 0..BOARD_SIZE {
            if let Some(entity) = board.get(x, read_y) {
                if read_y != write_y {
                    board.set(x, write_y, Some(entity));
                    board.set(x, read_y, None);
                    if let Ok((_, mut pos, mut transform)) = tiles.get_mut(entity) {
                        pos.y = write_y;
                        transform.translation = board.grid_to_world(x, write_y).extend(0.0);
                    }
                }
                write_y += 1;
            }
        }
    }

    cascade_state.pending_gravity = false;
    cascade_state.pending_spawn = true;
}

// Spawn new tiles in empty spaces
pub fn spawn_new_tiles(
    mut commands: Commands,
    mut board: ResMut<PuzzleBoard>,
    mut cascade_state: ResMut<CascadeState>,
) {
    if !cascade_state.pending_spawn {
        return;
    }

    for x in 0..BOARD_SIZE {
        for y in 0..BOARD_SIZE {
            if board.get(x, y).is_none() {
                let tile_type = TileType::random();
                let entity = commands.spawn((
                    Tile,
                    tile_type,
                    GridPosition::new(x, y),
                    Sprite { /* ... */ },
                    Transform::from_translation(board.grid_to_world(x, y).extend(0.0)),
                )).id();
                board.set(x, y, Some(entity));
            }
        }
    }

    cascade_state.pending_spawn = false;
}

// Check if cascade is complete
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
        // Cascade complete - emit event with combo bonus
        let mana = calculate_mana_from_combo(combo.current);
        if mana > 0.0 {
            commands.trigger(ManaSupplyEvent { amount: mana });
        }
        next_phase.set(PhaseState::Idle);
    } else {
        cascade_state.has_matches = false;
    }
}
```

### 4. System Registration

```rust
impl Plugin for PuzzlePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CascadeState>()
            .init_resource::<ComboCounter>()
            .add_systems(
                Update,
                (
                    detect_matches,
                    remove_matched_tiles,
                    start_cascade,
                    apply_gravity,
                    spawn_new_tiles,
                    check_cascade_complete,
                    reset_combo_on_idle,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}
```

## Key Points

### Execution Order

```
detect_matches → remove_matched → start_cascade → apply_gravity
→ spawn_new → check_cascade_complete → (loop or emit event + Idle)
```

### State Flags

| Flag | Purpose |
|------|---------|
| `has_matches` | Track if matches occurred this cascade |
| `pending_gravity` | Signal gravity needs to be applied |
| `pending_spawn` | Signal new tiles need to be spawned |

### Combo Bonus Formula Example

```rust
fn calculate_mana_from_combo(combo_count: u32) -> f32 {
    if combo_count == 0 { return 0.0; }
    let base = 10.0;
    let bonus = match combo_count {
        1 => 1.0,
        2 => 1.5,
        3 => 2.0,
        4 => 3.0,
        _ => 3.0 + (combo_count - 4) as f32 * 0.5,
    };
    base * bonus
}
```

## Pitfalls

1. **System Order**: Use `.chain()` to ensure correct execution order
2. **State Flags**: Reset flags properly to avoid infinite loops
3. **PhaseState Check**: Guard systems with state checks to prevent unwanted execution
4. **Match Detection Timing**: New tiles won't be checked for matches until next frame
5. **Board Sync**: Always update board resource when spawning/despawning tiles
