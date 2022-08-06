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
    // mut bezier_query: Query<(Entity, &Handle<Bezier>)>,
) {
    if action_event_reader.iter().any(|x| x == &Action::Undo) {
        //
        println!("undo: {:?}", history.actions.iter().count());
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
                bezier_handle,
                anchor,
                new_position,
                previous_position,
            } => {
                println!("undo: MovedAnchor");
                let bezier = bezier_curves.get_mut(&bezier_handle).unwrap();
                // let mut new_bezier = bezier.clone();
                // bezier.set_position(anchor, previous_position);
                // bezier.set_position(anchor, new_position);
                bezier.set_position(anchor, previous_position);
                // bezier.set_previous_pos(anchor, previous_position);
            }
            HistoryAction::SpawnedCurve {
                bezier_handle: _,
                bezier_hist: _,
                entity,
                id,
            } => {
                println!("undo: SpawnedCurve");
                commands.entity(entity).despawn_recursive();
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

        println!("hist event: {:?}", hist_event);

        match hist_event {
            x @ HistoryAction::MovedAnchor { .. } => {
                history.actions.push(x.clone());

                // let bezier = bezier_curves.get(&bezier_handle).unwrap();

                // history.actions.push(HistoryAction::MovedAnchor {
                //     bezier_handle: bezier_handle.clone(),
                //     anchor: *anchor,
                //     new_position: bezier.get_position(*anchor),
                //     previous_position: bezier.get_previous_position(*anchor),
                // });
            }
            x @ HistoryAction::SpawnedCurve {
                id: xid,
                entity: xentity,
                ..
            } => {
                // replace history element if it's a redo. Else, push
                let index = history.index as usize;
                if let Some(HistoryAction::SpawnedCurve { id, entity, .. }) =
                    history.actions.get_mut(index)
                {
                    // if the ids are identical, it means that the curve was issued
                    // from a redo. In that case, the history future is not wiped out
                    if id == xid {
                        *entity = *xentity;
                        // action_event_writer.send(Action::ComputeLut);

                        return;
                    }
                }

                history.actions.push(x.clone());
            }
            x @ HistoryAction::DeletedCurve { .. } => {
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

// TODO: not needed anymore... delete?
pub fn compute_lut_sender(
    mut lut_event_reader: EventReader<ComputeLut>,
    mut action_event_writer: EventWriter<Action>,
) {
    for _ in lut_event_reader.iter() {
        // println!("compute_lut_sender");
        action_event_writer.send(Action::ComputeLut);
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
) {
    if action_event_reader.iter().any(|x| x == &Action::Redo) {
        //
        println!("NUM ACTIONS: {:?}", history.actions.iter().count());
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
                bezier_handle,
                anchor,
                new_position,
                previous_position: _,
            } => {
                info!("redoing moving");
                let bezier = bezier_curves.get_mut(&bezier_handle).unwrap();
                // let mut new_bezier = bezier.clone();
                // bezier.set_position(anchor, previous_position);
                bezier.set_position(anchor, new_position);
                // lut_event_writer.send(ComputeLut);
                bezier.do_compute_lut = true;
                // bezier.set_previous_pos(anchor, previous_position);
            }
            // TODO: NEED TO CHANGE THE ENTITY INFO WHEN RESPAWNING WITH REDO
            HistoryAction::SpawnedCurve {
                bezier_handle,
                bezier_hist,
                entity: _,
                id,
            } => {
                println!("redo!@#@!#$#%$%^^:",);
                *user_state = UserState::SpawningCurve {
                    bezier_hist: Some(bezier_hist),
                    maybe_bezier_handle: Some(bezier_handle),
                };
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
