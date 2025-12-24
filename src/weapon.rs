use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::dino::{Dinosaur, DinoHealth, BodyPart, HitBox};
use crate::vehicle::WeaponTurret;

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
            .add_systems(Update, (
                handle_shooting,
                update_bullets,
                update_blood_particles,
            ));
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
    rapier_context: Res<RapierContext>,
    mut hit_events: EventWriter<BulletHitEvent>,
) {
    let current_time = time.elapsed_seconds();

    if !input.shooting {
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
    let bullet_entity = commands.spawn((
        Bullet {
            lifetime: Timer::from_seconds(2.0, TimerMode::Once),
            damage: 10.0,
        },
        PbrBundle {
            mesh: meshes.add(Sphere { radius: 0.08 }),
            material: materials.add(Color::srgb(1.0, 0.8, 0.2)),
            transform: Transform::from_translation(bullet_origin),
            ..default()
        },
        RigidBody::KinematicPositionBased,
        Collider::ball(0.08),
        Sensor,
    )).id();

    // Raycast for immediate hit detection
    let max_dist = 200.0;
    if let Some((entity, hit_pos)) = cast_ray(&rapier_context, bullet_origin, fire_direction, max_dist) {
        // Check if it's a dinosaur or hitbox
        if let Some((dino_entity, hit_box_part)) = find_dino_and_part(&mut commands, entity) {
            let damage = calculate_damage(hit_box_part);
            hit_events.send(BulletHitEvent {
                target: dino_entity,
                damage,
                position: hit_pos,
            });

            // Spawn blood particles
            spawn_blood_particles(&mut commands, &mut meshes, &mut materials, hit_pos);
        }
    }

    // Add velocity to bullet
    let bullet_speed = 100.0;
    commands.entity(bullet_entity).insert(Velocity::linear(fire_direction * bullet_speed));
}

fn cast_ray(
    rapier_context: &RapierContext,
    origin: Vec3,
    direction: Vec3,
    max_dist: f32,
) -> Option<(Entity, Vec3)> {
    let ray = Ray {
        origin: origin.into(),
        dir: direction.into(),
    };

    rapier_context.cast_rayAndGetNormal(
        ray.origin,
        ray.dir,
        max_dist,
        true,
        QueryFilter::default(),
    ).map(|(entity, hit)| {
        (entity, origin + direction * hit.toi)
    })
}

fn find_dino_and_part(
    commands: &mut Commands,
    entity: Entity,
) -> Option<(Entity, BodyPart)> {
    // Check if entity has a HitBox component
    if let Some(mut entity_commands) = commands.get_entity(entity) {
        if let Some(hit_box) = entity_commands.get::<HitBox>() {
            // Get parent dinosaur
            if let Some(parent) = entity_commands.get::<Parent>() {
                return Some((parent.get(), hit_box.part));
            }
        }
    }
    None
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
        transform.translation += forward * 100.0 * time.delta_seconds();
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
            PbrBundle {
                mesh: meshes.add(Sphere { radius: 0.08 }),
                material: blood_material.clone(),
                transform: Transform::from_translation(position + offset).with_scale(Vec3::splat(0.3)),
                ..default()
            },
        )).insert(Velocity::linear(velocity));
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
        transform.translation.y -= 9.8 * time.delta_seconds();

        // Shrink over time
        let scale = 1.0 - (particle.lifetime.elapsed() / particle.lifetime.duration());
        transform.scale = Vec3::splat(scale * 0.3);
    }
}
