use crate::actions::*;
use crate::io::{load, load_mesh, save};
use crate::moves::*;
use crate::pen::*;
use crate::undo::*;

use bevy::prelude::*;
use bevy_obj::*;
use bevy_pen_tool_model::*;
pub struct BevyPenToolPlugin;

// use bevy::render::{render_graph::RenderGraph, RenderApp};
// use bevy::window::{CreateWindow, WindowId};
// use bevy_inspector_egui::InspectorPlugin;
// use once_cell::sync::Lazy;

// TODO

// 0) unlatch bezier that is part of group
// 7) automatically group latched curves --
//      every chain is a group, even when it is just one curve
//      unlatching produces two new groups

// 1) use UI camera
// 2) disable and hide color picking
// 5) disable shortcuts
// 6) make most of the structs public(crate) and
//     leave only the API public
// 8) fix save all
// 9) move structs in correct files (ex: RoadMesh, FillMesh in mesh)
// 10) wrong unselect when moving a selected mesh
// 11) make io a feature
// 12) replace middle quads with line segments

// 13) scale meshes
// 14) no_std

impl Plugin for BevyPenToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(PenApiPlugin)
            .add_plugin(ObjPlugin)
            .add_plugin(SpawnerPlugin)
            .add_event::<RemoveMovingQuadEvent>()
            .add_event::<GroupBoxEvent>()
            .add_event::<SpawningCurve>()
            .add_event::<SpawnCurve>()
            .add_event::<UnlatchEvent>()
            .insert_resource(History::default())
            .add_startup_system(set_window_position)
            //
            .add_system(debug)
            .add_system(remove_all_moving_quad)
            .add_system(check_mouse_on_meshes)
            .add_system(unlatchy)
            .add_system(compute_group_lut)
            .add_system(load_mesh)
            //
            // Update model
            .add_system_set(
                SystemSet::on_update("ModelViewController")
                    // .with_system(groupy)
                    // .with_system(ungroupy)
                    .with_system(latchy)
                    .with_system(update_lut)
                    .with_system(officiate_latch_partnership)
                    .with_system(selection_box_init)
                    .with_system(selection_area_finalize)
                    .with_system(hide_anchors)
                    .with_system(delete)
                    .with_system(hide_control_points)
                    .with_system(unselect)
                    .with_system(undo)
                    .with_system(redo)
                    .with_system(redo_effects)
                    .with_system(add_to_history)
                    .with_system(update_anchors.exclusive_system().at_end())
                    .label("model"),
            )
            .add_system_set(
                SystemSet::on_update("ModelViewController")
                    .with_system(load)
                    .with_system(save)
                    .after("model"),
            )
            //
            // Update view
            .add_system_set(
                SystemSet::on_update("ModelViewController")
                    .with_system(bezier_anchor_order)
                    .with_system(move_end_quads)
                    .with_system(move_middle_quads)
                    .with_system(move_group_middle_quads)
                    .with_system(move_control_quads)
                    .with_system(move_bb_quads)
                    .with_system(move_ui)
                    .with_system(move_mesh)
                    .with_system(turn_round_animation)
                    .with_system(follow_bezier_group)
                    .label("view")
                    // .with_system(rescale)
                    .after("model"),
            );

        // ///////////////////////// inspector

        // app.add_plugin(InspectorPlugin::<History>::new())
        //     .add_plugin(InspectorPlugin::<HistoryInspector>::new().on_window(*SECOND_WINDOW_ID))
        //     .add_plugin(InspectorPlugin::<HistoryLenInspector>::new().on_window(*SECOND_WINDOW_ID))
        //     .insert_resource(HistoryInspector::default())
        //     .insert_resource(HistoryLenInspector::default())
        //     .add_startup_system(create_new_window)
        //     .add_system(update_history_inspector);

        // let render_app = app.sub_app_mut(RenderApp);
        // let mut graph = render_app.world.get_resource_mut::<RenderGraph>().unwrap();
        // bevy_egui::setup_pipeline(
        //     &mut graph,
        //     bevy_egui::RenderGraphConfig {
        //         window_id: *SECOND_WINDOW_ID,
        //         egui_pass: SECONDARY_EGUI_PASS,
        //     },
        // );

        // ///////////////////////// inspector
    }
}

pub fn remove_all_moving_quad(
    mut commands: Commands,
    mut events: EventReader<RemoveMovingQuadEvent>,
    mut query: Query<(Entity, &MovingAnchor)>,
) {
    for _ in events.iter() {
        for (entity, _anchor) in query.iter_mut() {
            commands.entity(entity).remove::<MovingAnchor>();
        }
    }
}

fn set_window_position(mut windows: ResMut<Windows>) {
    for window in windows.iter_mut() {
        window.set_position(IVec2::new(0, 0));
    }
}

//////////////////////////// Debugging ////////////////////////////
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Clone, Debug)]
struct BezierPrint {
    pub positions: BezierPositions,
    pub previous_positions: BezierPositions,
    // pub move_quad: Anchor,
    pub color: Option<Color>,
    pub do_compute_lut: bool,
    pub id: BezierId,
    pub latches: HashMap<AnchorEdge, LatchData>,
    pub potential_latch: Option<LatchData>,
    pub grouped: bool,
}

impl BezierPrint {
    #[allow(dead_code)]
    pub fn from_bezier(bezier: &Bezier) -> Self {
        Self {
            positions: bezier.positions.clone(),
            previous_positions: bezier.positions.clone(),
            // move_quad: bezier.move_quad,
            color: None,
            do_compute_lut: false,
            id: bezier.id,
            latches: bezier.latches.clone(),
            potential_latch: None,
            grouped: false,
        }
    }
}

use std::collections::HashSet;
#[derive(Clone, Debug)]
#[allow(dead_code)]
struct GroupPrint {
    // TODO: rid Group of redundancy
    pub group: HashSet<(Entity, Handle<Bezier>)>,
    pub bezier_handles: HashSet<Handle<Bezier>>,
    //
    // Attempts to store the start and end points of a group.
    // Fails if curves are not connected
    pub ends: Option<Vec<(Handle<Bezier>, AnchorEdge)>>,
}

impl GroupPrint {
    #[allow(dead_code)]
    pub fn from_group(group: &Group) -> Self {
        Self {
            group: group.group.clone(),
            bezier_handles: group.bezier_handles.clone(),
            ends: group.ends.clone(),
        }
    }
}

// static SECOND_WINDOW_ID: Lazy<WindowId> = Lazy::new(WindowId::new);
// const SECONDARY_EGUI_PASS: &str = "secondary_egui_pass";

// fn update_history_inspector(
//     history: ResMut<History>,
//     mut history_inspector: ResMut<HistoryInspector>,
//     mut history_len_inspector: ResMut<HistoryLenInspector>,
// ) {
//     if history.is_changed() {
//         *history_inspector = HistoryInspector::from(history.clone());
//         history_len_inspector.length = history_inspector.history.len();
//         history_len_inspector.index = history_inspector.index;
//     }
// }

// fn create_new_window(mut create_window_events: EventWriter<CreateWindow>) {
//     let window_id = *SECOND_WINDOW_ID;

//     create_window_events.send(CreateWindow {
//         id: window_id,
//         descriptor: WindowDescriptor {
//             width: 800.,
//             height: 600.,
//             position: WindowPosition::At(Vec2::new(1350., 0.)),
//             title: "Second window".to_string(),
//             ..Default::default()
//         },
//     });
// }

// pub fn compute_group_lut(
//     // mut commands: Commands,
//     bezier_curves: Res<Assets<Bezier>>,
//     mut groups: ResMut<Assets<Group>>,
//     mut group_lut_event_reader: EventReader<ComputeGroupLut>,
//     maps: Res<Maps>,
//     globals: Res<Globals>,
// ) {
//     for group_lut_event in group_lut_event_reader.iter() {
//         if let Some(group_handle) = maps.group_map.get(&group_lut_event.0) {
//             let group = groups.get_mut(group_handle).unwrap();

//             let bezier_assets = bezier_curves
//                 .iter()
//                 .collect::<HashMap<bevy::asset::HandleId, &Bezier>>();

//             group.find_connected_ends(&bezier_assets, maps.bezier_map.clone());
//             group.group_lut(&bezier_assets, maps.bezier_map.clone());
//             group.compute_standalone_lut(&bezier_assets, globals.group_lut_num_points);
//             println!("group lut computed");
//         }
//     }
// }

fn debug(
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<&Handle<Bezier>, With<BezierParent>>,
    bezier_curves: ResMut<Assets<Bezier>>,
    maps: Res<Maps>,
    globals: Res<Globals>,
    mut groups: ResMut<Assets<Group>>,
    // mids_groups: Query<&GroupMiddleQuad>,
    // maps: Res<Maps>,
    // history: Res<History>,
    // mut action_event_writer: EventWriter<Action>,
) {
    if keyboard_input.just_pressed(KeyCode::B)
        && !keyboard_input.pressed(KeyCode::LShift)
        && !keyboard_input.pressed(KeyCode::LControl)
    {
        // println!("group_handles: {:?}", maps.id_group_handle);
        // println!("'B' currently pressed");
        // let latches: Vec<Latch> = vec![];
        for _handle in query.iter() {
            // let bezier = bezier_curves.get_mut(handle).unwrap();
            // action_event_writer.send(Action::ComputeLut);

            // latches.push()

            // println!("group id: {:?}", bezier.group);
            // println!("latches: {:#?}", BezierPrint::from_bezier(bezier).latches);
            // println!("potential latche: {:#?}", bezier.potential_latch);
        }

        // println!("mids: {:?}", mids_groups.iter().count());
        // println!("history actions: {:#?}", history.actions);
        // println!("history actions len: {:#?}", history.actions.len());
        // println!("history index: {:?}", history.index);
        // println!("map: {:?}", maps.print_bezier_map());
        // println!("");
    }

    if keyboard_input.just_pressed(KeyCode::G) {
        // println!("'B' currently pressed");
        println!("groups: {:#?}", groups.iter().count());
        println!("group map: {:?}", maps.group_map);
        for (_, group) in groups.iter_mut() {
            // let bezier = bezier_curves.get_mut(handle).unwrap();

            let bezier_assets = bezier_curves
                .iter()
                .collect::<HashMap<bevy::asset::HandleId, &Bezier>>();

            group.find_connected_ends(&bezier_assets, maps.bezier_map.clone());
            group.group_lut(&bezier_assets, maps.bezier_map.clone());
            group.compute_standalone_lut(&bezier_assets, globals.group_lut_num_points);

            // println!("group: {:#?}", GroupPrint::from_group(group));
            // println!("");
        }
    }
}
//////////////////////////// Debugging ////////////////////////////

// const P1: Vec2 = Vec2::new(0.0, 0.0);
// const P2: Vec2 = Vec2::new(0.0, 0.0);
// const P3: Vec2 = Vec2::new(0.0, 0.0);
// const P4: Vec2 = Vec2::new(0.0, 0.0);
// const P5: Vec2 = Vec2::new(0.0, 0.0);
// const P6: Vec2 = Vec2::new(0.0, 0.0);
// const P7: Vec2 = Vec2::new(0.0, 0.0);
// const P8: Vec2 = Vec2::new(0.0, 0.0);

// pub fn first_test() {
//     // Setup app
//     let mut app = App::new();

//     app.add_plugins(DefaultPlugins);
//     app.add_plugin(BevyPenToolPlugin);

//     // Add Score resource

//     // Add helper system to access bezier curves
//     // (can't access assets through world directly)
//     // app.add_system(update_bez);

//     // Run systems once
//     app.update();

//     // // TODO: here, we have to create related systems that will do the logic PenCommands,
//     // // but we can enter the actual values here inside the #[test]

//     let mut pen_commands = app.world.get_resource_mut::<PenCommandVec>().unwrap();
//     // initiate a BezierPositions
//     let mut positions1 = BezierPositions {
//         start: Vec2::new(200., -100.),
//         end: Vec2::new(100., 100.),
//         control_start: Vec2::new(0., 100.),
//         control_end: Vec2::new(100., 100.),
//     };

//     let id1 = pen_commands.spawn(positions1);

//     let positions2 = BezierPositions {
//         start: Vec2::ZERO,
//         end: Vec2::new(-100., -100.),
//         control_start: Vec2::new(0., -100.),
//         control_end: Vec2::new(100., -200.),
//     };

//     let id2 = pen_commands.spawn(positions2);

//     app.update();

//     let mut pen_commands = app.world.get_resource_mut::<PenCommandVec>().unwrap();
//     pen_commands.move_anchor(id1, Anchor::Start, Vec2::ZERO);

//     positions1 = BezierPositions {
//         start: Vec2::ZERO,
//         end: Vec2::new(100., 100.),
//         control_start: Vec2::new(0., 100.) - Vec2::new(200., -100.), // moves with start
//         control_end: Vec2::new(100., 100.),
//     };

//     pen_commands.latch(
//         CurveIdEdge {
//             id: id1,
//             anchor_edge: AnchorEdge::Start,
//         },
//         CurveIdEdge {
//             id: id2,
//             anchor_edge: AnchorEdge::Start,
//         },
//     );

//     pen_commands.unlatch(
//         CurveIdEdge {
//             id: id1,
//             anchor_edge: AnchorEdge::Start,
//         },
//         CurveIdEdge {
//             id: id2,
//             anchor_edge: AnchorEdge::Start,
//         },
//     );

//     let mut target_positions = app.world.get_resource_mut::<TargetPositions>().unwrap();

//     target_positions.0.insert(id1, positions1);
//     target_positions.0.insert(id2, positions2);

//     app.update();

//     // let bezier_curves = app.world.resource::<BezierTestHashed>();
//     // let target_positions = app.world.resource::<TargetPositions>();

//     // for (id, target_pos) in target_positions.0.iter() {
//     //     let bezier = bezier_curves.0.get(&id).unwrap();
//     //     let bezier_state = BezierState::from(bezier);
//     //     assert_eq!(&bezier_state.positions, target_pos);
//     // }
// }
