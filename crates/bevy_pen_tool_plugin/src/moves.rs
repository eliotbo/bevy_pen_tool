use bevy::math::Vec3A;
use bevy::prelude::*;
use bevy_pen_tool_model::inputs::Cursor;
use bevy_pen_tool_model::materials::{BezierMidMat, SelectionMat};
use bevy_pen_tool_model::mesh::{FillMesh2dMaterial, RoadMesh2dMaterial, StartMovingMesh};
use bevy_pen_tool_model::model::{
    AchorEdgeQuad, AnchorEdge, Bezier, BezierParent, BoundingBoxQuad, ControlPointQuad,
    FollowBezierAnimation, Globals, Group, GroupMiddleQuad, MainUi, MiddlePointQuad, MovingAnchor,
    TurnRoundAnimation, UiAction, UiBoard,
};

use std::collections::HashMap;

pub fn move_ui(
    cursor: ResMut<Cursor>,
    mut ui_query: Query<(&mut Transform, &mut UiBoard), With<MainUi>>,
) {
    for (mut transform, ui_board) in ui_query.iter_mut() {
        //
        if ui_board.action == UiAction::MovingUi {
            //
            let z_pos = transform.translation.z;
            transform.translation =
                ui_board.previous_position.extend(z_pos) + cursor.pos_relative_to_click.extend(0.0);
        }
    }
}

pub fn move_middle_quads(
    time: Res<Time>,
    bezier_curves: ResMut<Assets<Bezier>>,
    mut my_shader_params: ResMut<Assets<BezierMidMat>>,
    mut query: Query<
        (&mut Transform, &Handle<Bezier>, &Handle<BezierMidMat>),
        With<MiddlePointQuad>,
    >,
    globals: ResMut<Globals>,
) {
    let number_of_bezier_curves = bezier_curves.len();
    let num_points = globals.num_points_on_curve + 1;
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

                use flo_curves::bezier::BezierCurve;

                let t_distance = bezier.compute_real_distance(t_time);
                let pos = curve.point_at_pos(t_distance);

                let z = transform.translation.z;

                transform.translation = Vec3::new(pos.0 as f32, pos.1 as f32, z);
            }
        }
    }
}

pub fn move_group_middle_quads(
    time: Res<Time>,
    bezier_curves: Res<Assets<Bezier>>,
    mut my_shader_params: ResMut<Assets<BezierMidMat>>,
    mut query: Query<(
        &mut Transform,
        &Handle<Group>,
        &Handle<BezierMidMat>,
        &GroupMiddleQuad,
    )>,
    // globals: ResMut<Globals>,
    groups: ResMut<Assets<Group>>,
    // mut action_event_reader: EventReader<Action>,
    // mut latch_event_reader: EventReader<OfficialLatch>,
) {
    let mut t = 0.0;
    // During the exact frame where either the ungroup action or the OfficialLatch event
    // takes places, the middle quads do not have time to be despawned before this system runs.
    // Instead of using this hack (checking whether these events take place), we could
    // simply use an "if let Some(group) = groups.get_mut(group_handle.id)" statement

    // if latch_event_reader.iter().next().is_none() {
    //     if !action_event_reader
    //         .iter()
    //         .any(|x| x == &Action::Ungroup || x == &Action::Latch)
    //     {
    if let Some(last_handle_tuple) = groups.iter().next() {
        let mut last_handle_id = last_handle_tuple.0;
        // println!("mids: {:?}", query.iter().count());
        let bezier_assets = bezier_curves
            .iter()
            .collect::<HashMap<bevy::asset::HandleId, &Bezier>>();

        for (mut transform, group_handle, shader_params_handle, GroupMiddleQuad(num_quads)) in
            query.iter_mut()
        {
            if group_handle.id != last_handle_id {
                t = 0.0;
                last_handle_id = group_handle.id;
            }
            t = t + 1.0 / (num_quads.clone() as f32);

            let mut shader_params = my_shader_params.get_mut(shader_params_handle).unwrap();

            if let Some(group) = groups.get(group_handle) {
                let t_time = (t as f64 + time.seconds_since_startup() * 0.02) % 1.0;
                shader_params.t = t_time as f32;
                // println!("time: {:?}", t_time);

                let pos = group.compute_position_with_bezier(&bezier_assets, t_time);

                let z = transform.translation.z;
                transform.translation = Vec3::new(pos.x, pos.y, z);
            }
        }
    }
    //     }
    // }
}

pub fn move_bb_quads(
    mut bezier_curves: ResMut<Assets<Bezier>>,
    mut query: Query<(
        &mut GlobalTransform,
        &Handle<Bezier>,
        &Handle<Mesh>,
        &Handle<SelectionMat>,
        &BoundingBoxQuad,
    )>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut my_shader_params: ResMut<Assets<SelectionMat>>,
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

        let bb_pos = (bound1 + bound0) / 2.0;

        // println!("{:?}, {:?}", bb_size,);

        let z = transform.translation().z;

        *transform.translation_mut() = Vec3A::new(bb_pos.x, bb_pos.y, z);
        // transform.translation = bb_pos.extend(transform.translation.z);

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
    mut commands: Commands,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    mut query: Query<(
        Entity,
        &mut Transform,
        &Handle<Bezier>,
        &AchorEdgeQuad,
        &Parent,
        &MovingAnchor,
    )>,
    q_parent: Query<&GlobalTransform, (With<BezierParent>, Without<AchorEdgeQuad>)>,
    // mut remove_moving_quad_event: EventWriter<RemoveMovingQuadEvent>,
    // globals: Res<Globals>,
) {
    for (entity, mut transform, bezier_handle, endpoint_quad_id, parent, moving_quad) in
        query.iter_mut()
    {
        //
        let AchorEdgeQuad(point) = endpoint_quad_id;
        // info!("point: {:?}", point);
        //
        // checks whether the transforms are equal to the positions in the Bezier data structure

        if let Some(bezier) = bezier_curves.get_mut(bezier_handle) {
            // TOOD: remove this condition
            if (*point == AnchorEdge::Start
                && transform.translation.truncate() != bezier.positions.start)
                || (*point == AnchorEdge::End
                    && transform.translation.truncate() != bezier.positions.end)
            {
                let parent_global_transform = q_parent.get(**parent).unwrap();

                let ((start_displacement, end_displacement), (start_rotation, end_rotation)) =
                    bezier.ends_displacement();

                if *point == AnchorEdge::Start {
                    transform.translation = (bezier.positions.start + start_displacement
                        - parent_global_transform.translation().truncate())
                    .extend(transform.translation.z);
                    transform.rotation = start_rotation;
                } else {
                    transform.translation = (bezier.positions.end + end_displacement
                        - parent_global_transform.translation().truncate())
                    .extend(transform.translation.z);
                    transform.rotation = end_rotation;
                }

                if moving_quad.once {
                    commands.entity(entity).remove::<MovingAnchor>();
                }
            }
        }
    }
}

pub fn move_control_quads(
    mut commands: Commands,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    mut query: Query<(
        Entity,
        &mut Transform,
        &Handle<Bezier>,
        &ControlPointQuad,
        &Parent,
        &MovingAnchor,
    )>,
    q_parent: Query<&GlobalTransform, (With<BezierParent>, Without<ControlPointQuad>)>,
) {
    for (entity, mut transform, bezier_handle, ctr_pt_id, parent, moving_quad) in query.iter_mut() {
        let ControlPointQuad(point) = ctr_pt_id;
        //
        if let Some(bezier) = bezier_curves.get_mut(bezier_handle) {
            let parent_global_transform = q_parent.get(**parent).unwrap().translation();

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
            if offset || rotated {
                // if offset {
                let z = transform.translation.z;
                transform.translation =
                    (control_point - parent_global_transform.truncate()).extend(z);
                // transform.translation = control_point.extend(transform.translation.z);
                transform.rotation = Quat::from_rotation_z(bezier_angle_90);
            }

            if moving_quad.once {
                commands.entity(entity).remove::<MovingAnchor>();
            }
        }
    }
}

// system that moves the fill mesh and the road mesh with mouse
pub fn move_mesh(
    cursor: ResMut<Cursor>,
    mut query: ParamSet<(
        Query<(
            &mut Transform,
            &StartMovingMesh,
            &Handle<FillMesh2dMaterial>,
        )>,
        Query<(
            &mut Transform,
            &StartMovingMesh,
            &Handle<RoadMesh2dMaterial>,
        )>,
    )>,
    mut fill_mesh_materials: ResMut<Assets<FillMesh2dMaterial>>,
    mut road_mesh_materials: ResMut<Assets<RoadMesh2dMaterial>>,
) {
    for (mut transform, start_move, mesh_mat_handle) in query.p0().iter_mut() {
        //
        let new_pos = cursor.pos_relative_to_click + start_move.start_position;

        transform.translation = new_pos.extend(transform.translation.z);

        let mut mesh_mat = fill_mesh_materials.get_mut(mesh_mat_handle).unwrap();
        mesh_mat.center_of_mass = new_pos;
    }

    for (mut transform, start_move, mesh_mat_handle) in query.p1().iter_mut() {
        //
        let new_pos = cursor.pos_relative_to_click + start_move.start_position;

        transform.translation = new_pos.extend(transform.translation.z);

        let mut mesh_mat = road_mesh_materials.get_mut(mesh_mat_handle).unwrap();
        mesh_mat.center_of_mass = new_pos;
    }
}

////////// helicopter animation
//
// // animates the helicopter blades
pub fn turn_round_animation(mut query: Query<(&mut Transform, &TurnRoundAnimation)>) {
    for (mut transform, _) in query.iter_mut() {
        let quat = Quat::from_rotation_z(0.2);
        transform.rotate(quat);
    }
}

////////// helicopter animation
//
// // moves the helicopter along the Group path
pub fn follow_bezier_group(
    mut query: Query<(&mut Transform, &FollowBezierAnimation)>,
    mut visible_query: Query<
        &mut Visibility,
        Or<(With<FollowBezierAnimation>, With<TurnRoundAnimation>)>,
    >,
    groups: Res<Assets<Group>>,
    curves: Res<Assets<Bezier>>,
    time: Res<Time>,
    globals: ResMut<Globals>,
) {
    if let Some(group) = groups.iter().next() {
        for mut visible in visible_query.iter_mut() {
            visible.is_visible = true;
        }

        let bezier_assets = curves
            .iter()
            .collect::<HashMap<bevy::asset::HandleId, &Bezier>>();

        for (mut transform, bezier_animation) in query.iter_mut() {
            let path_length = group.1.standalone_lut.path_length as f64;

            let multiplier: f64 = 1500.0 / path_length;
            let t_time = (bezier_animation.animation_offset
                + time.seconds_since_startup() * (0.1 * multiplier))
                % 1.0;
            let mut pos = group.1.compute_position_with_lut(t_time as f32);

            let road_line_offset = 4.0;
            let normal = group
                .1
                .compute_normal_with_bezier(&bezier_assets, t_time as f64)
                .normalize();
            pos += normal * road_line_offset;

            transform.translation.x = pos.x * globals.scale;
            transform.translation.y = pos.y * globals.scale;

            // the car looks ahead (5% of the curve length) to orient itself
            let further_pos = group
                .1
                .compute_position_with_lut(((t_time + 0.05 * multiplier) % 1.0) as f32);
            let further_normal = group
                .1
                .compute_normal_with_bezier(
                    &bezier_assets,
                    ((t_time + 0.05 * multiplier) % 1.0) as f64,
                )
                .normalize();

            let forward_direction =
                (further_pos + further_normal * road_line_offset - pos).normalize();

            // let initial_rot = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
            // let forward_direction = initial_rot.mul_vec3(forward_direction0.extend(0.0));

            let mut current_looking_dir = transform
                .rotation
                .mul_vec3(bezier_animation.initial_direction);
            current_looking_dir.z = 0.0;

            let quat = Quat::from_rotation_arc(current_looking_dir, forward_direction.extend(0.0));

            let (axis, mut angle) = quat.to_axis_angle();
            // println!(
            //     "current_looking_dir: {:?}, forward_direction: {:?}",
            //     current_looking_dir, forward_direction
            // );

            // maximum rotating speed
            angle = angle.clamp(0.0, 3.0 * std::f32::consts::PI / 180.0);
            let clamped_quat = Quat::from_axis_angle(axis, angle);

            transform.rotation = clamped_quat.mul_quat(transform.rotation);
        }
    }
}
