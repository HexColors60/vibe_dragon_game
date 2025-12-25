use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::dino::{BodyPart, HitBox};
use crate::vehicle::WeaponTurret;
use crate::input::TargetLock;

pub struct WeaponPlugin;

#[derive(Event)]
pub struct BulletHitEvent {
    pub target: Entity,
    pub damage: f32,
    pub position: Vec3,
}

#[derive(Resource)]
struct WeaponState {
    last_shot: f32,
    fire_rate: f32,
}

impl Default for WeaponState {
    fn default() -> Self {
        Self {
            last_shot: 0.0,
            fire_rate: 0.1,
        }
    }
}

#[derive(Component)]
struct Bullet {
    lifetime: Timer,
    damage: f32,
}

#[derive(Component)]
struct BloodParticle {
    lifetime: Timer,
}

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WeaponState>()
            .add_event::<BulletHitEvent>()
            .add_systems(Update, (
                handle_shooting,
                update_bullets,
                update_blood_particles,
            ).chain());
    }
}

fn handle_shooting(
    time: Res<Time>,
    input: Res<crate::input::PlayerInput>,
    mut weapon_state: ResMut<WeaponState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    turret_q: Query<&Transform, With<WeaponTurret>>,
    vehicle_q: Query<&Transform, (With<crate::vehicle::PlayerVehicle>, Without<WeaponTurret>)>,
    rapier_context: Query<&RapierContext>,
    hitbox_q: Query<&HitBox>,
    parent_q: Query<&Parent>,
    target_lock: Res<TargetLock>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let current_time = time.elapsed_secs();

    // Check for shooting input (either left click or spacebar when locked)
    let should_shoot = if target_lock.locked_entity.is_some() {
        input.shooting || keyboard.pressed(KeyCode::Space)
    } else {
        input.shooting
    };

    if !should_shoot {
        return;
    }

    if current_time - weapon_state.last_shot < weapon_state.fire_rate {
        return;
    }

    weapon_state.last_shot = current_time;

    let Ok(turret_transform) = turret_q.get_single() else {
        return;
    };

    let Ok(vehicle_transform) = vehicle_q.get_single() else {
        return;
    };

    // Fire direction
    let fire_direction = -turret_transform.forward();
    let bullet_origin = turret_transform.translation + vehicle_transform.translation + fire_direction * 1.0;

    // Spawn bullet
    commands.spawn((
        Bullet {
            lifetime: Timer::from_seconds(2.0, TimerMode::Once),
            damage: 10.0,
        },
        Mesh3d(meshes.add(Sphere { radius: 0.08 })),
        MeshMaterial3d(materials.add(Color::srgb(1.0, 0.8, 0.2))),
        Transform::from_translation(bullet_origin),
        RigidBody::KinematicPositionBased,
        Collider::ball(0.08),
        Sensor,
    ));

    // Raycast for immediate hit detection
    let Ok(rapier_context) = rapier_context.get_single() else {
        return;
    };

    let max_dist = 200.0;
    if let Some((entity, _hit_pos)) = cast_ray(rapier_context, bullet_origin, *fire_direction, max_dist) {
        // Check if entity has a HitBox component
        if let Ok(hit_box) = hitbox_q.get(entity) {
            // Get parent dinosaur
            if let Ok(_parent) = parent_q.get(entity) {
                let _damage = calculate_damage(hit_box.part);
                // We'll handle damage through collision detection instead of raycast
                // to avoid deferred world access issues
            }
        }
    }
}

fn cast_ray(
    rapier_context: &RapierContext,
    origin: Vec3,
    direction: Vec3,
    max_dist: f32,
) -> Option<(Entity, Vec3)> {
    rapier_context.cast_ray_and_get_normal(
        origin.into(),
        direction.into(),
        max_dist,
        true,
        QueryFilter::default(),
    ).map(|(entity, hit)| {
        (entity, origin + direction * hit.time_of_impact)
    })
}

fn calculate_damage(part: BodyPart) -> f32 {
    match part {
        BodyPart::Head => 50.0,
        BodyPart::Body => 15.0,
        BodyPart::Legs => 8.0,
    }
}

fn update_bullets(
    time: Res<Time>,
    mut commands: Commands,
    mut bullet_q: Query<(Entity, &mut Bullet, &mut Transform)>,
) {
    for (entity, mut bullet, mut transform) in bullet_q.iter_mut() {
        bullet.lifetime.tick(time.delta());

        if bullet.lifetime.finished() {
            commands.entity(entity).despawn_recursive();
            continue;
        }

        // Move bullet based on velocity
        let forward = transform.forward();
        transform.translation += forward * 100.0 * time.delta_secs();
    }
}

fn spawn_blood_particles(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
) {
    let blood_material = materials.add(Color::srgba(0.6, 0.05, 0.05, 0.8));

    for _ in 0..8 {
        let offset = Vec3::new(
            rand::random::<f32>() * 0.5 - 0.25,
            rand::random::<f32>() * 0.5,
            rand::random::<f32>() * 0.5 - 0.25,
        );

        let velocity = Vec3::new(
            rand::random::<f32>() * 4.0 - 2.0,
            rand::random::<f32>() * 4.0 + 1.0,
            rand::random::<f32>() * 4.0 - 2.0,
        );

        commands.spawn((
            BloodParticle {
                lifetime: Timer::from_seconds(0.5, TimerMode::Once),
            },
            Mesh3d(meshes.add(Sphere { radius: 0.08 })),
            MeshMaterial3d(blood_material.clone()),
            Transform::from_translation(position + offset).with_scale(Vec3::splat(0.3)),
            Velocity::linear(velocity),
        ));
    }
}

fn update_blood_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut particle_q: Query<(Entity, &mut BloodParticle, &mut Transform)>,
) {
    for (entity, mut particle, mut transform) in particle_q.iter_mut() {
        particle.lifetime.tick(time.delta());

        if particle.lifetime.finished() {
            commands.entity(entity).despawn_recursive();
            continue;
        }

        // Apply gravity
        transform.translation.y -= 9.8 * time.delta_secs();

        // Shrink over time
        let elapsed = particle.lifetime.elapsed_secs();
        let duration = particle.lifetime.duration().as_secs_f32();
        let scale = 1.0 - (elapsed / duration);
        transform.scale = Vec3::splat(scale * 0.3);
    }
}
