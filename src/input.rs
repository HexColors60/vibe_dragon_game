use bevy::prelude::*;
use bevy::window::CursorGrabMode;

pub struct InputPlugin;

#[derive(Resource, Default, Clone)]
pub struct PlayerInput {
    pub move_forward: bool,
    pub move_backward: bool,
    pub move_left: bool,
    pub move_right: bool,
    pub shooting: bool,
    pub mouse_position: Vec2,
}

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerInput>()
            .add_systems(Startup, grab_cursor)
            .add_systems(Update, (handle_key_input, handle_mouse_input));
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
}

fn handle_mouse_input(
    mut input: ResMut<PlayerInput>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<CursorMoved>,
) {
    input.shooting = mouse_button.pressed(MouseButton::Left);

    for event in mouse_motion.read() {
        input.mouse_position = event.position;
    }
}
