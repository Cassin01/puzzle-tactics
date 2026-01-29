# 氷メカニクス仕様書

> **Version**: 1.0.0
> **Last Updated**: 2026-01-29
> **関連ファイル**: `src/puzzle/obstacle.rs`, `src/battle/combat.rs`, `src/puzzle/board.rs`

## 概要

氷は「動的な盤面変化」システムの一部であり、敵の攻撃によってパズルボードに出現する妨害要素である。プレイヤーのパズル操作を制限し、戦略的な判断を要求する。

---

## 1. 出現条件

### トリガー

| 条件 | 確率 | 備考 |
|------|------|------|
| Wave 3 以降で敵が攻撃 | 10% | `maybe_spawn_obstacle_on_attack` 関数で判定 |

### 実装詳細

```rust
// src/battle/combat.rs
fn maybe_spawn_obstacle_on_attack(commands: &mut Commands, current_wave: u32) {
    // Wave 3+: 10% chance to spawn ice
    if current_wave >= 3 && rng.gen::<f32>() < 0.10 {
        let x = rng.gen_range(0..PUZZLE_BOARD_SIZE);
        let y = rng.gen_range(0..PUZZLE_BOARD_SIZE);
        commands.trigger(ObstacleSpawnEvent {
            position: (x, y),
            obstacle_type: ObstacleType::Ice,
            countdown: None,
        });
    }
}
```

### 生成位置

- パズルボード上のランダムな座標 (0..PUZZLE_BOARD_SIZE)
- 既存のタイルがある位置に重ねて生成される
- 既に氷がある位置にも重複して生成される可能性あり（要検討）

---

## 2. 効果

### スワップ不可

氷タイルは移動（スワップ）できない。

```rust
// src/puzzle/board.rs
impl PuzzleBoard {
    pub fn has_ice(&self, x: usize, y: usize) -> bool {
        self.get_obstacle(x, y) == Some(ObstacleType::Ice)
    }
}
```

入力処理（`input.rs`）でスワップ前に `has_ice` をチェックし、氷タイルの場合はスワップをキャンセルする。

### 視覚的表現

| プロパティ | 値 | 説明 |
|-----------|-----|------|
| 色 (RGBA) | `(0.7, 0.9, 1.0, 0.7)` | 半透明の淡い青 |
| サイズ | `TILE_SIZE` | タイルと同サイズ |
| Z座標 | `0.5` | タイルの上に表示 |

```rust
// src/puzzle/obstacle.rs
fn spawn_ice(commands: &mut Commands, board: &PuzzleBoard, x: usize, y: usize) {
    let pos = board.grid_to_world(x, y);

    commands.spawn((
        Obstacle::ice(),
        GridPosition::new(x, y),
        IceOverlay::default(),
        Sprite {
            color: Color::srgba(0.7, 0.9, 1.0, 0.7),
            custom_size: Some(Vec2::splat(TILE_SIZE)),
            ..default()
        },
        Transform::from_translation(pos.extend(0.5)),
        Visibility::default(),
    ));
}
```

### IceOverlay コンポーネント

```rust
#[derive(Component)]
pub struct IceOverlay {
    pub melting: bool,  // 溶け始めたかどうか
    pub alpha: f32,     // 現在の透明度（デフォルト: 0.7）
}
```

---

## 3. 解除方法

### 隣接マッチで溶ける

氷タイルの隣接タイル（上下左右）をマッチさせて消すと、氷が溶ける。

```rust
// src/puzzle/match_detector.rs (remove_matched_tiles 内)
// Clear ice obstacles adjacent to matched tiles
for (x, y) in &matched_positions {
    for (dx, dy) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
        let nx = *x as i32 + dx;
        let ny = *y as i32 + dy;
        if board.has_ice(nx as usize, ny as usize) {
            board.clear_obstacle(nx as usize, ny as usize);
        }
    }
}
```

### IceMeltEvent

氷を溶かす際は `IceMeltEvent` を発火し、アニメーション付きで消える。

```rust
#[derive(Event)]
pub struct IceMeltEvent {
    pub position: (usize, usize),
}
```

### フェードアウトアニメーション

| パラメータ | 値 | 説明 |
|-----------|-----|------|
| MELT_SPEED | `0.5` | 1秒あたりの透明度減少量 |
| 完了条件 | `alpha <= 0.0` | 完全に透明になったら削除 |

```rust
fn ice_melt_animation_system(...) {
    const MELT_SPEED: f32 = 0.5;

    for (entity, pos, mut overlay, mut sprite) in ice_query.iter_mut() {
        if overlay.melting {
            overlay.alpha -= MELT_SPEED * time.delta_secs();

            if overlay.alpha <= 0.0 {
                board.clear_obstacle(pos.x, pos.y);
                commands.entity(entity).despawn_recursive();
            } else {
                sprite.color = Color::srgba(0.7, 0.9, 1.0, overlay.alpha);
            }
        }
    }
}
```

---

## 4. 視覚的フィードバック

### スワップ不可の表現（IceShakeAnimation）

氷タイルをスワップしようとすると、横揺れアニメーションが発動し、操作不可を視覚的に伝える。

| パラメータ | 推奨値 | 説明 |
|-----------|--------|------|
| 振幅 | 3-5 px | 横方向の揺れ幅 |
| 周期 | 0.1 秒 | 1往復の時間 |
| 回数 | 2-3 回 | 揺れる回数 |

**実装ステータス**: 未実装（将来実装予定）

---

## 5. 戦略的意義

### プレイヤーへの影響

1. **盤面の動きを制限**
   - 氷タイルはスワップ不可のため、連鎖の起点として使えない
   - 盤面の自由度が低下し、パズル難易度が上昇

2. **早期解除の重要性**
   - 氷が増えると連鎖が困難になる
   - 隣接マッチを優先して氷を解除する戦略が有効

3. **連鎖妨害**
   - カスケード（落下）時に氷タイルが邪魔になる
   - 意図した連鎖が途切れる可能性

### 推奨プレイスタイル

- Wave 3 以降は氷の出現に備える
- 氷が出現したら早めに隣接マッチで解除
- 盤面端の氷は放置しても影響が少ない場合がある

---

## 6. 関連システム

| システム | ファイル | 関係 |
|---------|---------|------|
| 障害物スポーン | `src/puzzle/obstacle.rs` | 氷エンティティの生成 |
| 敵攻撃システム | `src/battle/combat.rs` | 氷生成のトリガー |
| マッチ検出 | `src/puzzle/match_detector.rs` | 隣接マッチで氷解除 |
| パズルボード | `src/puzzle/board.rs` | `has_ice()` チェック |
| 入力処理 | `src/puzzle/input.rs` | スワップ不可判定 |

---

## 7. 定数一覧

| 定数名 | 値 | 定義場所 | 説明 |
|--------|-----|---------|------|
| ICE_SPAWN_CHANCE | 0.10 (10%) | `combat.rs` | 氷生成確率 |
| ICE_SPAWN_MIN_WAVE | 3 | `combat.rs` | 氷生成開始 Wave |
| ICE_COLOR | (0.7, 0.9, 1.0, 0.7) | `obstacle.rs` | 氷の色 |
| MELT_SPEED | 0.5 | `obstacle.rs` | 溶けるアニメーション速度 |
| ICE_OVERLAY_ALPHA | 0.7 | `obstacle.rs` | 初期透明度 |

---

## 変更履歴

| 日付 | バージョン | 変更内容 |
|------|-----------|---------|
| 2026-01-29 | 1.0.0 | 初版作成 |
