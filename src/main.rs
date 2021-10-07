mod cam;

use bevy_pen_tool_plugin::{Bezier, Globals, Group, PenPlugin};
use cam::Cam;

use bevy::{math::Quat, prelude::*, render::camera::OrthographicProjection};

// TODO:
// 13. make whole group move when selected
// 16. Add RControl and RShift to keys

// long-term
// 1. Attach UI to a UI camera
// 7. ungroup
// 14. make undo/redo

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(camera_setup)
        // .add_plugin(CamPlugin)
        .add_plugin(PenPlugin)
        .add_system(spawn_heli)
        .add_system(tests)
        .add_system(follow_bezier_group.label("animation"))
        .add_system(turn_round_animation.label("turn").after("animation"))
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
                scale: 0.19, //view.camera_scale,
                far: 100000.0,
                near: -100000.0,
                // top: 115.0,
                // dummy
                ..Default::default()
            },
            ..OrthographicCameraBundle::new_2d()
        })
        .insert(Cam::default());

    // sets the number of rows in the animation position look-up table. More points will
    // make an animation smoother, but will take more space in memory
    globals.group_lut_num_points = 100;
}

fn tests(
    keyboard_input: Res<Input<KeyCode>>,
    // groups: ResMut<Assets<Group>>,
    // globals: Res<Globals>,
) {
    if keyboard_input.just_pressed(KeyCode::V) {
        println!(" ");
        println!(" ");
        println!(" ");
        println!(" ");
        println!(" ");
        println!(" ");
    }
}

struct TurnRoundAnimation;
struct FollowBezierAnimation;

fn spawn_heli(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    keyboard_input: Res<Input<KeyCode>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if keyboard_input.just_pressed(KeyCode::V) {
        let heli_handle = asset_server.load("textures/heli.png");
        let size = Vec2::new(50.0, 50.0);
        let heli_sprite = commands
            .spawn_bundle(SpriteBundle {
                material: materials.add(heli_handle.into()),
                // mesh: mesh_handle_button.clone(),
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, -222.0)),
                sprite: Sprite::new(size),
                ..Default::default()
            })
            .insert(FollowBezierAnimation)
            .id();
        let copter_handle = asset_server.load("textures/copter.png");
        let copter_sprite = commands
            .spawn_bundle(SpriteBundle {
                material: materials.add(copter_handle.into()),
                // mesh: mesh_handle_button.clone(),
                transform: Transform::from_translation(Vec3::new(6.0, 2.0, -122.0)),
                sprite: Sprite::new(size),
                ..Default::default()
            })
            .insert(TurnRoundAnimation)
            .id();

        commands.entity(heli_sprite).push_children(&[copter_sprite]);
    }
}

// animates the helicopter blades
fn turn_round_animation(mut query: Query<(&mut Transform, &TurnRoundAnimation)>) {
    for (mut transform, _) in query.iter_mut() {
        let quat = Quat::from_rotation_z(0.2);
        transform.rotate(quat);
    }
}

// moves the helicopter along the Group path
// a Group is made up of many latched Bezier curves
fn follow_bezier_group(
    mut query: Query<(&mut Transform, &FollowBezierAnimation)>,
    groups: Res<Assets<Group>>,
    bezier_curves: ResMut<Assets<Bezier>>,
    time: Res<Time>,
) {
    if let Some(group) = groups.iter().next() {
        let t_time = (time.seconds_since_startup() * 0.1) % 1.0;
        let pos = group.1.compute_position_with_lut(t_time as f32);

        // the heli looks ahead (10% of the curve length) to orient itself
        let further_pos = group
            .1
            .compute_position_with_lut(((t_time + 0.1) % 1.0) as f32);

        for (mut transform, _bezier_animation) in query.iter_mut() {
            transform.translation.x = pos.x;
            transform.translation.y = pos.y;

            // compute the forward direction
            let diff = pos - further_pos;
            let target_angle = diff.y.atan2(diff.x) + 3.1415;
            let mut next_angle = target_angle;

            let (_axis, current_angle) = transform.rotation.to_axis_angle();

            let max_angle = 3.0 / 180.0 * 3.1416;

            let target_360 = target_angle - 3.1416 * 2.0;
            let mut diff = target_angle - current_angle;
            let diff_360 = target_360 - current_angle;
            if diff_360.abs() < diff.abs() {
                diff = diff_360;
            }

            // clamp the angular speed of heli
            if diff.abs() > max_angle {
                next_angle = current_angle + diff.signum() * max_angle;
            }

            transform.rotation = Quat::from_rotation_z(next_angle as f32);
        }
    }
}
