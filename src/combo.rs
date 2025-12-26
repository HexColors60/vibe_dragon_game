use bevy::prelude::*;
use crate::pause::GameState;

/// Combo system tracking kill streaks
#[derive(Resource, Default)]
pub struct ComboSystem {
    pub current_combo: u32,
    pub max_combo: u32,
    pub combo_timer: Timer,
    pub last_kill_time: f32,
    pub combo_multiplier: f32,
}

impl ComboSystem {
    pub fn new() -> Self {
        Self {
            current_combo: 0,
            max_combo: 0,
            combo_timer: Timer::from_seconds(2.0, TimerMode::Once),
            last_kill_time: 0.0,
            combo_multiplier: 1.0,
        }
    }

    pub fn add_kill(&mut self) {
        self.current_combo += 1;
        self.combo_timer.reset();

        // Update max combo
        if self.current_combo > self.max_combo {
            self.max_combo = self.current_combo;
        }

        // Calculate multiplier based on combo count
        self.combo_multiplier = 1.0 + (self.current_combo as f32 * 0.1);
        // Cap at 5x multiplier
        self.combo_multiplier = self.combo_multiplier.min(5.0);
    }

    pub fn update(&mut self, delta: std::time::Duration) {
        self.combo_timer.tick(delta);

        // Reset combo if timer expires
        if self.combo_timer.finished() && self.current_combo > 0 {
            self.current_combo = 0;
            self.combo_multiplier = 1.0;
        }
    }

    pub fn get_score_multiplier(&self) -> f32 {
        self.combo_multiplier
    }

    pub fn get_combo_display(&self) -> String {
        if self.current_combo >= 2 {
            format!("{}x", self.current_combo)
        } else {
            String::new()
        }
    }
}

pub struct ComboPlugin;

impl Plugin for ComboPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ComboSystem>()
            .add_systems(Update, update_combo.run_if(in_state(GameState::Playing)));
    }
}

fn update_combo(
    time: Res<Time>,
    mut combo: ResMut<ComboSystem>,
) {
    combo.update(time.delta());
}
