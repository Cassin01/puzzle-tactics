# Pazzle Tactics - Implementation Status

> **Version**: 1.0.0
> **Last Updated**: 2026-01-29
> **Status**: Alpha (Core Systems Implemented)

## Project Overview

| Item | Details |
|------|---------|
| **Project Name** | Pazzle Tactics |
| **Path** | `/Users/koyo.jimbo/2025/pazzle_tactics/` |
| **Concept** | Match-3 Puzzle + Auto-Chess Hybrid Strategy Game |
| **Tech Stack** | Rust 2021 Edition + Bevy 0.15 (ECS Architecture) |
| **Platforms** | Native (wgpu: Vulkan/Metal/DX12), WASM/Web |
| **Build Size** | ~9.3GB (including build artifacts) |

---

## Architecture Overview

```
src/
├── main.rs              # Entry point, plugin registration
├── state.rs             # Game state management
├── puzzle/              # Match-3 puzzle system
│   ├── mod.rs
│   ├── board.rs         # Board structure, tile types
│   ├── input.rs         # Player input handling
│   ├── matching.rs      # Match detection algorithm
│   └── cascade.rs       # Gravity & cascade system
├── battle/              # Auto-chess battle system
│   ├── mod.rs
│   ├── grid.rs          # Hexagonal grid implementation
│   ├── unit.rs          # Unit stats & behavior
│   ├── combat.rs        # Combat mechanics
│   ├── synergy.rs       # Synergy bonus system
│   └── wave.rs          # Wave spawning system
├── bridge/              # Puzzle-to-Battle integration
│   └── mod.rs           # Event translation layer
└── ui/                  # User interface
    └── mod.rs           # HUD, overlays, status displays
```

---

## Implemented Features

### 1. Puzzle System (`src/puzzle/`)

#### Board Configuration

| Parameter | Value |
|-----------|-------|
| Grid Size | 8 × 8 (64 tiles) |
| Tile Size | 64px × 64px |
| Tile Spacing | 4px |
| Window Size | 800 × 900px |

#### Tile Types (5 types, 20% probability each)

| Type | RGB Color | Mapped Unit |
|------|-----------|-------------|
| Red | (0.9, 0.2, 0.2) | Warrior |
| Blue | (0.2, 0.4, 0.9) | Tank |
| Green | (0.2, 0.8, 0.3) | Ranger |
| Yellow | (0.9, 0.8, 0.2) | Assassin |
| Purple | (0.7, 0.3, 0.8) | Mage |

#### Crystal Core

- **Position**: (3,3), (3,4), (4,3), (4,4) - 2×2 center block
- **Effect**: Matches adjacent to core trigger `CoreAbilityEvent`

#### Input System

- Two-click selection (first click selects, second click swaps with adjacent)
- Ice tiles cannot be swapped
- Diagonal swaps prohibited (up/down/left/right only)

#### Match Detection

- Detects 3+ consecutive tiles horizontally and vertically
- T-shape and L-shape intersections count for both directions (duplicates removed)
- Adjacent ice obstacles destroyed after match

#### Cascade System

- Gravity applied (tiles compress upward)
- Empty cells filled with new random tiles
- Chain matches increment combo counter

#### Mana Supply Formula

```
base_mana = 10.0
combo_bonus:
  1 → 1.0x (10 mana)
  2 → 1.5x (15 mana)
  3 → 2.0x (20 mana)
  4 → 3.0x (30 mana)
  5+ → 3.0 + (n-4) × 0.5
```

#### Obstacle System

| Type | Behavior |
|------|----------|
| Ice | Cannot be swapped, destroyed by adjacent match |
| Bomb | Countdown timer, explodes dealing 10 damage to all player units |

---

### 2. Battle System (`src/battle/`)

#### Hexagonal Grid

| Parameter | Value |
|-----------|-------|
| Size | 7 columns × 4 rows |
| Coordinate System | Axial (q, r) |
| Hex Size | 40px |
| Distance Calculation | Manhattan-style hex distance |

#### Unit Statistics

| Type | HP | ATK | Attack Speed | Range | Trait |
|------|-----|-----|--------------|-------|-------|
| Red (Warrior) | 80 | 15 | 1.2 | 1 | High Attack |
| Blue (Tank) | 120 | 8 | 0.8 | 1 | High HP |
| Green (Ranger) | 90 | 12 | 1.0 | 3 | Long Range |
| Yellow (Assassin) | 70 | 18 | 1.5 | 1 | High DPS |
| Purple (Mage) | 100 | 10 | 1.0 | 2 | Mana-based |

#### Star Rank Multipliers

| Rank | Multiplier | Example: Red HP/ATK |
|------|------------|---------------------|
| 1★ | 1.0x | 80 / 15 |
| 2★ | 1.8x | 144 / 27 |
| 3★ | 3.0x | 240 / 45 |

#### Combat System

- **Targeting**: Automatically selects nearest enemy
- **Movement**: Greedy algorithm to adjacent hex (not A*)
- **Attack**: Cooldown-based (1.0 / attack_speed seconds)
- **Damage**: Direct attack power application (no defense calculation)

#### Health Bar

| Parameter | Value |
|-----------|-------|
| Size | 30px width × 4px height |
| Offset | +25px above unit |
| Color | Green (>50%) → Yellow (>25%) → Red (≤25%) |

---

### 3. Synergy System (`src/battle/synergy.rs`)

#### Synergy Levels

| Level | Unit Count | Base Multiplier |
|-------|------------|-----------------|
| None | 0-1 | 1.0x |
| Bronze | 2-3 | 1.15x (+15%) |
| Silver | 4-5 | 1.30x (+30%) |
| Gold | 6+ | 1.50x (+50%) |

#### Unit-Specific Bonuses

| Type | Bronze | Silver | Gold |
|------|--------|--------|------|
| Red (Warrior) | ATK+15%, HP+10% | ATK+30%, HP+20% | ATK+50%, HP+30% |
| Blue (Tank) | HP+15%, DEF+15% | HP+30%, DEF+30% | HP+50%, DEF+50% |
| Green (Ranger) | Range+1, SPD+15% | Range+2, SPD+30% | Range+3, SPD+50% |
| Yellow (Assassin) | ATK+15%, Crit+10% | ATK+30%, Crit+20% | ATK+50%, Crit+30% |
| Purple (Mage) | AP+15%, ManaRegen×1.2 | AP+30%, ManaRegen×1.4 | AP+50%, ManaRegen×1.6 |

---

### 4. Wave System (`src/battle/wave.rs`)

#### Enemy Spawning

| Parameter | Value |
|-----------|-------|
| Wave Interval | 10 seconds |
| Spawn Interval | 0.8 seconds |
| Enemy Count | `min(3 + wave × 2, 12)` |

#### Wave Progression

| Wave | Enemy Count | Star Rank Probability |
|------|-------------|----------------------|
| 0-2 | 3-7 | 100% 1★ |
| 3-5 | 9-11 | 70% 1★, 30% 2★ |
| 6+ | 12 | 50% 1★, 50% 2★ |

#### Win/Loss Conditions

**Victory**:
- Survive 10 waves + eliminate all enemies

**Defeat**:
- All units eliminated (after wave start)
- Enemy reaches baseline (r ≤ -2)
- 0 units for 5 seconds while enemies exist

---

### 5. Bridge System (`src/bridge/`)

#### Event Flow

```
MatchEvent (3+ tiles)
  ├─→ UnitSummonEvent (Star Rank: 3-4 tiles=1★, 5+ tiles=2★)
  │     └─→ Unit summon or merge
  └─→ SkillOrbEvent (4+ tiles only)
        ├─ Red/Purple → Meteor (50 damage to all enemies)
        ├─ Blue/Yellow → Buff (ally ATK ×1.2)
        └─ Green → Heal (ally HP +30%)
```

#### Unit Merge

- 2 units of same type and rank → 1 unit of next rank
- Maximum rank: 3★

---

### 6. UI System (`src/ui/`)

#### HUD Elements

| Position | Content |
|----------|---------|
| Top-Left | Wave number, Score |
| Top-Right | Active synergy display |
| Top-Center | Combo counter (displayed at 2x+) |
| Game Over | Semi-transparent overlay + VICTORY/GAME OVER text |

---

### 7. State Management (`src/state.rs`)

#### GameState

```
Loading → Playing → GameOver
         ↓
       Paused (defined only)
```

#### PhaseState

```
Idle → Cascading → Idle (cascade loop)
```

---

## Partial / Not Implemented

### Partially Implemented

| Feature | Status |
|---------|--------|
| Ability System | Framework only (Blue: 20% HP recovery implemented) |
| Critical Hit | Stats exist, damage calculation not implemented |
| Defense | Stats exist, damage reduction not implemented |
| Mana Regen | Stats exist, auto-recovery not implemented |
| Audio | `assets/audio/` directory only |

### Not Implemented

| Feature | Notes |
|---------|-------|
| Pause Menu | UI defined, interactions not implemented |
| Save/Load | No persistence system |
| Animations/Particles | No visual effects |
| Mobile Touch | Native only |
| Multiplayer | Single player only |
| Shop/Unit Selection | No economy system |
| 3★ Enemies | Maximum enemy rank is 2★ |

---

## Build & Run

```bash
# Development build
cargo run

# Release build
cargo build --release

# WASM build (requires wasm-pack)
cargo build --target wasm32-unknown-unknown
```

---

## Next Steps (Recommended Priority)

1. **Critical/Defense Calculation** - Enable existing stats in damage formula
2. **Ability System Completion** - Implement remaining unit abilities
3. **Pause Menu Functionality** - ESC to pause, resume, quit options
4. **Basic Sound Effects** - Match, attack, victory/defeat sounds
5. **Visual Polish** - Tile swap animations, damage numbers

---

## Technical Notes

### ECS Architecture
- All game logic implemented as Bevy systems
- Events used for cross-system communication
- Resources for shared state (board, wave info, synergies)

### Performance Considerations
- Match detection runs only on swap events
- Cascade uses phased state machine to prevent race conditions
- Unit targeting cached, recalculated only on death events

### Known Limitations
- Greedy pathfinding can cause unit clustering
- No unit collision avoidance
- Large matches (6+) may cause frame drops due to cascade iterations
