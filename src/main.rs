// use utilities::cam::{Cam, CamPlugin};
// use utilities::plugin::PenPlugin;

use utilities::*;

use bevy::{prelude::*, render::camera::OrthographicProjection};

// TODO:
// 1. Attach UI to a UI camera
// 4. Collapse the color UI
// 7. ungroup

// 11. reduce use of Globals
// 12. make save/load preserve groups
// 13. make whole group move when selected
// 14. make undo/redo work for moving anchors and control points
// 15. make compatible with a projective perspective
// 16. Add RControl and RShift to keys

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(camera_setup)
        .add_plugin(CamPlugin)
        .add_plugin(PenPlugin)
        .add_system(tests)
        .run();
}

fn camera_setup(mut commands: Commands, mut globals: ResMut<Globals>) {
    commands
        .spawn_bundle(OrthographicCameraBundle {
            transform: Transform::from_translation(Vec3::new(00.0, 0.0, 10.0))
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            orthographic_projection: OrthographicProjection {
                scale: 0.3, //view.camera_scale,
                far: 100000.0,
                near: -100000.0,
                // top: 115.0,
                // dummy
                ..Default::default()
            },
            ..OrthographicCameraBundle::new_2d()
        })
        .insert(Cam::default());

    globals.group_lut_num_points = 200;
}

fn tests(
    keyboard_input: Res<Input<KeyCode>>,
    groups: ResMut<Assets<Group>>,
    globals: Res<Globals>,
) {
    if keyboard_input.just_pressed(KeyCode::V) {
        for group in groups.iter() {
            let mut lut = group.1.standalone_lut.1.clone();
            let mut lut2 = group.1.standalone_lut.1.clone();
            lut.push(Vec2::new(0.0, 0.0));
            lut2.insert(0, Vec2::new(0.0, 0.0));
            println!(" ");
            println!(" ");
            println!(" ");
            println!(" ");
            println!(" ");
            println!(" ");
            for (l1, l2) in lut.iter().zip(lut2.iter()) {
                println!("number of points in lut: {:?} ", l1.distance(*l2));
            }
        }
    }
}
