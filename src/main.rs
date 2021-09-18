mod cam;
mod inputs;
mod moves;
mod plugin;
mod spawner;
mod util;

use cam::{Cam, CamPlugin};
use plugin::PenPlugin;

use bevy::{prelude::*, render::camera::OrthographicProjection};

// TODO:
// 0. Make everything scale dependent instead of absolute

// 6. make a plugin
// 8. add audio samples when latching / unlatching / grouping / ungrouping
// 10. add hide button
// 7. ungroup
// 1. Contour the control points with a dark color?
// 4. Collapse the UI
// 9. Compute higher quality lut for groups upon save
// 11. reduce use of Globals
// 12. make save/load preserve groups
// 13. make whole group move when selected
// 14. make undo/redo work for moving anchors and control points

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(CamPlugin)
        .add_plugin(PenPlugin)
        .add_startup_system(camera_setup)
        .run();
}

fn camera_setup(mut commands: Commands) {
    commands
        .spawn_bundle(OrthographicCameraBundle {
            transform: Transform::from_translation(Vec3::new(00.0, 0.0, 10.0))
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            orthographic_projection: OrthographicProjection {
                scale: 0.15, //view.camera_scale,
                far: 100000.0,
                near: -100000.0,
                // top: 115.0,
                ..Default::default()
            },
            ..OrthographicCameraBundle::new_2d()
        })
        .insert(Cam::default());
}
