use bevy_pen_tool_model::inputs::{Action, Cursor};
use bevy_pen_tool_model::mesh::PenMesh;
use bevy_pen_tool_model::model::*;

use bevy::prelude::*;

use std::collections::HashMap;
use std::collections::HashSet;

pub fn update_lut(
    mut bezier_curves: ResMut<Assets<Bezier>>,
    // mut bezier_assets_res: Res<Assets<Bezier>>,
    bezier_handles: Query<&Handle<Bezier>, (With<MovingAnchor>, With<AchorEdgeQuad>)>,
    globals: ResMut<Globals>,
    mut groups: ResMut<Assets<Group>>,
    maps: ResMut<Maps>,
) {
    let mut groups_to_update = HashSet::new();
    let mut bezier_partners_to_update = HashSet::new();
    for b_handle in bezier_handles.iter() {
        if let Some(bezier) = bezier_curves.get_mut(b_handle) {
            bezier.compute_lut_walk(globals.group_lut_num_points as usize);

            for (_parter_anchor, latch) in bezier.latches.iter() {
                if let Some(handle) = maps.bezier_map.get(&latch.latched_to_id) {
                    bezier_partners_to_update.insert(&handle.handle);
                }
            }

            // recompute the group lut
            // if let Some(_id) = bezier.group {
            for (group_handle_id, group) in groups.iter_mut() {
                // if _id == group_handle_id.id {
                if group.bezier_handles.contains(b_handle) {
                    groups_to_update.insert(group_handle_id);
                }
            }
            // }

            if bezier.do_compute_lut {
                bezier_partners_to_update.insert(b_handle);
                bezier.do_compute_lut = false;
            }
        }
    }
    for handle in bezier_partners_to_update.iter() {
        if let Some(bezier_partner) = bezier_curves.get_mut(&handle) {
            bezier_partner.compute_lut_walk(globals.group_lut_num_points as usize);
            // groups_to_update.insert(handle);
        }
    }
    let bezier_assets = bezier_curves
        .iter()
        .collect::<HashMap<bevy::asset::HandleId, &Bezier>>();

    // if the moving anchor is part of a group,
    for group_id in groups_to_update.iter() {
        let group_handle = groups.get_handle(*group_id);
        let group = groups.get_mut(&group_handle).unwrap();

        group.group_lut(&bezier_assets, maps.bezier_map.clone());
        group.compute_standalone_lut(&bezier_assets, globals.group_lut_num_points);
    }
}

pub fn update_anchors(
    mut bezier_curves: ResMut<Assets<Bezier>>,
    mut query: Query<(&Handle<Bezier>, &Anchor, &MovingAnchor)>,
    cursor: Res<Cursor>,
    maps: ResMut<Maps>,
) {
    // TODO: remove dependency on Cursor
    if cursor.latch.is_empty() {
        for (bezier_handle, anchor, moving_anchor) in query.iter_mut() {
            //
            if let Some(bezier) = bezier_curves.get_mut(bezier_handle) {
                //
                if moving_anchor.follow_mouse {
                    bezier.update_positions_cursor(&cursor, *anchor);

                    let latch_info = bezier.get_anchor_latch_info(*anchor);

                    if let Some(_) = latch_info {
                        update_latched_partner_position(
                            &maps.bezier_map,
                            &mut bezier_curves,
                            latch_info,
                        );
                    }
                }
            }
        }
    }
}

// TODO: separate into three separate systems:
// 1) move anchor order
// 2) unlatch anchor order
// 3) move connected chain
//
// After a mouse click on an anchor, orders to move either an anchor or the whole curve.
// The unlatch functionality is part of this function as well.

#[allow(dead_code)]
pub fn bezier_anchor_order(
    mut commands: Commands,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    globals: ResMut<Globals>,
    maps: ResMut<Maps>,
    mut move_anchor_event_reader: EventReader<MoveAnchorEvent>,
    mut unlatch_event_writer: EventWriter<UnlatchEvent>,
    // audio: Res<Audio>,
    // mut add_to_history_event_writer: EventWriter<HistoryAction>,
) {
    let mut latched_chain_whole_curve: Vec<Handle<Bezier>>; // = Vec::new();

    let mut latched_chain_whole_curve_set: HashSet<BezierId> = HashSet::new();

    let mut latched_beziers: Vec<BezierId> = Vec::new();

    ////////////////////////////////////
    for move_anchor in move_anchor_event_reader.iter() {
        //
        let chose_a_control_point =
            move_anchor.anchor == Anchor::ControlStart || move_anchor.anchor == Anchor::ControlEnd;
        let hidden_controls = globals.hide_control_points;

        // order to move the quad that was clicked on
        if let Some(bezier_handle_entity) = maps.bezier_map.get(&move_anchor.bezier_id) {
            // TODO: take care of the Anchor::All case

            let bezier = bezier_curves.get_mut(&bezier_handle_entity.handle).unwrap();

            // TODO: This "if" should be moved earlier: before the MoveAnchorEvent is sent
            //
            // cannot move a control point if it's hidden
            if !(chose_a_control_point && hidden_controls) {
                bezier.update_previous_pos();

                if move_anchor.anchor == Anchor::All {
                    latched_chain_whole_curve_set.insert(move_anchor.bezier_id);
                    latched_beziers.push(bezier.id);
                } else {
                    bezier.move_anchor(
                        &mut commands,
                        move_anchor.once,
                        true,
                        move_anchor.anchor,
                        maps.as_ref(),
                    );
                }
            }

            // unlatch event
            if move_anchor.unlatch {
                unlatch_event_writer.send(UnlatchEvent {
                    bezier_id: move_anchor.bezier_id,
                    anchor: move_anchor.anchor,
                });
            }
        } else {
            info!("no bezier handle found for {:?}", move_anchor.bezier_id);
        }
    }

    // TODO: allow for multiple latched_chain_whole_curves
    for bezier_id in latched_beziers {
        latched_chain_whole_curve =
            find_connected_curves(bezier_id, &mut bezier_curves, &maps.bezier_map);

        latched_chain_whole_curve_set = latched_chain_whole_curve
            .iter()
            .map(|x| x.id.into())
            .collect::<HashSet<BezierId>>()
            .union(&latched_chain_whole_curve_set)
            .cloned()
            .collect::<HashSet<BezierId>>();

        // println!(
        //     "latched_chain_whole_curve_set: {:?}",
        //     latched_chain_whole_curve_set
        // );
    }

    // Move the whole chain -> Anchor::All is sent

    for handle in latched_chain_whole_curve_set.iter() {
        let handle_entity = maps.bezier_map.get(&handle).unwrap();
        let bezier = bezier_curves.get_mut(&handle_entity.handle).unwrap();
        // bezier.move_quad = Anchor::All;

        bezier.update_previous_pos();

        bezier.move_anchor(
            &mut commands,
            false, /* once */
            true,  /* follow_mouse */
            Anchor::Start,
            maps.as_ref(),
        );

        bezier.move_anchor(
            &mut commands,
            false, /* once */
            true,  /* follow_mouse */
            Anchor::End,
            maps.as_ref(),
        );
    }
}

// Select by dragging the edge of a box
pub fn selection_box_init(
    mut commands: Commands,
    globals: ResMut<Globals>,

    cursor: ResMut<Cursor>,
    bezier_curves: ResMut<Assets<Bezier>>,
    query: Query<(Entity, &Handle<Bezier>), With<BezierParent>>,
    selection_box_query: Query<Entity, With<SelectingBoxQuad>>,
    mut action_event_reader: EventReader<Action>,
    mut visible_selection_query: Query<&mut Visibility, With<SelectingBoxQuad>>,
) {
    if action_event_reader
        .iter()
        .any(|x| x == &Action::SelectionBox)
    {
        if let Some((_distance, _anchor, _entity, _selected_handle)) =
            get_close_anchor_entity(2.0 * globals.scale, cursor.position, &bezier_curves, &query)
        {
        } else {
            // add CurrentlySelecting to the quad for the selection box
            for entity in selection_box_query.iter() {
                commands.entity(entity).insert(CurrentlySelecting);
            }

            for mut visible in visible_selection_query.iter_mut() {
                visible.is_visible = true;
            }
        }
    }
}

// inserts curves inside box in the Selection resource
pub fn selection_area_finalize(
    mut selection: ResMut<Selection>,
    cursor: ResMut<Cursor>,
    bezier_curves: ResMut<Assets<Bezier>>,
    // groups: ResMut<Assets<Group>>,
    mut query_set: ParamSet<(
        Query<&mut Visibility, With<SelectingBoxQuad>>,
        Query<&mut Visibility, With<SelectedBoxQuad>>,
        Query<&mut Visibility, With<GroupBoxQuad>>,
    )>,
    // group_query: Query<&Handle<Group>>,
    bezier_query: Query<(Entity, &Handle<Bezier>), With<BezierParent>>,
    mesh_query: Query<(&Transform, &PenMesh)>,
    mut action_event_reader: EventReader<Action>,
    globals: Res<Globals>,
    // mut group_box_event_writer: EventWriter<GroupBoxEvent>,
) {
    if action_event_reader.iter().any(|x| x == &Action::Selected) {
        selection.selected.clear();

        info!("selection_area_finalize");
        // check for meshes within the selection area
        for (transform, pen_mesh) in mesh_query.iter() {
            if cursor
                .anchor_is_within_selection_box(transform.translation.truncate() * globals.scale)
            {
                selection.selected.push(SelectionChoice::Mesh(
                    pen_mesh.clone(),
                    transform.translation.truncate(),
                ));
            }
        }

        let mut selected_curves = HashSet::new();
        let mut found_anchor_in_box = false;

        // check for anchors inside selection area
        for (_entity, bezier_handle) in bezier_query.iter() {
            let bezier = bezier_curves.get(bezier_handle).unwrap();

            if cursor.anchor_is_within_selection_box(bezier.positions.start * globals.scale)
                || cursor.anchor_is_within_selection_box(bezier.positions.end * globals.scale)
            {
                selected_curves.insert(bezier_handle.id.into());
                found_anchor_in_box = true;

                // selected_curves
                //     .group
                //     .insert((entity.clone(), bezier_handle.clone()));

                // selected_curves.bezier_handles.insert(bezier_handle.clone());
            }
        }

        if found_anchor_in_box {
            selection
                .selected
                .push(SelectionChoice::CurveSet(selected_curves));
        }

        // selection box visible
        for mut visible_selected in query_set.p1().iter_mut() {
            visible_selected.is_visible = true;
        }

        for mut visible_group_box_quad in query_set.p2().iter_mut() {
            visible_group_box_quad.is_visible = true;
        }

        // selecting box should be invisible
        for mut visible_selecting in query_set.p0().iter_mut() {
            visible_selecting.is_visible = false;
        }
    }
}

pub fn unselect(
    mut selection: ResMut<Selection>,
    mut visible_selection_query: Query<
        &mut Visibility,
        Or<(With<SelectedBoxQuad>, With<GroupBoxQuad>)>,
    >,
    mut action_event_reader: EventReader<Action>,
) {
    if action_event_reader.iter().any(|x| x == &Action::Unselect) {
        selection.selected = vec![];

        for mut visible in visible_selection_query.iter_mut() {
            visible.is_visible = false;
        }
    }
}

// // group curves together to form a more complex path
// pub fn groupy(
//     mut commands: Commands,
//     mut groups: ResMut<Assets<Group>>,
//     globals: ResMut<Globals>,
//     selection: ResMut<Selection>,
//     mut maps: ResMut<Maps>,
//     // bezier_curves: Res<Assets<Bezier>>,
//     mut bezier_curves_mut: ResMut<Assets<Bezier>>,

//     mid_bezier_query: Query<(Entity, &Handle<Bezier>), With<MiddlePointQuad>>,
//     group_query: Query<(Entity, &Handle<Group>), With<GroupParent>>,
//     mut event_writer: EventWriter<Handle<Group>>,

//     mut action_event_reader: EventReader<Action>,
//     mut loaded_event_reader: EventReader<Loaded>,
//     audio: Res<Audio>,
// ) {
//     let mut do_group = false;
//     let mut do_compute_lut = false;
//     // group selected curves
//     if action_event_reader.iter().any(|x| x == &Action::Group) {
//         do_group = true;
//         do_compute_lut = true;
//     }

//     // group loaded curves
//     if let Some(Loaded) = loaded_event_reader.iter().next() {
//         do_group = true;
//     }

//     if do_group && selection.selected.iter().count() == 1 {
//         let selected = selection.selected[0].clone();
//         let id_handle_map: HashMap<BezierId, BezierHandleEntity> = maps.bezier_map.clone();

//         let bezier_assets = bezier_curves_mut
//             .iter()
//             .collect::<HashMap<bevy::asset::HandleId, &Bezier>>();

//         if let SelectionChoice::CurveSet(mut selected) = selected.clone() {
//             selected.find_connected_ends(&bezier_assets, id_handle_map.clone());
//             // println!("connected ends: {:?}, ", selected.ends);

//             // abort grouping if the selection is not completely connected with latches
//             if selected.ends.is_none() {
//                 println!("Cannot group. Select multiple latched curves to successfully group");
//                 return;
//             }

//             // // if the selected curves are already in a group, abort
//             // for bez_handle in selected.bezier_handles.iter() {
//             //     let bez = bezier_curves_mut.get(bez_handle).unwrap();
//             //     if bez.group.is_some() {
//             //         println!("Cannot group. Selected curves are already in a group");
//             //         return;
//             //     }
//             // }

//             // get rid of the middle point quads
//             for (entity, bezier_handle) in mid_bezier_query.iter() {
//                 if selected.bezier_handles.contains(bezier_handle) {
//                     commands.entity(entity).despawn();
//                 }
//             }

//             if do_compute_lut {
//                 selected.group_lut(&bezier_assets, id_handle_map.clone());
//                 selected.compute_standalone_lut(&bezier_assets, globals.group_lut_num_points);
//             }

//             if globals.sound_on {
//                 if let Some(sound) = maps.sounds.get("group") {
//                     audio.play(sound.clone());
//                 }
//             }

//             // TODO: we must get rid of this to have more than one group allowed.
//             // get rid of the current group before making a new one
//             for (entity, group_handle) in group_query.iter() {
//                 let group = groups.get(group_handle).unwrap();
//                 for bezier_handle in group.bezier_handles.clone() {
//                     if selected.bezier_handles.contains(&bezier_handle) {
//                         commands.entity(entity).despawn();
//                         break;
//                     }
//                 }
//             }

//             for bezier_handle in selected.bezier_handles.clone() {
//                 let bezier = bezier_curves_mut.get_mut(&bezier_handle).unwrap();
//                 bezier.group = selected.id;
//             }

//             let group_handle = groups.add(selected.clone());

//             maps.group_map.insert(selected.id, group_handle.clone());

//             // spawn the middle quads and the bounding box

//             event_writer.send(group_handle.clone());
//         }
//     }
// }

pub fn latchy(
    // mut commands: Commands,
    cursor: ResMut<Cursor>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    query: Query<(&Handle<Bezier>, &AchorEdgeQuad), With<MovingAnchor>>,

    globals: ResMut<Globals>,
    mut action_event_reader: EventReader<Action>,
    non_moving_edge_query: Query<(&Handle<Bezier>, &AchorEdgeQuad), Without<MovingAnchor>>,
    // mut groups: ResMut<Assets<Group>>,
    // group_query: Query<(Entity, &Handle<Group>), With<GroupParent>>,
    maps: Res<Maps>,
) {
    if action_event_reader.iter().any(|x| x == &Action::Latch) {
        // let latching_distance = globals.anchor_clicking_dist;

        let mut potential_mover: Option<(Vec2, BezierId, AnchorEdge, Handle<Bezier>)> = None;
        let mut potential_partner: Option<(
            BezierId,
            AnchorEdge,
            AnchorEdge,
            Handle<Bezier>,
            Handle<Bezier>,
            GroupId,
        )> = None;

        // find moving quad and store its parameters
        for (bezier_handle, anchor_edge) in query.iter() {
            let mut bezier = bezier_curves.get(bezier_handle).unwrap().clone();
            bezier.potential_latch = None;

            // a latched point does not latch to an additional point
            let moving_anchor = anchor_edge.0;
            if bezier.quad_is_latched(&moving_anchor) {
                return (); // TODO: find out if this introduces a bug
            }

            let mover_pos = cursor.position;
            potential_mover = Some((mover_pos, bezier.id, moving_anchor, bezier_handle.clone()));

            // only runs once so as to not latch to multiple achor edges
            break;
        }

        // find quad within latching_distance. Upon success, setup a latch and store the
        // paramters of the latchee (partner)
        if let Some((pos, id, mover_edge, mover_handle)) = potential_mover {
            if let Some((_dist, anchor_edge, partner_handle)) = get_close_still_unlatched_anchor(
                // latching_distance * globals.scale,
                globals.anchor_clicking_dist,
                pos,
                &bezier_curves,
                // &query,
                &non_moving_edge_query,
            ) {
                // println!("processing Partner latch");
                let partner_bezier = bezier_curves.get_mut(&partner_handle.clone()).unwrap();

                // if the potential partner is free, continue
                if partner_bezier.quad_is_latched(&anchor_edge) {
                    println!("Cannot latch. Partner is latched");
                    return;
                }

                potential_partner = Some((
                    partner_bezier.id,
                    mover_edge,
                    anchor_edge,
                    mover_handle,
                    partner_handle.clone(),
                    partner_bezier.group,
                ));

                let partner_latch_data = LatchData {
                    latched_to_id: id,
                    self_edge: anchor_edge,
                    partners_edge: mover_edge,
                };

                partner_bezier.potential_latch = Some(partner_latch_data);
            } else {
                // if no partner is found, remove the potential latch
                let bezier = bezier_curves.get(&mover_handle).unwrap().clone();

                if let Some(potential_latch) = bezier.potential_latch {
                    if let Some(partner_handle) =
                        maps.bezier_map.get(&potential_latch.latched_to_id)
                    {
                        let partner_bezier = bezier_curves.get_mut(&partner_handle.handle).unwrap();
                        (*partner_bezier).potential_latch = None;
                    }
                }
                let bezier = bezier_curves.get_mut(&mover_handle).unwrap();
                (*bezier).potential_latch = None;
            }
        }

        // setup the latcher if a partner has been found
        if let Some((partner_id, mover_anchor, pa_edge, mover_handle, partner_handle, _group_id)) =
            potential_partner
        {
            let partner_bezier = bezier_curves.get(&partner_handle).unwrap().clone();
            let bezier = bezier_curves.get_mut(&mover_handle.clone()).unwrap();

            let latch_anchor_position = partner_bezier.get_position(pa_edge.to_anchor());
            let latch_control_position = partner_bezier.get_opposite_control(pa_edge);

            let mover_latch_data = LatchData {
                latched_to_id: partner_id,
                self_edge: mover_anchor,
                partners_edge: pa_edge,
            };

            bezier.potential_latch = Some(mover_latch_data.clone());

            // set the position of the latched moving quad and its control point
            if mover_anchor == AnchorEdge::Start {
                bezier.positions.start = latch_anchor_position;
                bezier.positions.control_start = latch_control_position;
            } else if mover_anchor == AnchorEdge::End {
                bezier.positions.end = latch_anchor_position;
                bezier.positions.control_end = latch_control_position;
            }
            //////////////////////////////////

            // let partner_bezier = bezier_curves.get(&partner_handle).unwrap().clone();
            // let bezier = bezier_curves.get_mut(&mover_handle.clone()).unwrap();

            // let group_id_to_delete = bezier.group;
            // let group_handle = maps.group_map.get(&group_id_to_delete).unwrap();
            // groups.remove(group_handle);

            // for (entity, queried_group_handle) in group_query.iter() {
            //     if queried_group_handle == group_handle {
            //         commands.entity(entity).despawn_recursive();
            //         println!("Removed group !!! !!! !!!");
            //     }
            // }

            // bezier.group = group_id;

            // let latch_anchor_position = partner_bezier.get_position(pa_edge.to_anchor());
            // let latch_control_position = partner_bezier.get_opposite_control(pa_edge);

            // let mover_latch_data = LatchData {
            //     latched_to_id: partner_id,
            //     self_edge: mover_anchor,
            //     partners_edge: pa_edge,
            // };

            // bezier.potential_latch = Some(mover_latch_data.clone());

            // // add the latcher's curve to the latchee's group
            // let group = groups.get_mut(&maps.group_map[&group_id]).unwrap();
            // let curve_handle_entity = maps.bezier_map.get(&bezier.id).unwrap().clone();
            // group.add_curve(curve_handle_entity.entity, mover_handle.clone());

            // // set the position of the latched moving quad and its control point
            // if mover_anchor == AnchorEdge::Start {
            //     bezier.positions.start = latch_anchor_position;
            //     bezier.positions.control_start = latch_control_position;
            // } else if mover_anchor == AnchorEdge::End {
            //     bezier.positions.end = latch_anchor_position;
            //     bezier.positions.control_end = latch_control_position;
            // }

            // // update group properties
            // let bezier_assets = bezier_curves
            //     .iter()
            //     .collect::<HashMap<bevy::asset::HandleId, &Bezier>>();

            // group.find_connected_ends(&bezier_assets, maps.bezier_map.clone());
            // group.group_lut(&bezier_assets, maps.bezier_map.clone());
            // group.compute_standalone_lut(&bezier_assets, globals.group_lut_num_points);

            // // delete the latcher's group
            // maps.group_map.remove(&group_id_to_delete);
        }
    }
}

struct BezierToRemoveFromGroup {
    group_id: GroupId,
    bezier_handle_entity: BezierHandleEntity,
    // color: Color,
    anchor_edge: AnchorEdge,
    old_partner_id: BezierId,
}

// TODO: separate in two distinct groups
pub fn unlatchy(
    // mut bezier_curves: Res<Assets<Bezier>>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    mut groups: ResMut<Assets<Group>>,
    globals: ResMut<Globals>,
    mut maps: ResMut<Maps>,
    mut unlatch_event_reader: EventReader<UnlatchEvent>,
    audio: Res<Audio>,
    mut add_to_history_event_writer: EventWriter<HistoryAction>,
    // mut spawn_mids_event_writer: EventWriter<SpawnMids>,
    mut event_writer: EventWriter<Handle<Group>>,
    // mut group_lut_event_writer: EventWriter<ComputeGroupLut>,
) {
    for unlatch in unlatch_event_reader.iter() {
        let mut latch_partner: Option<(BezierId, LatchData)> = None;
        let mut bezier_in_group: Option<BezierToRemoveFromGroup> = None;

        // unlatch primary
        if let Some(bezier_handle_entity) = maps.bezier_map.get(&unlatch.bezier_id) {
            // TODO: take care of the Anchor::All case

            // curve that was clicked on
            let bezier = bezier_curves.get_mut(&bezier_handle_entity.handle).unwrap();
            //
            // println!("Unlatching latches {:#?}", bezier.latches);
            // println!("unlatch.anchor {:#?}", unlatch.anchor);

            // remove latch and separate into two groups
            match unlatch.anchor {
                anchor @ (Anchor::Start | Anchor::End) => {
                    if let Some(temp_latch) = bezier.latches.get(&anchor.to_edge()) {
                        //
                        // keep the latch information in memory to unlatch the anchor's partner below
                        latch_partner = Some((bezier.id, temp_latch.clone()));

                        // let old_partner_id = maps
                        //     .bezier_map
                        //     .get(&temp_latch.latched_to_id)
                        //     .unwrap()
                        //     .clone();

                        // if let Some(group_id) = bezier.group {
                        bezier_in_group = Some(BezierToRemoveFromGroup {
                            group_id: bezier.group,
                            bezier_handle_entity: bezier_handle_entity.clone(),
                            // color: bezier
                            //     .color
                            //     .unwrap_or(globals.picked_color.unwrap_or(Color::WHITE)),
                            anchor_edge: anchor.to_edge(),
                            old_partner_id: temp_latch.latched_to_id,
                        });

                        bezier.latches.remove(&anchor.to_edge());
                        bezier.potential_latch = None;
                    }
                }

                _ => {}
            }
        }

        // unlatch partner
        if let Some((self_id, latch)) = latch_partner {
            //
            if let Some(handle) = maps.bezier_map.get(&latch.latched_to_id) {
                //
                let bezier = bezier_curves.get_mut(&handle.handle).unwrap();
                //
                if let Some(_) = bezier.latches.remove(&latch.partners_edge) {
                    bezier.potential_latch = None;
                    println!("unlatched partner: {:?}", bezier.group);

                    add_to_history_event_writer.send(HistoryAction::Unlatched {
                        self_id: self_id.into(),
                        partner_id: latch.latched_to_id.into(),
                        self_anchor: latch.self_edge,
                        partner_anchor: latch.partners_edge,
                    });

                    if globals.sound_on {
                        if let Some(sound) = maps.sounds.get("unlatch") {
                            audio.play(sound.clone());
                        }
                    }
                }
            }
        }

        // remove from group and create new group
        if let Some(BezierToRemoveFromGroup {
            group_id,
            bezier_handle_entity,
            // color: _,
            anchor_edge,
            old_partner_id,
        }) = bezier_in_group
        {
            // let group = groups.get_mut().unwrap();
            let mut new_group = Group::default();
            let new_group_id = new_group.id;
            // let bezier = bezier_curves.get_mut(&bezier_handle_entity.handle).unwrap();
            // bezier.group = new_group.group_id;

            let bezier_assets = bezier_curves
                .iter()
                .collect::<HashMap<bevy::asset::HandleId, &Bezier>>();

            let main_bezier = bezier_assets.get(&bezier_handle_entity.handle.id).unwrap();

            let curve_chain = find_onesided_chained_curves(
                main_bezier,
                &bezier_assets,
                maps.bezier_map.clone(),
                anchor_edge.other(),
            );

            let chain_ids = curve_chain
                .iter()
                .map(|x| x.handle.id.into())
                .collect::<Vec<BezierId>>();

            // If the chain is unchanged after unlatching, no need to take group-related actions
            if !chain_ids.contains(&old_partner_id) {
                //

                let group_handle = maps.group_map.get(&group_id).unwrap();
                let group = groups.get_mut(&group_handle).unwrap();

                // change group of each bezier curve in the chain for the new group id
                for bezier_from_chain in curve_chain.iter() {
                    let bezier = bezier_curves.get_mut(&bezier_from_chain.handle).unwrap();
                    bezier.group = new_group.id;
                }

                for bezier_from_chain in curve_chain.iter() {
                    // TODO: too much compute! find_connected_ends, group_lut and compute_standalone_lut
                    // are computed at each iteration. Only need to be computed at the end
                    group.remove_curve(&bezier_from_chain); //, &bezier_assets, &maps.bezier_map);
                    new_group.add_curve(bezier_from_chain.entity, bezier_from_chain.handle.clone());
                }

                // group_lut_event_writer.send(ComputeGroupLut(group.group_id));

                let bezier_assets = bezier_curves
                    .iter()
                    .collect::<HashMap<bevy::asset::HandleId, &Bezier>>();

                new_group.find_connected_ends(&bezier_assets, maps.bezier_map.clone());
                new_group.group_lut(&bezier_assets, maps.bezier_map.clone());
                new_group.compute_standalone_lut(&bezier_assets, globals.group_lut_num_points);

                let new_group_handle = groups.add(new_group);

                maps.group_map
                    .insert(new_group_id, new_group_handle.clone());

                event_writer.send(new_group_handle);
            }
        }
    }
}

pub fn officiate_latch_partnership(
    mut commands: Commands,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    mut latch_event_reader: EventReader<OfficialLatch>,
    mut history_action_event_writer: EventWriter<HistoryAction>,
    globals: ResMut<Globals>,
    audio: Res<Audio>,
    mut groups: ResMut<Assets<Group>>,
    group_query: Query<(Entity, &Handle<Group>), With<GroupParent>>,
    mut maps: ResMut<Maps>,
    mut group_lut_event_writer: EventWriter<ComputeGroupLut>,
) {
    for OfficialLatch(latch, bezier_1_handle) in latch_event_reader.iter() {
        //
        //
        // curve 1 is the curve that was being manipulated by the user
        let bezier_1 = bezier_curves.get_mut(bezier_1_handle).unwrap();

        bezier_1.latches.insert(latch.self_edge, latch.clone());
        bezier_1.compute_lut_walk(100); // TODO: is this useful? also, it should be dependent on a global var
        bezier_1.potential_latch = None;
        let bezier_1_id = bezier_1.id;
        let group_id_to_delete = bezier_1.group;

        ///////////// partner //////////////////////////////////
        //
        // curve 2 is the curve that was latched to
        let handle_entity_2 = maps.bezier_map[&latch.latched_to_id.into()].clone();
        let bezier_2 = bezier_curves.get_mut(&handle_entity_2.handle).unwrap();
        bezier_2.potential_latch = None;

        let latch_2 = LatchData {
            latched_to_id: bezier_1_id.into(),
            self_edge: latch.partners_edge,
            partners_edge: latch.self_edge,
        };

        bezier_2.latches.insert(latch.partners_edge, latch_2);
        // TODO: is this useful? also, it should be dependent on a global var
        bezier_2.compute_lut_walk(100);

        let bezier_2_group = bezier_2.group;
        //
        ///////////// partner //////////////////////////////////

        // send latch event to undo/redo history
        history_action_event_writer.send(HistoryAction::Latched {
            self_id: bezier_1_id.into(),
            partner_id: bezier_2.id.into(),
            self_anchor: latch.self_edge,
            partner_anchor: latch.partners_edge,
        });

        if globals.sound_on {
            if let Some(sound) = maps.sounds.get("latch") {
                audio.play(sound.clone());
            }
        }
        //
        // if the two curves are already in the same group, do no group-related actions
        if group_id_to_delete == bezier_2_group {
            //
        } else {
            //
            // if the two curves are not in the same group, merge the two groups
            //

            let group_1_handle = maps.group_map.get(&group_id_to_delete).unwrap();
            groups.remove(group_1_handle);

            // TODO: add entity to group_map to avoid looping here
            //
            // despawn the mid quads belonging to group_1
            for (entity, queried_group_handle) in group_query.iter() {
                if queried_group_handle == group_1_handle {
                    commands.entity(entity).despawn_recursive();
                }
            }

            let bezier_1 = bezier_curves.get_mut(bezier_1_handle).unwrap();
            bezier_1.group = bezier_2_group;

            // delete the latcher's group from group_map
            maps.group_map.remove(&group_id_to_delete);

            // // add bezier_1 to bezier_2's group
            // let curve_handle_entity = maps.bezier_map.get(&bezier_1_id).unwrap().clone();
            // group.add_curve(curve_handle_entity.entity, bezier_1_handle.clone());

            // Add curves latched to bezier_1 to bezier_2's group
            let bezier_assets = bezier_curves
                .iter()
                .collect::<HashMap<bevy::asset::HandleId, &Bezier>>();

            let bezier_1 = bezier_curves.get(bezier_1_handle).unwrap();

            let bezier_chain = find_onesided_chained_curves(
                &bezier_1,
                &bezier_assets,
                maps.bezier_map.clone(),
                latch.self_edge.other(),
            );

            // perform modifications to group 2
            let group = groups.get_mut(&maps.group_map[&bezier_2_group]).unwrap();
            for bez in bezier_chain.iter() {
                let bez = bezier_curves.get_mut(&bez.handle).unwrap();

                // change the group ids for all the curves that are latched to curve 1
                bez.group = bezier_2_group;
                let curve_handle_entity = maps.bezier_map.get(&bez.id).unwrap().clone();

                // add the curves to group_2
                group.add_curve(
                    curve_handle_entity.entity,
                    curve_handle_entity.handle.clone(),
                );
            }

            // Compute new look-up tables
            group_lut_event_writer.send(ComputeGroupLut(bezier_2_group));
        }
    }
}

pub fn compute_group_lut(
    bezier_curves: Res<Assets<Bezier>>,
    mut groups: ResMut<Assets<Group>>,
    mut group_lut_event_reader: EventReader<ComputeGroupLut>,
    mut group_asset_event: EventReader<AssetEvent<Group>>,
    mut bezier_asset_event: EventReader<AssetEvent<Bezier>>,
    maps: Res<Maps>,
    globals: Res<Globals>,
) {
    // On demand look-up table computation using the ComputeGroupLut event
    for group_lut_event in group_lut_event_reader.iter() {
        if let Some(group_handle) = maps.group_map.get(&group_lut_event.0) {
            // let group = groups.get_mut(group_handle).unwrap();
            if let Some(group) = groups.get_mut(group_handle) {
                let bezier_assets = bezier_curves
                    .iter()
                    .collect::<HashMap<bevy::asset::HandleId, &Bezier>>();

                group.find_connected_ends(&bezier_assets, maps.bezier_map.clone());
                group.group_lut(&bezier_assets, maps.bezier_map.clone());
                group.compute_standalone_lut(&bezier_assets, globals.group_lut_num_points);
            }
        }
    }

    // Every time a group is created, compute its look-up table.
    // Useful for unlatching.
    for ev in group_asset_event.iter() {
        match ev {
            AssetEvent::Created { handle } => {
                if let Some(group) = groups.get_mut(handle) {
                    let bezier_assets = bezier_curves
                        .iter()
                        .collect::<HashMap<bevy::asset::HandleId, &Bezier>>();

                    group.find_connected_ends(&bezier_assets, maps.bezier_map.clone());
                    group.group_lut(&bezier_assets, maps.bezier_map.clone());
                    group.compute_standalone_lut(&bezier_assets, globals.group_lut_num_points);
                }
            }
            _ => {}
        }
    }

    // Every time a bezier curve is modified, compute its group's look-up table
    for ev in bezier_asset_event.iter() {
        match ev {
            AssetEvent::Modified { handle } => {
                if let Some(bezier) = bezier_curves.get(handle) {
                    let bezier_assets = bezier_curves
                        .iter()
                        .collect::<HashMap<bevy::asset::HandleId, &Bezier>>();

                    if let Some(group_handle) = maps.group_map.get(&bezier.group) {
                        if let Some(group) = groups.get_mut(&group_handle) {
                            group.find_connected_ends(&bezier_assets, maps.bezier_map.clone());
                            group.group_lut(&bezier_assets, maps.bezier_map.clone());
                            group.compute_standalone_lut(
                                &bezier_assets,
                                globals.group_lut_num_points,
                            );
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

// // TODO: delete this
// pub fn ungroupy(
//     mut commands: Commands,
//     mut groups: ResMut<Assets<Group>>,
//     selection: ResMut<Selection>,
//     globals: ResMut<Globals>,
//     query: Query<(Entity, &Handle<Group>), With<GroupParent>>,
//     bezier_query: Query<(Entity, &Handle<Bezier>, &Parent)>,
//     mut maps: ResMut<Maps>,
//     mut bezier_curves: ResMut<Assets<Bezier>>,
//     mut action_event_reader: EventReader<Action>,
//     mut spawn_mids_event_writer: EventWriter<SpawnMids>,
// ) {
//     if action_event_reader.iter().any(|x| x == &Action::Ungroup) {
//         // let group = &selection.selected;
//         if selection.selected.iter().count() == 1 {
//             let selected = selection.selected.iter().next().unwrap();
//             if let SelectionChoice::CurveSet(selected) = selected.clone() {
//                 let group_beziers = selected.bezier_handles.clone();

//                 if group_beziers.is_empty() {
//                     println!("Cannot ungroup. No curves selected");
//                     return;
//                 }

//                 let bezier_handles = group_beziers
//                     .iter()
//                     .cloned()
//                     .collect::<Vec<Handle<Bezier>>>();

//                 // Check if the handles are all connected
//                 // bezier_handles is never empty at this point
//                 let first_bezier_handle = bezier_handles.iter().next().unwrap();

//                 let first_bezier = bezier_curves.get(first_bezier_handle).unwrap();

//                 let mut bezier_chain =
//                     find_connected_curves(first_bezier.id, &bezier_curves, &maps.bezier_map);

//                 // first_bezier.find_connected_curves(bezier_curve_hack, &maps.bezier_map);

//                 bezier_chain.push(first_bezier_handle.clone());

//                 let bezier_chain_hashset = bezier_chain
//                     .iter()
//                     .cloned()
//                     .collect::<HashSet<Handle<Bezier>>>();

//                 // check if all curves are part of the same group
//                 for handle in bezier_chain.iter() {
//                     let bezier = bezier_curves.get(handle).unwrap();
//                     // if let Some(group_id) = bezier.group {
//                     if bezier.group != selected.id {
//                         println!("Cannot ungroup. Not all curves are part of the same group");
//                         return;
//                     }
//                     // }
//                 }

//                 // TODO: this is not needed right?
//                 if group_beziers != bezier_chain_hashset {
//                     println!("Cannot ungroup. Curves are not part of the same chain");
//                     return;
//                 }

//                 // if let Some(id) = first_bezier.group {
//                 // println!("id: {:?}", id);
//                 // println!("maps.id_group_handle: {:?}", maps.id_group_handle.keys());
//                 if let Some(group_handle) = maps.group_map.get(&first_bezier.group) {
//                     // remove With
//                     let _what = groups.remove(group_handle);

//                     for (entity, queried_group_handle) in query.iter() {
//                         if queried_group_handle == group_handle {
//                             commands.entity(entity).despawn_recursive();
//                             println!("Removed group");
//                         }
//                     }
//                 } else {
//                     info!("Cannot delete group: wrong group id.")
//                 }
//                 maps.group_map.remove(&first_bezier.group);
//                 // }

//                 for bezier_handle in bezier_chain_hashset {
//                     let bezier = bezier_curves.get_mut(&bezier_handle).unwrap();

//                     // TODO group: separate into individual one-curve groups
//                     // bezier.group = None;

//                     // replace group mid quads by bezier mid quads
//                     for (_bez_entity, bez_handle, parent) in bezier_query.iter() {
//                         if let Some(chain_bezier_handle) = maps.bezier_map.get(&bezier.id) {
//                             if bez_handle == &chain_bezier_handle.handle {
//                                 // spawn mid quads
//                                 let spawn_mids = SpawnMids {
//                                     color: bezier
//                                         .color
//                                         .unwrap_or(globals.picked_color.unwrap_or(Color::WHITE)),
//                                     bezier_handle: bez_handle.clone(),
//                                     parent_entity: **parent,
//                                 };
//                                 // spawn bezier middle quads for each bezier
//                                 spawn_mids_event_writer.send(spawn_mids);

//                                 break;
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }

pub fn delete(
    mut commands: Commands,
    mut selection: ResMut<Selection>,
    mut maps: ResMut<Maps>,
    mut bezier_curves: ResMut<Assets<Bezier>>,

    mut groups: ResMut<Assets<Group>>,
    mut visible_query: Query<&mut Visibility, With<SelectedBoxQuad>>,
    // query: Query<&Handle<Bezier>, With<BezierParent>>,
    // query2: Query<&Handle<Group>, With<GroupParent>>, // TODO: change to GroupParent
    mut action_event_reader: EventReader<Action>,
    mut add_to_history_event_writer: EventWriter<HistoryAction>,
) {
    // if action_event_reader.iter().any(|x| x == &Action::Delete) {
    for action in action_event_reader.iter() {
        if let Action::Delete(is_from_redo) = action {
            // info!("MAPS: {:?}", maps.group_map);
            // list of partners that need to be unlatched

            //
            for selected in selection.selected.iter() {
                let mut delete_curve_events = Vec::new();

                let mut latched_partners: Vec<(BezierId, LatchData)> = Vec::new();
                match selected.clone() {
                    // if let SelectionChoice::Group(selected) = selection.selected.clone() {
                    SelectionChoice::CurveSet(selected) => {
                        let mut delete_groups: HashSet<GroupId> = HashSet::new();
                        for bezier_id in selected.iter() {
                            //
                            let handle_entity = maps.bezier_map.get(bezier_id).unwrap().clone();
                            let bezier = bezier_curves
                                .get_mut(&handle_entity.handle.clone())
                                .unwrap();
                            // println!("within DELETE ---> bezier: {:?}", bezier.id);

                            // latched_partners.push(bezier.latches[&AnchorEdge::Start].clone());
                            if let Some(latched_anchor) = bezier.latches.get(&AnchorEdge::Start) {
                                if !selected.contains(&latched_anchor.latched_to_id) {
                                    latched_partners.push((bezier.id, latched_anchor.clone()));
                                }
                            }

                            // latched_partners.push(bezier.latches[&AnchorEdge::End].clone());
                            if let Some(latched_anchor) = bezier.latches.get(&AnchorEdge::End) {
                                if !selected.contains(&latched_anchor.latched_to_id) {
                                    latched_partners.push((bezier.id, latched_anchor.clone()));
                                }
                            }

                            delete_curve_events.push(HistoryAction::DeletedCurve {
                                bezier: BezierHist::from(&bezier.clone()),
                                bezier_id: bezier.id.into(),
                            });

                            let bezier_id = bezier.id;

                            if let Some(group_handle) = maps.group_map.get_mut(&bezier.group) {
                                let group = groups.get_mut(&group_handle).unwrap();

                                // let bezier_assets =
                                //     bezier_curves
                                //         .iter()
                                //         .collect::<HashMap<bevy::asset::HandleId, &Bezier>>();

                                group.remove_curve(
                                    &handle_entity,
                                    // &bezier_assets,
                                    // &maps.bezier_map,
                                );

                                if group.bezier_handles.is_empty() {
                                    delete_groups.insert(group.id);
                                }
                            }

                            commands.entity(handle_entity.entity).despawn_recursive();
                            maps.bezier_map.remove(&bezier_id);
                            bezier_curves.remove(handle_entity.handle);

                            // maps.group_map.remove(&bezier.group);
                        }

                        for group_id in delete_groups.iter() {
                            //
                            let group_handle = maps.group_map.get(group_id).unwrap().clone();
                            let group = groups.get(&group_handle).unwrap();
                            //
                            if let Some(entity) = group.entity {
                                println!("DELETING GROUP");
                                commands.entity(entity).despawn_recursive();
                                maps.group_map.remove(&group.id);
                            }

                            groups.remove(group_handle);
                        }
                    }
                    SelectionChoice::Mesh(
                        PenMesh {
                            id,
                            bounding_box: _,
                        },
                        _pos,
                    ) => {
                        //
                        let entity = maps.mesh_map.get(&id).unwrap();
                        commands.entity(*entity).despawn();
                        maps.mesh_map.remove(&id);
                    }
                    _ => {}
                }

                // // delete whole group
                // // TODO: delete only the selected curves
                // for group_handle in query2.iter() {
                //     //
                //     let group = groups.get(group_handle).unwrap();
                //     if let SelectionChoice::CurveSet(selected) = selected.clone() {
                //         for bezier_id in selected.iter() {
                //             let bez_ids = group.bezier_handles.iter().map(|x| x.id).collect();
                //             if bez_ids.contains(&bezier_id) {
                //                 if let Some(entity_handle) = maps.bezier_map.get(&bezier_id) {
                //                     commands.entity(entity_handle.entity).despawn_recursive();
                //                 }
                //             }
                //         }
                //     }
                // }

                // unlatch partners of deleted curves
                let mut unlatched_pairs: Vec<HashSet<BezierId>> = Vec::new();
                for (self_id, latch_data) in latched_partners {
                    //
                    // if let Some(latch) = latch_vec {
                    //
                    if let Some(handle_entity) = maps.bezier_map.get(&latch_data.latched_to_id) {
                        //
                        let partner_bezier = bezier_curves.get_mut(&handle_entity.handle).unwrap();

                        // important to send the Unlatched to history before the DeletedCurve

                        let mut unlatched_pair = HashSet::new();
                        unlatched_pair.insert(partner_bezier.id);
                        unlatched_pair.insert(self_id);

                        // send Unlatched only once per pair
                        // if !*is_from_redo && !unlatched_pairs.contains(&unlatched_pair) {
                        if !*is_from_redo {
                            unlatched_pairs.push(unlatched_pair);

                            // from the point of view of the deleted curve's partner

                            let unlatched = HistoryAction::Unlatched {
                                self_id: self_id.into(),
                                partner_id: latch_data.latched_to_id.into(),
                                self_anchor: latch_data.self_edge,
                                partner_anchor: latch_data.partners_edge,
                            };

                            // info!("unlatched: {:?}", unlatched);
                            add_to_history_event_writer.send(unlatched);
                        }
                        // info!("unlatching partner: {:?}", partner_bezier.id);

                        partner_bezier.latches.remove(&latch_data.partners_edge);
                    }

                    // maps.id_handle_map.remove(&latch_data.latched_to_id);
                    // }
                }

                // make the group box quad invisible
                for mut visible in visible_query.iter_mut() {
                    visible.is_visible = false;
                }

                // reset selection
                // selection.selected = SelectionChoice::None;
                // selection.selected.bezier_handles = HashSet::new();

                // send the delete events, provided they are not from a redo
                if !*is_from_redo {
                    for e in delete_curve_events.iter() {
                        add_to_history_event_writer.send(e.clone());
                    }
                }
            }
            selection.selected.clear();
        }
    }
}

pub fn hide_anchors(
    mut globals: ResMut<Globals>,
    mut query: Query<&mut Visibility, Or<(With<ControlPointQuad>, With<AchorEdgeQuad>)>>,
    mut action_event_reader: EventReader<Action>,
) {
    // if let Some(Action::HideAnchors) = action_event_reader.iter().next() {
    if action_event_reader
        .iter()
        .any(|x| x == &Action::HideAnchors)
    {
        globals.do_hide_anchors = !globals.do_hide_anchors;
        for mut visible in query.iter_mut() {
            visible.is_visible = !globals.do_hide_anchors;
        }
    }
}

pub fn hide_control_points(
    mut globals: ResMut<Globals>,
    mut query_control: Query<&mut Visibility, With<ControlPointQuad>>,
    mut action_event_reader: EventReader<Action>,
) {
    if action_event_reader
        .iter()
        .any(|x| x == &Action::HideControls)
    {
        globals.hide_control_points = !globals.hide_control_points;
        for mut visible in query_control.iter_mut() {
            visible.is_visible = !globals.hide_control_points;
        }
    }
}
