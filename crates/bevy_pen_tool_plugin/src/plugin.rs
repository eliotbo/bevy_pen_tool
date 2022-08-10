use crate::actions::*;
use crate::undo::*;

use crate::moves::*;
use bevy_pen_tool_spawner::*;

use bevy::prelude::*;
use bevy::render::{render_graph::RenderGraph, RenderApp};

pub struct PenPlugin;
use bevy::window::{CreateWindow, WindowId};
use bevy_inspector_egui::InspectorPlugin;

use once_cell::sync::Lazy;
// TODO

// 1) undo/redo for groups
// 2) remove UserState
// 3) replace move_quad Anchor::All, by event

impl Plugin for PenPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_plugin(Material2dPlugin::<BezierMat>::default())
            .add_plugin(SpawnerPlugin)
            .add_plugin(InspectorPlugin::<History>::new())
            .add_plugin(InspectorPlugin::<HistoryInspector>::new().on_window(*SECOND_WINDOW_ID))
            .add_plugin(InspectorPlugin::<HistoryLenInspector>::new().on_window(*SECOND_WINDOW_ID))
            .add_event::<RemoveMovingQuadEvent>()
            .add_event::<GroupBoxEvent>()
            .insert_resource(History::default())
            .insert_resource(HistoryInspector::default())
            .insert_resource(HistoryLenInspector::default())
            .add_startup_system(set_window_position)
            .add_startup_system(create_new_window)
            //
            .add_system(debug)
            .add_system(update_history_inspector)
            .add_system(remove_all_moving_quad)
            //
            // Update controller
            .add_system_set(
                SystemSet::on_update("ModelViewController")
                    .with_system(rescale)
                    .label("controller"),
            )
            //
            // Update model
            .add_system_set(
                SystemSet::on_update("ModelViewController")
                    .with_system(groupy.label("group"))
                    .with_system(load.after("group"))
                    // .with_system(recompute_all_lut.label("recompute_lut"))
                    // .with_system(save.after("recompute_lut"))
                    .with_system(save)
                    .with_system(latchy)
                    .with_system(update_lut)
                    .with_system(officiate_latch_partnership)
                    .with_system(selection_box_init)
                    .with_system(selection_final)
                    .with_system(hide_anchors)
                    .with_system(delete)
                    .with_system(hide_control_points)
                    .with_system(unselect)
                    .with_system(ungroup)
                    .with_system(undo)
                    .with_system(redo)
                    .with_system(redo_effects)
                    .with_system(add_to_history)
                    .label("model")
                    .after("controller")
                    .with_system(update_anchors.exclusive_system().at_end()),
            )
            //
            // Update view
            .add_system_set(
                SystemSet::on_update("ModelViewController")
                    // TODO:
                    // mouse_release_actions should be in the controller,
                    // but there is a bug with the position of new latches when it's there
                    //
                    .with_system(bezier_anchor_order)
                    .with_system(move_end_quads)
                    .with_system(move_middle_quads)
                    .with_system(move_group_middle_quads)
                    .with_system(move_control_quads)
                    .with_system(move_bb_quads)
                    .with_system(move_ui)
                    .with_system(turn_round_animation)
                    .with_system(follow_bezier_group)
                    // .with_system(spawn_group_middle_quads)
                    .label("view")
                    .after("model"),
            );

        let render_app = app.sub_app_mut(RenderApp);
        let mut graph = render_app.world.get_resource_mut::<RenderGraph>().unwrap();
        bevy_egui::setup_pipeline(
            &mut graph,
            bevy_egui::RenderGraphConfig {
                window_id: *SECOND_WINDOW_ID,
                egui_pass: SECONDARY_EGUI_PASS,
            },
        );
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

static SECOND_WINDOW_ID: Lazy<WindowId> = Lazy::new(WindowId::new);
const SECONDARY_EGUI_PASS: &str = "secondary_egui_pass";

fn update_history_inspector(
    history: ResMut<History>,
    mut history_inspector: ResMut<HistoryInspector>,
    mut history_len_inspector: ResMut<HistoryLenInspector>,
) {
    if history.is_changed() {
        *history_inspector = HistoryInspector::from(history.clone());
        history_len_inspector.length = history_inspector.history.len();
        history_len_inspector.index = history_inspector.index;
    }
}

fn create_new_window(mut create_window_events: EventWriter<CreateWindow>) {
    let window_id = *SECOND_WINDOW_ID;

    create_window_events.send(CreateWindow {
        id: window_id,
        descriptor: WindowDescriptor {
            width: 800.,
            height: 600.,
            position: WindowPosition::At(Vec2::new(1350., 0.)),
            title: "Second window".to_string(),
            ..Default::default()
        },
    });
}

fn debug(
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<&Handle<Bezier>, With<BezierParent>>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    groups: Res<Assets<Group>>,
    // mids_groups: Query<&GroupMiddleQuad>,
    maps: Res<Maps>,
    history: Res<History>,
    mut action_event_writer: EventWriter<Action>,
) {
    if keyboard_input.just_pressed(KeyCode::B)
        && !keyboard_input.pressed(KeyCode::LShift)
        && !keyboard_input.pressed(KeyCode::LControl)
    {
        // println!("group_handles: {:?}", maps.id_group_handle);
        // println!("'B' currently pressed");
        for handle in query.iter() {
            let _bezier = bezier_curves.get_mut(handle).unwrap();
            action_event_writer.send(Action::ComputeLut);

            // println!("group id: {:?}", bezier.group);
            // println!("latches: {:#?}", BezierPrint::from_bezier(bezier));
        }

        // println!("mids: {:?}", mids_groups.iter().count());
        // println!("history actions: {:#?}", history.actions);
        println!("history actions len: {:#?}", history.actions.len());
        println!("history index: {:?}", history.index);
        println!("map: {:?}", maps.print_bezier_map());
        println!("");
    }

    if keyboard_input.just_pressed(KeyCode::G) {
        // println!("'B' currently pressed");
        println!("groups: {:#?}", groups.iter().count());
        for (_, _group) in groups.iter() {
            // let bezier = bezier_curves.get_mut(handle).unwrap();

            // println!("group: {:#?}", GroupPrint::from_group(group));
            // println!("");
        }
    }
}
//////////////////////////// Debugging ////////////////////////////
