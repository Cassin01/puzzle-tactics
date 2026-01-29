# 爆弾メカニクス仕様書

puzzle_tactics における爆弾（Bomb）障害物の仕様を定義する。

## 概要

爆弾は、敵の攻撃によってパズルボード上に出現する時限式障害物である。
カウントダウンが0になると爆発し、味方ユニット全体にダメージを与える。

---

## 1. 出現条件

### トリガー
- **Wave 5以降**に敵ユニットが攻撃を行った時
- **15%の確率**で出現

### 実装
```rust
// src/battle/combat.rs - maybe_spawn_obstacle_on_attack()
if current_wave >= 5 && rng.gen::<f32>() < 0.15 {
    commands.trigger(ObstacleSpawnEvent {
        position: (x, y),
        obstacle_type: ObstacleType::Bomb,
        countdown: Some(3),
    });
}
```

### 出現位置
- パズルボード上のランダムな座標
- 範囲: `(0..PUZZLE_BOARD_SIZE, 0..PUZZLE_BOARD_SIZE)`
- 既存のタイルや障害物との重複チェックなし

### 初期値
| パラメータ | 値 |
|-----------|-----|
| 初期カウントダウン | 3 |
| 出現確率 | 15% |
| 必要Wave | 5以上 |

---

## 2. カウントダウン

### システム
`bomb_countdown_system` が一定時間ごとにカウントを1減少させる。
カウントダウンは `BOMB_COUNTDOWN_INTERVAL` で定義された間隔（デフォルト1.5秒）で進行する。

### 実装
```rust
// src/battle/wave.rs - bomb_countdown_system()
pub const BOMB_COUNTDOWN_INTERVAL: f32 = 1.5;

countdown_timer.timer += time.delta_secs();
if countdown_timer.timer < BOMB_COUNTDOWN_INTERVAL {
    return;
}
countdown_timer.timer = 0.0;

if obstacle.is_bomb() {
    if let Some(ref mut countdown) = obstacle.countdown {
        if *countdown > 0 {
            *countdown -= 1;
        } else {
            // 爆発処理
        }
    }
}
```

### カウントダウン表示
- `BombCountdownText` コンポーネント（子エンティティ）
- 爆弾本体の中央に数字を表示
- `update_bomb_countdown_display` で毎フレーム更新

| パラメータ | 値 |
|-----------|-----|
| フォントサイズ | 28.0 |
| 文字色 | WHITE（白） |
| Z位置 | 0.1（爆弾本体より前面）|
| カウントダウン間隔 | 1.5秒 |

---

## 3. 爆発効果

### 爆発トリガー
カウントダウンが0になった時点で即座に爆発。

### 爆発処理の流れ

1. **エフェクト生成**
   ```rust
   commands.spawn((
       BombExplosionEffect { timer: 0.0, duration: 0.5 },
       Sprite {
           color: Color::srgba(1.0, 0.5, 0.0, 1.0),  // オレンジ
           custom_size: Some(Vec2::splat(TILE_SIZE)),
       },
   ));
   ```

2. **ダメージイベント発火**
   ```rust
   commands.trigger(BombDamageEvent {
       position: (pos.x, pos.y),
       damage: 10,
   });
   ```

3. **ボード状態クリア**
   ```rust
   board.clear_obstacle(pos.x, pos.y);
   ```

4. **エンティティ削除**
   ```rust
   commands.entity(entity).despawn_recursive();
   ```

### 爆発エフェクトアニメーション
`animate_bomb_explosion` システムで処理。

```rust
let progress = (effect.timer / effect.duration).min(1.0);
let scale = 1.0 + progress * 1.5;  // 拡大
sprite.color = Color::srgba(1.0, 0.5 - progress * 0.3, 0.0, 1.0 - progress);  // フェードアウト
```

| パラメータ | 値 |
|-----------|-----|
| 持続時間 | 0.5秒 |
| 最大スケール | 2.5倍 (1.0 + 1.5) |
| 色変化 | オレンジ → 赤みがかった色 + 透明化 |

---

## 4. ダメージ処理

### ダメージ対象
- **味方ユニット全体**にダメージを適用

### 実装
```rust
// src/battle/wave.rs - handle_bomb_damage()
pub fn handle_bomb_damage(
    trigger: Trigger<BombDamageEvent>,
    mut player_units: Query<&mut UnitStats, (With<Unit>, With<Team>)>,
) {
    let damage = event.damage as f32;
    for mut stats in player_units.iter_mut() {
        stats.health = (stats.health - damage).max(0.0);
    }
}
```

| パラメータ | 値 |
|-----------|-----|
| ダメージ量 | 10 |
| 対象 | 味方ユニット全員 |
| 最小HP | 0（死亡判定は別システム）|

---

## 5. 視覚表現

### 爆弾本体
```rust
// src/puzzle/obstacle.rs - spawn_bomb()
Sprite {
    color: Color::srgb(0.9, 0.4, 0.1),  // オレンジ（視認性向上のため）
    custom_size: Some(Vec2::splat(TILE_SIZE * 0.6)),
}
```

| パラメータ | 値 |
|-----------|-----|
| 色 | RGB(0.9, 0.4, 0.1) - オレンジ |
| サイズ | TILE_SIZE × 0.6 |
| Z位置 | 0.5 |

### カウントダウン数字
| パラメータ | 値 |
|-----------|-----|
| フォントサイズ | 28.0 |
| 色 | WHITE（白） |
| 位置 | 爆弾中央 (0, 0, 0.1) |

### 爆発エフェクト
| パラメータ | 値 |
|-----------|-----|
| 初期色 | RGBA(1.0, 0.5, 0.0, 1.0) - オレンジ |
| 初期サイズ | TILE_SIZE |
| アニメーション | 拡大 + フェードアウト |

---

## 6. 戦略的意義

### プレイヤーへの影響

1. **時間制限のプレッシャー**
   - 3ターン以内に対処する必要性
   - パズル操作の優先順位決定を迫られる

2. **リスク評価**
   - 爆発前に消すか、被害を受け入れるかの判断
   - 他の高価値マッチとのトレードオフ

3. **連鎖計画への影響**
   - 爆弾位置によっては連鎖ルートが制限される
   - 爆弾を含めた連鎖で効率的に処理

### ゲームバランス上の役割
- Wave 5以降の難易度上昇要素
- パズル→バトル連携の緊張感を高める
- 「動的な盤面変化」コンセプトの実現

---

## 関連ファイル

| ファイル | 役割 |
|---------|------|
| `src/battle/combat.rs` | 出現判定 (`maybe_spawn_obstacle_on_attack`) |
| `src/puzzle/obstacle.rs` | 生成処理 (`spawn_bomb`, `BombCountdownText`) |
| `src/battle/wave.rs` | カウントダウン・爆発処理 (`bomb_countdown_system`, `animate_bomb_explosion`, `handle_bomb_damage`) |
| `src/puzzle/tile.rs` | `Obstacle`, `ObstacleType` 定義 |
| `src/bridge/events.rs` | `ObstacleSpawnEvent` 定義 |
