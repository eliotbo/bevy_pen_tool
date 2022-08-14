use bevy_pen_tool_model::model::*;
use bevy_pen_tool_plugin::{pen::*, BevyPenToolPlugin, Bezier};

use bevy::prelude::*;
use std::collections::HashMap;

pub struct TargetLatches(pub HashMap<CurveIdEdge, CurveIdEdge>);

fn main() {
    let mut app = App::new();
    app.insert_resource(BezierTestHashed(HashMap::new()))
        .insert_resource(TargetLatches(HashMap::new()))
        .add_plugins(DefaultPlugins)
        .add_plugin(BevyPenToolPlugin)
        .add_system(update_bez);

    app.update();

    let mut pen_commands = app.world.get_resource_mut::<PenCommandVec>().unwrap();

    let positions1 = BezierPositions::ZERO;
    let positions2 = BezierPositions::ZERO;

    let id1 = pen_commands.spawn(positions1);
    let id2 = pen_commands.spawn(positions2);

    app.update();
    app.update();
    app.update();

    let mut pen_commands = app.world.get_resource_mut::<PenCommandVec>().unwrap();

    let latch1 = CurveIdEdge {
        id: id1,
        anchor_edge: AnchorEdge::Start,
    };
    let latch2 = CurveIdEdge {
        id: id2,
        anchor_edge: AnchorEdge::Start,
    };

    pen_commands.latch(latch1, latch2);

    app.update();
    app.update();
    app.update();

    let mut pen_commands = app.world.get_resource_mut::<PenCommandVec>().unwrap();
    pen_commands.undo();

    app.update();
    app.update();
    app.update();

    let bezier_curves = app.world.resource::<BezierTestHashed>();

    for (_id, bezier) in bezier_curves.0.iter() {
        assert!(bezier.latches.is_empty());
    }
    println!("undo_latch_test passed");
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
