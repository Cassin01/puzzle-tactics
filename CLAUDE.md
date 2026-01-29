# Puzzle Tactics - CLAUDE.md

> **Version**: 1.0.0
> **Last Updated**: 2026-01-29
> **Tech Stack**: Rust 2021 Edition + Bevy 0.15 (ECS Architecture)

## Project Overview

Match-3 Puzzle + Auto-Chess Hybrid Strategy Game

- **Puzzle Board**: 8×8 grid, match tiles to summon/upgrade units
- **Battle Field**: Hexagonal grid (7×4), automated unit combat
- **Core Mechanic**: Puzzle matches directly affect battle state in real-time

### Architecture

```
src/
├── main.rs              # Entry point, plugin registration
├── state.rs             # GameState, PhaseState management
├── puzzle/              # Match-3 system (independent)
│   ├── board.rs         # PuzzleBoard resource
│   ├── input.rs         # Tile selection/swap
│   ├── match_detector.rs # Match detection
│   └── cascade.rs       # Gravity & chain reactions
├── battle/              # Auto-chess system (independent)
│   ├── hex_grid.rs      # HexPosition, BattleGrid
│   ├── unit.rs          # UnitStats, StarRank
│   ├── combat.rs        # Targeting, movement, attack
│   ├── synergy.rs       # Team composition bonuses
│   └── wave.rs          # Enemy spawning
├── bridge/              # puzzle → battle event translation
│   └── mod.rs           # MatchEvent → UnitSummonEvent
└── ui/                  # HUD, overlays
    └── mod.rs
```

### Module Dependencies

```
puzzle ──→ bridge ──→ battle
   ↓          ↓          ↓
  (events)  (translate) (consume)
```

**Rule**: puzzle and battle must NOT directly depend on each other. All communication goes through bridge via events.

---

## TDD Workflow (MANDATORY)

All new features MUST follow the Red-Green-Refactor cycle:

### 1. RED: Write Failing Test First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn setup_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        // Add required resources/components
        app
    }

    #[test]
    fn test_feature_behavior() {
        let mut app = setup_test_app();
        // Setup initial state
        // Run system
        // Assert expected outcome
        assert!(false, "Test not implemented yet"); // RED
    }
}
```

### 2. GREEN: Minimum Implementation to Pass

Write only enough code to make the test pass. No extra features.

### 3. REFACTOR: Clean Up

Improve code quality while keeping tests green.

### Test File Organization

```
src/puzzle/
├── mod.rs
├── board.rs
├── match_detector.rs
└── tests/           # Test modules
    ├── mod.rs
    ├── board_tests.rs
    └── match_detector_tests.rs
```

Or inline tests in each module:

```rust
// src/puzzle/match_detector.rs
pub fn detect_matches(...) { ... }

#[cfg(test)]
mod tests {
    use super::*;
    // Tests here
}
```

### Coverage Target

- **Goal**: 80% code coverage for game logic
- **Focus**: Systems, event handlers, state transitions
- **Skip**: Rendering, asset loading, platform-specific code

---

## Bevy ECS Conventions

### Components

- Single responsibility per component
- Marker components for filtering: `#[derive(Component)] struct Selected;`
- Data components for state: `#[derive(Component)] struct Health(f32);`

### Systems Naming

| Pattern | Purpose | Example |
|---------|---------|---------|
| `*_system` | General update logic | `targeting_system` |
| `spawn_*` | Entity creation | `spawn_health_bars` |
| `update_*` | Component modification | `update_health_bars` |
| `check_*` | State validation | `check_cascade_complete` |
| `handle_*` | Event processing | `handle_tile_swap` |
| `animate_*` | Visual transitions | `animate_swap` |

### System Ordering

Always use `.chain()` for dependent systems:

```rust
.add_systems(
    Update,
    (
        detect_matches,
        start_cascade,
        remove_matched_tiles,
        apply_gravity,
        spawn_new_tiles,
        check_cascade_complete,
    )
        .chain()
        .run_if(in_state(GameState::Playing)),
)
```

### Events (Bevy 0.15 Observer Pattern)

```rust
// Define event
#[derive(Event)]
struct UnitSummonEvent {
    unit_type: TileType,
    star_rank: u8,
}

// Register observer
app.add_observer(handle_unit_summon);

// Trigger event
fn some_system(mut commands: Commands) {
    commands.trigger(UnitSummonEvent { ... });
}

// Handle event
fn handle_unit_summon(
    trigger: Trigger<UnitSummonEvent>,
    mut commands: Commands,
) {
    let event = trigger.event();
    // Process event
}
```

### Resources

Use `Resource` for shared state:

```rust
#[derive(Resource, Default)]
pub struct PuzzleBoard {
    tiles: [[Option<Entity>; 8]; 8],
}

// Initialize
app.init_resource::<PuzzleBoard>();

// Access
fn system(board: Res<PuzzleBoard>) { ... }
fn system_mut(mut board: ResMut<PuzzleBoard>) { ... }
```

---

## Project-Specific Rules

### Star Evolution

| Match Size | Star Rank |
|------------|-----------|
| 3-4 tiles | 1★ |
| 5+ tiles | 2★ |
| 3× same rank merge | Next rank |

Maximum: 3★

### Coordinate Systems

- **Puzzle**: `GridPosition { x: usize, y: usize }` (0-7, 0-7)
- **Battle**: `HexPosition { q: i32, r: i32 }` (axial coordinates)

### Combo Mana Formula

```rust
fn calculate_mana(combo: u32) -> f32 {
    let base = 10.0;
    let multiplier = match combo {
        0 => 0.0,
        1 => 1.0,
        2 => 1.5,
        3 => 2.0,
        4 => 3.0,
        _ => 3.0 + (combo - 4) as f32 * 0.5,
    };
    base * multiplier
}
```

---

## Build & Run Commands

```bash
# Development
cargo run

# Run tests
cargo test

# Run specific test
cargo test test_match_detection

# Static analysis
cargo clippy

# Format code
cargo fmt

# WASM build (requires trunk)
trunk build

# WASM serve locally
trunk serve
```

---

## Prohibited Patterns

### DO NOT

```rust
// ❌ Global mutable state
static mut SCORE: u32 = 0;

// ❌ spawn() followed by immediate component access
let entity = commands.spawn(Tile).id();
tiles.get(entity); // Entity doesn't exist yet!

// ❌ despawn() instead of despawn_recursive()
commands.entity(parent).despawn(); // Children become orphans!

// ❌ Direct module coupling
// In puzzle/match_detector.rs:
use crate::battle::Unit; // ❌ Use bridge events instead
```

### DO

```rust
// ✅ Use Resource for shared state
#[derive(Resource)]
struct Score(u32);

// ✅ Use Commands for deferred operations
commands.spawn(Tile);
// Access in next system via Query

// ✅ Use despawn_recursive()
commands.entity(parent).despawn_recursive();

// ✅ Communicate via events
commands.trigger(MatchEvent { ... });
```

---

## Testing Patterns for Bevy

### Testing Systems

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn setup_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<PuzzleBoard>()
            .init_resource::<CascadeState>()
            .add_systems(Update, detect_matches);
        app
    }

    #[test]
    fn test_horizontal_match() {
        let mut app = setup_test_app();

        // Setup: Create 3 adjacent same-type tiles
        let world = app.world_mut();
        for x in 0..3 {
            world.spawn((
                Tile,
                TileType::Red,
                GridPosition::new(x, 0),
            ));
        }

        // Run system
        app.update();

        // Assert: All 3 tiles should have Matched component
        let matched_count = app.world()
            .query_filtered::<Entity, With<Matched>>()
            .iter(app.world())
            .count();
        assert_eq!(matched_count, 3);
    }
}
```

### Testing Events

```rust
#[test]
fn test_match_triggers_event() {
    let mut app = setup_test_app();
    app.add_observer(|trigger: Trigger<MatchEvent>| {
        // Event received
    });

    // Setup match condition
    // Run systems
    app.update();

    // Assert event was triggered
}
```

### Testing State Transitions

```rust
#[test]
fn test_cascade_state_transition() {
    let mut app = setup_test_app();
    app.init_state::<PhaseState>();

    // Initial state
    assert_eq!(*app.world().resource::<State<PhaseState>>().get(), PhaseState::Idle);

    // Trigger cascade
    // ...
    app.update();

    // Verify state changed
    assert_eq!(*app.world().resource::<State<PhaseState>>().get(), PhaseState::Cascading);
}
```

---

## Documentation References

| Document | Purpose |
|----------|---------|
| `game_design.md` | Core game mechanics and design philosophy |
| `tech_stack.md` | Technology choices and tooling |
| `docs/IMPLEMENTATION_STATUS.md` | Current implementation status matrix |
| `.claude/skills/*.md` | Reusable Bevy patterns |

---

## Git Workflow

### Branch Naming

- `feature/` - New features
- `fix/` - Bug fixes
- `refactor/` - Code improvements
- `test/` - Test additions

### Commit Messages

- Write in Japanese (プロジェクト規約)
- No Co-Authored-By lines
- Format: `<type>: <description>`

```
feat: マッチ検出アルゴリズムを追加
fix: カスケード無限ループを修正
test: ユニット召喚テストを追加
refactor: コンボカウンター計算を簡略化
```

---

## Quick Reference

### Common Imports (prelude.rs)

```rust
use bevy::prelude::*;
use crate::state::{GameState, PhaseState, ComboCounter};
```

### System Run Conditions

```rust
.run_if(in_state(GameState::Playing))
.run_if(in_state(PhaseState::Cascading))
.run_if(resource_changed::<ComboCounter>)
```

### Entity Spawning Pattern

```rust
commands.spawn((
    Tile,
    TileType::Red,
    GridPosition::new(x, y),
    Sprite {
        color: TileType::Red.color(),
        custom_size: Some(Vec2::splat(TILE_SIZE)),
        ..default()
    },
    Transform::from_translation(position.extend(0.0)),
));
```
