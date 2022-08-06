use bevy_pen_tool_spawner::inputs::{Action, Cursor, MoveAnchor, UiButton};
use bevy_pen_tool_spawner::spawn_bezier;
use bevy_pen_tool_spawner::util::*;

use bevy::{asset::HandleId, prelude::*};

pub fn undo(
    mut commands: Commands,
    // mut globals: ResMut<Globals>,
    mut history: ResMut<History>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    mut action_event_reader: EventReader<Action>,
    mut user_state: ResMut<UserState>,
    maps: ResMut<Maps>,
    // mut bezier_query: Query<(Entity, &Handle<Bezier>)>,
) {
    if action_event_reader.iter().any(|x| x == &Action::Undo) {
        //
        // println!("undo: {:?}", history.actions.iter().count());
        //

        // let index = history.index;
        // println!("tried index: {:?}", index);

        if history.index == -1 {
            info!(
                "undo has reached the end of the history: {:?}",
                history.index
            );
            return ();
        }

        // if index + 1 > 0 {
        //     history.index -= 1;
        //     println!("history index now: {:?}", history.index);
        // }

        let previous_hist_action = history.actions[history.index as usize].clone();

        match previous_hist_action {
            HistoryAction::MovedAnchor {
                bezier_id,
                new_position: _,
                previous_position,
                anchor,
            } => {
                // println!("undo: MovedAnchor");
                let handle_entity = maps.bezier_map[&bezier_id.into()].clone();
                let bezier = bezier_curves.get_mut(&handle_entity.handle).unwrap();
                // let mut new_bezier = bezier.clone();
                // bezier.set_position(anchor, previous_position);
                // bezier.set_position(anchor, new_position);
                bezier.set_position(anchor, previous_position);
                // bezier.set_previous_pos(anchor, previous_position);
                bezier.do_compute_lut = true;
            }
            HistoryAction::SpawnedCurve {
                bezier_id,
                bezier_hist: _,
                // entity: _,
                // id,
            } => {
                // println!("undo: SpawnedCurve");
                if let Some(handle_entity) = maps.bezier_map.get(&bezier_id.into()) {
                    if let Some(entity) = handle_entity.entity {
                        commands.entity(entity).despawn_recursive();
                    }
                }
            }
            HistoryAction::DeletedCurve { bezier, bezier_id } => {
                println!("undo DeletedCurve, so spawning with id: {:?}", bezier_id);
                maps.print_bezier_map();
                // let handle_entity = maps.bezier_map[&bezier_id].clone();
                *user_state = UserState::SpawningCurve {
                    bezier_hist: Some(bezier),
                    maybe_bezier_id: Some(bezier_id.into()),
                };
            }

            _ => (),
        };
        history.index -= 1;
    }
}

// pub id_handle_map: HashMap<u128, Handle<Bezier>>,
pub fn add_to_history(
    mut history: ResMut<History>,
    mut add_to_history_event_reader: EventReader<HistoryAction>,
    // bezier_curves: ResMut<Assets<Bezier>>,
    // mut action_event_writer: EventWriter<Action>,
) {
    for hist_event in add_to_history_event_reader.iter() {
        //

        // println!("hist event: {:?}", hist_event);

        match hist_event {
            //
            x @ HistoryAction::MovedAnchor { .. } => {
                history.actions.push(x.clone());
            }

            x @ HistoryAction::SpawnedCurve {
                // bezier_id: xid,
                // entity: xentity,
                ..
            } => {
                // replace history element if it's a redo. Else, push
                // let index = history.index as usize;

                // if let Some(HistoryAction::SpawnedCurve { id, .. }) = history.actions.get_mut(index)
                // {
                //     // if the ids are identical, it means that the curve was issued
                //     // from a redo. In that case, the history future is not wiped out
                //     if id == xid {
                //         println!(
                //             "Spawned from redo (not increasing history index nor history actions)"
                //         );
                //         // *entity = *xentity;
                //         // action_event_writer.send(Action::ComputeLut);

                //         return;
                //     }
                // }

                // if let Some(HistoryAction::DeletedCurve { bezier, .. }) =
                //     history.actions.get_mut(index + 1)
                // {
                //     println!("added delete to history: {} -----> {}", bezier.id, xid);
                //     if bezier.id == *xid {
                //         // *entity = *xentity;
                //         // action_event_writer.send(Action::ComputeLut);
                //         println!("not increasing history index nor history actions");

                //         return;
                //     }
                // }

                history.actions.push(x.clone());
            }
            x @ HistoryAction::DeletedCurve { .. } => {
                info!("pushing deleted curve");
                history.actions.push(x.clone());
            }
            _ => {}
        }

        // move history head forward
        history.index += 1;

        // if history has a tail branching away from the head, remove it
        if history.index > -1 {
            history.actions = history.actions[0..(history.index + 1) as usize].to_vec();
        }
    }
}

pub fn history_effects(
    mut commands: Commands,
    mut lut_event_reader: EventReader<ComputeLut>,
    mut delete_curver_event_reader: EventReader<DeleteCurve>,
    mut action_event_writer: EventWriter<Action>,
    maps: ResMut<Maps>,
) {
    for _ in lut_event_reader.iter() {
        // println!("compute_lut_sender");
        action_event_writer.send(Action::ComputeLut);
    }

    for deleter in delete_curver_event_reader.iter() {
        if let Some(handle_entity) = maps.bezier_map.get(&deleter.bezier_id) {
            if let Some(entity) = handle_entity.entity {
                commands.entity(entity).despawn_recursive();
            }
        }
        // action_event_writer.send(Action::Delete);
    }
}

pub fn redo(
    // mut commands: Commands,
    // mut globals: ResMut<Globals>,
    mut history: ResMut<History>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    mut action_event_reader: EventReader<Action>,
    mut user_state: ResMut<UserState>,
    mut lut_event_writer: EventWriter<ComputeLut>,
    mut delete_curve_event_writer: EventWriter<DeleteCurve>,
    mut selection: ResMut<Selection>,
    maps: ResMut<Maps>,
) {
    if action_event_reader.iter().any(|x| x == &Action::Redo) {
        //
        // println!("NUM ACTIONS: {:?}", history.actions.iter().count());
        //

        let index = history.index + 1;

        // println!("redo index: {}", index);
        if index > history.actions.len() as i32 - 1 {
            info!("redo has reached the top of the history");
            return ();
        }

        let further_hist_action = history.actions[index as usize].clone();

        match further_hist_action {
            HistoryAction::MovedAnchor {
                bezier_id,
                anchor,
                new_position,
                previous_position: _,
            } => {
                // info!("redoing moving");
                let handle_entity = maps.bezier_map[&bezier_id.into()].clone();
                let bezier = bezier_curves.get_mut(&handle_entity.handle).unwrap();
                // let mut new_bezier = bezier.clone();
                // bezier.set_position(anchor, previous_position);
                bezier.set_position(anchor, new_position);
                // lut_event_writer.send(ComputeLut);
                bezier.do_compute_lut = true;
                // bezier.set_previous_pos(anchor, previous_position);
            }
            // TODO: NEED TO CHANGE THE ENTITY INFO WHEN RESPAWNING WITH REDO
            HistoryAction::SpawnedCurve {
                bezier_id,
                bezier_hist,
                // entity: _,
                // id,
            } => {
                // println!("redo!@#@!#$#%$%^^:",);
                // let handle_entity = maps.bezier_map[&bezier_id].clone();
                *user_state = UserState::SpawningCurve {
                    bezier_hist: Some(bezier_hist),
                    maybe_bezier_id: Some(bezier_id.into()),
                };
            }
            HistoryAction::DeletedCurve {
                bezier: _,
                bezier_id,
            } => {
                println!("redoing delete with id: {:?}", bezier_id);
                // selection.selected.group.clear();
                // if let Some(handle_entity) = maps.id_handle_map.get(&bezier.id) {
                // selection
                //     .selected
                //     .group
                //     .insert((handle_entity.entity.unwrap(), bezier_handle));
                delete_curve_event_writer.send(DeleteCurve {
                    bezier_id: bezier_id.into(),
                });
                //     return;
                // } else {
                //     info!("redo: could not find entity curve to delete");
                // }
            }
            _ => {}
        }
        history.index += 1;
    }
}

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
