use bevy_pen_tool_spawner::inputs::{Action, Cursor, Latch, MoveAnchor, UiButton};
use bevy_pen_tool_spawner::spawn_bezier;
use bevy_pen_tool_spawner::util::*;

use bevy::{asset::HandleId, prelude::*};

use bevy_inspector_egui::{Inspectable, InspectorPlugin};

#[derive(Debug, Clone, Inspectable)]
pub struct HistoryLenInspector {
    pub length: usize,
    pub index: i32,
}

impl Default for HistoryLenInspector {
    fn default() -> Self {
        Self {
            length: 0,
            index: -1,
        }
    }
}

#[derive(Debug, Clone, Inspectable)]
pub struct HistoryInspector {
    pub history: Vec<HistoryActionInspector>,
    pub index: i32,
}

impl Default for HistoryInspector {
    fn default() -> Self {
        Self {
            history: vec![],
            index: -1,
        }
    }
}

impl From<History> for HistoryInspector {
    fn from(history: History) -> Self {
        Self {
            history: history
                .actions
                .iter()
                .map(|x| HistoryActionInspector::from(x.clone()))
                .collect(),
            index: history.index,
        }
    }
}

#[derive(Debug, Clone, Inspectable)]
pub enum HistoryActionInspector {
    MovedAnchor {
        bezier_id: BezierHistId,
    },
    SpawnedCurve {
        bezier_id: BezierHistId,
    },
    DeletedCurve {
        bezier_id: BezierHistId,
    },
    Latched {
        bezier_id1: BezierHistId,
        bezier_id2: BezierHistId,
    },
    Unlatched {
        self_id: BezierHistId,
        partner_bezier_id: BezierHistId,
    },
    None,
}

impl From<HistoryAction> for HistoryActionInspector {
    fn from(action: HistoryAction) -> Self {
        match action {
            HistoryAction::MovedAnchor { bezier_id, .. } => {
                HistoryActionInspector::MovedAnchor { bezier_id }
            }
            HistoryAction::SpawnedCurve { bezier_id, .. } => {
                HistoryActionInspector::SpawnedCurve { bezier_id }
            }
            HistoryAction::DeletedCurve { bezier_id, .. } => {
                HistoryActionInspector::DeletedCurve { bezier_id }
            }
            HistoryAction::Latched {
                self_id: bezier_handle_1,
                partner_bezier_id: bezier_handle_2,
                ..
            } => HistoryActionInspector::Latched {
                bezier_id1: bezier_handle_1,
                bezier_id2: bezier_handle_2,
            },
            HistoryAction::Unlatched {
                self_id,
                partner_bezier_id,
                ..
            } => HistoryActionInspector::Unlatched {
                self_id,
                partner_bezier_id,
            },

            HistoryAction::None => HistoryActionInspector::None,
        }
    }
}

impl Default for HistoryActionInspector {
    fn default() -> Self {
        Self::None
    }
}

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

                bezier.set_position(anchor, previous_position);

                bezier.do_compute_lut = true;

                // move the latched partner anchor
                let anchor_edge = anchor.to_edge_with_controls();

                if let Some(_) = bezier.latches.get(&anchor_edge) {
                    let latch_info = bezier.get_anchor_latch_info(anchor);

                    update_latched_partner_position(
                        &maps.bezier_map,
                        &mut bezier_curves,
                        latch_info,
                    );
                }
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
            HistoryAction::Latched {
                self_id: bezier_id_1,
                partner_bezier_id: bezier_id_2,
                self_anchor: anchor_1,
                partner_anchor: anchor_2,
            } => {
                let handle_entity_1 = maps.bezier_map[&bezier_id_1.into()].clone();
                let bezier_1 = bezier_curves.get_mut(&handle_entity_1.handle).unwrap();
                bezier_1.latches.remove(&anchor_1.to_edge());

                let handle_entity_2 = maps.bezier_map[&bezier_id_2.into()].clone();
                let bezier_2 = bezier_curves.get_mut(&handle_entity_2.handle).unwrap();
                bezier_2.latches.remove(&anchor_2.to_edge());
            }

            HistoryAction::Unlatched {
                self_id,           //Handle<Bezier>,
                partner_bezier_id, //Handle<Bezier>,
                self_anchor,
                partner_anchor,
            } => {
                info!("undoing latch");
                let handle_entity_1 = maps.bezier_map[&self_id.into()].clone();
                let bezier_1 = bezier_curves.get_mut(&handle_entity_1.handle).unwrap();

                let latch_1 = LatchData {
                    latched_to_id: partner_bezier_id.into(),
                    self_edge: self_anchor.to_edge(),
                    partners_edge: partner_anchor.to_edge(),
                };

                bezier_1.latches.insert(self_anchor.to_edge(), latch_1);

                let handle_entity_2 = maps.bezier_map[&partner_bezier_id.into()].clone();
                let bezier_2 = bezier_curves.get_mut(&handle_entity_2.handle).unwrap();

                let latch_2 = LatchData {
                    latched_to_id: self_id.into(),
                    self_edge: partner_anchor.to_edge(),
                    partners_edge: self_anchor.to_edge(),
                };

                bezier_2.latches.insert(partner_anchor.to_edge(), latch_2);
            }

            _ => (),
        };
        history.index -= 1;
    }
}

pub fn add_to_history(
    mut history: ResMut<History>,
    mut add_to_history_event_reader: EventReader<HistoryAction>,
    // bezier_curves: ResMut<Assets<Bezier>>,
    // mut action_event_writer: EventWriter<Action>,
) {
    for (k, hist_event) in add_to_history_event_reader.iter().enumerate() {
        // if history has a tail branching away from the head, remove it, but
        // only once
        if k == 0 && history.index > -1 {
            history.actions = history.actions[0..(history.index + 1) as usize].to_vec();
        }

        match hist_event {
            //
            x @ HistoryAction::MovedAnchor { .. } => {
                history.actions.push(x.clone());
            }

            x @ HistoryAction::SpawnedCurve { .. } => {
                history.actions.push(x.clone());
            }
            x @ HistoryAction::DeletedCurve { .. } => {
                // info!("pushing deleted curve");
                history.actions.push(x.clone());
            }
            x @ HistoryAction::Latched { .. } => {
                history.actions.push(x.clone());
            }
            x @ HistoryAction::Unlatched { .. } => {
                history.actions.push(x.clone());
            }
            _ => {}
        }

        // move history head forward
        history.index += 1;
    }
}

pub fn redo_effects(
    // mut commands: Commands,
    // mut lut_event_reader: EventReader<ComputeLut>,
    mut redo_delete_event_reader: EventReader<RedoDelete>,
    mut action_event_writer: EventWriter<Action>,
    mut selection: ResMut<Selection>,
    mut maps: ResMut<Maps>,
) {
    // for _ in lut_event_reader.iter() {
    //     // println!("compute_lut_sender");
    //     action_event_writer.send(Action::ComputeLut);
    // }

    for redo_delete in redo_delete_event_reader.iter() {
        // to delete a curve, we programmatically select the curve and send
        // an Action::Delete event
        if let Some(handle_entity) = maps.bezier_map.get(&redo_delete.bezier_id) {
            selection
                .selected
                .group
                .insert((handle_entity.entity.unwrap(), handle_entity.handle.clone()));
            action_event_writer.send(Action::Delete(true));
        }
        if let None = maps.bezier_map.remove(&redo_delete.bezier_id) {
            info!(
                "COULD NOT DELETE CURVE FROM MAP: {:?}",
                redo_delete.bezier_id
            );
        }
    }
}

pub fn redo(
    // mut commands: Commands,
    // mut globals: ResMut<Globals>,
    mut history: ResMut<History>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    mut action_event_reader: EventReader<Action>,
    mut user_state: ResMut<UserState>,
    // mut lut_event_writer: EventWriter<ComputeLut>,
    mut delete_curve_event_writer: EventWriter<RedoDelete>,
    // mut selection: ResMut<Selection>,
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

                // let latch_info = bezier.get_mover_latch_info();
                // update_latched_partner_position(&maps.bezier_map, &mut bezier_curves, latch_info);

                // move the latched partner anchor
                let anchor_edge = anchor.to_edge_with_controls();

                if let Some(_) = bezier.latches.get(&anchor_edge) {
                    let latch_info = bezier.get_anchor_latch_info(anchor);

                    update_latched_partner_position(
                        &maps.bezier_map,
                        &mut bezier_curves,
                        latch_info,
                    );
                }

                // if let Some((partner_latch, mover_position, opposite_control)) = latch_info {
                //                 if let Some(bezier_handle) = maps.bezier_map.get(&partner_latch.latched_to_id) {
                //                     let partner_bezier = bezier_curves.get_mut(&bezier_handle.handle).unwrap();
                //                 }
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

                delete_curve_event_writer.send(RedoDelete {
                    bezier_id: bezier_id.into(),
                });
            }

            HistoryAction::Latched {
                self_id: bezier_id_1,
                self_anchor: anchor_1,
                partner_bezier_id: bezier_id_2,
                partner_anchor: anchor_2,
            } => {
                info!("redoing latching");
                let handle_entity_1 = maps.bezier_map[&bezier_id_1.into()].clone();
                let bezier_1 = bezier_curves.get_mut(&handle_entity_1.handle).unwrap();

                let latch_1 = LatchData {
                    latched_to_id: bezier_id_2.into(),
                    self_edge: anchor_1.to_edge(),
                    partners_edge: anchor_2.to_edge(),
                };

                bezier_1.latches.insert(anchor_1.to_edge(), latch_1);

                let handle_entity_2 = maps.bezier_map[&bezier_id_2.into()].clone();
                let bezier_2 = bezier_curves.get_mut(&handle_entity_2.handle).unwrap();

                let latch_2 = LatchData {
                    latched_to_id: bezier_id_1.into(),
                    self_edge: anchor_2.to_edge(),
                    partners_edge: anchor_1.to_edge(),
                };

                bezier_2.latches.insert(anchor_2.to_edge(), latch_2);
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
