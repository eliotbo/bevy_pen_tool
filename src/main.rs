mod cam;

use bevy_pen_tool_plugin::{Bezier, Globals, PenPlugin};
use cam::{Cam, CamPlugin};

use bevy::{prelude::*, render::camera::OrthographicProjection};

// TODO:
// - make whole group move when selected
// - make delete button
// - add Delete to readme
// - change drag a rectangle to drag a selection window
// - fix bug with selecting window appearing on default position for a frame
// - the position of a newly-latched anchor is incorrect
// - bug when loading group: middle quads not despawned

// long-term
// - ungroup
// - make undo/redo
// - Attach UI to a UI camera

fn main() {
    App::new()
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
        .spawn_bundle(OrthographicCameraBundle {
            transform: Transform::from_translation(Vec3::new(00.0, 0.0, 10.0))
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            orthographic_projection: OrthographicProjection {
                scale: 0.19,
                far: 100000.0,
                near: -100000.0,
                ..Default::default()
            },
            ..OrthographicCameraBundle::new_2d()
        })
        .insert(Cam::default());

    // sets the number of rows in the animation position look-up table. More points will
    // make an animation smoother, but will take more space in memory
    globals.group_lut_num_points = 100;
}

fn test(keyboard_input: Res<Input<KeyCode>>, mut bezier_curves: ResMut<Assets<Bezier>>) {
    if keyboard_input.just_pressed(KeyCode::V) {
        println!("test: {:?}", 123);
    }
}
