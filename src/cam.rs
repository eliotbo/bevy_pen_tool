use bevy_pen_tool_plugin::UiBoard;

use bevy::prelude::*;

#[derive(Component)]
pub struct Cam {
    pub speed: f32,
    pub key_left: KeyCode,
    pub key_right: KeyCode,
    pub key_up: KeyCode,
    pub key_down: KeyCode,
    pub enabled: bool,
}
impl Default for Cam {
    fn default() -> Self {
        Self {
            speed: 3.0,
            key_up: KeyCode::W,
            key_down: KeyCode::S,
            key_left: KeyCode::A,
            key_right: KeyCode::D,
            enabled: true,
        }
    }
}

pub struct CamPlugin;

impl Plugin for CamPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(camera_movevement_system);
        // .add_system(zoom_camera.system());
    }
}

pub fn movement_axis(input: &Res<Input<KeyCode>>, plus: KeyCode, minus: KeyCode) -> f32 {
    let mut axis = 0.0;
    if input.pressed(plus) && !input.pressed(KeyCode::LControl) && !input.pressed(KeyCode::LShift) {
        axis += 1.0;
    }
    if input.pressed(minus) && !input.pressed(KeyCode::LControl) && !input.pressed(KeyCode::LShift)
    {
        axis -= 1.0;
    }
    return axis;
}

pub fn camera_movevement_system(
    keyboard_input: Res<Input<KeyCode>>,
    // mut query: Query<(&Cam, &mut Transform)>,
    mut transforms: ParamSet<(
        Query<(&Cam, &mut Transform)>,
        Query<(&mut UiBoard, &mut Transform)>,
    )>,
    // mouse_button_input: Res<Input<MouseButton>>,
    // mut query_ui: Query<&mut Transform, With<UiBoard>>,
) {
    let mut cam_query = transforms.p0();
    let mut velocity = Vec3::ZERO;
    let mut do_move_cam = false;
    for (cam, mut transform) in cam_query.iter_mut() {
        let (axis_side, axis_up) = if cam.enabled {
            (
                movement_axis(&keyboard_input, cam.key_right, cam.key_left),
                movement_axis(&keyboard_input, cam.key_up, cam.key_down),
            )
        } else {
            (0.0, 0.0)
        };

        if axis_side.abs() > 0.0000001 || axis_up.abs() > 0.0000001 {
            do_move_cam = true;
        }

        velocity = Vec3::new(axis_side * cam.speed, axis_up * cam.speed, 0.0);

        transform.translation += velocity;
    }
    for (mut ui_board, mut ui_transform) in transforms.p1().iter_mut() {
        ui_transform.translation += velocity;
        if do_move_cam {
            ui_board.previous_position = ui_transform.translation.truncate();
        }
    }
}
