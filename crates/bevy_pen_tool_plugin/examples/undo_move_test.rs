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
    app.update();
    app.update();
    app.update();

    let mut pen_commands = app.world.get_resource_mut::<PenCommandVec>().unwrap();

    let pos1 = Vec2::new(200., -100.);
    pen_commands.move_anchor(id1, Anchor::Start, pos1);

    app.update();
    app.update();
    app.update();

    let bezier_curves = app.world.resource::<BezierTestHashed>();
    let bezier = bezier_curves.0.get(&id1).unwrap();
    assert_eq!(bezier.positions.start, pos1);

    let mut pen_commands = app.world.get_resource_mut::<PenCommandVec>().unwrap();
    pen_commands.undo();

    app.update();
    app.update();
    app.update();

    let bezier_curves = app.world.resource::<BezierTestHashed>();
    let bezier = bezier_curves.0.get(&id2).unwrap();

    assert_eq!(bezier.positions.start, Vec2::ZERO);

    println!("undo_move_test passed");
}

pub struct BezierTestHashed(pub HashMap<BezierId, Bezier>);

pub fn update_bez(
    bezier_curves: Res<Assets<Bezier>>,
    mut bezier_curves_test: ResMut<BezierTestHashed>,
) {
    bezier_curves_test.0.clear();
    for (handle_id, bez) in bezier_curves.iter() {
        let id = BezierId(handle_id);
        bezier_curves_test.0.insert(id, bez.clone());
    }
}
