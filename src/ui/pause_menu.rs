use crate::prelude::*;

#[derive(Component)]
pub struct PauseMenuRoot;

#[derive(Component)]
pub struct ResumeButton;

#[derive(Component)]
pub struct QuitButton;

pub fn handle_pause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match current_state.get() {
            GameState::Playing => {
                next_state.set(GameState::Paused);
            }
            GameState::Paused => {
                next_state.set(GameState::Playing);
            }
            _ => {}
        }
    }
}

pub fn setup_pause_menu(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            PauseMenuRoot,
        ))
        .with_children(|parent| {
            // PAUSED text
            parent.spawn((
                Text::new("PAUSED"),
                TextFont {
                    font_size: 64.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Resume button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.6, 0.2)),
                    ResumeButton,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Resume"),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Quit to Title button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.6, 0.2, 0.2)),
                    QuitButton,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Quit to Title"),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

pub fn cleanup_pause_menu(
    mut commands: Commands,
    menu_query: Query<Entity, With<PauseMenuRoot>>,
) {
    for entity in menu_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn handle_resume_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ResumeButton>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                next_state.set(GameState::Playing);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.3, 0.7, 0.3));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.6, 0.2));
            }
        }
    }
}

pub fn handle_quit_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<QuitButton>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                next_state.set(GameState::Loading);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.7, 0.3, 0.3));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.6, 0.2, 0.2));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::input::ButtonInput;
    use bevy::state::app::StatesPlugin;

    fn setup_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(StatesPlugin)
            .init_resource::<ButtonInput<KeyCode>>()
            .init_state::<GameState>()
            .add_systems(
                Update,
                handle_pause_input
                    .run_if(in_state(GameState::Playing).or(in_state(GameState::Paused))),
            );
        app
    }

    fn setup_button_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(StatesPlugin)
            .init_state::<GameState>()
            .add_systems(Update, handle_resume_button.run_if(in_state(GameState::Paused)))
            .add_systems(Update, handle_quit_button.run_if(in_state(GameState::Paused)));
        app
    }

    #[test]
    fn test_esc_playing_to_paused() {
        let mut app = setup_test_app();

        // Set initial state to Playing and apply
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update(); // Apply state transition

        // Simulate ESC key press
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);

        app.update(); // System runs, sets NextState
        app.update(); // State transition applies

        // Verify state changed to Paused
        let state = app.world().resource::<State<GameState>>();
        assert_eq!(*state.get(), GameState::Paused);
    }

    #[test]
    fn test_esc_paused_to_playing() {
        let mut app = setup_test_app();

        // Set initial state to Paused and apply
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Paused);
        app.update(); // Apply state transition

        // Simulate ESC key press
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);

        app.update(); // System runs, sets NextState
        app.update(); // State transition applies

        // Verify state changed to Playing
        let state = app.world().resource::<State<GameState>>();
        assert_eq!(*state.get(), GameState::Playing);
    }

    #[test]
    fn test_resume_button_transitions_to_playing() {
        let mut app = setup_button_test_app();

        // Set state to Paused and apply
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Paused);
        app.update(); // Apply state transition

        // Spawn a ResumeButton with None interaction first
        let entity = app
            .world_mut()
            .spawn((
                Button,
                ResumeButton,
                Interaction::None,
                BackgroundColor(Color::srgb(0.2, 0.6, 0.2)),
            ))
            .id();

        app.update(); // Entity registered

        // Change to Pressed to trigger Changed<Interaction>
        app.world_mut()
            .entity_mut(entity)
            .insert(Interaction::Pressed);

        app.update(); // System runs, sets NextState
        app.update(); // State transition applies

        // Verify state changed to Playing
        let state = app.world().resource::<State<GameState>>();
        assert_eq!(*state.get(), GameState::Playing);
    }

    #[test]
    fn test_quit_button_transitions_to_loading() {
        let mut app = setup_button_test_app();

        // Set state to Paused and apply
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Paused);
        app.update(); // Apply state transition

        // Spawn a QuitButton with None interaction first
        let entity = app
            .world_mut()
            .spawn((
                Button,
                QuitButton,
                Interaction::None,
                BackgroundColor(Color::srgb(0.6, 0.2, 0.2)),
            ))
            .id();

        app.update(); // Entity registered

        // Change to Pressed to trigger Changed<Interaction>
        app.world_mut()
            .entity_mut(entity)
            .insert(Interaction::Pressed);

        app.update(); // System runs, sets NextState
        app.update(); // State transition applies

        // Verify state changed to Loading
        let state = app.world().resource::<State<GameState>>();
        assert_eq!(*state.get(), GameState::Loading);
    }
}
