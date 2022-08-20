use bevy_pen_tool_model::inputs::Action;

use bevy_pen_tool_model::model::*;

use crate::pen::*;

use bevy::prelude::*;

use bevy_inspector_egui::Inspectable;

use std::collections::HashSet;

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
        let mut history_inspector: Vec<HistoryActionInspector> = history
            .actions
            .iter()
            .map(|x| HistoryActionInspector::from(x.clone()))
            .collect();
        history_inspector.reverse();
        Self {
            history: history_inspector,
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
                partner_id: bezier_handle_2,
                ..
            } => HistoryActionInspector::Latched {
                bezier_id1: bezier_handle_1,
                bezier_id2: bezier_handle_2,
            },
            HistoryAction::Unlatched {
                self_id,
                partner_id: partner_bezier_id,
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

pub fn latch_curves(
    mut commands: &mut Commands,
    l1: CurveIdEdge,
    l2: CurveIdEdge,
    maps: &ResMut<Maps>,
    bezier_curves: &mut ResMut<Assets<Bezier>>,
) {
    let bezier_id_1 = l1.id;
    let bezier_id_2 = l2.id;
    let anchor_1 = l1.anchor_edge;
    let anchor_2 = l2.anchor_edge;

    let handle_entity_1 = maps.bezier_map[&bezier_id_1.into()].clone();
    let bezier_1 = bezier_curves.get_mut(&handle_entity_1.handle).unwrap();

    let latch_1 = LatchData {
        latched_to_id: bezier_id_2.into(),
        self_edge: anchor_1,
        partners_edge: anchor_2,
    };

    bezier_1.do_compute_lut = true;

    bezier_1.latches.insert(anchor_1, latch_1);

    // control point position must be opposite from partner's
    let bezier_2_control_pos = bezier_1.get_opposite_control(anchor_1);

    bezier_1.set_position(
        anchor_1.to_anchor(),
        bezier_1.get_position(anchor_1.to_anchor()),
    );

    bezier_1.move_anchor(
        &mut commands,
        true,  // one move for a single frame
        false, // do not follow mouse
        anchor_1.to_anchor(),
        maps.as_ref(),
    );

    let handle_entity_2 = maps.bezier_map[&bezier_id_2.into()].clone();
    let bezier_2 = bezier_curves.get_mut(&handle_entity_2.handle).unwrap();

    let latch_2 = LatchData {
        latched_to_id: bezier_id_1.into(),
        self_edge: anchor_2,
        partners_edge: anchor_1,
    };

    bezier_2.do_compute_lut = true;
    bezier_2.latches.insert(anchor_2, latch_2);

    bezier_2.set_position(anchor_2.to_anchor().adjoint(), bezier_2_control_pos);
    bezier_2.move_anchor(
        &mut commands,
        true,  // one move for a single frame
        false, // do not follow mouse
        anchor_2.to_anchor().adjoint(),
        maps.as_ref(),
    );
}

pub fn undo(
    mut commands: Commands,
    mut history: ResMut<History>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    groups: Res<Assets<Group>>,
    mut action_event_reader: EventReader<Action>,
    maps: ResMut<Maps>,
    mut spawn_curve_event_writer: EventWriter<SpawningCurve>,
    audio: Res<Audio>,
    globals: ResMut<Globals>,
) {
    if action_event_reader.iter().any(|x| x == &Action::Undo) {
        if history.index == -1 {
            info!(
                "undo has reached the beginning of the history: {:?}",
                history.index
            );
            return ();
        }

        let previous_hist_action = history.actions[history.index as usize].clone();

        match previous_hist_action {
            HistoryAction::MovedAnchor {
                bezier_id,
                new_position: _,
                previous_position,
                anchor,
            } => {
                let move_command = MoveCommand {
                    id: bezier_id.into(),
                    anchor,
                    new_position: previous_position,
                };
                move_anchor(&mut commands, move_command, &mut bezier_curves, &maps);
            }
            HistoryAction::SpawnedCurve {
                bezier_id,
                bezier_hist: _,
            } => {
                if let Some(handle_entity) = maps.bezier_map.get(&bezier_id.into()) {
                    if let Some(bezier) = bezier_curves.get_mut(&handle_entity.handle) {
                        if let Some(group_handle) = maps.group_map.get(&bezier.group) {
                            let group = groups.get(group_handle).unwrap();

                            if let Some(group_entity) = group.entity {
                                commands.entity(group_entity).despawn_recursive();
                            }
                        }
                    }
                    commands.entity(handle_entity.entity).despawn_recursive();
                }
            }
            HistoryAction::DeletedCurve { bezier, bezier_id } => {
                spawn_curve_event_writer.send(SpawningCurve {
                    bezier_hist: Some(bezier),
                    maybe_bezier_id: Some(bezier_id.into()),
                    follow_mouse: false,
                });
            }
            HistoryAction::Latched {
                self_id: bezier_id_1,
                partner_id: bezier_id_2,
                self_anchor: anchor_1,
                partner_anchor: anchor_2,
            } => {
                let handle_entity_1 = maps.bezier_map[&bezier_id_1.into()].clone();
                let bezier_1 = bezier_curves.get_mut(&handle_entity_1.handle).unwrap();
                bezier_1.latches.remove(&anchor_1);
                bezier_1.potential_latch = None;

                let handle_entity_2 = maps.bezier_map[&bezier_id_2.into()].clone();
                let bezier_2 = bezier_curves.get_mut(&handle_entity_2.handle).unwrap();
                bezier_2.latches.remove(&anchor_2);
                bezier_2.potential_latch = None;

                if globals.sound_on {
                    if let Some(sound) = maps.sounds.get("unlatch") {
                        audio.play(sound.clone());
                    }
                }
            }

            HistoryAction::Unlatched {
                self_id,
                partner_id: partner_bezier_id,
                self_anchor,
                partner_anchor,
            } => {
                // info!("undoing unlatch");
                let handle_entity_1 = maps.bezier_map[&self_id.into()].clone();
                let bezier_1 = bezier_curves.get_mut(&handle_entity_1.handle).unwrap();

                let latch_1 = LatchData {
                    latched_to_id: partner_bezier_id.into(),
                    self_edge: self_anchor,
                    partners_edge: partner_anchor,
                };

                bezier_1.latches.insert(self_anchor, latch_1);

                let handle_entity_2 = maps.bezier_map[&partner_bezier_id.into()].clone();
                let bezier_2 = bezier_curves.get_mut(&handle_entity_2.handle).unwrap();

                let latch_2 = LatchData {
                    latched_to_id: self_id.into(),
                    self_edge: partner_anchor,
                    partners_edge: self_anchor,
                };

                bezier_2.latches.insert(partner_anchor, latch_2);

                if globals.sound_on {
                    if let Some(sound) = maps.sounds.get("latch") {
                        audio.play(sound.clone());
                    }
                }
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

        history.actions.push(hist_event.clone());

        // move history head forward
        history.index += 1;
    }
}

pub fn redo_effects(
    mut redo_delete_event_reader: EventReader<RedoDelete>,
    mut action_event_writer: EventWriter<Action>,
    mut selection: ResMut<Selection>,
    mut maps: ResMut<Maps>,
) {
    for redo_delete in redo_delete_event_reader.iter() {
        // to delete a curve, we programmatically select the curve and send
        // an Action::Delete event
        if let Some(handle_entity) = maps.bezier_map.get(&redo_delete.bezier_id) {
            let mut curve_set = HashSet::new();
            curve_set.insert(handle_entity.handle.id.into());
            selection.selected = vec![SelectionChoice::CurveSet(curve_set)];
            //     .group
            //     .insert((handle_entity.entity, handle_entity.handle.clone()));
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
    mut commands: Commands,
    // mut globals: ResMut<Globals>,
    mut history: ResMut<History>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    mut action_event_reader: EventReader<Action>,
    // mut user_state: ResMut<UserState>,
    // mut lut_event_writer: EventWriter<ComputeLut>,
    mut delete_curve_event_writer: EventWriter<RedoDelete>,
    mut spawn_curve_event_writer: EventWriter<SpawningCurve>,
    audio: Res<Audio>,
    globals: ResMut<Globals>,
    // mut move_anchor_event_writer: EventWriter<MoveAnchorEvent>,
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
                let move_command = MoveCommand {
                    id: bezier_id.into(),
                    anchor,
                    new_position,
                };
                move_anchor(&mut commands, move_command, &mut bezier_curves, &maps);
            }

            HistoryAction::SpawnedCurve {
                bezier_id,
                bezier_hist,
            } => {
                // *user_state = UserState::SpawningCurve {
                //     bezier_hist: Some(bezier_hist),
                //     maybe_bezier_id: Some(bezier_id.into()),
                // };
                spawn_curve_event_writer.send(SpawningCurve {
                    bezier_hist: Some(bezier_hist),
                    maybe_bezier_id: Some(bezier_id.into()),
                    follow_mouse: false,
                });
            }
            HistoryAction::DeletedCurve {
                bezier: _,
                bezier_id,
            } => {
                // println!("redoing delete with id: {:?}", bezier_id);

                delete_curve_event_writer.send(RedoDelete {
                    bezier_id: bezier_id.into(),
                });
            }

            HistoryAction::Latched {
                self_id: bezier_id_1,
                self_anchor: anchor_1,
                partner_id: bezier_id_2,
                partner_anchor: anchor_2,
            } => {
                let l1 = CurveIdEdge {
                    id: bezier_id_1.into(),
                    anchor_edge: anchor_1,
                };
                let l2 = CurveIdEdge {
                    id: bezier_id_2.into(),
                    anchor_edge: anchor_2,
                };

                latch_curves(&mut commands, l1, l2, &maps, &mut bezier_curves);

                if globals.sound_on {
                    if let Some(sound) = maps.sounds.get("latch") {
                        audio.play(sound.clone());
                    }
                }
            }
            HistoryAction::Unlatched {
                self_id,
                partner_id: partner_bezier_id,
                self_anchor,
                partner_anchor,
            } => {
                let handle_entity_1 = maps.bezier_map[&self_id.into()].clone();
                let bezier_1 = bezier_curves.get_mut(&handle_entity_1.handle).unwrap();
                bezier_1.latches.remove(&self_anchor);

                let handle_entity_2 = maps.bezier_map[&partner_bezier_id.into()].clone();
                let bezier_2 = bezier_curves.get_mut(&handle_entity_2.handle).unwrap();
                bezier_2.latches.remove(&partner_anchor);

                if globals.sound_on {
                    if let Some(sound) = maps.sounds.get("unlatch") {
                        audio.play(sound.clone());
                    }
                }
            }
            _ => {}
        }
        history.index += 1;
    }
}
