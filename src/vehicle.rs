use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::input::PlayerInput;

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

impl Plugin for VehiclePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_vehicle)
            .add_systems(Update, (handle_vehicle_movement, rotate_weapon_turret));
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
        RigidBody::KinematicPositionBased,
        Collider::cuboid(2.0, 1.0, 4.0),
        Friction::new(0.8),
        Mass::new(1500.0),
    )).id();

    // Vehicle body
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.0, 0.8, 4.0))),
        MeshMaterial3d(materials.add(body_color)),
        Transform::from_xyz(0.0, 0.5, 0.0),
        Parent(vehicle_entity),
    ));

    // Cabin
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.8, 0.7, 2.0))),
        MeshMaterial3d(materials.add(cabin_color)),
        Transform::from_xyz(0.0, 1.2, -0.5),
        Parent(vehicle_entity),
    ));

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
            Parent(vehicle_entity),
        ));
    }

    // Weapon mount base
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.2, 0.3))),
        MeshMaterial3d(materials.add(gun_color)),
        Transform::from_xyz(0.0, 1.8, 0.0),
        Parent(vehicle_entity),
    ));

    // Machine gun barrel (will rotate to face mouse direction)
    commands.spawn((
        WeaponTurret,
        Mesh3d(meshes.add(Cylinder::new(0.08, 1.5))),
        MeshMaterial3d(materials.add(gun_color)),
        Transform::from_xyz(0.0, 1.9, 0.0)
            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        Parent(vehicle_entity),
    ));
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

    let dt = time.delta_seconds();

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
    mut turret_q: Query<&mut Transform, (With<WeaponTurret>, Without<PlayerVehicle>)>,
    vehicle_q: Query<&Transform, (With<PlayerVehicle>, Without<WeaponTurret>)>,
    camera_q: Query<&Transform, (Without<WeaponTurret>, Without<PlayerVehicle>)>,
    window_q: Query<&Window>,
    camera_3d_q: Query<(&Camera, &GlobalTransform)>,
) {
    let Ok(mut turret_transform) = turret_q.get_single_mut() else {
        return;
    };

    let Ok(vehicle_transform) = vehicle_q.get_single() else {
        return;
    };

    let Ok(window) = window_q.get_single() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_3d_q.get_single() else {
        return;
    };

    // Get cursor position in world
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    // Raycast from camera
    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else {
        return;
    };

    // Project to a horizontal plane at vehicle height
    let plane_origin = vehicle_transform.translation + Vec3::Y * 2.0;
    let plane_normal = Vec3::Y;
    let t = (plane_origin - ray.origin).dot(plane_normal) / ray.direction.dot(plane_normal);

    if t > 0.0 {
        let target_point = ray.origin + ray.direction * t;

        // Rotate turret to face target
        let turret_pos = vehicle_transform.translation + Vec3::new(0.0, 1.9, 0.0);
        let direction = (target_point - turret_pos).normalize();

        if direction.length_squared() > 0.01 {
            let forward = Vec3::new(direction.x, 0.0, direction.z).normalize();
            let up = Vec3::Y;
            let right = forward.cross(up).normalize();
            let new_up = right.cross(forward).normalize();

            turret_transform.rotation = Quat::from_mat3(&glam::Mat3::from_cols(
                right,
                new_up,
                -forward,
            ));
        }
    }
}
