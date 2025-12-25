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
    dino_q: Query<(Entity, &DinoHealth, &GlobalTransform), (With<Dinosaur>, Without<HealthBar>)>,
    mut health_bar_q: Query<(&Parent, &mut Transform), With<HealthBar>>,
) {
    // Update existing health bars
    for (parent, mut transform) in health_bar_q.iter_mut() {
        if let Ok((_, health, global_transform)) = dino_q.get(parent.get()) {
            let health_percent = health.current / health.max;

            // Position above dinosaur
            let pos = global_transform.translation();
            transform.translation = Vec3::new(pos.x, pos.y + 3.5, pos.z);

            // Scale based on health
            transform.scale.x = health_percent;

            // Update color based on health (we'd need to access the material here)
        } else {
            // Dino is dead or health bar parent is invalid, remove health bar
            commands.entity(parent.get()).despawn_recursive();
        }
    }

    // Spawn new health bars for all dinosaurs (not just new ones)
    for (entity, health, global_transform) in dino_q.iter() {
        // Check if health bar already exists
        let has_bar = health_bar_q.iter().any(|(parent, _)| parent.get() == entity);
        if has_bar {
            continue;
        }

        let pos = global_transform.translation();

        // Background bar - directly in world space above dinosaur
        commands.spawn((
            HealthBarBackground,
            Sprite::from_color(Color::BLACK, Vec2::new(2.0, 0.25)),
            Transform::from_xyz(pos.x, pos.y + 3.5, pos.z),
            GlobalTransform::default(),
        )).set_parent(entity);

        // Health bar - directly in world space above dinosaur
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
            Sprite::from_color(bar_color, Vec2::new(2.0 * health_percent, 0.2)),
            Transform::from_xyz(pos.x - (2.0 * (1.0 - health_percent)) / 2.0, pos.y + 3.5, pos.z + 0.01),
            GlobalTransform::default(),
        )).set_parent(entity);
    }
}
