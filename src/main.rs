// use utilities::cam::{Cam, CamPlugin};
// use utilities::plugin::PenPlugin;

use utilities::*;

use bevy::{math::Quat, prelude::*, render::camera::OrthographicProjection};

// TODO:
// -2. Bug with group at latched points --> no bug when compute_position_with_lut
// 0. Make independent of camera
// 1. Attach UI to a UI camera
// 4. Collapse the color UI
// 7. ungroup

// 11. reduce use of Globals
// 12. make save/load preserve groups
// 13. make whole group move when selected
// 14. make undo/redo

// 16. Add RControl and RShift to keys

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(camera_setup)
        .add_plugin(CamPlugin)
        .add_plugin(PenPlugin)
        .add_system(spawn_heli)
        .add_system(tests)
        .add_system(follow_bezier_group.label("animation"))
        .add_system(turn_round_animation.label("turn").after("animation"))
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

    globals.group_lut_num_points = 100;
}

fn tests(
    keyboard_input: Res<Input<KeyCode>>,
    groups: ResMut<Assets<Group>>,
    globals: Res<Globals>,
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
    mut globals: ResMut<Globals>,
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

fn turn_round_animation(mut query: Query<(&mut Transform, &TurnRoundAnimation)>) {
    for (mut transform, _) in query.iter_mut() {
        let quat = Quat::from_rotation_z(0.2);
        transform.rotate(quat);
    }
}

fn follow_bezier_group(
    mut query: Query<(&mut Transform, &FollowBezierAnimation)>,
    mut globals: ResMut<Globals>,
    groups: Res<Assets<Group>>,
    bezier_curves: ResMut<Assets<Bezier>>,
    time: Res<Time>,
) {
    if let Some(group) = groups.iter().next() {
        let t_time = (time.seconds_since_startup() * 0.1) % 1.0;
        let pos = group.1.compute_position_with_bezier(&bezier_curves, t_time);
        // let pos2 = group
        //     .1
        //     // .compute_position_with_bezier(&bezier_curves, (t_time + 0.001) % 1.0);
        //     .compute_position_with_lut(((t_time + 0.001) % 1.0) as f32);

        let pos2 = group
            .1
            // .compute_position_with_bezier(&bezier_curves, (t_time + 0.001) % 1.0);
            .compute_position_with_lut(((t_time + 0.1) % 1.0) as f32);

        for (mut transform, _bezier_animation) in query.iter_mut() {
            transform.translation.x = pos.x;
            transform.translation.y = pos.y;

            let diff = pos - pos2;
            let target_angle = diff.y.atan2(diff.x) + 3.1415;
            let (axis, mut current_angle) = transform.rotation.to_axis_angle();
            let mut next_angle = target_angle;

            println!(
                "angles: {}, {}",
                target_angle * 180.0 / 3.1416,
                current_angle * 180.0 / 3.1416,
            );
            let max_angle = 3.0 / 180.0 * 3.1416;

            let target_360 = target_angle - 3.1416 * 2.0;
            let mut diff = target_angle - current_angle;
            let diff_360 = target_360 - current_angle;
            if diff_360.abs() < diff.abs() {
                diff = diff_360;
            }

            if diff.abs() > max_angle {
                next_angle = current_angle + diff.signum() * max_angle;
                println!("jutin");
            }

            transform.rotation = Quat::from_rotation_z(next_angle as f32);
        }
    }
}
