use bevy_pen_tool_model::model::*;
use bevy_pen_tool_plugin::{pen::*, BevyPenToolPlugin, Bezier};

use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Default, PartialEq)]
// pub(crate) struct TestState(Vec<BezierState>);

// #[derive(Default, PartialEq)]
// pub(crate) struct BezierState {
//     pub positions: BezierPositions,
//     pub previous_positions: BezierPositions,
//     pub id: BezierId,
//     pub latches: HashMap<AnchorEdge, LatchData>,
//     pub potential_latch: Option<LatchData>,
//     pub group: Option<GroupId>,
// }

// impl From<&Bezier> for BezierState {
//     fn from(bezier: &Bezier) -> Self {
//         Self {
//             positions: bezier.positions,
//             previous_positions: bezier.previous_positions,
//             id: bezier.id,
//             latches: bezier.latches.clone(),
//             potential_latch: bezier.potential_latch.clone(),
//             group: bezier.group,
//         }
//     }
// }

pub struct TargetPositions(pub HashMap<BezierId, BezierPositions>);
fn main() {
    let mut app = App::new();
    app.insert_resource(BezierTestHashed(HashMap::new()))
        .add_plugins(DefaultPlugins)
        .add_plugin(BevyPenToolPlugin)
        .insert_resource(TargetPositions(HashMap::new()))
        .add_system(update_bez);

    // Add Score resource

    // Run systems once
    app.update();

    // // TODO: here, we have to create related systems that will do the logic PenCommands,
    // // but we can enter the actual values here inside the #[test]

    let mut pen_commands = app.world.get_resource_mut::<PenCommandVec>().unwrap();
    // initiate a BezierPositions
    let mut positions1 = BezierPositions {
        start: Vec2::new(200., -100.),
        end: Vec2::new(100., 100.),
        control_start: Vec2::new(0., 100.),
        control_end: Vec2::new(100., 100.),
    };

    let id1 = pen_commands.spawn(positions1);

    let positions2 = BezierPositions {
        start: Vec2::ZERO,
        end: Vec2::new(-100., -100.),
        control_start: Vec2::new(0., -100.),
        control_end: Vec2::new(100., -200.),
    };

    let id2 = pen_commands.spawn(positions2);

    // the app needs some time to perform the tasks,
    // since they are event and asset based
    app.update();
    app.update();
    app.update();

    let mut pen_commands = app.world.get_resource_mut::<PenCommandVec>().unwrap();
    pen_commands.move_anchor(id1, Anchor::Start, Vec2::ZERO);

    // pen_commands.latch(
    //     CurveIdEdge {
    //         id: id1,
    //         anchor_edge: AnchorEdge::Start,
    //     },
    //     CurveIdEdge {
    //         id: id2,
    //         anchor_edge: AnchorEdge::Start,
    //     },
    // );

    // pen_commands.unlatch(
    //     CurveIdEdge {
    //         id: id1,
    //         anchor_edge: AnchorEdge::Start,
    //     },
    //     CurveIdEdge {
    //         id: id2,
    //         anchor_edge: AnchorEdge::Start,
    //     },
    // );

    let mut target_positions = app.world.get_resource_mut::<TargetPositions>().unwrap();

    positions1 = BezierPositions {
        start: Vec2::ZERO,
        end: Vec2::new(100., 100.),
        control_start: Vec2::new(0., 100.) - Vec2::new(200., -100.), // moves with start
        control_end: Vec2::new(100., 100.),
    };

    target_positions.0.insert(id1, positions1);
    target_positions.0.insert(id2, positions2);

    app.update();
    app.update();
    app.update();

    let bezier_curves = app.world.resource::<BezierTestHashed>();
    let target_positions = app.world.resource::<TargetPositions>();

    for (id, target_pos) in target_positions.0.iter() {
        let bezier = bezier_curves.0.get(&id).unwrap();
        // let bezier_state = BezierState::from(bezier);
        assert_eq!(&bezier.positions, target_pos);
    }
    println!("move_test passed");
}

pub struct BezierTestHashed(pub HashMap<BezierId, Bezier>);

pub fn update_bez(
    bezier_curves: Res<Assets<Bezier>>,
    mut bezier_curves_test: ResMut<BezierTestHashed>,
) {
    // if bezier_curves.is_changed() {
    for (handle_id, bez) in bezier_curves.iter() {
        let id = BezierId(handle_id);
        bezier_curves_test.0.insert(id, bez.clone());
    }
    // }
}
