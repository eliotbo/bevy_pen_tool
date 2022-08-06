use crate::actions::*;
use crate::undo::*;

use crate::moves::*;
use bevy_pen_tool_spawner::*;

use bevy::prelude::*;

pub struct PenPlugin;

// TODO
// 0) change to bevy 0.8
// 1) undo/redo
// History of actions
// on mouseclick, if moving, remember initial position (preivous_position in Cursor)
// on mouse release, add action to history
// moved anchor
// 2) remove UserState

impl Plugin for PenPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_plugin(Material2dPlugin::<BezierMat>::default())
            .add_plugin(SpawnerPlugin)
            .add_event::<GroupBoxEvent>()
            .insert_resource(History::default())
            .add_startup_system(set_window_position)
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
                    .with_system(recompute_lut.label("recompute_lut"))
                    .with_system(save.after("recompute_lut"))
                    .with_system(latchy)
                    .with_system(update_lut)
                    .with_system(officiate_latch_partnership)
                    .with_system(selection_box_init)
                    .with_system(selection_final)
                    .with_system(hide_anchors)
                    .with_system(delete)
                    .with_system(hide_control_points)
                    .with_system(unselect)
                    .with_system(debug)
                    .with_system(ungroup)
                    .with_system(undo)
                    .with_system(redo)
                    .with_system(compute_lut_sender)
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
    pub move_quad: Anchor,
    pub color: Option<Color>,
    pub do_compute_lut: bool,
    pub id: u128,
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
            move_quad: bezier.move_quad,
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

fn debug(
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<&Handle<Bezier>, With<BezierParent>>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    groups: Res<Assets<Group>>,
    mids_groups: Query<&GroupMiddleQuad>,
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
        println!("history actions: {:#?}", history.actions);
        println!("history actions len: {:#?}", history.actions.len());
        println!("history index: {:?}", history.index);
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
