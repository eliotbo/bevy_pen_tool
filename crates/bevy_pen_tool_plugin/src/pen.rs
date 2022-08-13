use crate::actions::*;
use crate::undo::*;

use crate::moves::*;
use bevy_pen_tool_spawner::*;

use bevy::prelude::*;
use bevy::render::{render_graph::RenderGraph, RenderApp};

use bevy::window::{CreateWindow, WindowId};
use bevy_inspector_egui::InspectorPlugin;

use once_cell::sync::Lazy;
use rand::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Default, PartialEq)]
pub struct TestState(Vec<BezierState>);

#[derive(Default, PartialEq)]
pub struct BezierState {
    pub positions: BezierPositions,
    pub previous_positions: BezierPositions,
    pub id: BezierId,
    pub latches: HashMap<AnchorEdge, LatchData>,
    pub potential_latch: Option<LatchData>,
    pub group: Option<GroupId>,
}

impl From<&Bezier> for BezierState {
    fn from(bezier: &Bezier) -> Self {
        Self {
            positions: bezier.positions,
            previous_positions: bezier.previous_positions,
            id: bezier.id,
            latches: bezier.latches.clone(),
            potential_latch: bezier.potential_latch.clone(),
            group: bezier.group,
        }
    }
}

pub struct PenCommandVec(pub Vec<PenCommand>);

impl PenCommandVec {
    pub fn spawn(&mut self, positions: BezierPositions) -> BezierId {
        let mut rng = thread_rng();
        let id: u64 = rng.gen();
        self.0.push(PenCommand::Spawn {
            positions,
            id: id.into(),
        });
        id.into()
    }

    pub fn move_anchor(&mut self, id: BezierId, anchor: Anchor, position: Vec2) {
        self.0.push(PenCommand::Move(MoveCommand {
            anchor,
            id,
            new_position: position,
        }));
    }

    pub fn latch(&mut self, l1: CurveIdEdge, l2: CurveIdEdge) {
        self.0.push(PenCommand::Latch { l1, l2 });
    }

    pub fn delete(&mut self, id: BezierId) {
        self.0.push(PenCommand::Delete { id });
    }

    pub fn unlatch(&mut self, l1: CurveIdEdge, l2: CurveIdEdge) {
        self.0.push(PenCommand::Unlatch { l1, l2 });
    }

    pub fn undo(&mut self) {
        self.0.push(PenCommand::Undo);
    }

    pub fn redo(&mut self) {
        self.0.push(PenCommand::Redo);
    }
}

pub struct PenApiPlugin;

#[derive(Copy, Clone)]
pub struct FrameNumber(pub i32);

impl Plugin for PenApiPlugin {
    fn build(&self, app: &mut App) {
        app

            .insert_resource(PenCommandVec(Vec::new()))
            .insert_resource(FrameNumber(0))
            .insert_resource(CurveVec(Vec::new()))
            // .add_startup_system(test.label("label"))
            // .add_system(test2)
            .add_system(direct_api_calls)
            // .add_system(test2_assert)
            // .add_system(test2)
            ;
    }
}

// pub fn test(mut pen_commands: ResMut<PenCommandVec>, mut all_curves: ResMut<CurveVec>) {
//     //

// }

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
                        selection
                            .selected
                            .group
                            .insert((handle_entity.entity, handle_entity.handle.clone()));
                        action_event_writer.send(Action::Delete(false));
                        info!("DELETING: {:?}", id);
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

    // for SpawnCurve { positions } in spawn_curve_event_reader.iter() {
    //     use rand::prelude::*;
    //     let mut rng = rand::thread_rng();
    //     let id = rng.gen::<u64>();

    //     spawning_curve_event_writer.send(SpawningCurve {
    //         bezier_hist: Some(BezierHist::new(*positions, id)),
    //         maybe_bezier_id: Some(id.into()),
    //         follow_mouse: false,
    //     });
    // }
}
