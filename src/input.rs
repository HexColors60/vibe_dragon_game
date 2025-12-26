use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use bevy::input::mouse::MouseMotion;
use crate::weapon_system::{WeaponType, WeaponSwitchedEvent, WeaponInventory};
use crate::pause::GameState;

pub struct InputPlugin;

#[derive(Resource, Default, Clone)]
pub struct PlayerInput {
    pub move_forward: bool,
    pub move_backward: bool,
    pub move_left: bool,
    pub move_right: bool,
    pub shooting: bool,
    pub mouse_position: Vec2,
    pub turret_left: bool,
    pub turret_right: bool,
    pub lock_target: bool,
    pub pause: bool,
    pub weapon_switch_1: bool,
    pub weapon_switch_2: bool,
    pub weapon_switch_3: bool,
    pub weapon_scroll: f32, // Positive = next weapon, Negative = previous
}

#[derive(Resource, Default)]
pub struct TargetLock {
    pub locked_entity: Option<Entity>,
    pub lock_position: Option<Vec3>,
}

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerInput>()
            .init_resource::<TargetLock>()
            .add_event::<WeaponSwitchedEvent>()
            .add_systems(Startup, grab_cursor)
            .add_systems(Update, (
                handle_key_input,
                handle_mouse_input,
                handle_mouse_motion,
                handle_mouse_wheel,
                handle_weapon_switching,
            ).run_if(in_state(GameState::Playing)));
    }
}

fn grab_cursor(mut window_q: Query<&mut Window>) {
    if let Ok(mut window) = window_q.get_single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
    }
}

fn handle_key_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut input: ResMut<PlayerInput>,
) {
    input.move_forward = keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp);
    input.move_backward = keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown);
    input.move_left = keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft);
    input.move_right = keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight);

    input.turret_left = keyboard.pressed(KeyCode::KeyQ);
    input.turret_right = keyboard.pressed(KeyCode::KeyE);
    input.pause = keyboard.just_pressed(KeyCode::Escape);

    // Weapon switching
    input.weapon_switch_1 = keyboard.just_pressed(KeyCode::Digit1);
    input.weapon_switch_2 = keyboard.just_pressed(KeyCode::Digit2);
    input.weapon_switch_3 = keyboard.just_pressed(KeyCode::Digit3);
}

fn handle_mouse_input(
    mut input: ResMut<PlayerInput>,
    mouse_button: Res<ButtonInput<MouseButton>>,
) {
    input.shooting = mouse_button.pressed(MouseButton::Left);
    input.lock_target = mouse_button.just_pressed(MouseButton::Right);
}

fn handle_mouse_motion(
    mut input: ResMut<PlayerInput>,
    mut mouse_motion: EventReader<MouseMotion>,
) {
    for event in mouse_motion.read() {
        input.mouse_position += event.delta;
    }
}

fn handle_mouse_wheel(
    mut input: ResMut<PlayerInput>,
    mut mouse_wheel: EventReader<bevy::input::mouse::MouseWheel>,
) {
    for event in mouse_wheel.read() {
        // Accumulate scroll value
        input.weapon_scroll += event.y;
    }
}

fn handle_weapon_switching(
    input: Res<PlayerInput>,
    mut weapon_inventory: ResMut<WeaponInventory>,
    mut weapon_events: EventWriter<WeaponSwitchedEvent>,
) {
    let mut switched = None;

    // Check keyboard shortcuts first
    if input.weapon_switch_1 {
        weapon_inventory.switch_to(WeaponType::MachineGun);
        switched = Some(WeaponType::MachineGun);
    } else if input.weapon_switch_2 {
        weapon_inventory.switch_to(WeaponType::Shotgun);
        switched = Some(WeaponType::Shotgun);
    } else if input.weapon_switch_3 {
        weapon_inventory.switch_to(WeaponType::RocketLauncher);
        switched = Some(WeaponType::RocketLauncher);
    }
    // Check mouse wheel
    else if input.weapon_scroll.abs() > 0.1 {
        if input.weapon_scroll > 0.0 {
            weapon_inventory.next_weapon();
        } else {
            weapon_inventory.previous_weapon();
        }
        switched = Some(weapon_inventory.current_weapon);
    }

    if let Some(weapon) = switched {
        weapon_events.send(WeaponSwitchedEvent { new_weapon: weapon });
    }
}
