use crate::prelude::*;
use crate::bridge::ObstacleSpawnEvent;
use super::board::PuzzleBoard;
use super::tile::{GridPosition, Obstacle, ObstacleType};

/// Marker component for bomb countdown text display
#[derive(Component)]
pub struct BombCountdownText;

/// Ice overlay visual effect component for enhanced ice rendering
#[derive(Component)]
pub struct IceOverlay {
    pub melting: bool,
    pub alpha: f32,
}

impl Default for IceOverlay {
    fn default() -> Self {
        Self {
            melting: false,
            alpha: 0.7,
        }
    }
}

/// Event to trigger ice melting animation
#[derive(Event)]
pub struct IceMeltEvent {
    pub position: (usize, usize),
}

/// Event to trigger bomb defuse (when matched tile with bomb is removed)
#[derive(Event)]
pub struct BombDefuseEvent {
    pub position: (usize, usize),
}

/// Visual effect component for bomb defuse animation
#[derive(Component)]
pub struct BombDefuseEffect {
    pub timer: f32,
}

pub struct ObstaclePlugin;

impl Plugin for ObstaclePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_obstacle_spawn)
            .add_observer(handle_ice_melt)
            .add_observer(handle_bomb_defuse)
            .add_systems(
                Update,
                (
                    sync_bomb_position_with_parent,
                    update_bomb_countdown_display,
                    ice_melt_animation_system,
                    bomb_defuse_animation_system,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn handle_obstacle_spawn(
    trigger: Trigger<ObstacleSpawnEvent>,
    mut commands: Commands,
    mut board: ResMut<PuzzleBoard>,
) {
    let event = trigger.event();
    let (x, y) = event.position;

    if x >= PUZZLE_BOARD_SIZE || y >= PUZZLE_BOARD_SIZE {
        return;
    }

    match event.obstacle_type {
        ObstacleType::Ice => {
            board.set_obstacle(x, y, Some(event.obstacle_type));
            spawn_ice(&mut commands, &board, x, y);
        }
        ObstacleType::Bomb => {
            // Bomb attaches to tile as child entity
            // Only set obstacle if tile exists to prevent board state inconsistency
            if let Some(tile_entity) = board.get(x, y) {
                board.set_obstacle(x, y, Some(event.obstacle_type));
                spawn_bomb(&mut commands, tile_entity, event.countdown.unwrap_or(3), x, y);
            }
        }
    }
}

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

fn spawn_bomb(commands: &mut Commands, parent_tile: Entity, countdown: u8, x: usize, y: usize) {
    // Spawn bomb as child of tile - it will move with the tile during swaps
    let bomb_entity = commands
        .spawn((
            Obstacle::bomb(countdown),
            GridPosition::new(x, y),
            Sprite {
                color: Color::srgb(0.9, 0.4, 0.1), // Orange color for better visibility
                custom_size: Some(Vec2::splat(TILE_SIZE * 0.6)),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.5)), // Relative to parent tile
            Visibility::default(),
        ))
        .with_child((
            BombCountdownText,
            Text2d::new(countdown.to_string()),
            TextFont {
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)),
        ))
        .id();

    commands.entity(parent_tile).add_child(bomb_entity);
}

/// Sync bomb GridPosition with parent tile's GridPosition after swaps
fn sync_bomb_position_with_parent(
    tiles: Query<(&GridPosition, &Children), With<super::tile::Tile>>,
    mut bombs: Query<&mut GridPosition, (With<Obstacle>, Without<super::tile::Tile>)>,
) {
    for (tile_pos, children) in tiles.iter() {
        for &child in children.iter() {
            if let Ok(mut bomb_pos) = bombs.get_mut(child) {
                if bomb_pos.x != tile_pos.x || bomb_pos.y != tile_pos.y {
                    bomb_pos.x = tile_pos.x;
                    bomb_pos.y = tile_pos.y;
                }
            }
        }
    }
}

/// Updates the bomb countdown display text to match the current countdown value
fn update_bomb_countdown_display(
    obstacles: Query<(&Obstacle, &Children)>,
    mut countdown_texts: Query<&mut Text2d, With<BombCountdownText>>,
) {
    for (obstacle, children) in obstacles.iter() {
        if let Some(countdown) = obstacle.countdown {
            for &child in children.iter() {
                if let Ok(mut text) = countdown_texts.get_mut(child) {
                    **text = countdown.to_string();
                }
            }
        }
    }
}

/// Handle ice melt event - starts the melting animation
fn handle_ice_melt(
    trigger: Trigger<IceMeltEvent>,
    mut ice_overlays: Query<(&GridPosition, &mut IceOverlay)>,
) {
    let event = trigger.event();

    for (pos, mut overlay) in ice_overlays.iter_mut() {
        if pos.x == event.position.0 && pos.y == event.position.1 {
            overlay.melting = true;
        }
    }
}

/// Animate ice melting with fade-out effect
fn ice_melt_animation_system(
    mut commands: Commands,
    time: Res<Time>,
    mut board: ResMut<PuzzleBoard>,
    mut ice_query: Query<(Entity, &GridPosition, &mut IceOverlay, &mut Sprite)>,
) {
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

/// Handle bomb defuse event - removes bomb when its parent tile is matched
fn handle_bomb_defuse(
    trigger: Trigger<BombDefuseEvent>,
    mut commands: Commands,
    board: Res<PuzzleBoard>,
    obstacles: Query<(Entity, &GridPosition), With<Obstacle>>,
) {
    let (x, y) = trigger.event().position;

    // Find and remove the bomb at this position
    for (entity, pos) in obstacles.iter() {
        if pos.x == x && pos.y == y {
            // Spawn defuse effect at world position
            let world_pos = board.grid_to_world(x, y);
            commands.spawn((
                BombDefuseEffect { timer: 0.0 },
                Sprite {
                    color: Color::srgba(0.2, 0.9, 0.3, 1.0),
                    custom_size: Some(Vec2::splat(TILE_SIZE)),
                    ..default()
                },
                Transform::from_translation(world_pos.extend(1.5)),
                Visibility::default(),
            ));

            // Spawn defuse effect. The bomb entity will be despawned with the parent tile
            // via despawn_recursive, but we trigger the visual effect here.
            commands.entity(entity).despawn_recursive();
            break;
        }
    }
}

/// Animate bomb defuse effect (green expanding circle that fades)
fn bomb_defuse_animation_system(
    mut commands: Commands,
    time: Res<Time>,
    mut effects: Query<(Entity, &mut BombDefuseEffect, &mut Sprite, &mut Transform)>,
) {
    const DEFUSE_DURATION: f32 = 0.4;

    for (entity, mut effect, mut sprite, mut transform) in effects.iter_mut() {
        effect.timer += time.delta_secs();
        let progress = effect.timer / DEFUSE_DURATION;

        if progress >= 1.0 {
            commands.entity(entity).despawn();
        } else {
            sprite.color = sprite.color.with_alpha(1.0 - progress);
            transform.scale = Vec3::splat(1.0 + progress * 0.5);
        }
    }
}
