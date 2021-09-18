use crate::util::{Globals, MyShader, UiBoard};

use bevy::{input::mouse::MouseWheel, prelude::*, render::camera::OrthographicProjection};

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
        app.add_system(camera_movevement_system.system())
            .add_system(zoom_camera.system());
    }
}

pub fn zoom_camera(
    mut query: QuerySet<(
        QueryState<(&mut OrthographicProjection, &mut Cam, &Transform)>,
        QueryState<(&mut Transform, &mut UiBoard)>,
    )>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut globals: ResMut<Globals>,
    entity_query: Query<(Entity, &Handle<MyShader>)>,
    mut res: ResMut<Assets<MyShader>>,
) {
    for event in mouse_wheel_events.iter() {
        let mut cam_query = query.q0();
        let mut cam_translation = Vec3::ZERO;

        for (mut ortho_proj, mut cam, transform) in cam_query.iter_mut() {
            let zoom_factor = 1.0 + event.y * 0.2;
            ortho_proj.scale = ortho_proj.scale * zoom_factor;

            globals.camera_scale = ortho_proj.scale;

            cam.speed = ortho_proj.scale * 3.0 / 0.15;

            for (_entity, params) in entity_query.iter() {
                res.get_mut(params).unwrap().zoom = ortho_proj.scale;
            }
            cam_translation = transform.translation;
        }

        let mut ui_query = query.q1();
        for (mut transform, mut ui_board) in ui_query.iter_mut() {
            let scale = globals.camera_scale / 0.15;
            transform.scale = Vec3::new(scale, scale, 1.0);
            transform.translation.x = (ui_board.position.x * scale) + cam_translation.x;
            transform.translation.y = (ui_board.position.y * scale) + cam_translation.y;

            // ui_board.position = transform.translation.truncate();
            ui_board.previous_position = transform.translation.truncate();
            // ui_board.size = Vec2::new(40.0, 75.0) * scale;
        }
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
    mut transforms: QuerySet<(
        QueryState<(&Cam, &mut Transform)>,
        QueryState<(&mut UiBoard, &mut Transform)>,
    )>,
    // mouse_button_input: Res<Input<MouseButton>>,
    // mut query_ui: Query<&mut Transform, With<UiBoard>>,
) {
    let mut cam_query = transforms.q0();
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
    for (mut ui_board, mut ui_transform) in transforms.q1().iter_mut() {
        ui_transform.translation += velocity;
        if do_move_cam {
            ui_board.previous_position = ui_transform.translation.truncate();
        }
    }
}
