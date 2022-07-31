mod cam;

use bevy_pen_tool_plugin::{Bezier, Globals, PenPlugin};
use cam::{Cam, CamPlugin};

use bevy::{prelude::*, render::camera::OrthographicProjection};

// TODO:
// - make whole group move when selected
// - add enabled/disabled to buttons

// long-term
// - ungroup
// - compatibility with multiple groups
// - undo/redo
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
        .add_plugins(DefaultPlugins)
        .add_plugin(CamPlugin)
        .add_startup_system(camera_setup)
        .add_plugin(PenPlugin)
        .add_system(test)
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

fn test(keyboard_input: Res<Input<KeyCode>>, mut _bezier_curves: ResMut<Assets<Bezier>>) {
    if keyboard_input.just_pressed(KeyCode::V) {
        println!("test: {:?}", 123);
    }
}
