use bevy::{prelude::*, render::camera::OrthographicProjection};

use serde::Deserialize;
use std::io::Read;

//
//
//
// This example shows how to load a look-up table created with bevy_pen_tool
// and how to run the corresponding animation totally independently of bevy_pen_tool
//
//
//

// data structure for the look-up table that will be deserialized (read from disk)
#[derive(Deserialize)]
struct Lut {
    path_length: f32,
    lut: Vec<Vec2>,
}

impl Lut {
    // loads a look-up table that was saved in assets/lut using bevy_pen_tool
    fn load() -> Lut {
        let lut_path = "saved/look_up_tables/my_group0.lut";
        let mut file = std::fs::File::open(lut_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let loaded_lut: Lut = serde_json::from_str(&contents).unwrap();
        return loaded_lut;
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        //
        // insert the look-up table as a resource.
        .insert_resource(Lut::load())
        .add_startup_system(camera_setup)
        .add_startup_system(spawn_quad)
        .add_system(follow_path)
        .run();
}

fn camera_setup(mut commands: Commands) {
    //
    // bevy_pen_tool is not compatible with Perspective Cameras
    commands.spawn_bundle(Camera2dBundle {
        transform: Transform::from_translation(Vec3::new(00.0, 0.0, 10.0))
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        projection: OrthographicProjection {
            scale: 0.19,
            far: 100000.0,
            near: -100000.0,
            ..Default::default()
        },
        ..Default::default()
    });
}

#[derive(Component)]
struct Animation;

fn spawn_quad(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>, lut: Res<Lut>) {
    // spawn sprite that will be animated
    commands
        .spawn_bundle(SpriteBundle {
            // material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
            transform: Transform::from_xyz(0.0, -0.0, 0.0),
            sprite: Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::new(10.0, 10.0)),
                ..Default::default()
            },

            ..Default::default()
        })
        // needed so that follow_path() can query the Sprite and animate it
        .insert(Animation);

    // show points from look-up table
    for position in lut.lut.iter() {
        commands.spawn_bundle(SpriteBundle {
            // material: materials.add(Color::rgb(0.7, 0.5, 1.0).into()),
            transform: Transform::from_translation(position.extend(-50.0)),
            // sprite: Sprite::new(Vec2::new(1.0, 1.0)),
            sprite: Sprite {
                color: Color::GREEN,
                custom_size: Some(Vec2::new(1.0, 1.0)),
                ..Default::default()
            },

            ..Default::default()
        });
    }
}

fn compute_position_with_lut(t: f32, lut: &Lut) -> Vec2 {
    let lut = lut.lut.clone();

    // indexing
    let idx_f64 = t * (lut.len() - 1) as f32;
    let p1 = lut[(idx_f64 as usize)];
    let p2 = lut[idx_f64 as usize + 1];
    let rem = idx_f64 % 1.0;

    // interpolation
    let position = p1 + rem * (p2 - p1); //interpolate_vec2(p1, p2, rem);
    return position;
}

fn follow_path(mut query: Query<(&mut Transform, &Animation)>, time: Res<Time>, lut: Res<Lut>) {
    let t_time = (time.seconds_since_startup() * 0.1) % 1.0;
    let pos = compute_position_with_lut(t_time as f32, lut.as_ref());

    for (mut transform, _bezier_animation) in query.iter_mut() {
        transform.translation = pos.extend(transform.translation.z);
    }
}
