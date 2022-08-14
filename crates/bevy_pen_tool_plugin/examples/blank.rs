use bevy_pen_tool_model::model::*;
use bevy_pen_tool_plugin::{BevyPenToolPlugin, Bezier, Globals};

// use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::{prelude::*, render::camera::OrthographicProjection};
// TODO:
// - add enabled/disabled to buttons
// - bug in officiate_latch_partnership(..) at let handle_entity_2 = maps.bezier_map[&latch.latched_to_id.into()].clone();
//       perhaps the bug involves deletion and redo but I'm not sure

// long-term
// - compatibility with multiple groups
// - Attach UI to a UI camera -- waiting for UI to be compatible with shaders

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "pen".to_string(),
            width: 1200.,
            height: 800.,
            // vsync: true,
            ..Default::default()
        })
        // .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(CamPlugin)
        .add_startup_system(camera_setup)
        .add_plugin(BevyPenToolPlugin)
        .add_system(tests)
        .run();
}

fn camera_setup(mut commands: Commands, mut globals: ResMut<Globals>) {
    //
    // bevy_pen_tool is not compatible with a Perspective Camera
    commands
        .spawn_bundle(Camera2dBundle {
            transform: Transform::from_translation(Vec3::new(00.0, 0.0, 1.0))
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            projection: OrthographicProjection {
                scale: 1.0,
                far: 100000.0,
                near: -100000.0,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Cam::default());

    // sets the number of rows in the animation position look-up table. More points will
    // make an animation smoother, but will take more space in memory
    globals.group_lut_num_points = 100;
    globals.road_width = 8.0;
}

fn tests(keyboard_input: Res<Input<KeyCode>>, mut _bezier_curves: ResMut<Assets<Bezier>>) {
    if keyboard_input.just_pressed(KeyCode::V) {
        println!("test: {:?}", 123);
    }
}

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
