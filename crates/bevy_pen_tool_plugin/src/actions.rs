use bevy_pen_tool_model::inputs::{Action, Cursor};
use bevy_pen_tool_model::materials::*;
use bevy_pen_tool_model::model::*;
use bevy_pen_tool_model::spawn_bezier;

use bevy::prelude::*;

use std::collections::HashMap;
use std::collections::HashSet;

use std::fs::File;
use std::io::Read;
use std::io::Write;

pub fn update_lut(
    mut bezier_assets: ResMut<Assets<Bezier>>,
    bezier_handles: Query<&Handle<Bezier>, (With<MovingAnchor>, With<AchorEdgeQuad>)>,
    globals: ResMut<Globals>,
    mut groups: ResMut<Assets<Group>>,
    maps: ResMut<Maps>,
) {
    let mut groups_to_update = HashSet::new();
    let mut bezier_partners_to_update = HashSet::new();
    for b_handle in bezier_handles.iter() {
        if let Some(bezier) = bezier_assets.get_mut(b_handle) {
            bezier.compute_lut_walk(globals.group_lut_num_points as usize);

            for (_parter_anchor, latch) in bezier.latches.iter() {
                if let Some(handle) = maps.bezier_map.get(&latch.latched_to_id) {
                    bezier_partners_to_update.insert(&handle.handle);
                }
            }

            // if curve is part of a group, recompute the group lut
            if let Some(_id) = bezier.group {
                for (group_handle_id, group) in groups.iter_mut() {
                    // if _id == group_handle_id.id {
                    if group.bezier_handles.contains(b_handle) {
                        groups_to_update.insert(group_handle_id);
                    }
                }
            }

            if bezier.do_compute_lut {
                bezier_partners_to_update.insert(b_handle);
                bezier.do_compute_lut = false;
            }
        }
    }
    for handle in bezier_partners_to_update.iter() {
        if let Some(bezier_partner) = bezier_assets.get_mut(&handle) {
            bezier_partner.compute_lut_walk(globals.group_lut_num_points as usize);
            // groups_to_update.insert(handle);
        }
    }

    // if the moving anchor is part of a group,
    for group_id in groups_to_update.iter() {
        let group_handle = groups.get_handle(*group_id);
        let group = groups.get_mut(&group_handle).unwrap();
        group.group_lut(&mut bezier_assets, maps.bezier_map.clone());
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
    audio: Res<Audio>,
    mut add_to_history_event_writer: EventWriter<HistoryAction>,
) {
    let mut latch_partner: Option<(BezierId, LatchData)> = None;

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

            // unlatch mechanism
            if move_anchor.unlatch {
                // if curve does not belong to a group
                if let None = bezier.group {
                    match move_anchor.anchor {
                        anchor @ (Anchor::Start | Anchor::End) => {
                            if let Some(temp_latch) = bezier.latches.get(&anchor.to_edge()) {
                                // keep the latch information in memory to unlatch the anchor's partner below
                                latch_partner = Some((bezier.id, temp_latch.clone()));
                            }
                            bezier.latches.remove(&anchor.to_edge());
                            bezier.potential_latch = None;
                            // println!("Unlatched primary : {:?}", bezier.id);
                        }

                        _ => {}
                    }
                }
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

    // unlatch partner
    if let Some((self_id, latch)) = latch_partner {
        //
        if let Some(handle) = maps.bezier_map.get(&latch.latched_to_id) {
            //
            let bezier = bezier_curves.get_mut(&handle.handle).unwrap();
            //
            // if let Some(latch_local) = bezier.latches.get_mut(&latch.partners_edge) {
            if let Some(_) = bezier.latches.remove(&latch.partners_edge) {
                bezier.potential_latch = None;

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
        info!("selection_box_init");
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
pub fn selection_final(
    mut selection: ResMut<Selection>,
    cursor: ResMut<Cursor>,
    bezier_curves: ResMut<Assets<Bezier>>,
    groups: ResMut<Assets<Group>>,
    mut query_set: ParamSet<(
        Query<&mut Visibility, With<SelectingBoxQuad>>,
        Query<&mut Visibility, With<SelectedBoxQuad>>,
        Query<&mut Visibility, With<GroupBoxQuad>>,
    )>,
    group_query: Query<&Handle<Group>>,
    query: Query<(Entity, &Handle<Bezier>), With<BezierParent>>,
    mut action_event_reader: EventReader<Action>,
    globals: Res<Globals>,

    mut group_box_event_writer: EventWriter<GroupBoxEvent>,
) {
    if action_event_reader.iter().any(|x| x == &Action::Selected) {
        let mut selected = Group::default();

        let release_position = cursor.position;
        let last_click_position = cursor.last_click_position;

        // check for anchors inside selection area
        for (entity, bezier_handle) in query.iter() {
            let bezier = bezier_curves.get(bezier_handle).unwrap();
            let bs = bezier.positions.start * globals.scale;
            let be = bezier.positions.end * globals.scale;

            if (bs.x < last_click_position.x.max(release_position.x)
                && bs.x > last_click_position.x.min(release_position.x)
                && bs.y < last_click_position.y.max(release_position.y)
                && bs.y > last_click_position.y.min(release_position.y))
                || (be.x < last_click_position.x.max(release_position.x)
                    && be.x > last_click_position.x.min(release_position.x)
                    && be.y < last_click_position.y.max(release_position.y)
                    && be.y > last_click_position.y.min(release_position.y))
            {
                // if the selected quad is part of a group, show group selection and return
                // Cannot select more than one group
                // Cannot select a group and individual curves together
                for group_handle in group_query.iter() {
                    let group = groups.get(group_handle).unwrap();
                    //
                    if group.bezier_handles.contains(&bezier_handle) {
                        selected = group.clone();

                        for mut visible in query_set.p1().iter_mut() {
                            visible.is_visible = true;
                            // println!("visible!!!");
                        }

                        for mut visible in query_set.p2().iter_mut() {
                            visible.is_visible = true;
                        }
                        for mut visible_selecting in query_set.p0().iter_mut() {
                            visible_selecting.is_visible = false;
                        }
                        selection.selected = Some(selected);
                        // let us = user_state.as_mut();
                        // *us = UserState::Idle;

                        // send event to adjust_selection_attributes(..) so that the group selection
                        // box is updated and shows on screen.
                        group_box_event_writer.send(GroupBoxEvent);
                        return ();
                    }
                }

                selected
                    .group
                    .insert((entity.clone(), bezier_handle.clone()));

                selected.bezier_handles.insert(bezier_handle.clone());
            }
        }

        selection.selected = Some(selected);

        for mut visible_selected in query_set.p1().iter_mut() {
            visible_selected.is_visible = true;
        }

        for mut visible_group_box_quad in query_set.p2().iter_mut() {
            visible_group_box_quad.is_visible = true;
        }
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
        selection.selected = None;

        for mut visible in visible_selection_query.iter_mut() {
            visible.is_visible = false;
        }
    }
}

// group curves together to form a more complex path
pub fn groupy(
    mut commands: Commands,
    mut groups: ResMut<Assets<Group>>,
    globals: ResMut<Globals>,
    selection: ResMut<Selection>,
    mut maps: ResMut<Maps>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    mid_bezier_query: Query<(Entity, &Handle<Bezier>), With<MiddlePointQuad>>,
    group_query: Query<(Entity, &Handle<Group>), With<GroupParent>>,
    mut event_writer: EventWriter<Handle<Group>>,
    mut action_event_reader: EventReader<Action>,
    mut loaded_event_reader: EventReader<Loaded>,
    audio: Res<Audio>,
) {
    let mut do_group = false;
    let mut do_compute_lut = false;
    // group selected curves
    if action_event_reader.iter().any(|x| x == &Action::Group) {
        do_group = true;
        do_compute_lut = true;
    }

    // group loaded curves
    if let Some(Loaded) = loaded_event_reader.iter().next() {
        do_group = true;
    }

    if do_group {
        let id_handle_map: HashMap<BezierId, BezierHandleEntity> = maps.bezier_map.clone();

        if let Some(mut selected) = selection.selected.clone() {
            selected.find_connected_ends(&mut bezier_curves, id_handle_map.clone());
            // println!("connected ends: {:?}, ", selected.ends);

            // abort grouping if the selection is not completely connected with latches
            if selected.ends.is_none() {
                println!("Cannot group. Select multiple latched curves to successfully group");
                return;
            }

            // if the selected curves are already in a group, abort
            for bez_handle in selected.bezier_handles.iter() {
                let bez = bezier_curves.get(bez_handle).unwrap();
                if bez.group.is_some() {
                    println!("Cannot group. Selected curves are already in a group");
                    return;
                }
            }

            // get rid of the middle point quads
            for (entity, bezier_handle) in mid_bezier_query.iter() {
                if selected.bezier_handles.contains(bezier_handle) {
                    commands.entity(entity).despawn();
                }
            }

            if do_compute_lut {
                selected.group_lut(&mut bezier_curves, id_handle_map.clone());
                selected.compute_standalone_lut(&bezier_curves, globals.group_lut_num_points);
            }

            if globals.sound_on {
                if let Some(sound) = maps.sounds.get("group") {
                    audio.play(sound.clone());
                }
            }

            // TODO: we must get rid of this to have more than one group allowed.
            // get rid of the current group before making a new one
            for (entity, group_handle) in group_query.iter() {
                let group = groups.get(group_handle).unwrap();
                for bezier_handle in group.bezier_handles.clone() {
                    if selected.bezier_handles.contains(&bezier_handle) {
                        commands.entity(entity).despawn();
                        break;
                    }
                }
            }

            for bezier_handle in selected.bezier_handles.clone() {
                let bezier = bezier_curves.get_mut(&bezier_handle).unwrap();
                bezier.group = Some(selected.group_id);
            }

            let group_handle = groups.add(selected.clone());

            maps.group_map
                .insert(selected.group_id, group_handle.clone());

            // spawn the middle quads and the bounding box

            event_writer.send(group_handle.clone());
        }
    }
}

pub fn latchy(
    cursor: ResMut<Cursor>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    query: Query<(&Handle<Bezier>, &AchorEdgeQuad), With<MovingAnchor>>,

    globals: ResMut<Globals>,
    mut action_event_reader: EventReader<Action>,
    non_moving_edge_query: Query<(&Handle<Bezier>, &AchorEdgeQuad), Without<MovingAnchor>>,

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
        if let Some((partner_id, mover_anchor, pa_edge, mover_handle, partner_handle)) =
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
        }
    }
}

pub fn officiate_latch_partnership(
    mut bezier_curves: ResMut<Assets<Bezier>>,
    mut latch_event_reader: EventReader<OfficialLatch>,
    mut history_action_event_writer: EventWriter<HistoryAction>,
    globals: ResMut<Globals>,
    audio: Res<Audio>,
    maps: ResMut<Maps>,
) {
    for OfficialLatch(latch, bezier_handle) in latch_event_reader.iter() {
        let bezier_1 = bezier_curves.get_mut(bezier_handle).unwrap();

        bezier_1.latches.insert(latch.self_edge, latch.clone());
        bezier_1.compute_lut_walk(100); // TODO: is this useful? also, it should be dependent on a global var
        bezier_1.potential_latch = None;
        let bezier_1_id = bezier_1.id;

        //
        ///////////// partner //////////////////////////////////

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

        ///////////// partner //////////////////////////////////
        //

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
    }
}

pub fn ungroup(
    mut commands: Commands,
    mut groups: ResMut<Assets<Group>>,
    selection: ResMut<Selection>,
    globals: ResMut<Globals>,
    query: Query<(Entity, &Handle<Group>), With<GroupParent>>,
    bezier_query: Query<(Entity, &Handle<Bezier>, &Parent)>,
    mut maps: ResMut<Maps>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    mut action_event_reader: EventReader<Action>,
    mut spawn_mids_event_writer: EventWriter<SpawnMids>,
) {
    if action_event_reader.iter().any(|x| x == &Action::Ungroup) {
        // let group = &selection.selected;
        if let Some(selected) = selection.selected.clone() {
            let group_beziers = selected.bezier_handles.clone();

            if group_beziers.is_empty() {
                println!("Cannot ungroup. No curves selected");
                return;
            }

            let bezier_handles = group_beziers
                .iter()
                .cloned()
                .collect::<Vec<Handle<Bezier>>>();

            // Check if the handles are all connected
            // bezier_handles is never empty at this point
            let first_bezier_handle = bezier_handles.iter().next().unwrap();

            let first_bezier = bezier_curves.get(first_bezier_handle).unwrap();

            let mut bezier_chain =
                find_connected_curves(first_bezier.id, &bezier_curves, &maps.bezier_map);

            // first_bezier.find_connected_curves(bezier_curve_hack, &maps.bezier_map);

            bezier_chain.push(first_bezier_handle.clone());

            let bezier_chain_hashset = bezier_chain
                .iter()
                .cloned()
                .collect::<HashSet<Handle<Bezier>>>();

            // check if all curves are part of the same group
            for handle in bezier_chain.iter() {
                let bezier = bezier_curves.get(handle).unwrap();
                if let Some(group_id) = bezier.group {
                    if group_id != selected.group_id {
                        println!("Cannot ungroup. Not all curves are part of the same group");
                        return;
                    }
                }
            }

            // TODO: this is not needed right?
            if group_beziers != bezier_chain_hashset {
                println!("Cannot ungroup. Curves are not part of the same chain");
                return;
            }

            if let Some(id) = first_bezier.group {
                // println!("id: {:?}", id);
                // println!("maps.id_group_handle: {:?}", maps.id_group_handle.keys());
                if let Some(group_handle) = maps.group_map.get(&id) {
                    // remove With
                    let _what = groups.remove(group_handle);

                    for (entity, queried_group_handle) in query.iter() {
                        if queried_group_handle == group_handle {
                            commands.entity(entity).despawn_recursive();
                            println!("Removed group");
                        }
                    }
                } else {
                    info!("Cannot delete group: wrong group id.")
                }
                maps.group_map.remove(&id);
            }

            for bezier_handle in bezier_chain_hashset {
                let bezier = bezier_curves.get_mut(&bezier_handle).unwrap();

                bezier.group = None;

                for (_bez_entity, bez_handle, parent) in bezier_query.iter() {
                    if let Some(chain_bezier_handle) = maps.bezier_map.get(&bezier.id) {
                        if bez_handle == &chain_bezier_handle.handle {
                            // spawn mid quads
                            let spawn_mids = SpawnMids {
                                color: bezier
                                    .color
                                    .unwrap_or(globals.picked_color.unwrap_or(Color::WHITE)),
                                bezier_handle: bez_handle.clone(),
                                parent_entity: **parent,
                            };
                            // spawn bezier middle quads for each bezier
                            spawn_mids_event_writer.send(spawn_mids);

                            break;
                        }
                    }
                }
            }
        }
    }
}

pub fn delete(
    mut commands: Commands,
    mut selection: ResMut<Selection>,
    mut maps: ResMut<Maps>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    groups: ResMut<Assets<Group>>,
    mut visible_query: Query<&mut Visibility, With<SelectedBoxQuad>>,
    query: Query<&Handle<Bezier>, With<BezierParent>>,
    query2: Query<&Handle<Group>, With<GroupParent>>, // TODO: change to GroupParent
    mut action_event_reader: EventReader<Action>,
    mut add_to_history_event_writer: EventWriter<HistoryAction>,
) {
    // if action_event_reader.iter().any(|x| x == &Action::Delete) {
    for action in action_event_reader.iter() {
        if let Action::Delete(is_from_redo) = action {
            // list of partners that need to be unlatched
            let mut delete_curve_events = Vec::new();

            let mut latched_partners: Vec<(BezierId, LatchData)> = Vec::new();
            for bezier_handle in query.iter() {
                //
                if let Some(selected) = selection.selected.clone() {
                    for (entity, handle) in selected.group {
                        //
                        let bezier = bezier_curves.get_mut(&handle.clone()).unwrap();
                        // println!("within DELETE ---> bezier: {:?}", bezier.id);

                        // latched_partners.push(bezier.latches[&AnchorEdge::Start].clone());
                        if let Some(latched_anchor) = bezier.latches.get(&AnchorEdge::Start) {
                            latched_partners.push((bezier.id, latched_anchor.clone()));
                        }

                        // latched_partners.push(bezier.latches[&AnchorEdge::End].clone());
                        if let Some(latched_anchor) = bezier.latches.get(&AnchorEdge::End) {
                            latched_partners.push((bezier.id, latched_anchor.clone()));
                        }

                        if &handle == bezier_handle {
                            delete_curve_events.push(HistoryAction::DeletedCurve {
                                bezier: BezierHist::from(&bezier.clone()),
                                bezier_id: bezier.id.into(),
                            });

                            commands.entity(entity).despawn_recursive();
                            maps.bezier_map.remove(&bezier.id);
                            if let Some(group_id) = bezier.group {
                                maps.group_map.remove(&group_id);
                            }
                        }
                    }
                }
            }

            for group_handle in query2.iter() {
                //
                let group = groups.get(group_handle).unwrap();
                if let Some(selected) = selection.selected.clone() {
                    for (entity, bezier_handle) in selected.group {
                        if group.bezier_handles.contains(&bezier_handle) {
                            commands.entity(entity).despawn_recursive();
                        }
                    }
                }
            }

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
                    info!("unlatching partner: {:?}", partner_bezier.id);

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
            selection.selected = None;
            // selection.selected.bezier_handles = HashSet::new();

            // send the delete events, provided they are not from a redo
            if !*is_from_redo {
                for e in delete_curve_events.iter() {
                    add_to_history_event_writer.send(e.clone());
                }
            }
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

pub fn save(
    mut bezier_curves: ResMut<Assets<Bezier>>,
    group_query: Query<&Handle<Group>, With<GroupParent>>,
    mesh_query: Query<(&Handle<Mesh>, &GroupMesh)>,
    road_mesh_query: Query<(&Handle<Mesh>, &RoadMesh)>,
    mut groups: ResMut<Assets<Group>>,
    meshes: Res<Assets<Mesh>>,
    globals: ResMut<Globals>,
    mut action_event_reader: EventReader<Action>,
) {
    if action_event_reader.iter().any(|x| x == &Action::Save) {
        //

        //
        // ////////////// start.  Save individual Bezier curves
        // let mut vec: Vec<Bezier> = Vec::new();
        // for bezier_handle in query.iter() {
        //     let bezier = bezier_curves.get(bezier_handle).unwrap();
        //     let mut bezier_clone = bezier.clone();
        //     bezier_clone.lut = Vec::new();
        //     vec.push(bezier_clone);
        // }

        // let serialized = serde_json::to_string_pretty(&vec).unwrap();

        // let path = "curves.txt";
        // let mut output = File::create(path).unwrap();
        // let _result = output.write(serialized.as_bytes());
        // ////////////// end.  Save individual Bezier curves
        //

        ////////////// start. Save Group and save Group look-up table
        if let Some(group_handle) = group_query.iter().next() {
            let mut group_vec = Vec::new();
            // for group_handle in group_query.iter() {
            let group = groups.get_mut(group_handle).unwrap();
            //
            ////////////// start. Save Group look-up table
            let lut_dialog_result = open_file_dialog("my_group", "look_up_tables", ".lut");
            if let Some(lut_path) = lut_dialog_result {
                group.compute_standalone_lut(&mut bezier_curves, globals.group_lut_num_points);
                let lut_serialized = serde_json::to_string_pretty(&group.standalone_lut).unwrap();
                // let lut_path = "assets/lut/my_group_lut.txt";
                let mut lut_output = File::create(&lut_path).unwrap();
                let _lut_write_result = lut_output.write(lut_serialized.as_bytes());
            }

            ////////////// start. Save Group
            let group_dialog_result = open_file_dialog("my_group", "groups", ".group");
            if let Some(group_path) = group_dialog_result {
                group_vec.push(group.into_group_save(&mut bezier_curves).clone());
                // }

                let serialized = serde_json::to_string_pretty(&group_vec).unwrap();

                // let path = "curve_groups.txt";
                let mut output = File::create(group_path).unwrap();
                let _group_write_result = output.write(serialized.as_bytes());
            }
        }
        ////////////// end. Save group and look-up table
        //

        ////////////// start. Save mesh in obj format
        if let Some((mesh_handle, GroupMesh(_color))) = mesh_query.iter().next() {
            let mesh_dialog_result = open_file_dialog("my_mesh", "meshes", ".obj");
            save_mesh(mesh_handle, &meshes, mesh_dialog_result);

            ////////////// end. Save mesh in obj format
        }

        ////////////// start. Save road in obj format
        if let Some((road_mesh_handle, RoadMesh(_color))) = road_mesh_query.iter().next() {
            let road_dialog_result = open_file_dialog("my_road", "meshes", ".obj");
            save_mesh(road_mesh_handle, &meshes, road_dialog_result);

            ////////////// end. Save road in obj format
        }
    }
}

// only loads groups

pub fn load(
    query: Query<Entity, Or<(With<BezierParent>, With<GroupParent>)>>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    // mut groups: ResMut<Assets<Group>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    // mut my_shader_params: ResMut<Assets<BezierMat>>,
    clearcolor_struct: Res<ClearColor>,
    mut globals: ResMut<Globals>,
    mut selection: ResMut<Selection>,
    mut maps: ResMut<Maps>,
    mut action_event_reader: EventReader<Action>,
    mut loaded_event_writer: EventWriter<Loaded>,
    mut selection_params: ResMut<Assets<SelectionMat>>,
    mut controls_params: ResMut<Assets<BezierControlsMat>>,
    mut ends_params: ResMut<Assets<BezierEndsMat>>,
    mut mid_params: ResMut<Assets<BezierMidMat>>,
    mut add_to_history_event_writer: EventWriter<HistoryAction>,
) {
    if action_event_reader.iter().any(|x| x == &Action::Load) {
        let mut default_path = std::env::current_dir().unwrap();
        default_path.push("saved");
        default_path.push("groups");

        let res = rfd::FileDialog::new()
            .add_filter("text", &["group"])
            .set_directory(&default_path)
            .pick_files();

        // cancel loading if user cancelled the file dialog
        let path: std::path::PathBuf;
        if let Some(chosen_path) = res.clone() {
            let path_some = chosen_path.get(0);
            if let Some(path_local) = path_some {
                path = path_local.clone();
            } else {
                return ();
            }
        } else {
            return ();
        }

        let clearcolor = clearcolor_struct.0;

        // delete all current groups and curves before spawning the saved ones
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        globals.do_hide_anchors = false;
        globals.do_hide_bounding_boxes = true;

        let mut file = std::fs::File::open(path).unwrap();

        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        let loaded_groups_vec: Vec<GroupSaveLoad> = serde_json::from_str(&contents).unwrap();

        let id: GroupId = GroupId::default();

        let mut group = Group {
            group: HashSet::new(),
            bezier_handles: HashSet::new(),
            lut: Vec::new(),
            ends: None,
            standalone_lut: StandaloneLut {
                path_length: 0.0,
                lut: Vec::new(),
            },
            group_id: id,
        };

        for group_load_save in loaded_groups_vec {
            for (mut bezier, anchor, t_ends, local_lut) in group_load_save.lut {
                let (entity, handle) = spawn_bezier(
                    &mut bezier,
                    &mut bezier_curves,
                    &mut commands,
                    &mut meshes,
                    &mut selection_params,
                    &mut controls_params,
                    &mut ends_params,
                    &mut mid_params,
                    clearcolor,
                    &mut globals,
                    &mut maps,
                    &mut add_to_history_event_writer,
                    &None, // does not have handle information
                    true,  // do send to history
                    false, // do not follow mouse
                );
                group.group.insert((entity.clone(), handle.clone()));
                group.bezier_handles.insert(handle.clone());
                group.standalone_lut = group_load_save.standalone_lut.clone();
                group.lut.push((handle.clone(), anchor, t_ends, local_lut));
            }
        }
        selection.selected = Some(group.clone());

        // to create a group: select all the curves programmatically, and send a UiButton::Group event
        loaded_event_writer.send(Loaded);
        println!("{:?}", "loaded groups");
    }
}
