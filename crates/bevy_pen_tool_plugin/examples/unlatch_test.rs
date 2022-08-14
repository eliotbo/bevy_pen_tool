use bevy_pen_tool_model::model::*;
use bevy_pen_tool_plugin::{pen::*, BevyPenToolPlugin, Bezier};

use bevy::prelude::*;
use std::collections::HashMap;

fn main() {
    let mut app = App::new();
    app.insert_resource(BezierTestHashed(HashMap::new()))
        .add_plugins(DefaultPlugins)
        .add_plugin(BevyPenToolPlugin)
        .add_system(update_bez);

    // Run systems once
    app.update();

    let mut pen_commands = app.world.get_resource_mut::<PenCommandVec>().unwrap();

    let positions1 = BezierPositions::ZERO;
    let positions2 = BezierPositions::ZERO;

    let id1 = pen_commands.spawn(positions1);
    let id2 = pen_commands.spawn(positions2);

    // the app needs some time to perform the tasks,
    // since they are event and asset based
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

    pen_commands.unlatch(
        CurveIdEdge {
            id: id1,
            anchor_edge: AnchorEdge::Start,
        },
        CurveIdEdge {
            id: id2,
            anchor_edge: AnchorEdge::Start,
        },
    );

    app.update();
    app.update();
    app.update();

    let maps = app.world.resource::<Maps>();
    let bezier_curves = app.world.resource::<BezierTestHashed>();

    for id in maps.bezier_map.keys() {
        let bezier = bezier_curves.0.get(&id).unwrap();
        assert_eq!(bezier.latches, HashMap::new());
    }
    println!("unlatch_test passed");
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
