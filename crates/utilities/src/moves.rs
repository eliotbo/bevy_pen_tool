use crate::inputs::Cursor;
use crate::util::{
    interpolate, AnchorEdge, Bezier, BoundingBoxQuad, ControlPointQuad, EndpointQuad, Globals,
    GrandParent, Group, GroupMiddleQuad, MiddlePointQuad, MyShader, UiAction, UiBoard,
};

use bevy::prelude::*;

pub fn move_ui(
    cursor: ResMut<Cursor>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut ui_query: Query<(&mut Transform, &mut UiBoard), With<GrandParent>>,
    // mut globals: ResMut<Globals>,
) {
    for (mut transform, mut ui_board) in ui_query.iter_mut() {
        //
        if mouse_button_input.pressed(MouseButton::Left) && ui_board.action == UiAction::MovingUi {
            //
            let z_pos = transform.translation.z;
            transform.translation =
                ui_board.previous_position.extend(z_pos) + cursor.pos_relative_to_click.extend(0.0);
            // ui_board.position = transform.translation.truncate();
        }

        if mouse_button_input.just_released(MouseButton::Left) {
            ui_board.action = UiAction::None;
            ui_board.previous_position = transform.translation.truncate();
        }
    }
}

pub fn move_middle_quads(
    time: Res<Time>,
    bezier_curves: ResMut<Assets<Bezier>>,
    mut my_shader_params: ResMut<Assets<MyShader>>,
    mut query: Query<
        (&mut GlobalTransform, &Handle<Bezier>, &Handle<MyShader>),
        With<MiddlePointQuad>,
    >,
    globals: ResMut<Globals>,
) {
    let number_of_bezier_curves = bezier_curves.len();
    let num_points = globals.num_points + 1;
    let vrange: Vec<f32> = (0..num_points * number_of_bezier_curves)
        .map(|x| ((x) as f32) / (num_points as f32 - 1.0))
        .collect();

    for (handle_id, bezier) in bezier_curves.iter() {
        //
        let curve = bezier.to_curve();

        for ((mut transform, bezier_handle, shader_params_handle), t) in
            query.iter_mut().zip(vrange.clone())
        {
            if handle_id == bezier_handle.id {
                //
                let mut shader_params = my_shader_params.get_mut(shader_params_handle).unwrap();

                let t_time = (t as f64 + time.seconds_since_startup() * 0.1) % 1.0;
                shader_params.t = t_time as f32;

                // let idx_f64 = t_time * (bezier.lut.len() - 1) as f64;
                // let p1 = bezier.lut[(idx_f64 as usize)];
                // let p2 = bezier.lut[idx_f64 as usize + 1];
                // //
                // // TODO: is the minus one useful here?
                // let rem = (idx_f64 - 1.0) % 1.0;
                // let t_distance = interpolate(p1, p2, rem);

                use flo_curves::bezier::BezierCurve;

                let t_distance = bezier.compute_real_distance(t_time);
                let pos = curve.point_at_pos(t_distance);

                transform.translation.x = pos.0 as f32;
                transform.translation.y = pos.1 as f32;
            }
        }
    }
}

pub fn move_group_middle_quads(
    time: Res<Time>,
    bezier_curves: ResMut<Assets<Bezier>>,
    mut my_shader_params: ResMut<Assets<MyShader>>,
    mut query: Query<(
        &mut GlobalTransform,
        &Handle<Group>,
        &Handle<MyShader>,
        &GroupMiddleQuad,
    )>,
    // globals: ResMut<Globals>,
    groups: ResMut<Assets<Group>>,
) {
    // let num_points = (globals.num_points + 1) * groups.len();
    // let vrange: Vec<f32> = (0..num_points * number_of_bezier_curves)
    // let vrange: Vec<f32> = (0..num_points)
    //     .map(|x| ((x) as f32) / (num_points as f32 - 1.0))
    //     .collect();

    let mut t = 0.0;
    // println!("START:");
    if let Some(last_handle_tuple) = groups.iter().next() {
        let mut last_handle_id = last_handle_tuple.0;
        for (mut transform, group_handle, shader_params_handle, GroupMiddleQuad(num_quads)) in
            query.iter_mut()
        {
            if group_handle.id != last_handle_id {
                t = 0.0;
                last_handle_id = group_handle.id;
            }
            t = t + 1.0 / (num_quads.clone() as f32);

            let mut shader_params = my_shader_params.get_mut(shader_params_handle).unwrap();

            // println!("groups handle: {:?}", group_handle);
            let group = groups.get(group_handle).unwrap();

            let t_time = (t as f64 + time.seconds_since_startup() * 0.02) % 1.0;
            shader_params.t = t_time as f32;
            // println!("time: {:?}", t_time);

            let pos = group.compute_position_with_bezier(&bezier_curves, t_time);
            // let pos = group.compute_position_with_lut(t_time as f32);

            transform.translation.x = pos.x;
            transform.translation.y = pos.y;
        }
    }
}

pub fn move_bb_quads(
    mut bezier_curves: ResMut<Assets<Bezier>>,
    mut query: Query<(
        &mut GlobalTransform,
        &Handle<Bezier>,
        &Handle<Mesh>,
        &Handle<MyShader>,
        &BoundingBoxQuad,
    )>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut my_shader_params: ResMut<Assets<MyShader>>,
) {
    for (mut transform, bezier_handle, mesh_handle, shader_params_handle, _bbquad) in
        query.iter_mut()
    {
        let bezier = bezier_curves.get_mut(bezier_handle).unwrap();

        let mesh = meshes.get_mut(mesh_handle).unwrap();
        let mut shader_params = my_shader_params.get_mut(shader_params_handle).unwrap();

        let (bound0, bound1) = bezier.bounding_box();

        // makes the quad bigger than the bounding box so that we can have smooth edges made in the shader
        let bigger_size = (bound1 - bound0) * 1.1;
        // let bb_size = (bound1 - bound0);
        let bb_pos = (bound1 + bound0) / 2.0;

        // println!("{:?}, {:?}", bb_size,);
        transform.translation = bb_pos.extend(transform.translation.z);
        shader_params.size = bigger_size;

        // TODO: change the transform scale instead of the mesh
        let new_mesh = Mesh::from(shape::Quad {
            size: bigger_size,
            flip: false,
        });
        *mesh = new_mesh;
    }
}

pub fn move_end_quads(
    mut bezier_curves: ResMut<Assets<Bezier>>,
    mut query: Query<(&mut GlobalTransform, &Handle<Bezier>, &EndpointQuad)>,
    globals: Res<Globals>,
) {
    for (mut transform, bezier_handle, endpoint_quad_id) in query.iter_mut() {
        //
        let EndpointQuad(point) = endpoint_quad_id;
        //
        if let Some(bezier) = bezier_curves.get_mut(bezier_handle) {
            if (*point == AnchorEdge::Start
                && transform.translation.truncate() != bezier.positions.start)
                || (*point == AnchorEdge::End
                    && transform.translation.truncate() != bezier.positions.end)
                || bezier.just_created
            {
                let ((start_displacement, end_displacement), (start_rotation, end_rotation)) =
                    bezier.ends_displacement(globals.scale);
                // println!("{}", globals.scale);

                if *point == AnchorEdge::Start {
                    transform.translation = (bezier.positions.start + start_displacement)
                        .extend(transform.translation.z);
                    transform.rotation = start_rotation;
                } else {
                    transform.translation =
                        (bezier.positions.end + end_displacement).extend(transform.translation.z);
                    transform.rotation = end_rotation;
                }

                bezier.do_compute_lut = true;
            }
        }
    }
}

pub fn move_control_quads(
    mut bezier_curves: ResMut<Assets<Bezier>>,
    mut query: Query<(&mut GlobalTransform, &Handle<Bezier>, &ControlPointQuad)>,
) {
    for (mut transform, bezier_handle, ctr_pt_id) in query.iter_mut() {
        let ControlPointQuad(point) = ctr_pt_id;
        //
        if let Some(bezier) = bezier_curves.get_mut(bezier_handle) {
            //
            let (_axis, quad_angle) = transform.rotation.to_axis_angle();

            let control_point: Vec2;
            let anchor_point: Vec2;
            let constant_angle: f32;

            if *point == AnchorEdge::Start {
                control_point = bezier.positions.control_start;
                anchor_point = bezier.positions.start;
                constant_angle = std::f32::consts::PI / 2.0;
            } else {
                control_point = bezier.positions.control_end;
                anchor_point = bezier.positions.end;
                constant_angle = std::f32::consts::PI / 2.0;
            }

            let relative_position: Vec2 = control_point - anchor_point;
            let bezier_angle: f32 = relative_position.y.atan2(relative_position.x);

            let bezier_angle_90: f32 = bezier_angle + constant_angle;

            let offset: bool = transform.translation.truncate() != control_point;
            let rotated: bool = !((quad_angle.abs() - bezier_angle_90.abs()).abs() < 0.01);

            // if the quad's translation and rotation are not equal to the corresponding control point, fix them
            if offset || rotated || bezier.just_created {
                //
                transform.translation = control_point.extend(transform.translation.z);

                transform.rotation = Quat::from_rotation_z(bezier_angle_90);

                // println!("tac: {:?}, {:?}, {:?}", offset, rotated, id);

                bezier.do_compute_lut = true;
            }
        }
    }
}