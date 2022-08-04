use crate::util::{
    Bezier, BezierMidMat, FollowBezierAnimation, Globals, Group, GroupBoxQuad, GroupMiddleQuad,
    GroupParent, Maps, SelectedBoxQuad, SelectingBoxQuad, SelectingMat, SelectionMat,
    TurnRoundAnimation,
};

use crate::inputs::Action;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

pub fn spawn_selection_bounding_box(
    mut commands: Commands,
    // asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    // mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    // mut render_graph: ResMut<RenderGraph>,
    globals: ResMut<Globals>,
    // query: Query<(Entity, &BezierParent)>,
    // maps: ResMut<Maps>,
    mut my_shader_params: ResMut<Assets<SelectionMat>>,
    clearcolor_struct: Res<ClearColor>,
) {
    // Bounding Box for group
    let bb_group_size = Vec2::new(10.0, 10.0);
    let shader_params_handle_group_bb = my_shader_params.add(SelectionMat {
        color: Color::ALICE_BLUE.into(),
        t: 0.5,
        zoom: 0.15 / globals.scale,
        size: bb_group_size / (globals.scale / 0.15),
        clearcolor: clearcolor_struct.0.clone().into(),
        ..Default::default()
    });
    let visible_bb_group = Visibility { is_visible: false };
    // let mesh_handle_bb_group = meshes.add(Mesh::from(shape::Quad {
    //     size: bb_group_size,
    //     flip: false,
    // }));
    let mesh_handle_bb_group =
        bevy::sprite::Mesh2dHandle(meshes.add(Mesh::from(shape::Quad::new(bb_group_size))));

    let bb_group_transform =
        Transform::from_translation(Vec3::new(0.0, 0.0, globals.z_pos.selection_box));

    // let bb_group_pipeline_handle = maps.pipeline_handles["bounding_box"].clone();

    let _hild = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle_bb_group,
            visibility: visible_bb_group,
            // render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
            //     bb_group_pipeline_handle,
            // )]),
            transform: bb_group_transform,
            material: shader_params_handle_group_bb,
            ..Default::default()
        })
        // .insert(shader_params_handle_group_bb)
        .insert(SelectedBoxQuad)
        .id();

    // let
    // println!("spawn_selection_bounding_box: child: {:?}", child);
}

pub fn spawn_selecting_bounding_box(
    mut commands: Commands,
    // asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    // mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    // mut render_graph: ResMut<RenderGraph>,
    globals: ResMut<Globals>,
    // maps: ResMut<Maps>,
    mut my_shader_params: ResMut<Assets<SelectingMat>>,
    clearcolor_struct: Res<ClearColor>,
) {
    // Bounding Box for group
    let bb_group_size = Vec2::new(100.0, 100.0);
    let shader_params_handle_group_bb = my_shader_params.add(SelectingMat {
        color: Color::DARK_GRAY.into(),
        t: 0.5,
        zoom: 0.15 / globals.scale,
        size: bb_group_size / (globals.scale / 0.15),
        clearcolor: clearcolor_struct.0.clone().into(),
        ..Default::default()
    });
    let visible_bb_group = Visibility { is_visible: false };
    // let mesh_handle_bb_group = meshes.add(Mesh::from(shape::Quad {
    //     size: bb_group_size,
    //     flip: false,
    // }));
    let mesh_handle_bb_group =
        bevy::sprite::Mesh2dHandle(meshes.add(Mesh::from(shape::Quad::new(bb_group_size))));
    let bb_group_transform =
        Transform::from_translation(Vec3::new(0.0, 0.0, globals.z_pos.selecting_box));
    // let bb_group_pipeline_handle = maps.pipeline_handles["selecting"].clone();

    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle_bb_group,
            visibility: visible_bb_group,
            // render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
            //     bb_group_pipeline_handle,
            // )]),
            transform: bb_group_transform,
            material: shader_params_handle_group_bb,
            ..Default::default()
        })
        // .insert(shader_params_handle_group_bb)
        .insert(SelectingBoxQuad);
}

pub fn spawn_group_entities(
    mut commands: Commands,
    // asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    // mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    // mut render_graph: ResMut<RenderGraph>,
    globals: ResMut<Globals>,
    mut my_shader_params: ResMut<Assets<SelectionMat>>,
    mut mids_shader_params: ResMut<Assets<BezierMidMat>>,
    clearcolor_struct: Res<ClearColor>,
    mut group_event_reader: EventReader<Handle<Group>>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    maps: ResMut<Maps>,
    mut groups: ResMut<Assets<Group>>,
    // maps: ResMut<Maps>,
    // group: &Group,
) {
    // Bounding Box for group
    for group_handle in group_event_reader.iter() {
        let bb_group_size = Vec2::new(10.0, 10.0);

        let shader_params_handle_group_bb = my_shader_params.add(SelectionMat {
            color: Color::BLACK.into(),
            t: 0.5,
            zoom: 0.15 / globals.scale,
            size: bb_group_size / (globals.scale / 0.15),
            clearcolor: clearcolor_struct.0.clone().into(),
            ..Default::default()
        });

        // let mesh_handle_bb_group = meshes.add(Mesh::from(shape::Quad {
        //     size: bb_group_size,
        //     flip: false,
        // }));

        let mut init_pos = Transform::default();
        init_pos.scale = Vec3::new(globals.scale, globals.scale, 1.0);

        let group_parent_transform =
            GlobalTransform::from_translation(Vec3::new(0.0, 0.0, globals.z_pos.group_parent));

        // let bb_group_pipeline_handle = maps.pipeline_handles["bounding_box"].clone();

        /////////    group bounding box (also the Parent)  ///////////////////

        let parent_id = commands
            .spawn_bundle((
                GroupParent,
                init_pos,
                group_parent_transform.clone(),
                Visibility { is_visible: true }, // visibility is inherited by all children
                group_handle.clone(),
                ComputedVisibility::not_visible(), // the parent entity is not a rendered object
            ))
            .id();

        let visible_bb_group = Visibility { is_visible: false };
        let mesh_handle_bb_group =
            bevy::sprite::Mesh2dHandle(meshes.add(Mesh::from(shape::Quad::new(bb_group_size))));
        let bb_group_transform =
            Transform::from_translation(Vec3::new(0.0, 0.0, globals.z_pos.group_bouding_box));
        let bb_id = commands
            .spawn_bundle(MaterialMesh2dBundle {
                mesh: mesh_handle_bb_group,
                visibility: visible_bb_group,
                // render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                //     bb_group_pipeline_handle,
                // )]),
                transform: bb_group_transform,
                material: shader_params_handle_group_bb,
                ..Default::default()
            })
            // .insert(shader_params_handle_group_bb)
            .insert(GroupBoxQuad)
            // .insert(GroupParent)
            .insert(group_handle.clone())
            .id();

        commands.entity(parent_id).push_children(&[bb_id]);
        /////////    group bounding box (also the Parent)  ///////////////////

        let visible = Visibility { is_visible: true };
        let middle_mesh_handle = maps.mesh_handles["middles"].clone();

        let num_mid_quads = 50;

        let group = groups.get_mut(&group_handle.clone()).unwrap();
        group.group_lut(&mut bezier_curves, maps.id_handle_map.clone());

        // let (parent, _handle) = group.group.iter().next().unwrap();

        let first_bezier_handle = group.bezier_handles.iter().next().unwrap();
        let first_bezier = bezier_curves.get(first_bezier_handle).unwrap();

        let color = first_bezier.color.unwrap(); //Color::hex("2e003e").unwrap();
        let vrange: Vec<f32> = (0..num_mid_quads)
            .map(|x| (x as f32) / (num_mid_quads as f32 - 1.0) - 0.0000001)
            .collect();

        let mut z = 0.0;
        let mut x = -20.0;
        // let mut k = 0;

        /////////    group middle quads  ///////////////////
        for t in vrange {
            // let pos = group.compute_position_with_bezier(&bezier_curves, t as f64);
            let pos = group.compute_position_with_lut(t as f32);

            let mid_shader_params_handle = mids_shader_params.add(BezierMidMat {
                color: color.into(),
                t: 0.5,
                zoom: 0.15 / globals.scale,
                size: Vec2::new(1.0, 1.0),
                clearcolor: clearcolor_struct.0.clone().into(),
                ..Default::default()
            });

            x = x + 2.0;
            z = z + 5.0;
            let child = commands
                // // left
                .spawn_bundle(MaterialMesh2dBundle {
                    mesh: middle_mesh_handle.clone(),
                    visibility: visible.clone(),
                    // render_pipelines: render_piplines.clone(),
                    transform: Transform::from_xyz(pos.x, pos.y, globals.z_pos.group_middles),
                    material: mid_shader_params_handle,
                    ..Default::default()
                })
                .insert(GroupMiddleQuad(num_mid_quads))
                // .insert(mid_shader_params_handle.clone())
                .insert(group_handle.clone())
                .id();

            commands.entity(parent_id).push_children(&[child]);
        }
    }
}

// pub fn spawn_group_middle_quads(
//     mut commands: Commands,
//     bezier_curves: ResMut<Assets<Bezier>>,
//     globals: ResMut<Globals>,
//     mut my_shader_params: ResMut<Assets<BezierMidMat>>,
//     clearcolor_struct: Res<ClearColor>,
//     // group_handle: Handle<Group>,
//     groups: ResMut<Assets<Group>>,
//     maps: ResMut<Maps>,
//     // mut group_event_reader: EventReader<Group>,
//     mut group_event_reader: EventReader<Handle<Group>>,
// ) {
//     for group_handle in group_event_reader.iter() {
//         let visible = Visibility { is_visible: true };
//         let middle_mesh_handle = maps.mesh_handles["middles"].clone();

//         let num_mid_quads = 50;

//         let group = groups.get(&group_handle.clone()).unwrap();

//         let (parent, _handle) = group.group.iter().next().unwrap();

//         let first_bezier_handle = group.bezier_handles.iter().next().unwrap();
//         let first_bezier = bezier_curves.get(first_bezier_handle).unwrap();

//         let color = first_bezier.color.unwrap(); //Color::hex("2e003e").unwrap();
//         let vrange: Vec<f32> = (0..num_mid_quads)
//             .map(|x| (x as f32) / (num_mid_quads as f32 - 1.0) - 0.0000001)
//             .collect();

//         let mut z = 0.0;
//         let mut x = -20.0;
//         // let mut k = 0;

//         for t in vrange {
//             // let pos = group.compute_position_with_bezier(&bezier_curves, t as f64);
//             let pos = group.compute_position_with_lut(t as f32);

//             let mid_shader_params_handle = my_shader_params.add(BezierMidMat {
//                 color: color.into(),
//                 t: 0.5,
//                 zoom: 0.15 / globals.scale,
//                 size: Vec2::new(1.0, 1.0),
//                 clearcolor: clearcolor_struct.0.clone().into(),
//                 ..Default::default()
//             });

//             x = x + 2.0;
//             z = z + 5.0;
//             let child = commands
//                 // // left
//                 .spawn_bundle(MaterialMesh2dBundle {
//                     mesh: middle_mesh_handle.clone(),
//                     visibility: visible.clone(),
//                     // render_pipelines: render_piplines.clone(),
//                     transform: Transform::from_xyz(pos.x, pos.y, globals.z_pos.group_middles),
//                     material: mid_shader_params_handle,
//                     ..Default::default()
//                 })
//                 .insert(GroupMiddleQuad(num_mid_quads))
//                 // .insert(mid_shader_params_handle.clone())
//                 .insert(group_handle.clone())
//                 .id();

//             commands.entity(*parent).push_children(&[child]);
//         }
//     }
// }

pub fn spawn_heli(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    globals: ResMut<Globals>,
    // mut materials: ResMut<Assets<ColorMaterial>>,
    mut action_event_reader: EventReader<Action>,
    groups: Res<Assets<Group>>,
) {
    if action_event_reader.iter().any(|x| x == &Action::SpawnHeli) {
        if let Some(_) = groups.iter().next() {
            // let rotation = Quat::IDENTITY;
            let _rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)
                .mul_quat(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));

            let heli_handle = asset_server.load("textures/heli.png");
            let size = Vec2::new(125.0, 125.0);
            let heli_sprite = commands
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(size),
                        ..Default::default()
                    },
                    texture: heli_handle,
                    // mesh: mesh_handle_button.clone(),
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, globals.z_pos.heli)),
                    // sprite: Sprite::new(size),
                    visibility: Visibility { is_visible: true },
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
                    texture: copter_handle,
                    sprite: Sprite {
                        custom_size: Some(size),
                        ..Default::default()
                    },
                    // mesh: mesh_handle_button.clone(),
                    transform: Transform::from_translation(Vec3::new(
                        3.0,
                        1.0,
                        globals.z_pos.heli_top,
                    )),
                    // sprite: Sprite::new(size),
                    visibility: Visibility { is_visible: true },
                    ..Default::default()
                })
                .insert(TurnRoundAnimation)
                .id();

            commands.entity(heli_sprite).push_children(&[copter_sprite]);
        }
    }
}
