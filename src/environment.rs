use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;
use crate::pause::GameState;
use crate::vehicle::PlayerVehicle;

#[derive(Component)]
pub struct WaterBody {
    pub slow_factor: f32, // Reduces vehicle speed to this factor (0.5 = 50% speed)
}

#[derive(Component)]
pub struct Obstacle;

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_environment)
            .add_systems(Update, (
                apply_water_effects,
            ).run_if(in_state(GameState::Playing)));
    }
}

fn spawn_environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let water_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.2, 0.5, 0.8, 0.7),
        unlit: true,
        ..default()
    });

    let obstacle_material = materials.add(Color::srgb(0.4, 0.4, 0.45));

    let mut rng = rand::thread_rng();

    // Spawn water bodies (rivers and lakes)
    // Create a river flowing through the map
    for i in -5..5 {
        let z = i as f32 * 30.0;
        let width = 15.0 + (rand::random::<f32>() * 5.0);

        commands.spawn((
            WaterBody { slow_factor: 0.5 },
            Transform::from_xyz(0.0, -0.3, z),
            Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::new(500.0, width)))),
            MeshMaterial3d(water_material.clone()),
        ));

        // Add collision for water (optional - makes it a physical body)
        commands.spawn((
            Transform::from_xyz(0.0, -0.5, z),
            Collider::halfspace(Vec3::new(0.0, 1.0, 0.0)).unwrap(),
            Sensor, // Make it a sensor so it detects but doesn't block
        ));
    }

    // Spawn some lakes
    for _ in 0..3 {
        let x = rng.gen_range(-100.0..100.0);
        let z = rng.gen_range(-100.0..100.0);
        let radius = rng.gen_range(10.0..20.0);

        commands.spawn((
            WaterBody { slow_factor: 0.5 },
            Transform::from_xyz(x, -0.3, z),
            Mesh3d(meshes.add(Circle { radius })),
            MeshMaterial3d(water_material.clone()),
        ));
    }

    // Spawn rock obstacles
    for _ in 0..30 {
        let x: f32 = rng.gen_range(-150.0..150.0);
        let z: f32 = rng.gen_range(-150.0..150.0);
        let scale: f32 = rng.gen_range(1.0..3.0);

        // Don't spawn too close to origin
        if x.abs() < 10.0 && z.abs() < 10.0 {
            continue;
        }

        commands.spawn((
            Obstacle,
            Transform::from_xyz(x, scale * 0.3, z).with_scale(Vec3::splat(scale)),
            Mesh3d(meshes.add(Sphere { radius: 0.5 })),
            MeshMaterial3d(obstacle_material.clone()),
            RigidBody::Fixed,
            Collider::ball(scale * 0.5),
        ));
    }

    // Spawn fallen tree obstacles
    for _ in 0..15 {
        let x: f32 = rng.gen_range(-120.0..120.0);
        let z: f32 = rng.gen_range(-120.0..120.0);
        let rotation: f32 = rng.gen_range(0.0..std::f32::consts::PI);

        // Don't spawn too close to origin
        if x.abs() < 10.0 && z.abs() < 10.0 {
            continue;
        }

        commands.spawn((
            Obstacle,
            Transform::from_xyz(x, 0.5, z)
                .with_rotation(Quat::from_rotation_y(rotation))
                .with_scale(Vec3::new(0.8, 0.8, 6.0)),
            Mesh3d(meshes.add(Cylinder::new(0.5, 1.0))),
            MeshMaterial3d(materials.add(Color::srgb(0.4, 0.25, 0.15))),
            RigidBody::Fixed,
            Collider::cylinder(0.5, 3.0),
        ));
    }
}

fn apply_water_effects(
    water_q: Query<&WaterBody, (Without<PlayerVehicle>,)>,
    vehicle_q: Query<&Transform, With<PlayerVehicle>>,
    mut vehicle_speed: EventWriter<crate::vehicle::SpeedModifierEvent>,
) {
    let Ok(vehicle_transform) = vehicle_q.get_single() else {
        return;
    };

    let vehicle_pos = vehicle_transform.translation;

    // Check if vehicle is in any water body
    for water in water_q.iter() {
        // Simple distance check for water bodies
        // In a real implementation, you'd check actual overlap
        let distance = vehicle_pos.length(); // Simplified check

        // Check if roughly in water (z-coordinate near water bodies)
        let in_water = (vehicle_pos.z % 30.0).abs() < 10.0;

        if in_water {
            // Send speed modification event
            vehicle_speed.send(crate::vehicle::SpeedModifierEvent {
                multiplier: water.slow_factor,
            });
            return;
        }
    }
}
