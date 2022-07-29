use crate::inputs::{Cursor, Latch};
use crate::util::{
    Anchor, AnchorEdge, Bezier, BezierControlsMat, BezierEndsMat, BezierGrandParent, BezierMidMat,
    BezierParent, BezierPositions, BoundingBoxQuad, ControlPointQuad, EndpointQuad, Globals,
    LatchData, Maps, MiddlePointQuad, SelectionMat, UserState,
};

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use std::collections::HashMap;

use rand::prelude::*;

// TODO: merge spawn_bezier_system and spawn_bezier
pub fn spawn_bezier_system(
    mut bezier_curves: ResMut<Assets<Bezier>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut cursor: ResMut<Cursor>,
    // mut my_shader_params: ResMut<Assets<BezierMat>>,
    mut selection_params: ResMut<Assets<SelectionMat>>,
    mut controls_params: ResMut<Assets<BezierControlsMat>>,
    mut ends_params: ResMut<Assets<BezierEndsMat>>,
    mut mid_params: ResMut<Assets<BezierMidMat>>,
    clearcolor_struct: Res<ClearColor>,
    mut globals: ResMut<Globals>,
    mut maps: ResMut<Maps>,
    mut latch_event_reader: EventReader<Latch>,
    mut user_state: ResMut<UserState>,
) {
    if user_state.as_ref() == &UserState::SpawningCurve {
        //
        let us = user_state.as_mut();
        *us = UserState::MovingAnchor;

        let clearcolor = clearcolor_struct.0;

        let mut rng = thread_rng();
        let mut spawner_id: u128 = rng.gen();

        let mut start = cursor.position;

        // the control points cannot be exactly in the same positions as the anchors
        // because the algorithm for finding position along the curves fail in that case
        let mut epsilon = 5.01;
        if globals.hide_control_points {
            epsilon = 0.01;
        }

        let mut control_start: Vec2 = cursor.position + Vec2::new(epsilon, epsilon);
        let control_end: Vec2 = cursor.position + Vec2::new(epsilon, epsilon);

        let mut latches = HashMap::new();
        latches.insert(AnchorEdge::Start, Vec::new());
        latches.insert(AnchorEdge::End, Vec::new());

        for latch_received in latch_event_reader.iter() {
            //
            start = latch_received.position;
            control_start = latch_received.control_point;
            spawner_id = latch_received.latcher_id;

            if let Some(latch_local) = latches.get_mut(&AnchorEdge::Start) {
                *latch_local = vec![LatchData {
                    latched_to_id: latch_received.latchee_id,
                    self_edge: AnchorEdge::Start,
                    partners_edge: latch_received.latchee_edge,
                }];
            }
        }

        cursor.latch = Vec::new();

        let mut bezier = Bezier {
            positions: BezierPositions {
                start,
                end: cursor.position,
                control_start,
                control_end,
            },
            previous_positions: BezierPositions::default(),
            move_quad: Anchor::End,
            id: spawner_id,
            latches,
            ..Default::default()
        };
        bezier.update_previous_pos();

        spawn_bezier(
            &mut bezier,
            &mut bezier_curves,
            &mut commands,
            &mut meshes,
            // &mut my_shader_params,
            &mut selection_params,
            &mut controls_params,
            &mut ends_params,
            &mut mid_params,
            clearcolor,
            &mut globals,
            &mut maps,
        );
    }
}

pub fn spawn_bezier(
    mut bezier: &mut Bezier,
    bezier_curves: &mut ResMut<Assets<Bezier>>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    selection_params: &mut ResMut<Assets<SelectionMat>>,
    controls_params: &mut ResMut<Assets<BezierControlsMat>>,
    ends_params: &mut ResMut<Assets<BezierEndsMat>>,
    mid_params: &mut ResMut<Assets<BezierMidMat>>,
    clearcolor: Color,
    globals: &mut ResMut<Globals>,
    maps: &mut ResMut<Maps>,
) -> (Entity, Handle<Bezier>) {
    bezier.compute_lut_walk(100);

    let ends_controls_mesh_handle = maps.mesh_handles["ends_controls"].clone();
    let ends_mesh_handle = maps.mesh_handles["ends"].clone();
    let middle_mesh_handle = maps.mesh_handles["middles"].clone();

    let num_mid_quads = globals.num_points_on_curve;

    let mut color = Color::hex("3CB44B").unwrap();

    if let Some(color_in_params) = bezier.color {
        color = color_in_params;
    } else if let Some(color_in_globals) = globals.picked_color {
        color = color_in_globals;
    }
    bezier.color = Some(color);

    //////////////////// Bounding box parameters ////////////////////
    // need to import the whole library in order to use the .min() and .max() methods: file issue?
    use flo_curves::*;
    let curve0 = bezier.to_curve();
    let bb: Bounds<Coord2> = curve0.bounding_box();

    let Coord2(ax, ay) = bb.min();
    let Coord2(bx, by) = bb.max();

    let bound0 = Vec2::new(ax as f32, ay as f32);
    let bound1 = Vec2::new(bx as f32, by as f32);

    let qq = 0.0;
    let bigger_size = (bound1 - bound0) + Vec2::new(qq, qq);
    let bb_size = bound1 - bound0;
    let bb_pos = (bound1 + bound0) / 2.0;

    let mut start_pt_pos = bezier.positions.start - bb_pos;
    let mut end_pt_pos = bezier.positions.end - bb_pos;
    let ctr0_pos = bezier.positions.control_start; // - bb_pos;
    let ctr1_pos = bezier.positions.control_end - bb_pos;

    // let mesh_handle_bb = meshes.add(Mesh::from(shape::Quad {
    //     size: bigger_size,
    //     flip: false,
    // }));

    let mesh_handle_bb =
        bevy::sprite::Mesh2dHandle(meshes.add(Mesh::from(shape::Quad::new(bigger_size))));

    // since bezier is cloned, be careful about modifying it after the cloning, it won't have any side-effects
    let bezier_handle = bezier_curves.add(bezier.clone());

    maps.id_handle_map.insert(bezier.id, bezier_handle.clone());
    //////////////////// Bounding box ////////////////////

    let visible_bb = Visibility {
        // is_visible: !globals.do_hide_bounding_boxes,
        is_visible: false,
    };

    let visible_anchors = Visibility {
        is_visible: !globals.do_hide_anchors,
    };

    let shader_params_handle_bb = selection_params.add(SelectionMat {
        color: color.into(),
        t: 0.5,
        zoom: 0.15 / globals.scale,
        size: bb_size,
        clearcolor: clearcolor.clone().into(),
        ..Default::default()
    });

    // TODO: make the depth deterministic
    let mut rng = thread_rng();
    let pos_z = -rng.gen::<f32>() * 5000.0 - 1110.0;
    // let mut init_pos = Transform::from_translation(bb_pos.extend(-20.0));
    let global_init_pos = GlobalTransform::from_translation(bb_pos.extend(-20.0));
    let mut init_pos = Transform::default();

    init_pos.scale = Vec3::new(globals.scale, globals.scale, 1.0);

    // This is the parent of every entity belonging to a rendered bezier curve.
    let parent = commands
        .spawn_bundle((
            BezierParent,
            BezierGrandParent,
            init_pos.clone(),
            Visibility { is_visible: true }, // visibility is inherited by all children
            global_init_pos,
            bezier_handle.clone(),
            ComputedVisibility::not_visible(), // the parent entity is not a rendered object
        ))
        .id();

    let bbquad_entity = commands
        // let parent = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle_bb,
            visibility: visible_bb,
            transform: Transform::from_translation(Vec2::ZERO.extend(-0.01)),
            material: shader_params_handle_bb,
            ..Default::default()
        })
        .insert(BoundingBoxQuad)
        .insert(bezier_handle.clone())
        .id();

    commands.entity(parent).push_children(&[bbquad_entity]);

    //////////////////// Bounding box ////////////////////

    // let render_piplines_ends =
    //     RenderPipelines::from_pipelines(vec![RenderPipeline::new(ends_pipeline_handle)]);

    // let render_piplines =
    //     RenderPipelines::from_pipelines(vec![RenderPipeline::new(ecm_pipeline_handle)]);

    // Although the interface is two-dimensional, the z position of the quads is important for transparency

    let ((start_displacement, end_displacement), (start_rotation, end_rotation)) =
        bezier.ends_displacement(globals.scale);

    start_pt_pos += start_displacement;
    end_pt_pos += end_displacement;

    let mut start_pt_transform = Transform::from_translation(start_pt_pos.extend(pos_z + 30.0));
    let mut end_pt_transform = Transform::from_translation(end_pt_pos.extend(pos_z + 40.0));

    start_pt_transform.rotation = start_rotation;
    end_pt_transform.rotation = end_rotation;

    let ends_params_handle = ends_params.add(BezierEndsMat {
        color: color.into(),
        t: 0.5,
        zoom: 0.15 / globals.scale,
        size: bb_size,
        clearcolor: clearcolor.clone().into(),
        ..Default::default()
    });

    let child_start = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: ends_mesh_handle.clone(),
            visibility: visible_anchors.clone(),
            // render_pipelines: render_piplines_ends.clone(),
            transform: start_pt_transform,
            material: ends_params_handle.clone(),
            ..Default::default()
        })
        .insert(EndpointQuad(AnchorEdge::Start))
        .insert(bezier_handle.clone())
        // .insert(shader_params_handle_bb.clone())
        .id();

    commands.entity(parent).push_children(&[child_start]);

    let child_end = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: ends_mesh_handle.clone(),
            visibility: visible_anchors.clone(),
            transform: end_pt_transform,
            material: ends_params_handle.clone(),
            // render_pipelines: render_piplines_ends.clone(),
            // RenderPipelines::from_pipelines(vec![RenderPipeline::new(
            //     pipeline_handle.clone(),
            // )]),
            ..Default::default()
        })
        .insert(EndpointQuad(AnchorEdge::End))
        .insert(bezier_handle.clone())
        // .insert(shader_params_handle_bb.clone())
        .id();

    commands.entity(parent).push_children(&[child_end]);

    // let ctrl_render_piplines =
    //     RenderPipelines::from_pipelines(vec![RenderPipeline::new(ctrl_pipeline_handle)]);

    let mut visible_ctrl = Visibility {
        is_visible: true,
        // is_transparent: true,
    };
    if globals.hide_control_points {
        visible_ctrl.is_visible = true;
    };

    // control points
    for k in 0..2 {
        let z_ctr = pos_z + 50.0 + (k as f32) * 10.0;
        let mut ctr_pos_transform = Transform::from_translation(ctr0_pos.extend(z_ctr));

        let mut point = AnchorEdge::Start;
        if k == 1 {
            point = AnchorEdge::End;
            ctr_pos_transform = Transform::from_translation(ctr1_pos.extend(z_ctr));
        }

        let controls_params_handle = controls_params.add(BezierControlsMat {
            color: color.into(),
            t: 0.5,
            zoom: 0.15 / globals.scale,
            size: Vec2::new(1.0, 1.0) * globals.scale,
            clearcolor: clearcolor.clone().into(),
            ..Default::default()
        });

        let child = commands
            .spawn_bundle(MaterialMesh2dBundle {
                mesh: ends_controls_mesh_handle.clone(),
                visibility: visible_ctrl.clone(),
                // render_pipelines: ctrl_render_piplines.clone(),
                transform: ctr_pos_transform,
                material: controls_params_handle.clone(),
                ..Default::default()
            })
            .insert(ControlPointQuad(point))
            .insert(bezier_handle.clone())
            // .insert(shader_params_handle_bb.clone())
            .id();

        commands.entity(parent).push_children(&[child]);

        // if k == 0 {
        //     commands.entity(child_start).push_children(&[child]);
        // } else {
        //     commands.entity(child_end).push_children(&[child]);
        // }
    }

    //////////////////// Small moving rings aka middle quads ////////////////////
    let visible = Visibility { is_visible: true };

    let vrange: Vec<f32> = (0..num_mid_quads)
        .map(|x| ((x as f32) / (num_mid_quads as f32 - 1.0) - 0.5) * 2.0 * 50.0)
        .collect();

    let mut z = 0.0;
    let mut x = -20.0;
    for _t in vrange {
        let mid_shader_params_handle = mid_params.add(BezierMidMat {
            color: color.into(),
            t: 0.5,
            zoom: 0.15 / globals.scale,
            size: Vec2::new(1.0, 1.0) * globals.scale,
            clearcolor: clearcolor.clone().into(),
            ..Default::default()
        });

        x = x + 2.0;
        z = z + 10.0;
        let child = commands
            // // left
            .spawn_bundle(MaterialMesh2dBundle {
                mesh: middle_mesh_handle.clone(),
                visibility: visible.clone(),
                // render_pipelines: render_piplines.clone(),
                transform: Transform::from_xyz(0.0, 0.0, pos_z + 100.0 + z),
                material: mid_shader_params_handle,
                ..Default::default()
            })
            .insert(MiddlePointQuad)
            .insert(bezier_handle.clone())
            // .insert(mid_shader_params_handle.clone())
            .id();

        commands.entity(parent).push_children(&[child]);
    }

    return (parent, bezier_handle);
}
