use bevy::prelude::*;
use crate::dino::{Dinosaur, DinoHealth};
use crate::pause::GameState;

pub struct UIPlugin;

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct HealthBar;

#[derive(Component)]
pub struct HealthBarBackground;

#[derive(Component)]
pub struct Crosshair;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui)
            .add_systems(Update, (
                update_health_bars,
            ).run_if(in_state(GameState::Playing)));
    }
}

fn setup_ui(mut commands: Commands) {
    // Score text
    commands.spawn((
        ScoreText,
        Text2d::new("Score: 0"),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(-400.0, 320.0, 0.0),
    ));

    // Crosshair (horizontal line)
    commands.spawn((
        Crosshair,
        Sprite::from_color(Color::WHITE, Vec2::new(20.0, 2.0)),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Crosshair (vertical line)
    commands.spawn((
        Crosshair,
        Sprite::from_color(Color::WHITE, Vec2::new(2.0, 20.0)),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

fn update_health_bars(
    mut commands: Commands,
    dino_q: Query<(Entity, &DinoHealth, &GlobalTransform), With<Dinosaur>>,
    health_bar_bg_q: Query<(Entity, &Parent), With<HealthBarBackground>>,
    health_bar_q: Query<(Entity, &Parent), (With<HealthBar>, Without<HealthBarBackground>)>,
) {
    // Get set of dinosaurs that already have health bars
    let dinos_with_bars: std::collections::HashSet<Entity> = health_bar_bg_q.iter()
        .map(|(_, parent)| parent.get())
        .chain(health_bar_q.iter().map(|(_, parent)| parent.get()))
        .collect();

    // Spawn health bars for dinosaurs that don't have them yet
    for (entity, health, global_transform) in dino_q.iter() {
        if dinos_with_bars.contains(&entity) {
            continue;
        }

        let pos = global_transform.translation();

        // Background bar
        commands.spawn((
            HealthBarBackground,
            Sprite::from_color(Color::BLACK, Vec2::new(3.0, 0.3)),
            Transform::from_xyz(pos.x, pos.y + 4.0, pos.z),
        )).set_parent(entity);

        // Health bar - colored based on health percentage
        let health_percent = health.current / health.max;
        let bar_color = if health_percent < 0.3 {
            Color::srgb(0.8, 0.2, 0.2) // Red for low health
        } else if health_percent < 0.6 {
            Color::srgb(0.8, 0.8, 0.2) // Yellow for medium health
        } else {
            Color::srgb(0.2, 0.8, 0.2) // Green for high health
        };

        commands.spawn((
            HealthBar,
            Sprite::from_color(bar_color, Vec2::new(3.0 * health_percent, 0.25)),
            Transform::from_xyz(pos.x - (3.0 * (1.0 - health_percent)) / 2.0, pos.y + 4.0, pos.z + 0.01),
        )).set_parent(entity);
    }
}
