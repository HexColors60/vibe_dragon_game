use bevy::prelude::*;

use crate::vehicle::PlayerVehicle;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(Update, camera_follow);
    }
}

#[derive(Component)]
pub struct MainCamera;

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        MainCamera,
        Transform::from_xyz(0.0, 8.0, 15.0).looking_at(Vec3::Y * 2.0, Vec3::Y),
        Projection::Perspective {
            fov: 60.0_f32.to_radians(),
            ..default()
        },
    ));
}

fn camera_follow(
    mut camera_q: Query<&mut Transform, (With<MainCamera>, Without<PlayerVehicle>)>,
    vehicle_q: Query<&Transform, (With<PlayerVehicle>, Without<MainCamera>)>,
) {
    let Ok(mut camera_transform) = camera_q.get_single_mut() else {
        return;
    };

    let Ok(vehicle_transform) = vehicle_q.get_single() else {
        return;
    };

    let vehicle_pos = vehicle_transform.translation;
    let vehicle_forward = vehicle_transform.forward();

    // Camera position behind and above the vehicle
    let offset = Vec3::new(0.0, 6.0, 12.0);
    let target_pos = vehicle_pos + offset;

    // Smooth follow
    camera_transform.translation = camera_transform.translation.lerp(target_pos, 0.1);

    // Look at vehicle (slightly above)
    let look_at = vehicle_pos + Vec3::new(0.0, 2.0, 0.0);
    camera_transform.look_at(look_at, Vec3::Y);
}
