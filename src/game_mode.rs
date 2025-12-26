use bevy::prelude::*;
use crate::pause::GameState;
use crate::GameScore;
use crate::combo::ComboSystem;

#[derive(Resource, Default)]
pub struct TimeAttackMode {
    pub is_active: bool,
    pub time_remaining: Timer,
    pub total_time: f32,
    pub kills: u32,
    pub max_combo: u32,
}

impl TimeAttackMode {
    pub fn new(duration_seconds: f32) -> Self {
        Self {
            is_active: false,
            time_remaining: Timer::from_seconds(duration_seconds, TimerMode::Once),
            total_time: duration_seconds,
            kills: 0,
            max_combo: 0,
        }
    }

    pub fn start(&mut self) {
        self.is_active = true;
        self.time_remaining.reset();
        self.kills = 0;
        self.max_combo = 0;
    }

    pub fn stop(&mut self) {
        self.is_active = false;
    }

    pub fn is_finished(&self) -> bool {
        self.is_active && self.time_remaining.finished()
    }

    pub fn get_rank(&self) -> &str {
        let score = self.kills as f32 * (1.0 + self.max_combo as f32 * 0.1);
        if score >= 50.0 { "S" }
        else if score >= 35.0 { "A" }
        else if score >= 20.0 { "B" }
        else { "C" }
    }
}

pub struct GameModePlugin;

impl Plugin for GameModePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TimeAttackMode>()
            .add_systems(Update, (
                update_time_attack,
                check_time_attack_end,
            ).run_if(in_state(GameState::Playing)));
    }
}

fn update_time_attack(
    time: Res<Time>,
    mut mode: ResMut<TimeAttackMode>,
    combo: Res<ComboSystem>,
) {
    if !mode.is_active {
        return;
    }

    mode.time_remaining.tick(time.delta());

    // Track max combo
    if combo.current_combo > mode.max_combo {
        mode.max_combo = combo.current_combo;
    }
}

fn check_time_attack_end(
    mode: Res<TimeAttackMode>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if mode.is_finished() {
        // Switch to pause/menu state when time is up
        next_state.set(GameState::Paused);
    }
}

#[derive(Component)]
pub struct TimeAttackText;

#[derive(Component)]
pub struct TimeAttackResultText;
