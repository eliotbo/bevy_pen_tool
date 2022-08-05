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
) {
    if action_event_reader.iter().any(|x| x == &Action::Undo) {
        //
        println!("undo: {:?}", history.actions.iter().count());
        //

        if history.index == 0 {
            info!("undo has reached the end of the history");
            return ();
        }
        history.index -= 1;
        println!("undo index: {}", history.index);

        let index = history.index as usize;
        let previous_hist_action = history.actions[index].clone();

        match previous_hist_action {
            HistoryAction::MovedAnchor {
                bezier_handle,
                anchor,
                new_position,
                previous_position,
            } => {
                let bezier = bezier_curves.get_mut(&bezier_handle).unwrap();
                // let mut new_bezier = bezier.clone();
                // bezier.set_position(anchor, previous_position);
                bezier.set_position(anchor, new_position);
                bezier.set_previous_pos(anchor, previous_position);
            }
            HistoryAction::SpawnedCurve {
                bezier_handle: _,
                bezier_hist: _,
                entity,
            } => {
                commands.entity(entity).despawn_recursive();
            }

            _ => (),
        };
    }
}

pub fn add_to_history(
    mut history: ResMut<History>,
    mut add_to_history_event_reader: EventReader<HistoryAction>,
    bezier_curves: ResMut<Assets<Bezier>>,
) {
    for hist_event in add_to_history_event_reader.iter() {
        //
        history.index += 1;
        if history.index > -1 {
            history.actions = history.actions[0..history.index as usize].to_vec();
        }

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
            x @ HistoryAction::SpawnedCurve { .. } => {
                history.actions.push(x.clone());
            }
            x @ HistoryAction::DeletedCurve { .. } => {
                history.actions.push(x.clone());
            }
            _ => {}
        }
    }
}

pub fn redo(
    mut commands: Commands,
    // mut globals: ResMut<Globals>,
    mut history: ResMut<History>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    mut action_event_reader: EventReader<Action>,
    mut user_state: ResMut<UserState>,
) {
    if action_event_reader.iter().any(|x| x == &Action::Redo) {
        //
        println!("undo: {:?}", history.actions.iter().count());
        //

        if history.index < history.actions.len() as i32 {
            history.index += 1;
        } else {
            info!("redo has reached the top of the history");
        }

        println!("undo index: {}", history.index);

        let index = history.index as usize;
        let further_hist_action = history.actions[index - 1].clone();

        match further_hist_action {
            HistoryAction::MovedAnchor {
                bezier_handle,
                anchor,
                new_position,
                previous_position,
            } => {
                let bezier = bezier_curves.get_mut(&bezier_handle).unwrap();
                // let mut new_bezier = bezier.clone();
                // bezier.set_position(anchor, previous_position);
                bezier.set_position(anchor, new_position);
                bezier.set_previous_pos(anchor, previous_position);
            }
            HistoryAction::SpawnedCurve {
                bezier_handle: _,
                bezier_hist,
                entity: _,
            } => {
                println!("redo!@#@!#$#%$%^^:",);
                *user_state = UserState::SpawningCurve {
                    bezier_hist: Some(bezier_hist),
                };
            }
            _ => {}
        }
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
