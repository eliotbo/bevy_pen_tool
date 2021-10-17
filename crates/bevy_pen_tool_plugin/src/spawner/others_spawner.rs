use crate::util::{
    Bezier, FollowBezierAnimation, Globals, Group, GroupBoxQuad, GroupMiddleQuad, Maps, MyShader,
    SelectedBoxQuad, SelectingBoxQuad, TurnRoundAnimation,
};

use crate::inputs::Action;

use bevy::{
    prelude::*,
    render::pipeline::{RenderPipeline, RenderPipelines},
};

// There is culling between two transparent quads at the same distance from the camera.
// Is this normal behavior?
// To avoid culling, the quads that can intercept each other in the xy plane need
// to have different z-values
//
//
///////////////////////////////////////////// z positions
// spawn_group_middle_quads:        -1000.0
// car:  -720.0
// helicopter: -715.0
// heli rotor blades: -710.0
// spawn_group_bounding_box:        0.0
// spawn_selecting_bounding_box:    5.0
// spawn_selection_bounding_box:    -10.0

// button_ui: -550.0
// color_ui:  -500.0
// buttons:   -400.0
// icon:        10.1
// icon2:       20.1

// pos_z in bezier_spawner: -5110.0 to -1110.0
// bezier_bounding_box: -20.0
// start anchor: 30.0 + pos_z
// end anchor: 40.0 + pos_z
// ctrl start: 50 + pos_z
// ctrl end: 60 + pos_z
// middle quads: 110 + pos_z + 10 per quad

// road: -725.0
// light: -700.0
// mesh: -730.0
// helicopter: -715.0
// heli rotor blades: -710.0
// car:  -720.0
///////////////////////////////////////////// z positions

pub fn spawn_selection_bounding_box(
    mut commands: Commands,
    // asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    // mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    // mut render_graph: ResMut<RenderGraph>,
    globals: ResMut<Globals>,
    maps: ResMut<Maps>,
    mut my_shader_params: ResMut<Assets<MyShader>>,
    clearcolor_struct: Res<ClearColor>,
) {
    // Bounding Box for group
    let bb_group_size = Vec2::new(10.0, 10.0);
    let shader_params_handle_group_bb = my_shader_params.add(MyShader {
        color: Color::DARK_GRAY,
        t: 0.5,
        zoom: 0.15 / globals.scale,
        size: bb_group_size / (globals.scale / 0.15),
        clearcolor: clearcolor_struct.0.clone(),
        ..Default::default()
    });
    let visible_bb_group = Visible {
        is_visible: false,
        is_transparent: true,
    };
    let mesh_handle_bb_group = meshes.add(Mesh::from(shape::Quad {
        size: bb_group_size,
        flip: false,
    }));
    let bb_group_transform = Transform::from_translation(Vec3::new(0.0, 0.0, -10.0));
    let bb_group_pipeline_handle = maps.pipeline_handles["bounding_box"].clone();

    commands
        .spawn_bundle(MeshBundle {
            mesh: mesh_handle_bb_group,
            visible: visible_bb_group,
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                bb_group_pipeline_handle,
            )]),
            transform: bb_group_transform,
            ..Default::default()
        })
        .insert(shader_params_handle_group_bb)
        .insert(SelectedBoxQuad);
}

pub fn spawn_selecting_bounding_box(
    mut commands: Commands,
    // asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    // mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    // mut render_graph: ResMut<RenderGraph>,
    globals: ResMut<Globals>,
    maps: ResMut<Maps>,
    mut my_shader_params: ResMut<Assets<MyShader>>,
    clearcolor_struct: Res<ClearColor>,
) {
    // Bounding Box for group
    let bb_group_size = Vec2::new(100.0, 100.0);
    let shader_params_handle_group_bb = my_shader_params.add(MyShader {
        color: Color::DARK_GRAY,
        t: 0.5,
        zoom: 0.15 / globals.scale,
        size: bb_group_size / (globals.scale / 0.15),
        clearcolor: clearcolor_struct.0.clone(),
        ..Default::default()
    });
    let visible_bb_group = Visible {
        is_visible: false,
        is_transparent: true,
    };
    let mesh_handle_bb_group = meshes.add(Mesh::from(shape::Quad {
        size: bb_group_size,
        flip: false,
    }));
    let bb_group_transform = Transform::from_translation(Vec3::new(0.0, 0.0, 5.0));
    let bb_group_pipeline_handle = maps.pipeline_handles["selecting"].clone();

    commands
        .spawn_bundle(MeshBundle {
            mesh: mesh_handle_bb_group,
            visible: visible_bb_group,
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                bb_group_pipeline_handle,
            )]),
            transform: bb_group_transform,
            ..Default::default()
        })
        .insert(shader_params_handle_group_bb)
        .insert(SelectingBoxQuad);
}

pub fn spawn_group_bounding_box(
    mut commands: Commands,
    // asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    // mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    // mut render_graph: ResMut<RenderGraph>,
    globals: ResMut<Globals>,
    mut my_shader_params: ResMut<Assets<MyShader>>,
    clearcolor_struct: Res<ClearColor>,
    mut group_event_reader: EventReader<Handle<Group>>,
    maps: ResMut<Maps>,
    // group: &Group,
) {
    // Bounding Box for group
    for group_handle in group_event_reader.iter() {
        let bb_group_size = Vec2::new(10.0, 10.0);

        let shader_params_handle_group_bb = my_shader_params.add(MyShader {
            color: Color::BLACK,
            t: 0.5,
            zoom: 0.15 / globals.scale,
            size: bb_group_size / (globals.scale / 0.15),
            clearcolor: clearcolor_struct.0.clone(),
            ..Default::default()
        });
        let visible_bb_group = Visible {
            is_visible: false,
            is_transparent: true,
        };
        let mesh_handle_bb_group = meshes.add(Mesh::from(shape::Quad {
            size: bb_group_size,
            flip: false,
        }));
        let bb_group_transform = Transform::from_translation(Vec3::new(0.0, 0.0, 0.0));
        let bb_group_pipeline_handle = maps.pipeline_handles["bounding_box"].clone();

        commands
            .spawn_bundle(MeshBundle {
                mesh: mesh_handle_bb_group,
                visible: visible_bb_group,
                render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                    bb_group_pipeline_handle,
                )]),
                transform: bb_group_transform,
                ..Default::default()
            })
            .insert(shader_params_handle_group_bb)
            .insert(GroupBoxQuad)
            .insert(group_handle.clone());
    }
}

pub fn spawn_group_middle_quads(
    mut commands: Commands,
    bezier_curves: ResMut<Assets<Bezier>>,
    globals: ResMut<Globals>,
    mut my_shader_params: ResMut<Assets<MyShader>>,
    clearcolor_struct: Res<ClearColor>,
    // group_handle: Handle<Group>,
    groups: ResMut<Assets<Group>>,
    maps: ResMut<Maps>,
    // mut group_event_reader: EventReader<Group>,
    mut group_event_reader: EventReader<Handle<Group>>,
) {
    for group_handle in group_event_reader.iter() {
        let visible = Visible {
            is_visible: true,
            is_transparent: true,
        };
        let middle_mesh_handle = maps.mesh_handles["middles"].clone();

        let pos_z = -1000.0;

        let num_mid_quads = 50;

        let group = groups.get(group_handle.clone()).unwrap();

        let (parent, _handle) = group.group.iter().next().unwrap();

        let first_bezier_handle = group.handles.iter().next().unwrap();
        let first_bezier = bezier_curves.get(first_bezier_handle).unwrap();

        let color = first_bezier.color.unwrap(); //Color::hex("2e003e").unwrap();
        let vrange: Vec<f32> = (0..num_mid_quads)
            .map(|x| (x as f32) / (num_mid_quads as f32 - 1.0) - 0.0000001)
            .collect();
        // println!("total length: {:?}", vrange);

        let ecm_pipeline_handle = maps.pipeline_handles["mids"].clone();
        let render_piplines =
            RenderPipelines::from_pipelines(vec![RenderPipeline::new(ecm_pipeline_handle)]);

        let mut z = 0.0;
        let mut x = -20.0;
        // let mut k = 0;

        for t in vrange {
            // let pos = group.compute_position_with_bezier(&bezier_curves, t as f64);
            let pos = group.compute_position_with_lut(t as f32);

            let mid_shader_params_handle = my_shader_params.add(MyShader {
                color,
                t: 0.5,
                zoom: 0.15 / globals.scale,
                size: Vec2::new(1.0, 1.0),
                clearcolor: clearcolor_struct.0.clone(),
                ..Default::default()
            });

            x = x + 2.0;
            z = z + 5.0;
            let child = commands
                // // left
                .spawn_bundle(MeshBundle {
                    mesh: middle_mesh_handle.clone(),
                    visible: visible.clone(),
                    render_pipelines: render_piplines.clone(),
                    transform: Transform::from_xyz(pos.x, pos.y, pos_z),
                    ..Default::default()
                })
                .insert(GroupMiddleQuad(num_mid_quads))
                .insert(mid_shader_params_handle.clone())
                .insert(group_handle.clone())
                .id();

            commands.entity(*parent).push_children(&[child]);
        }
    }
}

pub fn spawn_heli(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut action_event_reader: EventReader<Action>,
    groups: Res<Assets<Group>>,
) {
    if action_event_reader.iter().any(|x| x == &Action::SpawnHeli) {
        if let Some(_) = groups.iter().next() {
            // let rotation = Quat::IDENTITY;
            let rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)
                .mul_quat(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));

            commands
                .spawn_bundle((
                    Transform {
                        translation: Vec3::new(0.0, 0.0, -720.0),
                        rotation,
                        scale: Vec3::splat(3.0),
                        ..Default::default()
                    },
                    GlobalTransform::identity(),
                ))
                .insert(FollowBezierAnimation {
                    animation_offset: 0.0,
                    initial_direction: Vec3::Z,
                })
                // .insert(TurnRoundAnimation)
                .with_children(|cell| {
                    cell.spawn_scene(asset_server.load("models/car.gltf#Scene0"));
                })
                .id();

            let heli_handle = asset_server.load("textures/heli.png");
            let size = Vec2::new(25.0, 25.0);
            let heli_sprite = commands
                .spawn_bundle(SpriteBundle {
                    material: materials.add(heli_handle.into()),
                    // mesh: mesh_handle_button.clone(),
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, -710.0)),
                    sprite: Sprite::new(size),
                    visible: Visible {
                        is_visible: true,
                        is_transparent: true,
                    },
                    ..Default::default()
                })
                .insert(FollowBezierAnimation {
                    animation_offset: -0.1,
                    initial_direction: Vec3::X,
                })
                .id();
            let copter_handle = asset_server.load("textures/copter.png");
            let copter_sprite = commands
                .spawn_bundle(SpriteBundle {
                    material: materials.add(copter_handle.into()),
                    // mesh: mesh_handle_button.clone(),
                    transform: Transform::from_translation(Vec3::new(3.0, 1.0, 5.0)),
                    sprite: Sprite::new(size),
                    visible: Visible {
                        is_visible: true,
                        is_transparent: true,
                    },
                    ..Default::default()
                })
                .insert(TurnRoundAnimation)
                .id();

            commands.entity(heli_sprite).push_children(&[copter_sprite]);

            // light
            commands
                .spawn_bundle(PointLightBundle {
                    transform: Transform::from_translation(Vec3::new(0.0, 25.0, -700.0)),
                    point_light: PointLight {
                        intensity: 50000.,
                        range: 1000.,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(FollowBezierAnimation {
                    animation_offset: 0.0,
                    initial_direction: Vec3::X,
                });
        }
    }
}
