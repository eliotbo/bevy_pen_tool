use super::buttons::{ButtonState, UiButton};
use super::inputs::{Action, Cursor, MoveAnchor};
use crate::spawner::spawn_bezier;
use crate::GroupMiddleQuad;

use crate::util::{
    compute_lut, compute_lut_long, get_close_anchor, get_close_anchor_entity,
    get_close_still_anchor, Anchor, AnchorEdge, Bezier, BoundingBoxQuad, ControlPointQuad,
    EndpointQuad, Globals, GrandParent, Group, GroupBoxQuad, GroupSaveLoad, LatchData, Loaded,
    MiddlePointQuad, MyShader, OfficialLatch, SelectedBoxQuad, SelectingBoxQuad, UiAction, UiBoard,
    UserState,
};

use bevy::prelude::*;

// use rand::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;
// use std::ops::DerefMut;

use std::fs::File;
use std::io::Read;
use std::io::Write;

pub fn recompute_lut(
    // keyboard_input: Res<Input<KeyCode>>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    mut query: Query<&Handle<Bezier>, With<BoundingBoxQuad>>,
    query_group: Query<&Handle<Group>>,
    mut groups: ResMut<Assets<Group>>,
    // mut ui_event_reader: EventReader<UiButton>,
    mut action_event_reader: EventReader<Action>,
    globals: ResMut<Globals>,
    time: Res<Time>,
) {
    if let Some(Action::ComputeLut) = action_event_reader.iter().next() {
        for bezier_handle in query.iter_mut() {
            let mut bezier = bezier_curves.get_mut(bezier_handle).unwrap();

            let bezier_c = bezier.to_coord2();

            // this import is heavy, but from_points() does not work without importing everything
            use flo_curves::*;
            let curve = flo_curves::bezier::Curve::from_points(
                bezier_c.start,
                bezier_c.control_points,
                bezier_c.end,
            );

            let lut_option =
                compute_lut_long(curve, globals.group_lut_num_points as usize, time.clone());
            if let Some(lut) = lut_option {
                bezier.lut = lut;
            } else {
                bezier.lut = compute_lut(curve, globals.group_lut_num_points as usize);
            }

            bezier.do_compute_lut = false;

            for group_handle in query_group.iter() {
                let group = groups.get_mut(group_handle).unwrap();
                if group.handles.contains(bezier_handle) {
                    let id_handle_map = globals.id_handle_map.clone();
                    group.group_lut(&mut bezier_curves, id_handle_map);
                    group.compute_standalone_lut(&bezier_curves, globals.group_lut_num_points);
                }
            }
        }
    }
}

// unlatch is part of this function
pub fn begin_move_on_mouseclick(
    mut bezier_curves: ResMut<Assets<Bezier>>,
    globals: ResMut<Globals>,
    mut move_event_reader: EventReader<MoveAnchor>,
    audio: Res<Audio>,
) {
    let mut latch_partners: Vec<LatchData> = Vec::new();

    if let Some(move_anchor) = move_event_reader.iter().next() {
        let mut bezier = bezier_curves.get_mut(move_anchor.handle.clone()).unwrap();

        let chose_a_control_point =
            move_anchor.anchor == Anchor::ControlStart || move_anchor.anchor == Anchor::ControlEnd;
        let hidden_controls = globals.hide_control_points;

        // order to move the quad that was clicked on
        if move_anchor.anchor != Anchor::None && !(chose_a_control_point && hidden_controls) {
            bezier.move_quad = move_anchor.anchor;

            bezier.update_previous_pos();
        }

        // unlatch mechanism
        if move_anchor.unlatch {
            if !bezier.grouped {
                match move_anchor.anchor {
                    Anchor::Start => {
                        // keep the latch information in memory to unlatch the anchor's partner below
                        latch_partners = bezier.latches[&AnchorEdge::Start].clone();
                        if let Some(latch) = bezier.latches.get_mut(&AnchorEdge::Start) {
                            *latch = Vec::new();
                        }
                    }
                    Anchor::End => {
                        latch_partners = bezier.latches[&AnchorEdge::End].clone();
                        // bezier.latches[&AnchorEdge::End] = Vec::new();
                        if let Some(latch) = bezier.latches.get_mut(&AnchorEdge::End) {
                            *latch = Vec::new();
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // unlatch partner
    if let Some(latch) = latch_partners.get(0) {
        //
        if let Some(handle) = globals.id_handle_map.get(&latch.latched_to_id) {
            //
            let bezier = bezier_curves.get_mut(handle).unwrap();
            //
            if let Some(latch_local) = bezier.latches.get_mut(&latch.partners_edge) {
                *latch_local = Vec::new();
                if globals.sound_on {
                    if let Some(sound) = globals.sounds.get("unlatch") {
                        audio.play(sound.clone());
                    }
                }
            }
        }
    }
}

pub fn selection(
    mut globals: ResMut<Globals>,
    cursor: ResMut<Cursor>,
    bezier_curves: ResMut<Assets<Bezier>>,
    groups: ResMut<Assets<Group>>,
    mut visible_selection_query: Query<&mut Visible, With<SelectedBoxQuad>>,
    group_query: Query<&Handle<Group>>,
    query: Query<(Entity, &Handle<Bezier>), With<BoundingBoxQuad>>,
    mut action_event_reader: EventReader<Action>,
) {
    if let Some(Action::Select) = action_event_reader.iter().next() {
        println!("select");
        if let Some((_distance, _anchor, entity, selected_handle)) = get_close_anchor_entity(
            2.0 * globals.scale,
            cursor.position,
            &bezier_curves,
            &query,
            globals.scale,
        ) {
            // if the selected quad is part of a group, show group selection
            for group_handle in group_query.iter() {
                let group = groups.get(group_handle).unwrap();
                //
                if group.handles.contains(&selected_handle) {
                    globals.selected = group.clone();
                    for mut visible in visible_selection_query.iter_mut() {
                        visible.is_visible = true;
                    }

                    return ();
                }
            }

            let selected_entity = entity.clone();

            // add the selected quad to the selected group
            globals
                .selected
                .group
                .insert((selected_entity.clone(), selected_handle.clone()));

            globals.selected.handles.insert(selected_handle.clone());

            // these will be computed when a group order has been emitted
            globals.selected.ends = None;
            globals.selected.lut = Vec::new();

            for mut visible in visible_selection_query.iter_mut() {
                visible.is_visible = true;
            }
            // println!("selectd: {:?}", &globals.selected);
        }
    }
}

pub fn selection_box_init(
    globals: ResMut<Globals>,
    mut user_state: ResMut<UserState>,
    cursor: ResMut<Cursor>,
    bezier_curves: ResMut<Assets<Bezier>>,
    query: Query<(Entity, &Handle<Bezier>), With<BoundingBoxQuad>>,
    mut action_event_reader: EventReader<Action>,
    mut visible_selection_query: Query<&mut Visible, With<SelectingBoxQuad>>,
) {
    if let Some(Action::SelectionBox) = action_event_reader.iter().next() {
        if let Some((_distance, _anchor, _entity, _selected_handle)) = get_close_anchor_entity(
            2.0 * globals.scale,
            cursor.position,
            &bezier_curves,
            &query,
            globals.scale,
        ) {
        } else {
            println!("select_box_init");
            let us = user_state.as_mut();
            *us = UserState::Selecting(cursor.position);
            println!("changed UserState to Selecting");

            for mut visible in visible_selection_query.iter_mut() {
                visible.is_visible = true;
            }
        }
    }
}

pub fn selection_final(
    mut globals: ResMut<Globals>,
    mut user_state: ResMut<UserState>,
    cursor: ResMut<Cursor>,
    bezier_curves: ResMut<Assets<Bezier>>,
    groups: ResMut<Assets<Group>>,
    // mut visible_selecting_query: Query<&mut Visible, With<SelectingBoxQuad>>,
    // mut visible_selected_query: Query<&mut Visible, With<SelectedBoxQuad>>,
    mut query_set: QuerySet<(
        QueryState<&mut Visible, With<SelectingBoxQuad>>,
        QueryState<&mut Visible, With<SelectedBoxQuad>>,
    )>,
    group_query: Query<&Handle<Group>>,
    query: Query<(Entity, &Handle<Bezier>), With<BoundingBoxQuad>>,
    mut action_event_reader: EventReader<Action>,
) {
    if let Some(Action::Selected) = action_event_reader.iter().next() {
        println!("select_box_final");

        // let mut changed_user_state = false;
        let mut selected = Group::default();
        if let UserState::Selecting(click_position) = user_state.as_ref() {
            // changed_user_state = true;
            let release_position = cursor.position;

            // check for anchors inside selection area
            for (entity, bezier_handle) in query.iter() {
                let bezier = bezier_curves.get(bezier_handle).unwrap();
                let bs = bezier.positions.start;
                let be = bezier.positions.end;
                if (bs.x < click_position.x.max(release_position.x)
                    && bs.x > click_position.x.min(release_position.x)
                    && bs.y < click_position.y.max(release_position.y)
                    && bs.y > click_position.y.min(release_position.y))
                    || (be.x < click_position.x.max(release_position.x)
                        && be.x > click_position.x.min(release_position.x)
                        && be.y < click_position.y.max(release_position.y)
                        && be.y > click_position.y.min(release_position.y))
                {
                    // if the selected quad is part of a group, show group selection and return
                    // Cannot select more than one group
                    // Cannot select a group and individual curves together
                    for group_handle in group_query.iter() {
                        let group = groups.get(group_handle).unwrap();
                        //
                        if group.handles.contains(&bezier_handle) {
                            selected = group.clone();
                            for mut visible in query_set.q0().iter_mut() {
                                visible.is_visible = true;
                            }
                            for mut visible_selecting in query_set.q0().iter_mut() {
                                visible_selecting.is_visible = false;
                            }
                            globals.selected = selected;
                            let us = user_state.as_mut();
                            *us = UserState::Idle;
                            return ();
                        }
                    }

                    selected
                        .group
                        .insert((entity.clone(), bezier_handle.clone()));
                    selected.handles.insert(bezier_handle.clone());
                }
            }
            globals.selected = selected;
            println!("selected: {:?}", globals.selected);
        }
        let us = user_state.as_mut();
        *us = UserState::Idle;

        for mut visible_selected in query_set.q1().iter_mut() {
            visible_selected.is_visible = true;
        }
        for mut visible_selecting in query_set.q0().iter_mut() {
            visible_selecting.is_visible = false;
        }
    }
}

pub fn unselect(
    mut globals: ResMut<Globals>,
    mut visible_selection_query: Query<&mut Visible, With<SelectedBoxQuad>>,
    mut action_event_reader: EventReader<Action>,
) {
    if let Some(Action::Unselect) = action_event_reader.iter().next() {
        globals.selected.group = HashSet::new();
        globals.selected.handles = HashSet::new();
        globals.selected.ends = None;
        globals.selected.lut = Vec::new();

        for mut visible in visible_selection_query.iter_mut() {
            visible.is_visible = false;
        }
    }
}

pub fn groupy(
    mut commands: Commands,
    mut groups: ResMut<Assets<Group>>,
    globals: ResMut<Globals>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    query: Query<(Entity, &Handle<Bezier>), With<MiddlePointQuad>>,
    group_query: Query<(Entity, &Handle<Group>), Or<(With<GroupBoxQuad>, With<GroupMiddleQuad>)>>,
    mut event_writer: EventWriter<Handle<Group>>,
    mut action_event_reader: EventReader<Action>,
    mut loaded_event_reader: EventReader<Loaded>,
    audio: Res<Audio>,
) {
    let mut do_group = false;
    let mut do_compute_lut = false;
    // group selected curves
    if let Some(Action::Group) = action_event_reader.iter().next() {
        do_group = true;
        do_compute_lut = true;
    }

    // group loaded curves
    if let Some(Loaded) = loaded_event_reader.iter().next() {
        do_group = true;
    }

    if do_group {
        let id_handle_map: HashMap<u128, Handle<Bezier>> = globals.id_handle_map.clone();

        let mut selected = globals.selected.clone();

        selected.find_connected_ends(&mut bezier_curves, id_handle_map.clone());
        // println!("connected ends: {:?}, ", selected.ends);

        // abort grouping if the selection is not completely connected with latches
        if selected.ends.is_none() {
            println!("Cannot group. Select multiple latched curves to successfully group");
            return;
        }

        if globals.sound_on {
            if let Some(sound) = globals.sounds.get("group") {
                audio.play(sound.clone());
            }
        }

        if do_compute_lut {
            selected.group_lut(&mut bezier_curves, id_handle_map.clone());
            selected.compute_standalone_lut(&bezier_curves, globals.group_lut_num_points);
        }

        // get rid of the middle point quads
        for (entity, bezier_handle) in query.iter() {
            if selected.handles.contains(bezier_handle) {
                commands.entity(entity).despawn();
            }
        }

        // get rid of the current group before making a new one
        for (entity, group_handle) in group_query.iter() {
            let group = groups.get(group_handle).unwrap();
            for bezier_handle in group.handles.clone() {
                if selected.handles.contains(&bezier_handle) {
                    commands.entity(entity).despawn();
                    break;
                }
            }
        }

        for bezier_handle in selected.handles.clone() {
            let bezier = bezier_curves.get_mut(bezier_handle).unwrap();
            bezier.grouped = true;
        }

        let group_handle = groups.add(selected);

        // spawn the middle quads and the bounding box

        event_writer.send(group_handle.clone());
    }
}

// pub fn ungroup(
//     mut commands: Commands,
//     mut groups: ResMut<Assets<Group>>,
//     globals: ResMut<Globals>,
//     keyboard_input: Res<Input<KeyCode>>,
//     mut bezier_curves: ResMut<Assets<Bezier>>,
//     query: Query<(Entity, &Handle<Bezier>), With<MiddlePointQuad>>,
//     mut ui_event_reader: EventReader<UiButton>,
//     mut group_event_reader: EventReader<Group>,
//     mut event_writer: EventWriter<Handle<Group>>,
//     audio: Res<Audio>,
// ) {
//     if !globals.selected.group.is_empty()
//         && (keyboard_input.pressed(KeyCode::LControl)
//             && keyboard_input.just_pressed(KeyCode::G)
//             && keyboard_input.pressed(KeyCode::LShift)
//             && !keyboard_input.pressed(KeyCode::Space))
//     {
//         let mut selected = globals.selected.clone();
//         for bezier_handle in selected.handles {
//             for (idx, group) in groups.clone().iter().enumerate() {
//                 for handle_in_group in group.handles {

//                 }
//             }
//         }
//     }
// }

pub fn delete(
    // keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut globals: ResMut<Globals>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    groups: ResMut<Assets<Group>>,
    mut visible_query: Query<&mut Visible, With<SelectedBoxQuad>>,
    query: Query<(Entity, &Handle<Bezier>), With<BoundingBoxQuad>>,
    query2: Query<(Entity, &Handle<Group>), With<GroupBoxQuad>>,
    mut action_event_reader: EventReader<Action>,
) {
    // if keyboard_input.pressed(KeyCode::Delete) {
    if let Some(Action::Delete) = action_event_reader.iter().next() {
        // println!("{:?}", globals.selected.clone());

        // list of partners that need to be unlatched
        let mut latched_partners: Vec<Vec<LatchData>> = Vec::new();
        for (entity, bezier_handle) in query.iter() {
            //
            for (_entity_selected, handle) in globals.selected.group.clone() {
                //
                let bezier = bezier_curves.get_mut(handle.clone()).unwrap();

                latched_partners.push(bezier.latches[&AnchorEdge::Start].clone());
                latched_partners.push(bezier.latches[&AnchorEdge::End].clone());

                if &handle == bezier_handle {
                    commands.entity(entity).despawn_recursive();
                }
            }
        }

        for (entity, group_handle) in query2.iter() {
            //
            let group = groups.get(group_handle).unwrap();
            for (_entity_selected, bezier_handle) in globals.selected.group.clone() {
                if group.handles.contains(&bezier_handle) {
                    commands.entity(entity).despawn_recursive();
                }
            }
        }

        // unlatch partners of deleted curves
        for latch_vec in latched_partners {
            //
            if let Some(latch) = latch_vec.get(0) {
                //
                if let Some(handle) = globals.id_handle_map.get(&latch.latched_to_id) {
                    //
                    let bezier = bezier_curves.get_mut(handle).unwrap();

                    if let Some(latch_local) = bezier.latches.get_mut(&latch.partners_edge) {
                        // println!("selectd: {:?}", &latch_local);
                        *latch_local = Vec::new();
                        println!("deleted partner latches");
                    }
                }
            }
        }

        // make the group box quad invisible
        for mut visible in visible_query.iter_mut() {
            visible.is_visible = false;
        }

        // reset selection
        globals.selected.group = HashSet::new();
        globals.selected.handles = HashSet::new();
    }
}

// pub fn undo(
//     keyboard_input: Res<Input<KeyCode>>,
//     mut commands: Commands,
//     mut globals: ResMut<Globals>,
//     mut bezier_curves: ResMut<Assets<Bezier>>,
//     mut event_reader: EventReader<UiButton>,
//     query: Query<(Entity, &Handle<Bezier>), With<BoundingBoxQuad>>,
// ) {
//     let mut pressed_undo_button = false;
//     for ui_button in event_reader.iter() {
//         pressed_undo_button = ui_button == &UiButton::Undo;
//         break;
//     }

//     if pressed_undo_button
//         || (keyboard_input.pressed(KeyCode::LControl)
//             && keyboard_input.just_pressed(KeyCode::Z)
//             && !keyboard_input.pressed(KeyCode::LShift))
//     {
//         // let mut latched_start: Vec<LatchData> = Vec::new();
//         // let mut latched_end: Vec<LatchData> = Vec::new();
//         let mut latches: Vec<Vec<LatchData>> = Vec::new();

//         if let Some((entity, bezier_handle)) = query.iter().last() {
//             globals.history.push(bezier_handle.clone());

//             let bezier = bezier_curves.get(bezier_handle).unwrap();
//             latches.push(bezier.latches[&AnchorEdge::Start].clone());
//             latches.push(bezier.latches[&AnchorEdge::End].clone());

//             commands.entity(entity).despawn_recursive();
//         }

//         // This piece of code is shared with delete()
//         // unlatch partners of deleted curves
//         for latch_vec in latches {
//             //
//             if let Some(latch) = latch_vec.get(0) {
//                 //
//                 if let Some(handle) = globals.id_handle_map.get(&latch.latched_to_id) {
//                     //
//                     let bezier_partner = bezier_curves.get_mut(handle).unwrap();
//                     //
//                     if let Some(latch_local) = bezier_partner.latches.get_mut(&latch.partners_edge)
//                     {
//                         *latch_local = Vec::new();
//                         println!("deleted partner's latch {:?}", latch.partners_edge);
//                     }
//                 }
//             }
//         }
//     }
// }

// // Warning: undo followed by redo does not preserve the latch data
// // spawn_bezier() does not allow the end point to be latched
// pub fn redo(
//     keyboard_input: Res<Input<KeyCode>>,
//     mut bezier_curves: ResMut<Assets<Bezier>>,
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     // mut pipelines: ResMut<Assets<PipelineDescriptor>>,
//     mut my_shader_params: ResMut<Assets<MyShader>>,
//     clearcolor_struct: Res<ClearColor>,
//     mut globals: ResMut<Globals>,
//     mut event_reader: EventReader<UiButton>,
// ) {
//     let mut pressed_redo_button = false;
//     for ui_button in event_reader.iter() {
//         pressed_redo_button = ui_button == &UiButton::Redo;
//         break;
//     }

//     if pressed_redo_button
//         || (keyboard_input.pressed(KeyCode::LControl)
//             && keyboard_input.just_pressed(KeyCode::Z)
//             && keyboard_input.pressed(KeyCode::LShift))
//     {
//         let clearcolor = clearcolor_struct.0;
//         let length = globals.history.len();
//         let mut should_remove_last_from_history = false;
//         if let Some(bezier_handle) = globals.history.last() {
//             should_remove_last_from_history = true;
//             let mut bezier = bezier_curves.get_mut(bezier_handle).unwrap().clone();
//             bezier_curves.remove(bezier_handle);
//             globals.do_spawn_curve = false;
//             // println!("{:?}", bezier.color);

//             spawn_bezier(
//                 &mut bezier,
//                 &mut bezier_curves,
//                 &mut commands,
//                 &mut meshes,
//                 // &mut pipelines,
//                 &mut my_shader_params,
//                 clearcolor,
//                 &mut globals,
//             );
//         }

//         if should_remove_last_from_history {
//             globals.history.swap_remove(length - 1);
//         }
//     }
// }

pub fn hide_anchors(
    mut globals: ResMut<Globals>,
    mut query: Query<&mut Visible, Or<(With<ControlPointQuad>, With<EndpointQuad>)>>,
    mut action_event_reader: EventReader<Action>,
) {
    if let Some(Action::HideAnchors) = action_event_reader.iter().next() {
        globals.do_hide_anchors = !globals.do_hide_anchors;
        for mut visible in query.iter_mut() {
            visible.is_visible = !globals.do_hide_anchors;
        }
    }
}

pub fn hide_control_points(
    mut globals: ResMut<Globals>,
    mut query_control: Query<&mut Visible, With<ControlPointQuad>>,
    mut action_event_reader: EventReader<Action>,
) {
    if let Some(Action::HideControls) = action_event_reader.iter().next() {
        globals.hide_control_points = !globals.hide_control_points;
        for mut visible in query_control.iter_mut() {
            visible.is_visible = !globals.hide_control_points;
        }
    }
}

// pub fn toggle_sound(
//     // asset_server: Res<AssetServer>,
//     mut globals: ResMut<Globals>,
//     // mut materials: ResMut<Assets<ColorMaterial>>,
//     mut query: Query<(&mut Handle<ColorMaterial>, &mut OnOffMaterial)>,
//     mut event_reader: EventReader<UiButton>,
// ) {
//     for ui_button in event_reader.iter() {
//         //
//         if ui_button == &UiButton::Sound {
//             //
//             globals.sound_on = !globals.sound_on;
//             //
//             for (mut material_handle, mut on_off_mat) in query.iter_mut() {
//                 // toggle sprite
//                 use std::ops::DerefMut;
//                 let other_material = on_off_mat.material.clone();
//                 let current_material = material_handle.clone();
//                 let mat = material_handle.deref_mut();
//                 *mat = other_material.clone();
//                 on_off_mat.material = current_material;
//             }
//         }
//     }
// }

pub fn save(
    // keyboard_input: Res<Input<KeyCode>>,
    query: Query<&Handle<Bezier>, With<BoundingBoxQuad>>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    group_query: Query<&Handle<Group>, With<GroupBoxQuad>>,
    mut groups: ResMut<Assets<Group>>,
    // mut event_reader: EventReader<UiButton>,
    globals: ResMut<Globals>,
    mut action_event_reader: EventReader<Action>,
) {
    if let Some(Action::Save) = action_event_reader.iter().next() {
        let mut vec: Vec<Bezier> = Vec::new();
        for bezier_handle in query.iter() {
            let bezier = bezier_curves.get(bezier_handle).unwrap();
            let mut bezier_clone = bezier.clone();
            bezier_clone.lut = Vec::new();
            vec.push(bezier_clone);
        }

        let serialized = serde_json::to_string_pretty(&vec).unwrap();

        let path = "curves.txt";
        let mut output = File::create(path).unwrap();
        let _result = output.write(serialized.as_bytes());

        println!("{:?}", "saved individual Bezier curves");

        let mut group_vec = Vec::new();
        for group_handle in group_query.iter() {
            let group = groups.get_mut(group_handle).unwrap();

            group.compute_standalone_lut(&mut bezier_curves, globals.group_lut_num_points);
            let lut_serialized = serde_json::to_string_pretty(&group.standalone_lut).unwrap();
            let lut_path = "group_lut.txt";
            let mut lut_output = File::create(lut_path).unwrap();
            let _lut_result = lut_output.write(lut_serialized.as_bytes());

            group_vec.push(group.into_group_save(&mut bezier_curves).clone());
        }

        let serialized = serde_json::to_string_pretty(&group_vec).unwrap();

        let path = "curve_groups.txt";
        let mut output = File::create(path).unwrap();
        let _result = output.write(serialized.as_bytes());

        println!("{:?}", "saved groups");
    }
}

pub fn load(
    // keyboard_input: Res<Input<KeyCode>>,
    query: Query<Entity, Or<(With<BoundingBoxQuad>, With<GroupBoxQuad>)>>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    // mut groups: ResMut<Assets<Group>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    // mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut my_shader_params: ResMut<Assets<MyShader>>,
    clearcolor_struct: Res<ClearColor>,
    mut globals: ResMut<Globals>,
    // mut event_reader: EventReader<UiButton>,
    mut event_writer: EventWriter<Group>,
    mut action_event_reader: EventReader<Action>,
    mut loaded_event_writer: EventWriter<Loaded>,
    // mut query: Query<(Entity, &Handle<Bezier>), With<BoundingBoxQuad>>,
) {
    if let Some(Action::Load) = action_event_reader.iter().next() {
        let clearcolor = clearcolor_struct.0;

        let path = "bezier.txt";
        let mut file = std::fs::File::open(path).unwrap();

        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        // delete all current groups and curves before spawning the saved ones
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        globals.history = Vec::new();
        globals.do_hide_anchors = false;
        globals.do_hide_bounding_boxes = true;

        // let loaded_bezier_vec: Vec<Bezier> = serde_json::from_str(&contents).unwrap();
        // for mut bezier in loaded_bezier_vec {
        //     spawn_bezier(
        //         &mut bezier,
        //         &mut bezier_curves,
        //         &mut commands,
        //         &mut meshes,
        //         // &mut pipelines,
        //         &mut my_shader_params,
        //         clearcolor,
        //         &mut globals,
        //     )
        // }
        // println!("{:?}", "loaded Bezier curves");

        let path = "curve_groups.txt";
        let mut file = std::fs::File::open(path).unwrap();

        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let loaded_groups_vec: Vec<GroupSaveLoad> = serde_json::from_str(&contents).unwrap();

        let mut group = Group {
            group: HashSet::new(),
            handles: HashSet::new(),
            lut: Vec::new(),
            ends: None,
            standalone_lut: (0.0, Vec::new()),
        };

        println!("group_load_save length: {:?}", loaded_groups_vec.len());

        for group_load_save in loaded_groups_vec {
            for (mut bezier, anchor, t_ends, local_lut) in group_load_save.lut {
                let (entity, handle) = spawn_bezier(
                    &mut bezier,
                    &mut bezier_curves,
                    &mut commands,
                    &mut meshes,
                    // &mut pipelines,
                    &mut my_shader_params,
                    clearcolor,
                    &mut globals,
                );
                group.group.insert((entity.clone(), handle.clone()));
                group.handles.insert(handle.clone());
                group.standalone_lut = group_load_save.standalone_lut.clone();
                group.lut.push((handle.clone(), anchor, t_ends, local_lut));
            }
        }
        globals.selected = group.clone();

        // event_writer.send(group);

        // to create a group: select all the curves programmatically, and send a UiButton::Group event
        loaded_event_writer.send(Loaded);
        println!("{:?}", "loaded groups");
    }
}

pub fn latch2(
    cursor: ResMut<Cursor>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    mut query: QuerySet<(
        QueryState<(&Handle<Bezier>, &BoundingBoxQuad)>,
        QueryState<(&Handle<Bezier>, &BoundingBoxQuad)>,
    )>,
    globals: ResMut<Globals>,
    mut event_writer: EventWriter<OfficialLatch>,
    mut action_event_reader: EventReader<Action>,
) {
    if let Some(Action::Latch) = action_event_reader.iter().next() {
        let latching_distance = 5.0;

        let mut potential_mover: Option<(Vec2, u128, AnchorEdge, Handle<Bezier>)> = None;
        let mut potential_partner: Option<(
            u128,
            AnchorEdge,
            AnchorEdge,
            Handle<Bezier>,
            Handle<Bezier>,
        )> = None;

        // find moving quad and store its parameters
        for (bezier_handle, _bb) in query.q0().iter() {
            let bezier = bezier_curves.get(bezier_handle).unwrap().clone();
            if bezier.edge_is_moving() {
                // a latched point does not latch to an additional point
                let moving_anchor = bezier.get_mover_edge();
                if bezier.quad_is_latched(moving_anchor) {
                    return;
                }

                let mover_pos = cursor.position;
                potential_mover = Some((
                    mover_pos,
                    bezier.id,
                    bezier.get_mover_edge(),
                    bezier_handle.clone(),
                ));

                break;
            }
        }

        // find quad within latching_distance. Upon success, setup a latch and store the
        // paramters of the latchee (partner)
        if let Some((pos, id, mover_edge, mover_handle)) = potential_mover {
            if let Some((_dist, anchor_edge, partner_handle)) = get_close_still_anchor(
                latching_distance * globals.scale,
                pos,
                &bezier_curves,
                &query.q0(),
            ) {
                let partner_bezier = bezier_curves.get_mut(partner_handle.clone()).unwrap();

                // if the potential partner is free, continue
                if partner_bezier.quad_is_latched(anchor_edge) {
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

                event_writer.send(OfficialLatch(partner_latch_data, partner_handle.clone()));
            }
        }

        // setup the latcher if a partner has been found
        if let Some((partner_id, mover_anchor, pa_edge, mover_handle, partner_handle)) =
            potential_partner
        {
            let partner_bezier = bezier_curves.get(partner_handle).unwrap().clone();
            let bezier = bezier_curves.get_mut(mover_handle.clone()).unwrap();

            let mover_latch_data = LatchData {
                latched_to_id: partner_id,
                self_edge: mover_anchor,
                partners_edge: pa_edge,
            };

            // set the position of the latched moving quad and its control point
            if mover_anchor == AnchorEdge::Start {
                bezier.positions.start = partner_bezier.get_position(pa_edge.to_anchor());
                bezier.positions.control_start = partner_bezier.get_opposite_control(pa_edge)
            } else if mover_anchor == AnchorEdge::End {
                bezier.positions.end = partner_bezier.get_position(pa_edge.to_anchor());
                bezier.positions.control_end = partner_bezier.get_opposite_control(pa_edge)
            }

            event_writer.send(OfficialLatch(mover_latch_data, mover_handle.clone()));
        }
    }
}

pub fn officiate_latch_partnership(
    mouse_button_input: Res<Input<MouseButton>>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    mut latch_event_reader: EventReader<OfficialLatch>,
    globals: ResMut<Globals>,
    audio: Res<Audio>,
) {
    if mouse_button_input.just_released(MouseButton::Left) {
        for OfficialLatch(latch, bezier_handle) in latch_event_reader.iter() {
            let bezier = bezier_curves.get_mut(bezier_handle).unwrap();
            bezier.set_latch(latch.clone());
            println!("latched, {:?}", bezier.latches);

            if globals.sound_on {
                if let Some(sound) = globals.sounds.get("latch") {
                    audio.play(sound.clone());
                }
            }
        }
    }
}

//
pub fn rescale(
    mut grandparent_query: Query<&mut Transform, With<GrandParent>>,
    shader_param_query: Query<&Handle<MyShader>>,
    mut my_shaders: ResMut<Assets<MyShader>>,
    mut globals: ResMut<Globals>,
    mut action_event_reader: EventReader<Action>,
) {
    for action in action_event_reader.iter() {
        //
        let mut pressed_rescale_button = false;
        let mut zoom_direction = 0.0;
        //
        if action == &Action::ScaleUp {
            pressed_rescale_button = true;
            zoom_direction = 1.0;
        } else if action == &Action::ScaleDown {
            pressed_rescale_button = true;
            zoom_direction = -1.0;
        }
        if pressed_rescale_button {
            let zoom_factor = 1.0 + zoom_direction * 0.1;
            globals.scale = globals.scale * zoom_factor;

            // the bounding box, the ends and the control points share the same shader parameters
            for mut transform in grandparent_query.iter_mut() {
                transform.scale = Vec2::new(globals.scale, globals.scale).extend(1.0);
            }

            // update the shader params for the middle quads (animated quads)
            for shader_handle in shader_param_query.iter() {
                let shader_param = my_shaders.get_mut(shader_handle).unwrap();
                shader_param.zoom = 0.15 / globals.scale;
                shader_param.size *= 1.0 / zoom_factor;
            }
        }
    }
}
