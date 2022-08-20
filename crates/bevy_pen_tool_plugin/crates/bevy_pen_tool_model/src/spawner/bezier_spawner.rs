use crate::inputs::{Cursor, Latch};

use crate::materials::{BezierControlsMat, BezierEndsMat, SelectionMat};

use crate::model::{
    AchorEdgeQuad, Anchor, AnchorEdge, Bezier, BezierHandleEntity, BezierHist, BezierId,
    BezierParent, BezierPositions, BoundingBoxQuad, ControlPointQuad, Globals, Group, GroupId,
    HistoryAction, LatchData, MainUi, Maps, MovingAnchor, SpawningCurve,
};

use bevy::{asset::HandleId, prelude::*, sprite::MaterialMesh2dBundle};

use std::collections::HashMap;

// TODO: merge spawn_bezier_system and spawn_bezier
pub fn spawn_bezier_system(
    mut bezier_curves: ResMut<Assets<Bezier>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut cursor: ResMut<Cursor>,
    mut selection_params: ResMut<Assets<SelectionMat>>,
    mut controls_params: ResMut<Assets<BezierControlsMat>>,
    mut ends_params: ResMut<Assets<BezierEndsMat>>,
    // mut mid_params: ResMut<Assets<BezierMidMat>>,
    clearcolor_struct: Res<ClearColor>,
    mut globals: ResMut<Globals>,
    mut maps: ResMut<Maps>,
    mut latch_event_reader: EventReader<Latch>,
    // mut user_state: ResMut<UserState>,
    mut add_to_history_event_writer: EventWriter<HistoryAction>,
    mut spawn_curve_event_reader: EventReader<SpawningCurve>,
    mut groups: ResMut<Assets<Group>>,
    // mut spawn_mids_event_writer: EventWriter<SpawnMids>,
    mut group_event_writer: EventWriter<Handle<Group>>,
    // mut group_lut_event_writer: EventWriter<ComputeGroupLut>,
    // mut move_quad_event_writer: EventWriter<MoveAnchorEvent>,
    // cam_query: Query<&Transform, With<OrthographicProjection>>,
) {
    let mut do_send_to_history = true;
    // let mut do_move_anchor = false;
    // let mut do_nothing = false;
    // if let UserState::SpawningCurve {
    //     bezier_hist: maybe_bezier_hist,
    //     maybe_bezier_id,
    // } = &*user_state

    for SpawningCurve {
        bezier_hist: maybe_bezier_hist,
        maybe_bezier_id,
        follow_mouse,
    } in spawn_curve_event_reader.iter()
    {
        let clearcolor = clearcolor_struct.0;

        // actually generates a random HandleId
        let mut default_spawner_id = BezierId::default();

        if let Some(bezier_hist) = maybe_bezier_hist {
            default_spawner_id = bezier_hist.id.into();
        }

        let mut start = cursor.last_click_position;

        // the control points cannot be exactly in the same positions as the anchors
        // because the algorithm for finding position along the curves fail in that case
        let mut epsilon = 25.01;
        if globals.hide_control_points {
            epsilon = 0.01;
        }

        let mut control_start: Vec2 = start + Vec2::new(epsilon, epsilon);
        let control_end: Vec2 = start + Vec2::new(epsilon, epsilon);

        let mut latches: HashMap<AnchorEdge, LatchData> = HashMap::new();

        let mut group_id = GroupId::default();

        let mut latch_received_bool: bool = false;
        for latch_received in latch_event_reader.iter() {
            //
            latch_received_bool = true;
            println!("latch received: {:?}", latch_received);
            start = latch_received.position;
            control_start = latch_received.control_point;

            // if a latch is taking place, the id will have been generated
            // in the generate_start_latch_on_spawn(..) method for Bezier
            default_spawner_id = latch_received.latcher_id;

            let latch_local = LatchData {
                latched_to_id: latch_received.latchee_id,
                self_edge: AnchorEdge::Start,
                partners_edge: latch_received.latchee_edge,
            };

            latches.insert(AnchorEdge::Start, latch_local);

            group_id = latch_received.group_id;
        }

        cursor.latch = Vec::new();

        let mut bezier = Bezier {
            positions: BezierPositions {
                start,
                end: start,
                control_start,
                control_end,
            },
            previous_positions: BezierPositions::default(),
            latches,
            // move_quad: Anchor::End,
            id: default_spawner_id,
            group: group_id,
            ..Default::default()
        };

        // if the spawn if sent from a redo action
        if let Some(bezier_hist) = maybe_bezier_hist {
            do_send_to_history = bezier_hist.do_send_to_history;
            bezier.positions = bezier_hist.positions.clone();
            bezier.latches = bezier_hist.latches.clone();
            bezier.color = bezier_hist.color.clone();
            bezier.id = bezier_hist.id.into();
            bezier.do_compute_lut = true;
        }

        bezier.update_previous_pos();

        let (entity, handle) = spawn_bezier(
            &mut bezier,
            &mut bezier_curves,
            &mut commands,
            &mut meshes,
            // &mut my_shader_params,
            &mut selection_params,
            &mut controls_params,
            &mut ends_params,
            // &mut mid_params,
            clearcolor,
            &mut globals,
            &mut maps,
            &mut add_to_history_event_writer,
            &maybe_bezier_id,
            do_send_to_history,
            *follow_mouse,
        );

        if latch_received_bool {
            let group_handle = maps
                .group_map
                .get(&bezier.group)
                .expect("Could not find group id in group_map");

            let group = groups.get_mut(&group_handle).unwrap();

            group.add_curve(entity, handle);

            // group.find_connected_ends(&bezier_assets, maps.bezier_map.clone());
            // group.group_lut(&bezier_assets, maps.bezier_map.clone());
            // group.compute_standalone_lut(&bezier_assets, globals.group_lut_num_points);

            // group_event_writer.send(group_handle.clone());
            // group_lut_event_writer.send(ComputeGroupLut(group.id));
        } else {
            // produce a new group
            let mut group = Group::default();
            group.id = bezier.group;
            group.add_curve(entity, handle.clone());

            // group.find_connected_ends(&bezier_assets, maps.bezier_map.clone());
            // let group_handle: Handle<Group> = Handle::weak(bezier.group.0);

            group.ends = Some(vec![
                (handle.clone(), AnchorEdge::Start),
                (handle.clone(), AnchorEdge::End),
            ]);

            // maps.group_map.insert(bezier.group, group_handle.clone());

            let mut bezier_assets = HashMap::new();
            let bez = bezier.clone();
            bezier_assets.insert(handle.id, &bez);

            group.find_connected_ends(&bezier_assets, maps.bezier_map.clone());
            group.group_lut(&bezier_assets, maps.bezier_map.clone());
            group.compute_standalone_lut(&bezier_assets, globals.group_lut_num_points);
            // group.id = group_handle.id.into();
            // let group_handle = groups.add(group);
            let mut group_handle: Handle<Group> = bevy::asset::Handle::weak(group.id.0);
            group_handle.make_strong(&groups);
            group.id = group_handle.id.into();
            let strong_handle = groups.set(group_handle.clone(), group);
            // bezier.group = strong_handle.id.into();

            maps.group_map.insert(bezier.group, strong_handle.clone());

            group_event_writer.send(strong_handle.clone());
        }
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
    // mid_params: &mut ResMut<Assets<BezierMidMat>>,
    clearcolor: Color,
    globals: &mut ResMut<Globals>,
    maps: &mut ResMut<Maps>,
    add_to_history_event_writer: &mut EventWriter<HistoryAction>,
    maybe_bezier_id: &Option<BezierId>,
    do_send_to_history: bool,
    follow_mouse: bool,
) -> (Entity, Handle<Bezier>) {
    bezier.compute_lut_walk(100);

    let ends_controls_mesh_handle = maps.mesh_handles["ends_controls"].clone();
    let ends_mesh_handle = maps.mesh_handles["ends"].clone();

    // let middle_mesh_handle = maps.mesh_handles["middles"].clone();
    // let num_mid_quads = globals.num_points_on_curve;

    let mut color = Color::hex("3CB44B").unwrap();

    if let Some(color_in_params) = bezier.color {
        color = color_in_params;
    } else if let Some(color_in_globals) = globals.picked_color {
        color = color_in_globals;
    }
    bezier.color = Some(color);

    //
    //
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

    // let mut start_pt_pos = bezier.positions.start - bb_pos;
    // let mut end_pt_pos = bezier.positions.end - bb_pos;

    let mut start_pt_pos = bezier.positions.start;
    let mut end_pt_pos = bezier.positions.end;

    let ctr0_pos = bezier.positions.control_start; // - bb_pos;
                                                   // let ctr1_pos = bezier.positions.control_end - bb_pos;
    let ctr1_pos = bezier.positions.control_end;

    let mesh_handle_bb =
        bevy::sprite::Mesh2dHandle(meshes.add(Mesh::from(shape::Quad::new(bigger_size))));

    // since bezier is cloned, be careful about modifying it after the cloning, it won't have any side-effects
    let bezier_handle = if let Some(b_id) = maybe_bezier_id {
        // handle.clone()
        // let handle_entity = maps.bezier_map.get(b_id).unwrap().clone();
        bezier_curves.set(b_id.0, bezier.clone())
    } else {
        // bezier_curves.add(bezier.clone())
        let handle_id: HandleId = bezier.id.0.into();
        bezier_curves.set(handle_id, bezier.clone())
    };

    // assign the bezier handle id to the bezier id
    let bezier_got = bezier_curves.get_mut(&bezier_handle).unwrap();
    bezier_got.id = bezier_handle.id.into();

    //////////////////// Bounding box ////////////////////
    //
    //

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
        zoom: 1.0 / globals.scale,
        size: bb_size,
        clearcolor: clearcolor.clone().into(),
        ..Default::default()
    });

    let global_init_pos =
        GlobalTransform::from_translation(bb_pos.extend(globals.z_pos.bezier_parent));
    let mut init_pos = Transform::default();

    init_pos.scale = Vec3::new(globals.scale, globals.scale, 1.0);

    // TODO: remove BezierGrandParent and replace by BezierParent everywhere
    // This is the parent of every entity belonging to a rendered bezier curve.
    let parent = commands
        .spawn_bundle((
            BezierParent,
            MainUi,
            init_pos.clone(),
            Visibility { is_visible: true }, // visibility is inherited by all children
            global_init_pos,
            bezier_handle.clone(),
            ComputedVisibility::not_visible(), // the parent entity is not a rendered object
        ))
        .id();

    bezier_got.entity = Some(parent);

    if do_send_to_history {
        add_to_history_event_writer.send(HistoryAction::SpawnedCurve {
            // bezier_handle: bezier_handle.clone(),
            bezier_id: bezier.id.into(),
            bezier_hist: BezierHist::from(&bezier.clone()),
            // entity: parent,
            // id: bezier.id,
        });

        // send the latch to history
        if !bezier.latches.is_empty() {
            let latch = &bezier
                .latches
                .clone()
                .into_values()
                .collect::<Vec<LatchData>>()[0];

            add_to_history_event_writer.send(HistoryAction::Latched {
                self_id: bezier.id.into(),
                self_anchor: latch.self_edge,
                partner_id: latch.latched_to_id.into(),
                partner_anchor: latch.partners_edge,
            });
        }
    }

    // println!("spawned bezier curve with id: {:?}", bezier.latches);

    let bbquad_entity = commands
        // let parent = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle_bb,
            visibility: visible_bb,
            transform: Transform::from_translation(Vec2::ZERO.extend(globals.z_pos.bounding_box)),
            material: shader_params_handle_bb,
            ..Default::default()
        })
        .insert(BoundingBoxQuad)
        .insert(bezier_handle.clone())
        .id();

    commands.entity(parent).push_children(&[bbquad_entity]);

    //////////////////// Bounding box ////////////////////

    let ((start_displacement, end_displacement), (start_rotation, end_rotation)) =
        bezier.ends_displacement();

    start_pt_pos += start_displacement;
    end_pt_pos += end_displacement;

    let mut start_pt_transform =
        Transform::from_translation(start_pt_pos.extend(globals.z_pos.anchors));
    let mut end_pt_transform =
        Transform::from_translation(end_pt_pos.extend(globals.z_pos.anchors));

    start_pt_transform.rotation = start_rotation;
    end_pt_transform.rotation = end_rotation;

    let ends_params_handle = ends_params.add(BezierEndsMat {
        color: color.into(),
        t: 0.5,
        zoom: 1.0 / globals.scale,
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
        .insert(AchorEdgeQuad(AnchorEdge::Start))
        .insert(Anchor::Start)
        .insert(bezier_handle.clone())
        .insert(MovingAnchor {
            once: true,
            follow_mouse: false,
        })
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
        .insert(AchorEdgeQuad(AnchorEdge::End))
        .insert(Anchor::End)
        .insert(MovingAnchor {
            once: false,
            // follow mouse only if the spawn originate from a mouse click,
            // in which case an HistoryAction::SpawnedCurve is sent to the history
            follow_mouse,
        })
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
        visible_ctrl.is_visible = false;
    };

    /////////////////////////////////////////
    // Control start

    let ctr_pos_transform = Transform::from_translation(ctr0_pos.extend(globals.z_pos.controls));

    let controls_params_handle = controls_params.add(BezierControlsMat {
        color: color.into(),
        t: 2.5,
        zoom: 1.0 / globals.scale,
        size: Vec2::new(5.0, 5.0) * globals.scale,
        clearcolor: clearcolor.clone().into(),
        ..Default::default()
    });

    let control_start = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: ends_controls_mesh_handle.clone(),
            visibility: visible_ctrl.clone(),
            // render_pipelines: ctrl_render_piplines.clone(),
            transform: ctr_pos_transform,
            material: controls_params_handle.clone(),
            ..Default::default()
        })
        .insert(ControlPointQuad(AnchorEdge::Start))
        .insert(Anchor::ControlStart)
        .insert(MovingAnchor {
            once: true,
            follow_mouse: false,
        })
        .insert(bezier_handle.clone())
        // .insert(shader_params_handle_bb.clone())
        .id();

    commands.entity(parent).push_children(&[control_start]);

    /////////////////////////////////////////
    // Control end

    let ctr_pos_transform = Transform::from_translation(ctr1_pos.extend(globals.z_pos.controls));

    let controls_params_handle = controls_params.add(BezierControlsMat {
        color: color.into(),
        t: 2.5,
        zoom: 1.0 / globals.scale,
        size: Vec2::new(5.0, 5.0) * globals.scale,
        clearcolor: clearcolor.clone().into(),
        ..Default::default()
    });

    let control_end = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: ends_controls_mesh_handle.clone(),
            visibility: visible_ctrl.clone(),
            // render_pipelines: ctrl_render_piplines.clone(),
            transform: ctr_pos_transform,
            material: controls_params_handle.clone(),
            ..Default::default()
        })
        .insert(ControlPointQuad(AnchorEdge::End))
        .insert(Anchor::ControlEnd)
        .insert(MovingAnchor {
            once: false,
            follow_mouse: false,
        })
        .insert(bezier_handle.clone())
        // .insert(shader_params_handle_bb.clone())
        .id();

    commands.entity(parent).push_children(&[control_end]);

    // add entities to the Bezier map.
    // This map is used to improve query performance

    let anchor_entities = vec![
        (Anchor::Start, child_start),
        (Anchor::End, child_end),
        (Anchor::ControlStart, control_start),
        (Anchor::ControlEnd, control_end),
    ]
    .iter()
    .cloned()
    .collect::<HashMap<Anchor, Entity>>();

    maps.bezier_map.insert(
        bezier.id,
        BezierHandleEntity {
            handle: bezier_handle.clone(),
            entity: parent,
            anchor_entities,
        },
    );

    // //////////////////// Small moving rings aka middle quads ////////////////////
    // let visible = Visibility { is_visible: true };

    // let vrange: Vec<f32> = (0..num_mid_quads)
    //     .map(|x| ((x as f32) / (num_mid_quads as f32 - 1.0) - 0.5) * 2.0 * 50.0)
    //     .collect();

    // let mut z = 0.0;
    // let mut x = -20.0;

    // for _t in vrange {
    //     let mid_shader_params_handle = mid_params.add(BezierMidMat {
    //         color: color.into(),
    //         t: 0.5,
    //         zoom: 1.0 / globals.scale,
    //         size: Vec2::new(1.0, 1.0) * globals.scale,
    //         clearcolor: clearcolor.clone().into(),
    //         ..Default::default()
    //     });

    //     x = x + 2.0;
    //     z = z + 10.0;
    //     let child = commands
    //         // // left
    //         .spawn_bundle(MaterialMesh2dBundle {
    //             mesh: middle_mesh_handle.clone(),
    //             visibility: visible.clone(),
    //             // render_pipelines: render_piplines.clone(),
    //             transform: Transform::from_translation(start_pt_pos.extend(globals.z_pos.middles)),
    //             material: mid_shader_params_handle,
    //             ..Default::default()
    //         })
    //         .insert(MiddlePointQuad)
    //         .insert(bezier_handle.clone())
    //         // .insert(mid_shader_params_handle.clone())
    //         .id();

    //     commands.entity(parent).push_children(&[child]);
    // }

    return (parent, bezier_handle);
}

// pub fn spawn_middle_quads(
//     mut commands: Commands,
//     globals: Res<Globals>,
//     mut mid_params: ResMut<Assets<BezierMidMat>>,
//     clearcolor: Res<ClearColor>,
//     maps: Res<Maps>,
//     mut spawn_mids_event: EventReader<SpawnMids>,
// ) {
//     for spawn_mids in spawn_mids_event.iter() {
//         // let middle_mesh_handle = maps.mesh_handles["middles"].clone();
//         // let num_mid_quads = globals.num_points_on_curve;

//         // let visible = Visibility { is_visible: true };

//         // let vrange: Vec<f32> = (0..num_mid_quads)
//         //     .map(|x| ((x as f32) / (num_mid_quads as f32 - 1.0) - 0.5) * 2.0 * 50.0)
//         //     .collect();

//         // let mut z = 0.0;
//         // let mut x = -20.0;
//         // for _t in vrange {
//         //     let mid_shader_params_handle = mid_params.add(BezierMidMat {
//         //         color: spawn_mids.color.into(),
//         //         t: 0.5,
//         //         zoom: 1.0 / globals.scale,
//         //         size: Vec2::new(1.0, 1.0) * globals.scale,
//         //         clearcolor: clearcolor.0.clone().into(),
//         //         ..Default::default()
//         //     });

//         //     x = x + 2.0;
//         //     z = z + 10.0;
//         //     let child = commands
//         //         // // left
//         //         .spawn_bundle(MaterialMesh2dBundle {
//         //             mesh: middle_mesh_handle.clone(),
//         //             visibility: visible.clone(),
//         //             // render_pipelines: render_piplines.clone(),
//         //             transform: Transform::from_xyz(0.0, 0.0, globals.z_pos.middles),
//         //             material: mid_shader_params_handle,
//         //             ..Default::default()
//         //         })
//         //         .insert(MiddlePointQuad)
//         //         .insert(spawn_mids.bezier_handle.clone())
//         //         // .insert(mid_shader_params_handle.clone())
//         //         .id();

//         //     commands
//         //         .entity(spawn_mids.parent_entity)
//         //         .push_children(&[child]);
//         // }
//     }
// }
