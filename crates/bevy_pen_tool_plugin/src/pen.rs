//! Small API for programmatically producing Bezier curves, moving anchors, attaching curves together, and more.

use crate::undo::*;

use bevy::prelude::*;
use bevy_pen_tool_model::*;

use rand::prelude::*;
use std::collections::HashSet;

pub(crate) enum PenCommand {
    Spawn {
        positions: BezierPositions,
        id: BezierId,
    },

    Move(MoveCommand),

    Latch {
        l1: CurveIdEdge,
        l2: CurveIdEdge,
    },

    Unlatch {
        l1: CurveIdEdge,
        l2: CurveIdEdge,
    },

    Delete {
        id: BezierId,
    },

    Undo,
    Redo,
}

/// Identifies a specific anchor edge (start or end point) of a specific Bezier curve.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct CurveIdEdge {
    pub id: BezierId,
    pub anchor_edge: AnchorEdge,
}

/// Identifies the new position for a single anchor of a Bezier curve.
#[derive(Copy, Clone)]
pub struct MoveCommand {
    pub anchor: Anchor,
    pub id: BezierId,
    pub new_position: Vec2,
}

/// Commands that have effects over Bezier curves. Although sending simultaneous [`Move`] commands is supported,
/// in general, sending simultaneous commands (in the same frame) is not supported: they may lead to a panic!
/// It is recommended to separate successive method calls by about ten frames.
pub struct PenCommandVec(Vec<PenCommand>);

impl PenCommandVec {
    /// Spawn a new Bezier curve with the given anchor positions.
    pub fn spawn(&mut self, positions: BezierPositions) -> BezierId {
        let mut rng = thread_rng();
        let id: u64 = rng.gen();
        self.0.push(PenCommand::Spawn {
            positions,
            id: id.into(),
        });
        id.into()
    }

    /// Move a single anchor of a given Bezier curve.
    pub fn move_anchor(&mut self, id: BezierId, anchor: Anchor, position: Vec2) {
        self.0.push(PenCommand::Move(MoveCommand {
            anchor,
            id,
            new_position: position,
        }));
    }

    /// Latch two Bezier curves together, given the two anchor edges.
    pub fn latch(&mut self, l1: CurveIdEdge, l2: CurveIdEdge) {
        self.0.push(PenCommand::Latch { l1, l2 });
    }

    /// Delete a Bezier curve. This command will also unlatch any anchor that is connected to this curve.
    pub fn delete(&mut self, id: BezierId) {
        self.0.push(PenCommand::Delete { id });
    }

    /// Unlatches two Bezier curves, given the two anchor edges that will be unlatched.
    pub fn unlatch(&mut self, l1: CurveIdEdge, l2: CurveIdEdge) {
        self.0.push(PenCommand::Unlatch { l1, l2 });
    }

    /// Undo a command. Useful for internal tests, but not very useful for users of the API.
    pub fn undo(&mut self) {
        self.0.push(PenCommand::Undo);
    }

    /// Redo a command. Useful for internal tests, but not very useful for users of the API.
    pub fn redo(&mut self) {
        self.0.push(PenCommand::Redo);
    }
}

pub(crate) fn move_anchor(
    commands: &mut Commands,
    move_command: MoveCommand,
    mut bezier_curves: &mut ResMut<Assets<Bezier>>,
    maps: &ResMut<Maps>,
) {
    // println!("undo: MovedAnchor");
    let anchor = move_command.anchor;
    let handle_entities = maps.bezier_map[&move_command.id.into()].clone();
    let bezier = bezier_curves.get_mut(&handle_entities.handle).unwrap();

    bezier.set_position(anchor, move_command.new_position);

    // attaches MovingAnchor component to the entity
    bezier.move_anchor(
        commands,
        true,  // one move for a single frame
        false, // do not follow mouse
        anchor,
        maps.as_ref(),
    );

    let anchor_edge = anchor.to_edge_with_controls();
    if let Some(_) = bezier.latches.get(&anchor_edge) {
        let latch_info = bezier.get_anchor_latch_info(anchor);

        update_latched_partner_position(&maps.bezier_map, &mut bezier_curves, latch_info);
    }
}

pub(crate) struct PenApiPlugin;

impl Plugin for PenApiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PenCommandVec(Vec::new()))
            .add_system(direct_api_calls);
    }
}

fn direct_api_calls(
    mut commands: Commands,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    // mut spawn_curve_event_reader: EventReader<SpawnCurve>,
    mut spawning_curve_event_writer: EventWriter<SpawningCurve>,
    mut pen_command_vec: ResMut<PenCommandVec>,
    mut action_event_writer: EventWriter<Action>,
    mut selection: ResMut<Selection>,
    mut maps: ResMut<Maps>,
    mut add_to_history_event_writer: EventWriter<HistoryAction>,
) {
    if pen_command_vec.is_changed() {
        for pen_command in pen_command_vec.0.iter() {
            match pen_command {
                PenCommand::Spawn { positions, id } => {
                    spawning_curve_event_writer.send(SpawningCurve {
                        bezier_hist: Some(BezierHist::new(*positions, (*id).into())),
                        maybe_bezier_id: Some((*id).into()),
                        follow_mouse: false,
                    });
                }
                PenCommand::Move(move_command) => {
                    move_anchor(&mut commands, *move_command, &mut bezier_curves, &maps);
                    let handle_entity = maps.bezier_map[&move_command.id].clone();

                    let bezier = bezier_curves.get_mut(&handle_entity.handle).unwrap();

                    // info!("Anchor position history: {:?}", history_action);

                    add_to_history_event_writer.send(HistoryAction::MovedAnchor {
                        anchor: move_command.anchor,
                        bezier_id: move_command.id.into(),
                        previous_position: bezier.get_position(move_command.anchor),
                        new_position: move_command.new_position,
                    });
                }
                PenCommand::Latch { l1, l2 } => {
                    info!("latch");
                    latch_curves(&mut commands, *l1, *l2, &maps, &mut bezier_curves);
                    add_to_history_event_writer.send(HistoryAction::Latched {
                        self_id: l1.id.into(),
                        self_anchor: l1.anchor_edge,
                        partner_id: l2.id.into(),
                        partner_anchor: l2.anchor_edge,
                    });
                }

                PenCommand::Unlatch { l1, l2 } => {
                    info!("unlatch");
                    let handle_entity_1 = maps.bezier_map[&l1.id.into()].clone();
                    let bezier_1 = bezier_curves.get_mut(&handle_entity_1.handle).unwrap();
                    bezier_1.latches.remove(&l1.anchor_edge);

                    let handle_entity_2 = maps.bezier_map[&l2.id.into()].clone();
                    let bezier_2 = bezier_curves.get_mut(&handle_entity_2.handle).unwrap();
                    bezier_2.latches.remove(&l2.anchor_edge);

                    add_to_history_event_writer.send(HistoryAction::Unlatched {
                        self_id: l1.id.into(),
                        partner_id: l2.id.into(),
                        self_anchor: l1.anchor_edge,
                        partner_anchor: l2.anchor_edge,
                    });
                }
                PenCommand::Delete { id } => {
                    if let Some(handle_entity) = maps.bezier_map.get(&id) {
                        let mut new_set = HashSet::new();
                        new_set.insert(handle_entity.handle.id.into());

                        selection.selected = vec![SelectionChoice::CurveSet(new_set)];

                        action_event_writer
                            .send(Action::Delete(false /* do not add to history */));
                    }
                    if let None = maps.bezier_map.remove(&id) {
                        info!("COULD NOT DELETE CURVE FROM MAP: {:?}", id);
                    }
                }
                PenCommand::Undo => {
                    action_event_writer.send(Action::Undo);
                }
                PenCommand::Redo => {
                    action_event_writer.send(Action::Redo);
                }
            }
        }
        pen_command_vec.0.clear();
    }
}
