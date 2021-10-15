mod cam;

use bevy_pen_tool_plugin::{Globals, Group, PenPlugin};
use cam::{Cam, CamPlugin};

use bevy::{math::Quat, prelude::*, render::camera::OrthographicProjection};

// How to
//
// 1. run main.rs
// 2. explore and have fun
// 3. spawn multiple curves
// 4. compute the look-up tables for each curve by pressing Shift + T
// (step 4 can be repeated anytime an anchor or control point is moved)
// 5. latch the curves together if they are not already latched at spawn
// 6. moves the anchors and control points to a desired position
// (there is a "hide control points" button for when they overlap with anchors)
// 7. select the latched curves by clicking and dragging a selection box
// 8. group the curves with Ctrl + G and repeat step 4
// 9. save the look-up table with Ctrl + S
// 10. use the look-up table in your app (see the simple_animation.rs example)

// Notes
//
// - bevy_pen_tool does not work with a Perspective Camera (only Orthographic)
// - cannot save multiple groups at once, only a single one
// - currently, the plugin only works with bevy version 0.5, rev="615d43b",
//      but this will change
// - pressing load will delete everything on the canvas before loading

// TODO:
// - make whole group move when selected

// - popups for save/load
// - add Delete to readme
// - change drag a rectangle to drag a selection window
// - fix bug with selecting window appearing on default position for a frame

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
        // .add_startup_system(spawn_heli)
        // .add_system(follow_bezier_group.label("animation"))
        // .add_system(turn_round_animation)
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

// #[derive(Component)]
// struct TurnRoundAnimation;

// #[derive(Component)]
// struct FollowBezierAnimation;

// fn spawn_heli(
//     mut commands: Commands,
//     asset_server: Res<AssetServer>,
//     mut materials: ResMut<Assets<ColorMaterial>>,
// ) {
//     let heli_handle = asset_server.load("textures/heli.png");
//     let size = Vec2::new(50.0, 50.0);
//     let heli_sprite = commands
//         .spawn_bundle(SpriteBundle {
//             material: materials.add(heli_handle.into()),
//             // mesh: mesh_handle_button.clone(),
//             transform: Transform::from_translation(Vec3::new(0.0, 0.0, -222.0)),
//             sprite: Sprite::new(size),
//             visible: Visible {
//                 is_visible: false,
//                 is_transparent: true,
//             },
//             ..Default::default()
//         })
//         .insert(FollowBezierAnimation)
//         .id();
//     let copter_handle = asset_server.load("textures/copter.png");
//     let copter_sprite = commands
//         .spawn_bundle(SpriteBundle {
//             material: materials.add(copter_handle.into()),
//             // mesh: mesh_handle_button.clone(),
//             transform: Transform::from_translation(Vec3::new(6.0, 2.0, -122.0)),
//             sprite: Sprite::new(size),
//             visible: Visible {
//                 is_visible: false,
//                 is_transparent: true,
//             },
//             ..Default::default()
//         })
//         .insert(TurnRoundAnimation)
//         .id();

//     commands.entity(heli_sprite).push_children(&[copter_sprite]);
// }

// // animates the helicopter blades
// fn turn_round_animation(mut query: Query<(&mut Transform, &TurnRoundAnimation)>) {
//     for (mut transform, _) in query.iter_mut() {
//         let quat = Quat::from_rotation_z(0.2);
//         transform.rotate(quat);
//     }
// }

// // moves the helicopter along the Group path
// // a Group is made up of many latched Bezier curves
// fn follow_bezier_group(
//     mut query: Query<(&mut Transform, &FollowBezierAnimation)>,
//     mut visible_query: Query<
//         &mut Visible,
//         Or<(With<FollowBezierAnimation>, With<TurnRoundAnimation>)>,
//     >,
//     groups: Res<Assets<Group>>,
//     time: Res<Time>,
// ) {
//     if let Some(group) = groups.iter().next() {
//         for mut visible in visible_query.iter_mut() {
//             visible.is_visible = true;
//         }

//         let t_time = (time.seconds_since_startup() * 0.1) % 1.0;
//         let pos = group.1.compute_position_with_lut(t_time as f32);

//         // the heli looks ahead (10% of the curve length) to orient itself
//         let further_pos = group
//             .1
//             .compute_position_with_lut(((t_time + 0.1) % 1.0) as f32);

//         for (mut transform, _bezier_animation) in query.iter_mut() {
//             transform.translation.x = pos.x;
//             transform.translation.y = pos.y;

//             // compute the forward direction
//             let diff = pos - further_pos;
//             let target_angle = diff.y.atan2(diff.x) + 3.1415;
//             let mut next_angle = target_angle;

//             let (_axis, current_angle) = transform.rotation.to_axis_angle();

//             let max_angle = 3.0 / 180.0 * 3.1416;

//             let target_360 = target_angle - 3.1416 * 2.0;
//             let mut diff = target_angle - current_angle;
//             let diff_360 = target_360 - current_angle;
//             if diff_360.abs() < diff.abs() {
//                 diff = diff_360;
//             }

//             // clamp the angular speed of heli
//             if diff.abs() > max_angle {
//                 next_angle = current_angle + diff.signum() * max_angle;
//             }

//             transform.rotation = Quat::from_rotation_z(next_angle as f32);
//         }
//     }
// }
