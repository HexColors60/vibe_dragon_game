use bevy::prelude::*;
use crate::vehicle::PlayerVehicle;
use crate::input::PlayerInput;

pub struct CameraPlugin;

#[derive(Resource, Default)]
pub struct CameraSettings {
    pub height: f32,
    pub distance: f32,
    pub angle: f32,
}

impl CameraSettings {
    pub fn new() -> Self {
        Self {
            height: 60.0,   // High bird's eye view
            distance: 30.0, // Distance behind vehicle
            angle: 60.0,    // Look-down angle in degrees
        }
    }

    pub fn adjust_height(&mut self, delta: f32) {
        self.height = (self.height + delta).clamp(10.0, 150.0);
    }

    pub fn adjust_distance(&mut self, delta: f32) {
        self.distance = (self.distance + delta).clamp(5.0, 100.0);
    }

    pub fn adjust_angle(&mut self, delta: f32) {
        self.angle = (self.angle + delta).clamp(10.0, 85.0);
    }
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraSettings>()
            .add_systems(Startup, setup_camera)
            .add_systems(Update, (update_camera_settings, camera_follow));
    }
}

#[derive(Component)]
pub struct MainCamera;

fn setup_camera(mut commands: Commands) {
    // Initial spawn position, will be updated by camera_follow system
    commands.spawn((
        Camera3d::default(),
        MainCamera,
        Transform::from_xyz(0.0, 60.0, 30.0).looking_at(Vec3::ZERO, Vec3::Y),
        Projection::Perspective(default()),
    ));
}

fn update_camera_settings(
    input: Res<PlayerInput>,
    mut settings: ResMut<CameraSettings>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let adjust_speed = 50.0 * dt;

    // Page Up: Raise camera and increase distance (move away from ground)
    if input.camera_up {
        settings.adjust_height(adjust_speed);
        settings.adjust_distance(adjust_speed * 0.5);
    }
    // Page Down: Lower camera and decrease distance (move closer to ground)
    if input.camera_down {
        settings.adjust_height(-adjust_speed);
        settings.adjust_distance(-adjust_speed * 0.5);
    }
}

fn camera_follow(
    mut camera_q: Query<&mut Transform, (With<MainCamera>, Without<PlayerVehicle>)>,
    vehicle_q: Query<&Transform, (With<PlayerVehicle>, Without<MainCamera>)>,
    settings: Res<CameraSettings>,
) {
    let Ok(mut camera_transform) = camera_q.get_single_mut() else {
        return;
    };

    let Ok(vehicle_transform) = vehicle_q.get_single() else {
        return;
    };

    let vehicle_pos = vehicle_transform.translation;

    // Calculate camera position based on vehicle position and settings
    // Camera is positioned at (height) units above and (distance) units behind
    let angle_rad = settings.angle.to_radians();
    let vertical_offset = settings.height;
    let horizontal_offset = settings.distance * angle_rad.cos();

    let offset = Vec3::new(0.0, vertical_offset, horizontal_offset);
    let target_pos = vehicle_pos + offset;

    // Smooth follow
    camera_transform.translation = camera_transform.translation.lerp(target_pos, 0.1);

    // Look at vehicle from above
    let look_at = vehicle_pos + Vec3::new(0.0, 0.0, 0.0);
    camera_transform.look_at(look_at, Vec3::Y);
}
