use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use crate::input::PlayerInput;
use crate::dino::RespawnDinosEvent;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Playing,
    Paused,
}

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_event::<RestartGameEvent>()
            .add_systems(OnEnter(GameState::Playing), setup_cursor)
            .add_systems(OnEnter(GameState::Paused), (show_cursor, spawn_pause_menu))
            .add_systems(OnExit(GameState::Paused), (hide_cursor, despawn_pause_menu))
            .add_systems(Update, (
                handle_pause_input.run_if(in_state(GameState::Playing)),
                handle_pause_menu_input.run_if(in_state(GameState::Paused)),
                handle_restart_game,
            ));
    }
}

#[derive(Event)]
pub struct RestartGameEvent;

#[derive(Component)]
pub struct PauseMenu;

#[derive(Component)]
pub struct ResumeButton;

#[derive(Component)]
pub struct RestartButton;

#[derive(Component)]
pub struct QuitButton;

fn setup_cursor(mut window_q: Query<&mut Window>) {
    if let Ok(mut window) = window_q.get_single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
    }
}

fn show_cursor(mut window_q: Query<&mut Window>) {
    if let Ok(mut window) = window_q.get_single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
    }
}

fn hide_cursor(mut window_q: Query<&mut Window>) {
    if let Ok(mut window) = window_q.get_single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
    }
}

fn spawn_pause_menu(mut commands: Commands) {
    // Pause menu container
    commands.spawn((
        PauseMenu,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
    )).with_children(|parent| {
        // Title
        parent.spawn((
            Text::new("PAUSED"),
            TextFont {
                font_size: 60.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::bottom(Val::Px(40.0)),
                ..default()
            },
        ));

        // Instructions text
        parent.spawn((
            Text::new("Click buttons or press keys: [R] Restart  [Q] Quit  [ESC] Resume"),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
            Node {
                margin: UiRect::bottom(Val::Px(30.0)),
                ..default()
            },
        ));

        // Resume button
        parent.spawn((
            ResumeButton,
            Button {
                ..default()
            },
            Node {
                width: Val::Px(200.0),
                height: Val::Px(50.0),
                margin: UiRect::bottom(Val::Px(20.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.5, 0.8)),
        )).with_children(|parent| {
            parent.spawn((
                Text::new("Resume [ESC]"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });

        // Restart button
        parent.spawn((
            RestartButton,
            Button {
                ..default()
            },
            Node {
                width: Val::Px(200.0),
                height: Val::Px(50.0),
                margin: UiRect::bottom(Val::Px(20.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.5, 0.5, 0.2)),
        )).with_children(|parent| {
            parent.spawn((
                Text::new("Restart [R]"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });

        // Quit button
        parent.spawn((
            QuitButton,
            Button {
                ..default()
            },
            Node {
                width: Val::Px(200.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.8, 0.2, 0.2)),
        )).with_children(|parent| {
            parent.spawn((
                Text::new("Quit [Q]"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
    });
}

fn despawn_pause_menu(
    mut commands: Commands,
    menu_q: Query<Entity, With<PauseMenu>>,
) {
    for entity in menu_q.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn handle_pause_input(
    input: Res<PlayerInput>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if input.pause {
        next_state.set(GameState::Paused);
    }
}

fn handle_pause_menu_input(
    mut next_state: ResMut<NextState<GameState>>,
    mut restart_events: EventWriter<RestartGameEvent>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut interaction_q: Query<
        (Option<&ResumeButton>, Option<&RestartButton>, Option<&QuitButton>),
        (With<Button>, Changed<Interaction>),
    >,
    mut app_exit: EventWriter<bevy::app::AppExit>,
) {
    // Handle keyboard shortcuts
    if keyboard.just_pressed(KeyCode::KeyR) {
        restart_events.send(RestartGameEvent);
        next_state.set(GameState::Playing);
        return;
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Playing);
        return;
    }
    if keyboard.just_pressed(KeyCode::KeyQ) {
        app_exit.send(bevy::app::AppExit::Success);
        return;
    }

    // Handle mouse clicks on buttons
    for (resume_opt, restart_opt, quit_opt) in interaction_q.iter_mut() {
        // Check if button was just clicked (interaction changed to Pressed)
        if resume_opt.is_some() {
            next_state.set(GameState::Playing);
        } else if restart_opt.is_some() {
            restart_events.send(RestartGameEvent);
            next_state.set(GameState::Playing);
        } else if quit_opt.is_some() {
            app_exit.send(bevy::app::AppExit::Success);
        }
    }
}

fn handle_restart_game(
    mut events: EventReader<RestartGameEvent>,
    mut commands: Commands,
    dino_q: Query<Entity, With<crate::dino::Dinosaur>>,
    bullet_q: Query<Entity, With<crate::weapon::Bullet>>,
    mut score: ResMut<crate::GameScore>,
    mut target_lock: ResMut<crate::input::TargetLock>,
    mut respawn_events: EventWriter<RespawnDinosEvent>,
) {
    for _event in events.read() {
        // Reset score
        score.score = 0;

        // Reset target lock
        target_lock.locked_entity = None;
        target_lock.lock_position = None;

        // Despawn all dinosaurs
        for entity in dino_q.iter() {
            commands.entity(entity).despawn_recursive();
        }

        // Despawn all bullets
        for entity in bullet_q.iter() {
            commands.entity(entity).despawn_recursive();
        }

        // Note: Blood particles will despawn themselves on their timer
        // No need to manually despawn them

        // Respawn dinosaurs
        respawn_events.send(RespawnDinosEvent);
    }
}
