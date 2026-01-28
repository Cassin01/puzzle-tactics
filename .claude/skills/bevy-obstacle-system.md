# Bevy Obstacle System

## Overview

Puzzle Tactics における障害物システムの実装パターン。`ObstacleType` enum で障害物の種類を定義し、`PuzzleBoard` のグリッドとは別レイヤーで管理する。

## Use Cases

- パズルボードに動的な障害物を配置したい
- マッチ3パズルに戦略要素を追加したい
- Wave ベースのゲームで難易度調整をしたい

## Implementation Patterns

### 1. ObstacleType Enum Definition

```rust
// src/puzzle/tile.rs

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ObstacleType {
    Ice,   // Blocks tile movement, cleared by adjacent match
    Bomb,  // Countdown timer, damages player units on explosion
}

#[derive(Component)]
pub struct Obstacle {
    pub obstacle_type: ObstacleType,
    pub countdown: Option<u8>,  // Only used for Bomb
}

impl Obstacle {
    pub fn ice() -> Self {
        Self {
            obstacle_type: ObstacleType::Ice,
            countdown: None,
        }
    }

    pub fn bomb(countdown: u8) -> Self {
        Self {
            obstacle_type: ObstacleType::Bomb,
            countdown: Some(countdown),
        }
    }

    pub fn is_ice(&self) -> bool {
        self.obstacle_type == ObstacleType::Ice
    }

    pub fn is_bomb(&self) -> bool {
        self.obstacle_type == ObstacleType::Bomb
    }
}
```

### 2. PuzzleBoard Obstacle Layer

```rust
// src/puzzle/board.rs

#[derive(Resource)]
pub struct PuzzleBoard {
    pub grid: [[Option<Entity>; PUZZLE_BOARD_SIZE]; PUZZLE_BOARD_SIZE],
    pub obstacles: [[Option<ObstacleType>; PUZZLE_BOARD_SIZE]; PUZZLE_BOARD_SIZE],
    // ...
}

impl PuzzleBoard {
    pub fn get_obstacle(&self, x: usize, y: usize) -> Option<ObstacleType> {
        self.obstacles.get(y).and_then(|row| row.get(x)).copied().flatten()
    }

    pub fn set_obstacle(&mut self, x: usize, y: usize, obstacle: Option<ObstacleType>) {
        if y < PUZZLE_BOARD_SIZE && x < PUZZLE_BOARD_SIZE {
            self.obstacles[y][x] = obstacle;
        }
    }

    pub fn has_ice(&self, x: usize, y: usize) -> bool {
        self.get_obstacle(x, y) == Some(ObstacleType::Ice)
    }

    pub fn has_bomb(&self, x: usize, y: usize) -> bool {
        self.get_obstacle(x, y) == Some(ObstacleType::Bomb)
    }

    pub fn clear_obstacle(&mut self, x: usize, y: usize) {
        self.set_obstacle(x, y, None);
    }
}
```

### 3. Ice Clearing on Adjacent Match

```rust
// src/puzzle/match_detector.rs

pub fn remove_matched_tiles(
    mut commands: Commands,
    mut board: ResMut<PuzzleBoard>,
    matched: Query<(Entity, &GridPosition), With<Matched>>,
) {
    let matched_positions: Vec<(usize, usize)> = matched
        .iter()
        .map(|(_, pos)| (pos.x, pos.y))
        .collect();

    // Clear ice obstacles adjacent to matched tiles
    for (x, y) in &matched_positions {
        for (dx, dy) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
            let nx = (*x as i32 + dx) as usize;
            let ny = (*y as i32 + dy) as usize;
            if nx < PUZZLE_BOARD_SIZE && ny < PUZZLE_BOARD_SIZE {
                if board.has_ice(nx, ny) {
                    board.clear_obstacle(nx, ny);
                }
            }
        }
    }

    for (entity, pos) in matched.iter() {
        board.set(pos.x, pos.y, None);
        commands.entity(entity).despawn();
    }
}
```

### 4. Bomb Countdown and Damage Event

```rust
// src/battle/wave.rs

#[derive(Event)]
pub struct BombDamageEvent {
    pub position: (usize, usize),
    pub damage: u32,
}

pub fn bomb_countdown_system(
    mut commands: Commands,
    mut board: ResMut<PuzzleBoard>,
    mut obstacles: Query<(Entity, &GridPosition, &mut Obstacle)>,
) {
    for (entity, pos, mut obstacle) in obstacles.iter_mut() {
        if obstacle.is_bomb() {
            if let Some(ref mut countdown) = obstacle.countdown {
                if *countdown > 0 {
                    *countdown -= 1;
                } else {
                    // Bomb explodes
                    commands.trigger(BombDamageEvent {
                        position: (pos.x, pos.y),
                        damage: 10,
                    });
                    board.clear_obstacle(pos.x, pos.y);
                    commands.entity(entity).remove::<Obstacle>();
                }
            }
        }
    }
}

pub fn handle_bomb_damage(
    trigger: Trigger<BombDamageEvent>,
    mut player_units: Query<&mut UnitStats, (With<Unit>, With<Team>)>,
) {
    let event = trigger.event();
    let damage = event.damage as f32;

    for mut stats in player_units.iter_mut() {
        stats.health = (stats.health - damage).max(0.0);
    }
}
```

## Notes

### Ice Obstacles

- **Behavior**: Tiles on ice positions cannot be swapped/moved
- **Clearing**: Adjacent match (4-directional) clears ice
- **Visual**: Consider adding semi-transparent overlay sprite

### Bomb Obstacles

- **Countdown**: Decrements each turn (or game tick)
- **Explosion**: At countdown=0, triggers `BombDamageEvent`
- **Defusal**: Not implemented yet - could add defusal mechanic via specific match pattern

### Common Pitfalls

1. **Boundary Check**: Always validate `nx`, `ny` before accessing grid
   ```rust
   // Wrong: Can overflow for x=0
   let nx = (x as i32 + dx) as usize;

   // Right: Check bounds after conversion
   if nx < PUZZLE_BOARD_SIZE && ny < PUZZLE_BOARD_SIZE { ... }
   ```

2. **Dual Layer Management**: Keep `grid` and `obstacles` in sync
   - Clear obstacle when tile is removed
   - Check obstacle before allowing tile swap

3. **Event Ordering**: `bomb_countdown_system` should run in appropriate schedule
   - After player turn ends
   - Before cascade/refill phase

### Extension Ideas

- `Chain` obstacle: Links multiple tiles, must clear all at once
- `Lock` obstacle: Requires specific tile type to unlock
- `Portal` obstacle: Teleports matched tiles to another position
