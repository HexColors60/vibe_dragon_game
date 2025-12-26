use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy::math::Mat3;
use crate::input::{PlayerInput, TargetLock};
use crate::dino::Dinosaur;
use crate::camera::MainCamera;

pub struct VehiclePlugin;

#[derive(Component)]
pub struct PlayerVehicle;

#[derive(Component)]
pub struct VehicleVelocity {
    pub current: f32,
    pub max_speed: f32,
    pub acceleration: f32,
    pub deceleration: f32,
    pub turn_speed: f32,
}

#[derive(Component)]
pub struct VehicleHealth {
    pub current: f32,
    pub max: f32,
}

impl Default for VehicleHealth {
    fn default() -> Self {
        Self { current: 100.0, max: 100.0 }
    }
}

impl Plugin for VehiclePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_vehicle)
            .add_systems(Update, (
                handle_vehicle_movement,
                rotate_weapon_turret,
                update_target_lock,
                update_indicator_position,
            ));
    }
}

fn spawn_vehicle(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let body_color = Color::srgb(0.7, 0.2, 0.15);
    let cabin_color = Color::srgb(0.9, 0.85, 0.7);
    let wheel_color = Color::srgb(0.1, 0.1, 0.1);
    let gun_color = Color::srgb(0.3, 0.3, 0.35);

    // Vehicle root entity
    let vehicle_entity = commands.spawn((
        PlayerVehicle,
        Transform::from_xyz(0.0, 1.0, 0.0),
        VehicleVelocity {
            current: 0.0,
            max_speed: 25.0,
            acceleration: 15.0,
            deceleration: 10.0,
            turn_speed: 2.5,
        },
        VehicleHealth::default(),
        RigidBody::KinematicPositionBased,
        Collider::cuboid(2.0, 1.0, 4.0),
        Friction::new(0.8),
        AdditionalMassProperties::Mass(1500.0),
    )).id();

    // Vehicle body
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.0, 0.8, 4.0))),
        MeshMaterial3d(materials.add(body_color)),
        Transform::from_xyz(0.0, 0.5, 0.0),
    )).set_parent(vehicle_entity);

    // Cabin
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.8, 0.7, 2.0))),
        MeshMaterial3d(materials.add(cabin_color)),
        Transform::from_xyz(0.0, 1.2, -0.5),
    )).set_parent(vehicle_entity);

    // Wheels
    let wheel_positions = [
        (-1.1, 0.0, 1.3),
        (1.1, 0.0, 1.3),
        (-1.1, 0.0, -1.3),
        (1.1, 0.0, -1.3),
    ];

    for pos in wheel_positions {
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.4, 0.3))),
            MeshMaterial3d(materials.add(wheel_color)),
            Transform::from_xyz(pos.0, pos.1, pos.2)
                .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
        )).set_parent(vehicle_entity);
    }

    // Weapon mount base
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.2, 0.3))),
        MeshMaterial3d(materials.add(gun_color)),
        Transform::from_xyz(0.0, 1.8, 0.0),
    )).set_parent(vehicle_entity);

    // Machine gun barrel (will rotate to face mouse direction)
    commands.spawn((
        WeaponTurret,
        Mesh3d(meshes.add(Cylinder::new(0.08, 1.5))),
        MeshMaterial3d(materials.add(gun_color)),
        Transform::from_xyz(0.0, 1.9, 0.0)
            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
    )).set_parent(vehicle_entity);
}

#[derive(Component)]
pub struct WeaponTurret;

fn handle_vehicle_movement(
    input: Res<PlayerInput>,
    time: Res<Time>,
    mut vehicle_q: Query<(&mut Transform, &mut VehicleVelocity), With<PlayerVehicle>>,
) {
    let Ok((mut transform, mut velocity)) = vehicle_q.get_single_mut() else {
        return;
    };

    let dt = time.delta_secs();

    // Acceleration
    if input.move_forward {
        velocity.current += velocity.acceleration * dt;
    } else if input.move_backward {
        velocity.current -= velocity.acceleration * dt;
    } else {
        // Decelerate when not moving
        if velocity.current > 0.0 {
            velocity.current -= velocity.deceleration * dt;
            velocity.current = velocity.current.max(0.0);
        } else if velocity.current < 0.0 {
            velocity.current += velocity.deceleration * dt;
            velocity.current = velocity.current.min(0.0);
        }
    }

    // Clamp speed
    velocity.current = velocity.current.clamp(-velocity.max_speed * 0.3, velocity.max_speed);

    // Turning (only when moving)
    if velocity.current.abs() > 0.1 {
        let turn_direction = if input.move_backward { -1.0 } else { 1.0 };
        if input.move_left {
            transform.rotate_y(velocity.turn_speed * dt * turn_direction);
        }
        if input.move_right {
            transform.rotate_y(-velocity.turn_speed * dt * turn_direction);
        }
    }

    // Apply velocity
    let forward = transform.forward();
    transform.translation += forward * velocity.current * dt;
}

fn rotate_weapon_turret(
    time: Res<Time>,
    input: Res<PlayerInput>,
    target_lock: Res<TargetLock>,
    mut turret_q: Query<&mut Transform, (With<WeaponTurret>, Without<PlayerVehicle>)>,
    vehicle_q: Query<&Transform, (With<PlayerVehicle>, Without<WeaponTurret>)>,
    dino_q: Query<&GlobalTransform, With<Dinosaur>>,
) {
    let Ok(mut turret_transform) = turret_q.get_single_mut() else {
        return;
    };

    let Ok(vehicle_transform) = vehicle_q.get_single() else {
        return;
    };

    let dt = time.delta_secs();
    let turret_rotation_speed = 2.0;

    // Check if we have a locked target
    if let Some(locked_entity) = target_lock.locked_entity {
        if let Ok(dino_transform) = dino_q.get(locked_entity) {
            let turret_pos = vehicle_transform.translation + Vec3::new(0.0, 1.9, 0.0);
            let target_pos = dino_transform.translation();
            let direction = (target_pos - turret_pos).normalize();

            if direction.length_squared() > 0.01 {
                let forward = Vec3::new(direction.x, 0.0, direction.z).normalize();
                let up = Vec3::Y;
                let right = forward.cross(up).normalize();
                let new_up = right.cross(forward).normalize();

                turret_transform.rotation = Quat::from_mat3(&Mat3::from_cols(
                    right,
                    new_up,
                    -forward,
                ));
            }
        }
    } else if let Some(lock_pos) = target_lock.lock_position {
        // Use locked position
        let turret_pos = vehicle_transform.translation + Vec3::new(0.0, 1.9, 0.0);
        let direction = (lock_pos - turret_pos).normalize();

        if direction.length_squared() > 0.01 {
            let forward = Vec3::new(direction.x, 0.0, direction.z).normalize();
            let up = Vec3::Y;
            let right = forward.cross(up).normalize();
            let new_up = right.cross(forward).normalize();

            turret_transform.rotation = Quat::from_mat3(&Mat3::from_cols(
                right,
                new_up,
                -forward,
            ));
        }
    } else {
        // Use mouse movement for rotation
        if input.turret_left {
            turret_transform.rotate_y(turret_rotation_speed * dt);
        }
        if input.turret_right {
            turret_transform.rotate_y(-turret_rotation_speed * dt);
        }
    }
}

fn update_target_lock(
    mut commands: Commands,
    input: Res<PlayerInput>,
    mut target_lock: ResMut<TargetLock>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    dino_q: Query<(Entity, &GlobalTransform), With<Dinosaur>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    indicator_q: Query<Entity, With<TargetLockIndicator>>,
) {
    // Handle target locking when right mouse button is pressed
    if input.lock_target {
        let Ok((_camera, camera_transform)) = camera_q.get_single() else {
            return;
        };

        // Remove old indicator if exists
        for indicator_entity in indicator_q.iter() {
            commands.entity(indicator_entity).despawn_recursive();
        }

        // Camera forward direction and position
        let cam_pos = camera_transform.translation();
        let cam_forward = camera_transform.forward();

        // Get list of visible dinosaurs (in front of camera)
        let visible_dinos: Vec<(Entity, Vec3, f32)> = dino_q.iter()
            .filter_map(|(entity, transform)| {
                let dino_pos = transform.translation();
                let to_dino = dino_pos - cam_pos;
                let distance = to_dino.length();

                // Check if dinosaur is in front of camera (within 90 degree FOV cone)
                let to_dino_norm = to_dino.normalize();
                let dot = cam_forward.dot(to_dino_norm);

                if dot > 0.3 && distance < 200.0 {
                    // Dinosaur is in front of camera and within range
                    return Some((entity, dino_pos, distance));
                }
                None
            })
            .collect();

        if visible_dinos.is_empty() {
            // No visible dinosaurs, clear lock
            target_lock.locked_entity = None;
            target_lock.lock_position = None;
            return;
        }

        // Sort by distance (closest first)
        let mut sorted_dinos = visible_dinos;
        sorted_dinos.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

        // If we already have a lock, cycle to the next visible dinosaur
        let target_entity = if let Some(current_lock) = target_lock.locked_entity {
            // Find the current lock's index
            if let Some(current_idx) = sorted_dinos.iter().position(|(e, _, _)| *e == current_lock) {
                // Cycle to next dinosaur (loop around)
                let next_idx = (current_idx + 1) % sorted_dinos.len();
                sorted_dinos[next_idx].0
            } else {
                // Current lock not in visible list, start with closest
                sorted_dinos[0].0
            }
        } else {
            // No current lock, lock onto closest visible
            sorted_dinos[0].0
        };

        // Update the lock
        target_lock.locked_entity = Some(target_entity);
        if let Ok((_, transform)) = dino_q.get(target_entity) {
            target_lock.lock_position = Some(transform.translation());
        }

        // Spawn red circle indicator for the new target
        commands.spawn((
            TargetLockIndicator,
            Mesh3d(meshes.add(Torus::new(1.5, 0.1))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 0.0, 0.0, 0.8),
                unlit: true,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.5, 0.0),
        )).set_parent(target_entity);
    }
}

fn update_indicator_position(
    target_lock: Res<TargetLock>,
    dino_q: Query<&GlobalTransform, With<Dinosaur>>,
    mut indicator_q: Query<&mut Transform, With<TargetLockIndicator>>,
) {
    // Update indicator position
    if let Some(locked_entity) = target_lock.locked_entity {
        if let Ok(dino_transform) = dino_q.get(locked_entity) {
            for mut transform in indicator_q.iter_mut() {
                let pos = dino_transform.translation();
                transform.translation = Vec3::new(pos.x, pos.y + 0.5, pos.z);
            }
        }
    }
}

#[derive(Component)]
struct TargetLockIndicator;
