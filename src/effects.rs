use bevy::prelude::*;
use crate::pause::GameState;

/// Event triggered when a kill happens
#[derive(Event)]
pub struct KillShakeEvent;

/// Event triggered when a hit happens
#[derive(Event)]
pub struct HitFeedbackEvent;

/// Screen shake effect resource
#[derive(Resource, Default)]
pub struct ScreenShake {
    pub intensity: f32,
    pub duration: Timer,
}

impl ScreenShake {
    pub fn trigger(&mut self, intensity: f32, duration: f32) {
        self.intensity = intensity;
        self.duration = Timer::from_seconds(duration, TimerMode::Once);
    }
}

/// Crosshair hit feedback
#[derive(Resource, Default)]
pub struct CrosshairFeedback {
    pub scale: f32,
    pub velocity: f32,
}

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScreenShake>()
            .init_resource::<CrosshairFeedback>()
            .add_event::<KillShakeEvent>()
            .add_event::<HitFeedbackEvent>()
            .add_systems(Update, (
                handle_kill_shake,
                handle_hit_feedback,
                update_screen_shake,
                update_crosshair_feedback,
            ).run_if(in_state(GameState::Playing)));
    }
}

fn handle_kill_shake(
    mut events: EventReader<KillShakeEvent>,
    mut shake: ResMut<ScreenShake>,
) {
    for _event in events.read() {
        shake.trigger(0.3, 0.15); // Intensity, Duration
    }
}

fn handle_hit_feedback(
    mut events: EventReader<HitFeedbackEvent>,
    mut feedback: ResMut<CrosshairFeedback>,
) {
    for _event in events.read() {
        feedback.scale = 2.0;
        feedback.velocity = -1.0; // Will snap back to normal
    }
}

fn update_screen_shake(
    time: Res<Time>,
    mut shake: ResMut<ScreenShake>,
    mut camera_q: Query<&mut Transform, (With<crate::camera::MainCamera>, Without<crate::ui::Crosshair>)>,
) {
    if shake.duration.finished() {
        shake.intensity = 0.0;
        // Reset camera position
        if let Ok(mut transform) = camera_q.get_single_mut() {
            transform.translation = Vec3::ZERO;
        }
        return;
    }

    shake.duration.tick(time.delta());
    let elapsed = shake.duration.elapsed_secs();
    let total = shake.duration.duration().as_secs_f32();

    // Decay shake over time
    let current_intensity = shake.intensity * (1.0 - (elapsed / total));

    // Apply random offset to camera
    if let Ok(mut transform) = camera_q.get_single_mut() {
        let offset_x = (rand::random::<f32>() - 0.5) * 2.0 * current_intensity;
        let offset_y = (rand::random::<f32>() - 0.5) * current_intensity;
        transform.translation = Vec3::new(offset_x, offset_y, transform.translation.z);
    }
}

fn update_crosshair_feedback(
    time: Res<Time>,
    mut feedback: ResMut<CrosshairFeedback>,
    mut crosshair_q: Query<&mut Sprite, With<crate::ui::Crosshair>>,
) {
    // Spring back to normal
    feedback.velocity += (1.0 - feedback.scale) * 15.0 * time.delta_secs();
    feedback.scale += feedback.velocity * time.delta_secs();
    feedback.velocity *= 0.8; // Damping

    // Clamp scale
    feedback.scale = feedback.scale.clamp(1.0, 3.0);

    for mut sprite in crosshair_q.iter_mut() {
        if let Some(ref mut size) = sprite.custom_size {
            let horizontal_size = Vec2::new(20.0 * feedback.scale, 2.0);
            let vertical_size = Vec2::new(2.0, 20.0 * feedback.scale);

            // Apply to both crosshair lines
            sprite.custom_size = Some(horizontal_size);
        }
    }
}
