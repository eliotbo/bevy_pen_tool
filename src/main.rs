// use utilities::cam::{Cam, CamPlugin};
// use utilities::plugin::PenPlugin;

use utilities::*;

use bevy::{prelude::*, render::camera::OrthographicProjection};

use std::fs::File;
// use std::io::Read;
use std::io::Write;

// TODO:
// 1. Attach UI to a UI camera
// 4. Collapse the color UI
// 7. ungroup
// 9. Compute higher quality lut for groups upon save
// 11. reduce use of Globals
// 12. make save/load preserve groups
// 13. make whole group move when selected
// 14. make undo/redo work for moving anchors and control points
// 15. make compatible with a projective perspective
// 16. Add RControl and RShift to keys
// 17. Disable move anchors and control points when hiding

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(camera_setup)
        .add_plugin(CamPlugin)
        .add_plugin(PenPlugin)
        .add_system(save_group)
        .run();
}

fn camera_setup(mut commands: Commands) {
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
}
//
//
//

pub fn save_group(
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<&Handle<Bezier>, With<BoundingBoxQuad>>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    group_query: Query<&Handle<Group>, With<GroupBoxQuad>>,
    groups: Res<Assets<Group>>,
    mut event_reader: EventReader<UiButton>,
    // mut event_writer: EventWriter<Handle<Group>>,
    // mut query: Query<(Entity, &Handle<Bezier>), With<BoundingBoxQuad>>,
) {
}
