use bevy::{prelude::*, render::camera::OrthographicProjection};

use serde::{Deserialize, Serialize};
use std::io::Read;

use lyon::math::{point, Point};

use lyon::path::Path;

use lyon::tessellation::geometry_builder::simple_builder;
use lyon::tessellation::{FillOptions, FillTessellator, VertexBuffers};

// look-up table
#[derive(Serialize, Deserialize)]
struct Lut {
    path_length: f32,
    lut: Vec<Vec2>,
}

struct Inds {
    indices: Vec<u32>,
}

impl Default for Inds {
    fn default() -> Self {
        Self {
            indices: Vec::new(),
        }
    }
}

struct K {
    k: u32,
}

impl Lut {
    // loads a look-up table that was saved in assets/lut using bevy_pen_tool
    fn load() -> Lut {
        let lut_path = "assets/lut/my_group_lut.txt";
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
        .insert_resource(Inds::default())
        .insert_resource(K { k: 0 })
        .add_startup_system(camera_setup)
        .add_startup_system(make_mesh)
        .add_startup_system(spawn_quad)
        .add_system(follow_path)
        .add_system(show_triangle)
        .run();
}

fn make_mesh(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut lut: ResMut<Lut>,
    mut inds: ResMut<Inds>,
) {
    // let lut = Lut::load();

    // println!("{:?}", lut.lut.len());

    let mut path_builder = Path::builder();

    let first = lut.lut.get(0).unwrap();
    path_builder.begin(point(first.x, first.y));
    let resto: Vec<Vec2> = lut.lut[1..].to_vec();

    for e in resto.iter() {
        path_builder.line_to(point(e.x, e.y));
    }

    path_builder.end(true);
    let path = path_builder.build();

    // Create the destination vertex and index buffers.
    let mut buffers: VertexBuffers<Point, u16> = VertexBuffers::new();

    {
        let mut vertex_builder = simple_builder(&mut buffers);

        // Create the tessellator.
        let mut tessellator = FillTessellator::new();

        // Compute the tessellation.
        let result =
            tessellator.tessellate_path(&path, &FillOptions::default(), &mut vertex_builder);
        assert!(result.is_ok());
    }

    let mut new_lut = Vec::new();
    let mut new_indices = Vec::new();
    // show points from look-up table
    for position in buffers.vertices[..].iter() {
        let v = Vec3::new(position.x, position.y, -100.0);
        // let v = Vec3::new(0.0, 0.0, -100.0);
        commands.spawn_bundle(SpriteBundle {
            material: materials.add(Color::rgb(0.9, 0.5, 0.8).into()),
            transform: Transform::from_translation(v),
            sprite: Sprite::new(Vec2::new(1.2, 1.2)),

            ..Default::default()
        });
        new_lut.push(Vec2::new(position.x, position.y));
    }

    for ind in buffers.indices[..].iter() {
        new_indices.push(ind.clone() as u32);
    }

    lut.lut = new_lut;
    inds.indices = new_indices;

    println!("The generated vertices are: {:?}.", &buffers.vertices[..]);
    println!("The generated indices are: {:?}.", &buffers.indices[..]);
}
#[derive(Component)]
struct Triangle;

fn show_triangle(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut k: ResMut<K>,
    lut: Res<Lut>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    inds: Res<Inds>,
    query: Query<Entity, With<Triangle>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for entity in query.iter() {
            commands.entity(entity).despawn();
        }

        k.k += 1;
        let kk = (k.k * 3) as usize;
        let kkk = ((k.k + 1) * 3) as usize;
        let indices = inds.indices[kk..kkk].to_vec();
        for ind in indices.iter() {
            let v = lut.lut.get(*ind as usize).unwrap().extend(-300.0);
            commands
                .spawn_bundle(SpriteBundle {
                    material: materials.add(Color::rgb(0.95, 0.86, 0.76).into()),
                    transform: Transform::from_translation(v),
                    sprite: Sprite::new(Vec2::new(2.0, 2.0)),

                    ..Default::default()
                })
                .insert(Triangle);
        }
    }
}

fn camera_setup(mut commands: Commands) {
    //
    // bevy_pen_tool is not compatible with Perspective Cameras
    commands.spawn_bundle(OrthographicCameraBundle {
        transform: Transform::from_translation(Vec3::new(00.0, 0.0, 10.0))
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        orthographic_projection: OrthographicProjection {
            scale: 0.20,
            far: 100000.0,
            near: -100000.0,
            ..Default::default()
        },
        ..OrthographicCameraBundle::new_2d()
    });
}

#[derive(Component)]
struct Animation;

fn spawn_quad(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>, lut: Res<Lut>) {
    // spawn sprite that will be animated
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
            transform: Transform::from_xyz(0.0, -0.0, 0.0),
            sprite: Sprite::new(Vec2::new(10.0, 10.0)),

            ..Default::default()
        })
        // needed so that follow_path() can query the Sprite and animate it
        .insert(Animation);

    // show points from look-up table
    for position in lut.lut.iter() {
        commands.spawn_bundle(SpriteBundle {
            material: materials.add(Color::rgb(0.7, 0.5, 1.0).into()),
            transform: Transform::from_translation(position.extend(-50.0)),
            sprite: Sprite::new(Vec2::new(1.0, 1.0)),

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
