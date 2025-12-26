use bevy::prelude::*;
use crate::dino::{BodyPart, HitBox, Dinosaur};
use crate::vehicle::WeaponTurret;
use crate::input::TargetLock;
use crate::pause::GameState;

pub struct WeaponPlugin;

#[derive(Event)]
pub struct BulletHitEvent {
    pub target: Entity,
    pub damage: f32,
    pub position: Vec3,
    pub hit_part: BodyPart,
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
pub struct Bullet {
    pub lifetime: Timer,
    pub damage: f32,
}

#[derive(Component)]
struct BulletVelocity {
    vec: Vec3,
}

#[derive(Component)]
pub struct BloodParticle {
    pub lifetime: Timer,
}

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WeaponState>()
            .add_event::<BulletHitEvent>()
            .add_systems(Update, (
                handle_shooting,
                update_bullets,
                check_bullet_collisions,
                update_blood_particles,
            ).chain().run_if(in_state(GameState::Playing)));
    }
}

fn handle_shooting(
    time: Res<Time>,
    input: Res<crate::input::PlayerInput>,
    mut weapon_state: ResMut<WeaponState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    turret_q: Query<&GlobalTransform, With<WeaponTurret>>,
    vehicle_q: Query<&GlobalTransform, (With<crate::vehicle::PlayerVehicle>, Without<WeaponTurret>)>,
    target_lock: Res<TargetLock>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let current_time = time.elapsed_secs();

    // Check for shooting input (either left click or spacebar)
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

    let Ok(turret_global) = turret_q.get_single() else {
        return;
    };

    let Ok(vehicle_global) = vehicle_q.get_single() else {
        return;
    };

    // Get world positions
    let turret_pos = turret_global.translation();
    let _vehicle_pos = vehicle_global.translation();

    // Fire direction from turret's forward vector (in world space)
    let fire_direction = turret_global.forward();

    // Bullet origin at turret position, slightly forward
    let bullet_origin = turret_pos + fire_direction * 1.0;

    // Spawn bullet with custom velocity component
    let bullet_speed = 100.0;
    commands.spawn((
        Bullet {
            lifetime: Timer::from_seconds(3.0, TimerMode::Once),
            damage: 10.0,
        },
        BulletVelocity {
            vec: fire_direction * bullet_speed,
        },
        Mesh3d(meshes.add(Sphere { radius: 0.15 })),
        MeshMaterial3d(materials.add(Color::srgb(1.0, 0.8, 0.2))),
        Transform::from_translation(bullet_origin),
    ));
}

fn update_bullets(
    time: Res<Time>,
    mut commands: Commands,
    mut bullet_q: Query<(Entity, &mut Bullet, &mut Transform, &BulletVelocity)>,
) {
    let dt = time.delta_secs();

    for (entity, mut bullet, mut transform, velocity) in bullet_q.iter_mut() {
        bullet.lifetime.tick(time.delta());

        if bullet.lifetime.finished() {
            commands.entity(entity).despawn_recursive();
            continue;
        }

        // Move bullet manually
        transform.translation += velocity.vec * dt;
    }
}

fn check_bullet_collisions(
    mut commands: Commands,
    mut bullet_q: Query<(Entity, &Bullet, &Transform)>,
    hitbox_q: Query<(&HitBox, &Parent)>,
    dino_q: Query<&GlobalTransform, With<Dinosaur>>,
    parent_q: Query<&Parent>,
    mut hit_events: EventWriter<BulletHitEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (bullet_entity, _bullet, bullet_transform) in bullet_q.iter_mut() {
        let bullet_pos = bullet_transform.translation;

        // Check collision with all dinosaurs
        for (hit_box, parent) in hitbox_q.iter() {
            // Get the dinosaur entity
            let Ok(dino_parent) = parent_q.get(parent.get()) else { continue };

            // Get dinosaur position
            let Ok(dino_transform) = dino_q.get(dino_parent.get()) else { continue };
            let dino_pos = dino_transform.translation();

            // Simple distance check for collision
            let distance = (bullet_pos - dino_pos).length();

            // Hit detection threshold
            if distance < 2.5 {
                // Calculate damage based on body part
                let damage = calculate_damage(hit_box.part);

                // Send hit event
                hit_events.send(BulletHitEvent {
                    target: dino_parent.get(),
                    damage,
                    position: bullet_pos,
                    hit_part: hit_box.part,
                });

                // Spawn blood particles
                spawn_blood_particles(&mut commands, &mut meshes, &mut materials, bullet_pos);

                // Despawn bullet
                commands.entity(bullet_entity).despawn_recursive();

                // Only one hit per bullet
                break;
            }
        }
    }
}

fn calculate_damage(part: BodyPart) -> f32 {
    match part {
        BodyPart::Head => 50.0,
        BodyPart::Body => 15.0,
        BodyPart::Legs => 8.0,
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
            BulletVelocity { vec: velocity },
            Mesh3d(meshes.add(Sphere { radius: 0.12 })),
            MeshMaterial3d(blood_material.clone()),
            Transform::from_translation(position + offset).with_scale(Vec3::splat(0.4)),
        ));
    }
}

fn update_blood_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut particle_q: Query<(Entity, &mut BloodParticle, &mut Transform, &BulletVelocity)>,
) {
    let dt = time.delta_secs();

    for (entity, mut particle, mut transform, velocity) in particle_q.iter_mut() {
        particle.lifetime.tick(time.delta());

        if particle.lifetime.finished() {
            commands.entity(entity).despawn_recursive();
            continue;
        }

        // Apply gravity to velocity
        let mut vel = velocity.vec;
        vel.y -= 9.8 * dt;
        transform.translation += vel * dt;

        // Shrink over time
        let elapsed = particle.lifetime.elapsed_secs();
        let duration = particle.lifetime.duration().as_secs_f32();
        let scale = 1.0 - (elapsed / duration);
        transform.scale = Vec3::splat(scale * 0.4);
    }
}
