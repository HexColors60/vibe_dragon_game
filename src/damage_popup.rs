use bevy::prelude::*;
use crate::pause::GameState;

/// Floating damage number that appears when hitting enemies
#[derive(Component)]
pub struct DamagePopup {
    pub lifetime: Timer,
    pub velocity: Vec3,
}

/// Damage type for visual distinction
#[derive(Clone, Copy)]
pub enum DamageType {
    Normal,
    Critical,   // Head shot
    Weak,       // Leg shot
}

#[derive(Component)]
pub struct DamageText;

pub struct DamagePopupPlugin;

impl Plugin for DamagePopupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            spawn_damage_popups,
            update_damage_popups,
        ).chain().run_if(in_state(GameState::Playing)));
    }
}

/// Spawn damage numbers - call this when a hit occurs
pub fn spawn_damage_popups(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut damage_events: EventReader<crate::weapon::BulletHitEvent>,
) {
    for event in damage_events.read() {
        let damage = event.damage as i32;
        let damage_type = match event.hit_part {
            crate::dino::BodyPart::Head => DamageType::Critical,
            crate::dino::BodyPart::Legs => DamageType::Weak,
            _ => DamageType::Normal,
        };

        let (color, scale) = match damage_type {
            DamageType::Critical => (
                Color::srgba(1.0, 0.84, 0.0, 1.0), // Gold
                0.4,
            ),
            DamageType::Weak => (
                Color::srgba(0.8, 0.6, 0.6, 1.0), // Light red
                0.25,
            ),
            DamageType::Normal => (
                Color::srgba(1.0, 1.0, 1.0, 1.0), // White
                0.3,
            ),
        };

        // Spawn damage number as 3D object (using colored spheres)
        let damage_value = damage as f32;
        let size = scale * (damage_value.min(100.0) / 100.0 + 0.5);

        commands.spawn((
            DamagePopup {
                lifetime: Timer::from_seconds(1.5, TimerMode::Once),
                velocity: Vec3::new(0.0, 4.0, 0.0), // Float upward
            },
            Mesh3d(meshes.add(Sphere { radius: size * 0.3 })),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                emissive: LinearRgba::new(0.3, 0.3, 0.3, 1.0),
                unlit: true,
                ..default()
            })),
            Transform::from_translation(event.position + Vec3::new(0.0, 1.0, 0.0)),
        ));
    }
}

pub fn update_damage_popups(
    time: Res<Time>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut popup_q: Query<(Entity, &mut DamagePopup, &mut Transform, &MeshMaterial3d<StandardMaterial>)>,
) {
    let dt = time.delta_secs();

    for (entity, mut popup, mut transform, material) in popup_q.iter_mut() {
        popup.lifetime.tick(time.delta());

        if popup.lifetime.finished() {
            commands.entity(entity).despawn_recursive();
            continue;
        }

        // Move upward
        transform.translation += popup.velocity * dt;

        // Fade out based on remaining lifetime
        let elapsed = popup.lifetime.elapsed_secs();
        let duration = popup.lifetime.duration().as_secs_f32();
        let alpha = 1.0 - (elapsed / duration);

        // Update material transparency
        if let Some(mat) = materials.get_mut(material.id()) {
            mat.base_color.set_alpha(alpha);
            mat.emissive.set_alpha(alpha);
        }

        // Slow down velocity
        popup.velocity *= 0.95;
    }
}
