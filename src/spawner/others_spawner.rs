use crate::util::{
    Bezier, Globals, Group, GroupBoxQuad, GroupMiddleQuad, MyShader, SelectionBoxQuad,
};

use bevy::{
    prelude::*,
    render::pipeline::{RenderPipeline, RenderPipelines},
};

pub fn spawn_selection_bounding_box(
    mut commands: Commands,
    // asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    // mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    // mut render_graph: ResMut<RenderGraph>,
    globals: ResMut<Globals>,
    mut my_shader_params: ResMut<Assets<MyShader>>,
    clearcolor_struct: Res<ClearColor>,
) {
    // Bounding Box for group
    let bb_group_size = Vec2::new(10.0, 10.0);
    let shader_params_handle_group_bb = my_shader_params.add(MyShader {
        color: Color::DARK_GRAY,
        t: 0.5,
        zoom: globals.camera_scale,
        size: bb_group_size,
        clearcolor: clearcolor_struct.0.clone(),
    });
    let visible_bb_group = Visible {
        is_visible: false,
        is_transparent: true,
    };
    let mesh_handle_bb_group = meshes.add(Mesh::from(shape::Quad {
        size: bb_group_size,
        flip: false,
    }));
    let bb_group_transform = Transform::from_translation(Vec3::new(0.0, 0.0, -455.0));
    let bb_group_pipeline_handle = globals.pipeline_handles["bounding_box"].clone();

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
        .insert(SelectionBoxQuad);
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
    // group: &Group,
) {
    // Bounding Box for group
    for group_handle in group_event_reader.iter() {
        let bb_group_size = Vec2::new(10.0, 10.0);

        let shader_params_handle_group_bb = my_shader_params.add(MyShader {
            color: Color::BLACK,
            t: 0.5,
            zoom: globals.camera_scale,
            size: bb_group_size,
            clearcolor: clearcolor_struct.0.clone(),
        });
        let visible_bb_group = Visible {
            is_visible: false,
            is_transparent: true,
        };
        let mesh_handle_bb_group = meshes.add(Mesh::from(shape::Quad {
            size: bb_group_size,
            flip: false,
        }));
        let bb_group_transform = Transform::from_translation(Vec3::new(0.0, 0.0, -455.0));
        let bb_group_pipeline_handle = globals.pipeline_handles["bounding_box"].clone();

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
    // mut group_event_reader: EventReader<Group>,
    mut group_event_reader: EventReader<Handle<Group>>,
) {
    for group_handle in group_event_reader.iter() {
        let visible = Visible {
            is_visible: true,
            is_transparent: true,
        };
        let middle_mesh_handle = globals.mesh_handles["middles"].clone();

        let pos_z = -1111.11;

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

        let ecm_pipeline_handle = globals.pipeline_handles["mids"].clone();
        let render_piplines =
            RenderPipelines::from_pipelines(vec![RenderPipeline::new(ecm_pipeline_handle)]);

        let mut z = 0.0;
        let mut x = -20.0;
        // let mut k = 0;

        for t in vrange {
            let pos = group.compute_position(&bezier_curves, t as f64);

            let mid_shader_params_handle = my_shader_params.add(MyShader {
                color,
                t: 0.5,
                zoom: globals.camera_scale,
                size: Vec2::new(1.0, 1.0),
                clearcolor: clearcolor_struct.0.clone(),
            });

            x = x + 2.0;
            z = z + 10.0;
            let child = commands
                // // left
                .spawn_bundle(MeshBundle {
                    mesh: middle_mesh_handle.clone(),
                    visible: visible.clone(),
                    render_pipelines: render_piplines.clone(),
                    transform: Transform::from_xyz(pos.x, pos.y, pos_z + 50.0 + z),
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
