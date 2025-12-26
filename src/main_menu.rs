use bevy::prelude::*;
use crate::pause::GameState;
use crate::game_mode::TimeAttackMode;

#[derive(Component)]
pub struct MainMenu;

#[derive(Component)]
pub struct MenuButton;

#[derive(Component)]
pub struct StartButton;

#[derive(Component)]
pub struct TimeAttackButton;

#[derive(Component)]
pub struct QuitButton;

#[derive(Component)]
pub struct ResumeButton;

#[derive(Resource, Default)]
pub struct MenuState {
    pub is_in_menu: bool,
}

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuState>()
            .add_systems(OnEnter(GameState::Paused), setup_main_menu)
            .add_systems(Update, handle_menu_input.run_if(in_state(GameState::Paused)))
            .add_systems(OnExit(GameState::Paused), cleanup_main_menu);
    }
}

fn setup_main_menu(
    mut commands: Commands,
    mode: Res<TimeAttackMode>,
) {
    let is_game_active = mode.kills > 0 || mode.is_active;

    // Menu background
    commands.spawn((
        MainMenu,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(20.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.2, 0.9)),
    )).with_children(|parent| {
        // Title
        parent.spawn((
            Text::new("DINO HUNTER"),
            TextFont {
                font_size: 60.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.8, 0.2)),
            Node {
                margin: UiRect::bottom(Val::Px(40.0)),
                ..default()
            },
        ));

        // Show game stats if game was active
        if is_game_active {
            parent.spawn((
                Text::new(format!(
                    "Kills: {} | Max Combo: {} | Rank: {}",
                    mode.kills,
                    mode.max_combo,
                    mode.get_rank()
                )),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));
        }

        // Resume Button (only if game was active)
        if is_game_active {
            parent.spawn((
                ResumeButton,
                MenuButton,
                Node {
                    width: Val::Px(250.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.6, 0.2)),
            )).with_children(|parent| {
                parent.spawn((
                    Text::new("Resume Game"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
        }

        // Start New Game Button
        parent.spawn((
            StartButton,
            MenuButton,
            Node {
                width: Val::Px(250.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.4, 0.7)),
        )).with_children(|parent| {
            parent.spawn((
                Text::new("Free Hunt Mode"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });

        // Time Attack Button
        parent.spawn((
            TimeAttackButton,
            MenuButton,
            Node {
                width: Val::Px(250.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.7, 0.3, 0.2)),
        )).with_children(|parent| {
            parent.spawn((
                Text::new("Time Attack (5 min)"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });

        // Quit Button
        parent.spawn((
            QuitButton,
            MenuButton,
            Node {
                width: Val::Px(250.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.6, 0.2, 0.2)),
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

        // Instructions
        parent.spawn((
            Text::new("WASD: Move | Mouse: Aim | Click: Shoot | 1/2/3: Weapons | ESC: Pause"),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.6, 0.6, 0.6)),
            Node {
                margin: UiRect::top(Val::Px(40.0)),
                ..default()
            },
        ));
    });
}

fn handle_menu_input(
    mut next_state: ResMut<NextState<GameState>>,
    mut interaction_q: Query<
        (&Interaction, &mut BackgroundColor),
        (With<MenuButton>, Changed<Interaction>)
    >,
    button_types: Query<
        (Option<&ResumeButton>, Option<&StartButton>, Option<&TimeAttackButton>, Option<&QuitButton>),
        With<MenuButton>
    >,
    mut time_attack: ResMut<TimeAttackMode>,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // ESC to resume if in menu
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Playing);
        return;
    }

    for (interaction, mut bg_color) in interaction_q.iter_mut() {
        let (is_resume, is_start, is_time_attack, is_quit) = button_types.get_single().ok().unwrap_or_default();

        match *interaction {
            Interaction::Pressed => {
                if is_resume.is_some() {
                    // Resume game
                    next_state.set(GameState::Playing);
                } else if is_start.is_some() {
                    // Start free hunt mode
                    time_attack.stop();
                    next_state.set(GameState::Playing);
                } else if is_time_attack.is_some() {
                    // Start time attack mode
                    time_attack.start();
                    next_state.set(GameState::Playing);
                } else if is_quit.is_some() {
                    // Quit game
                    app_exit_events.send(bevy::app::AppExit::Success);
                }
            }
            Interaction::Hovered => {
                // Lighten the color on hover
                // For now, just set a lighter gray color
                bg_color.0 = Color::srgb(0.4, 0.4, 0.4);
            }
            Interaction::None => {
                // Color will be reset to original
            }
        }
    }
}

fn cleanup_main_menu(
    mut commands: Commands,
    menu_q: Query<Entity, With<MainMenu>>,
) {
    for entity in menu_q.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
