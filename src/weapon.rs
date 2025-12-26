use bevy::prelude::*;
use crate::dino::{BodyPart, HitBox, Dinosaur};
use crate::vehicle::WeaponTurret;
use crate::input::TargetLock;
use crate::pause::GameState;
use crate::weapon_system::WeaponInventory;
use crate::effects::HitFeedbackEvent;

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
}

impl Default for WeaponState {
    fn default() -> Self {
        Self {
            last_shot: 0.0,
        }
    }
}

#[derive(Component)]
pub struct Bullet {
    pub lifetime: Timer,
    pub damage: f32,
    pub weapon_type: crate::weapon_system::WeaponType,
}

#[derive(Component)]
struct BulletVelocity {
    vec: Vec3,
}

#[derive(Component)]
pub struct BloodParticle {
    pub lifetime: Timer,
}

/// For rocket delayed explosions
#[derive(Component)]
pub struct Rocket {
    pub timer: Timer,
    pub damage: f32,
    pub explosion_radius: f32,
}

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WeaponState>()
            .add_event::<BulletHitEvent>()
            .add_event::<RocketExplosionEvent>()
            .add_event::<HitFeedbackEvent>()
            .add_systems(Update, (
                handle_shooting,
                update_bullets,
                check_bullet_collisions,
                update_blood_particles,
                update_rockets,
            ).chain().run_if(in_state(GameState::Playing)));
    }
}

#[derive(Event)]
pub struct RocketExplosionEvent {
    pub position: Vec3,
    pub damage: f32,
    pub radius: f32,
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
    dino_q: Query<&GlobalTransform, With<Dinosaur>>,
    weapon_inv: Res<WeaponInventory>,
) {
    let current_time = time.elapsed_secs();

    // Check if shooting with locked target (Space) or free aim (Left Click)
    let shooting_at_lock = keyboard.pressed(KeyCode::Space) && target_lock.locked_entity.is_some();
    let should_shoot = input.shooting || shooting_at_lock;

    if !should_shoot {
        return;
    }

    let current_weapon = weapon_inv.current_weapon;
    let fire_rate = current_weapon.fire_rate();

    if current_time - weapon_state.last_shot < fire_rate {
        return;
    }

    weapon_state.last_shot = current_time;

    let Ok(turret_global) = turret_q.get_single() else {
        return;
    };

    let Ok(_vehicle_global) = vehicle_q.get_single() else {
        return;
    };

    // Get world positions
    let turret_pos = turret_global.translation();

    // Determine fire direction
    let fire_direction = if shooting_at_lock {
        // Shooting at locked target - aim directly at it
        if let Some(locked_entity) = target_lock.locked_entity {
            if let Ok(dino_global) = dino_q.get(locked_entity) {
                let target_pos = dino_global.translation();
                (target_pos - turret_pos).normalize()
            } else {
                *turret_global.forward()
            }
        } else {
            *turret_global.forward()
        }
    } else {
        // Free aim - use turret's facing direction
        *turret_global.forward()
    };

    let base_damage = current_weapon.damage();
    let pellet_count = current_weapon.pellet_count();
    let spread = current_weapon.spread();
    let bullet_speed = current_weapon.bullet_speed();
    let bullet_radius = current_weapon.bullet_radius();

    // Spawn bullets
    for i in 0..pellet_count {
        let bullet_origin = turret_pos + fire_direction * 1.0;

        // Apply spread for shotgun
        let bullet_direction = if spread > 0.0 && pellet_count > 1 {
            let spread_angle = spread;
            let horizontal_angle = (i as f32 / pellet_count as f32 - 0.5) * spread_angle;
            let vertical_angle = (rand::random::<f32>() - 0.5) * spread_angle * 0.5;

            let mut dir = fire_direction;
            dir = Quat::from_rotation_y(horizontal_angle) * dir;
            dir = Quat::from_rotation_x(vertical_angle) * dir;
            dir.normalize()
        } else {
            fire_direction
        };

        // Rocket launcher creates rockets instead of bullets
        if current_weapon.explosive() {
            commands.spawn((
                Bullet {
                    lifetime: Timer::from_seconds(5.0, TimerMode::Once),
                    damage: base_damage,
                    weapon_type: current_weapon,
                },
                Rocket {
                    timer: Timer::from_seconds(current_weapon.rocket_delay(), TimerMode::Once),
                    damage: base_damage,
                    explosion_radius: current_weapon.explosion_radius(),
                },
                BulletVelocity {
                    vec: bullet_direction * bullet_speed,
                },
                Mesh3d(meshes.add(Sphere { radius: bullet_radius })),
                MeshMaterial3d(materials.add(Color::srgb(1.0, 0.3, 0.1))),
                Transform::from_translation(bullet_origin),
            ));
        } else {
            // Normal bullets
            commands.spawn((
                Bullet {
                    lifetime: Timer::from_seconds(3.0, TimerMode::Once),
                    damage: base_damage,
                    weapon_type: current_weapon,
                },
                BulletVelocity {
                    vec: bullet_direction * bullet_speed,
                },
                Mesh3d(meshes.add(Sphere { radius: bullet_radius })),
                MeshMaterial3d(materials.add(if current_weapon == crate::weapon_system::WeaponType::Shotgun {
                    Color::srgb(0.8, 0.6, 0.3) // Buckshot color
                } else {
                    Color::srgb(1.0, 0.8, 0.2) // Machine gun color
                })),
                Transform::from_translation(bullet_origin),
            ));
        }
    }
}

fn update_bullets(
    time: Res<Time>,
    mut commands: Commands,
    mut bullet_q: Query<(Entity, &mut Bullet, &mut Transform, &BulletVelocity), Without<Rocket>>,
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

fn update_rockets(
    time: Res<Time>,
    mut commands: Commands,
    mut rocket_q: Query<(Entity, &mut Rocket, &mut Transform, &BulletVelocity)>,
    mut explosion_events: EventWriter<RocketExplosionEvent>,
) {
    let dt = time.delta_secs();

    for (entity, mut rocket, mut transform, velocity) in rocket_q.iter_mut() {
        // Move rocket
        transform.translation += velocity.vec * dt;

        // Update explosion timer
        rocket.timer.tick(time.delta());

        if rocket.timer.finished() {
            // Trigger explosion
            explosion_events.send(RocketExplosionEvent {
                position: transform.translation,
                damage: rocket.damage,
                radius: rocket.explosion_radius,
            });
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn check_bullet_collisions(
    mut commands: Commands,
    mut bullet_q: Query<(Entity, &Bullet, &Transform)>,
    dino_q: Query<(Entity, &GlobalTransform), With<Dinosaur>>,
    hitbox_q: Query<(&HitBox, &GlobalTransform, &Parent)>,
    _parent_q: Query<&Parent>,
    mut hit_events: EventWriter<BulletHitEvent>,
    mut hit_feedback: EventWriter<HitFeedbackEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut explosion_events: EventReader<RocketExplosionEvent>,
) {
    // Handle rocket explosions first
    for event in explosion_events.read() {
        // Find all dinosaurs in explosion radius
        for (dino_entity, dino_global) in dino_q.iter() {
            let dino_pos = dino_global.translation();
            let distance = (dino_pos - event.position).length();

            if distance < event.radius {
                // Damage decreases with distance
                let falloff = 1.0 - (distance / event.radius);
                let damage = event.damage * falloff;

                hit_events.send(BulletHitEvent {
                    target: dino_entity,
                    damage,
                    position: event.position,
                    hit_part: BodyPart::Body, // Explosion hits body
                });

                // Spawn blood particles
                spawn_blood_particles(&mut commands, &mut meshes, &mut materials, dino_pos);

                // Trigger crosshair feedback
                hit_feedback.send(HitFeedbackEvent);
            }
        }

        // Spawn explosion particles
        spawn_explosion_particles(&mut commands, &mut meshes, &mut materials, event.position);
    }

    // Handle bullet collisions
    for (bullet_entity, bullet, bullet_transform) in bullet_q.iter_mut() {
        // Skip rockets (they're handled by update_rockets)
        if bullet.weapon_type.explosive() {
            continue;
        }

        let bullet_pos = bullet_transform.translation;

        // Check collision with all dinosaurs
        for (dino_entity, dino_global) in dino_q.iter() {
            let dino_pos = dino_global.translation();

            // Simple distance check for collision (larger hitbox)
            let distance = (bullet_pos - dino_pos).length();

            // Hit detection threshold - generous hitbox
            if distance < 4.0 {
                // Find which body part was hit by checking all hitboxes
                let mut hit_part = BodyPart::Body; // default
                let mut found_hit = false;

                for (hit_box, hitbox_global, _parent) in hitbox_q.iter() {
                    let hitbox_pos = hitbox_global.translation();
                    let hitbox_distance = (bullet_pos - hitbox_pos).length();

                    if hitbox_distance < 1.5 {
                        hit_part = hit_box.part;
                        found_hit = true;
                        break;
                    }
                }

                // Calculate damage based on body part
                let damage = calculate_damage(if found_hit { hit_part } else { BodyPart::Body });

                // Send hit event
                hit_events.send(BulletHitEvent {
                    target: dino_entity,
                    damage,
                    position: bullet_pos,
                    hit_part: hit_part,
                });

                // Trigger crosshair feedback on hit
                hit_feedback.send(HitFeedbackEvent);

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

    for _ in 0..12 {
        let offset = Vec3::new(
            rand::random::<f32>() * 0.8 - 0.4,
            rand::random::<f32>() * 0.8,
            rand::random::<f32>() * 0.8 - 0.4,
        );

        let velocity = Vec3::new(
            rand::random::<f32>() * 6.0 - 3.0,
            rand::random::<f32>() * 6.0 + 2.0,
            rand::random::<f32>() * 6.0 - 3.0,
        );

        commands.spawn((
            BloodParticle {
                lifetime: Timer::from_seconds(0.8, TimerMode::Once),
            },
            BulletVelocity { vec: velocity },
            Mesh3d(meshes.add(Sphere { radius: 0.15 })),
            MeshMaterial3d(blood_material.clone()),
            Transform::from_translation(position + offset).with_scale(Vec3::splat(0.5)),
        ));
    }
}

fn spawn_explosion_particles(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
) {
    let explosion_material = materials.add(Color::srgba(1.0, 0.5, 0.1, 0.9));

    for _ in 0..20 {
        let offset = Vec3::new(
            rand::random::<f32>() * 0.5 - 0.25,
            rand::random::<f32>() * 0.5,
            rand::random::<f32>() * 0.5 - 0.25,
        );

        let velocity = Vec3::new(
            rand::random::<f32>() * 10.0 - 5.0,
            rand::random::<f32>() * 8.0 + 3.0,
            rand::random::<f32>() * 10.0 - 5.0,
        );

        commands.spawn((
            BloodParticle {
                lifetime: Timer::from_seconds(0.6, TimerMode::Once),
            },
            BulletVelocity { vec: velocity },
            Mesh3d(meshes.add(Sphere { radius: 0.3 })),
            MeshMaterial3d(explosion_material.clone()),
            Transform::from_translation(position + offset).with_scale(Vec3::splat(0.8)),
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
        transform.scale = Vec3::splat(scale * 0.5);
    }
}
