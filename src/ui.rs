use bevy::prelude::*;
use crate::dino::{Dinosaur, DinoHealth};
use crate::weapon::BulletHitEvent;
use crate::GameScore;

pub struct UIPlugin;

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct HealthBar;

#[derive(Component)]
pub struct Crosshair;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui)
            .add_systems(Update, (
                update_health_bars,
                handle_bullet_hits,
            ));
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
            transform.translation = Vec3::new(pos.x, pos.y + 3.0, pos.z);

            // Scale based on health
            transform.scale.x = health_percent;

            // Change color based on health
            if health_percent < 0.3 {
                // Red for low health
            }
        } else {
            // Dino is dead, remove health bar
            commands.entity(parent.get()).despawn_recursive();
        }
    }

    // Spawn new health bars
    for (entity, health, global_transform) in dino_q.iter() {
        // Check if health bar already exists
        let has_bar = health_bar_q.iter().any(|(parent, _)| parent.get() == entity);
        if has_bar {
            continue;
        }

        let pos = global_transform.translation();

        // Background bar
        commands.spawn((
            Sprite::from_color(Color::BLACK, Vec2::new(2.0, 0.2)),
            Transform::from_xyz(pos.x, pos.y + 3.0, pos.z),
            GlobalTransform::default(),
        )).set_parent(entity);

        // Health bar
        commands.spawn((
            HealthBar,
            Sprite::from_color(Color::srgb(0.2, 0.8, 0.2), Vec2::new(2.0, 0.2)),
            Transform::from_xyz(0.0, 0.0, 0.01).with_scale(Vec3::new(
                health.current / health.max,
                1.0,
                1.0,
            )),
            GlobalTransform::default(),
        )).set_parent(entity);
    }
}

fn handle_bullet_hits(
    mut commands: Commands,
    mut events: EventReader<BulletHitEvent>,
    mut dino_q: Query<&mut DinoHealth>,
    mut score: ResMut<GameScore>,
) {
    for event in events.read() {
        if let Ok(mut health) = dino_q.get_mut(event.target) {
            health.current -= event.damage;

            if health.current <= 0.0 {
                // Dino killed, add score
                score.score += 100;
            }
        }
    }
}
