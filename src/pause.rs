use bevy::prelude::*;
use crate::input::PlayerInput;

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
            .add_systems(OnEnter(GameState::Paused), spawn_pause_menu)
            .add_systems(OnExit(GameState::Paused), despawn_pause_menu)
            .add_systems(Update, (handle_pause_input.run_if(in_state(GameState::Playing)), handle_pause_menu_input.run_if(in_state(GameState::Paused))));
    }
}

#[derive(Component)]
pub struct PauseMenu;

#[derive(Component)]
pub struct ResumeButton;

#[derive(Component)]
pub struct QuitButton;

fn spawn_pause_menu(mut commands: Commands) {
    // Create camera for UI
    commands.spawn((
        Camera2d,
        Camera {
            order: 999,
            ..default()
        },
    ));

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
                Text::new("Resume"),
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
                Text::new("Quit Game"),
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
    camera_q: Query<Entity, (With<Camera2d>, Without<Camera3d>)>,
) {
    for entity in menu_q.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in camera_q.iter() {
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
    mut interaction_q: Query<
        (&Interaction, &Parent),
        (With<Button>, Changed<Interaction>),
    >,
    children_q: Query<&Children>,
    resume_button_q: Query<&ResumeButton>,
    quit_button_q: Query<&QuitButton>,
    mut app_exit: EventWriter<bevy::app::AppExit>,
) {
    for (interaction, parent) in interaction_q.iter_mut() {
        if *interaction == Interaction::Pressed {
            // Check all children to find button type
            for child in children_q.iter_descendants(parent.get()) {
                if resume_button_q.get(child).is_ok() {
                    next_state.set(GameState::Playing);
                    return;
                } else if quit_button_q.get(child).is_ok() {
                    app_exit.send(bevy::app::AppExit::Success);
                    return;
                }
            }
        }
    }
}
