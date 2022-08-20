use crate::inputs::*;
use crate::materials::*;
use crate::mesh::*;
use crate::model::bezier::*;
use crate::model::group::*;

use bevy::{asset::HandleId, prelude::*, sprite::Mesh2dHandle, utils::Uuid};

// use rand::distributions::Open01;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::collections::HashSet;

use bevy_inspector_egui::Inspectable;

#[derive(Component)]
pub struct CurrentlySelecting;

#[derive(Component)]
pub struct OnOffMaterial {
    pub material: Handle<Image>,
}

#[derive(Component)]
pub struct FollowBezierAnimation {
    pub animation_offset: f64,
    pub initial_direction: Vec3,
}

// helicopter animation
#[derive(Component)]
pub struct TurnRoundAnimation;

#[derive(Component)]
pub struct MainUi;

#[derive(Component)]
pub struct Icon;

#[derive(Debug, Component)]
pub struct BoundingBoxQuad;

#[derive(Component)]
pub struct SelectedBoxQuad;

#[derive(Component)]
pub struct SelectingBoxQuad;

#[derive(Component)]
pub struct GroupBoxQuad;

#[derive(Debug)]
pub struct Maps {
    pub mesh_handles: HashMap<&'static str, Mesh2dHandle>,
    // pub pipeline_handles: HashMap<&'static str, Handle<PipelineDescriptor>>,
    pub bezier_map: HashMap<BezierId, BezierHandleEntity>,
    pub group_map: HashMap<GroupId, Handle<Group>>,
    pub mesh_map: HashMap<MeshId, Entity>,
    pub sounds: HashMap<&'static str, Handle<AudioSource>>,
    pub textures: HashMap<&'static str, Handle<Image>>,
}

impl Maps {
    pub fn print_bezier_map(&self) {
        info!(
            "bezier maps: {:?}",
            self.bezier_map
                .iter()
                .map(|(key, val)| {
                    if let HandleId::Id(_, id) = val.handle.id {
                        if let HandleId::Id(_, id2) = key.0 {
                            (id2, id)
                        } else {
                            panic!("no id 1")
                        }
                    } else {
                        panic!("no id 2")
                    }
                })
                .collect::<Vec<(u64, u64)>>()
        )
    }
}

impl Default for Maps {
    fn default() -> Self {
        Maps {
            mesh_handles: HashMap::new(),
            // pipeline_handles: HashMap::new(),
            mesh_map: HashMap::new(),
            bezier_map: HashMap::new(),
            group_map: HashMap::new(),
            sounds: HashMap::new(),
            textures: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum SelectionChoice {
    CurveSet(HashSet<BezierId>),
    Mesh(PenMesh, Vec2), // Vec2 is translation
    None,
}

pub struct Selection {
    pub selected: Vec<SelectionChoice>,
}

impl Default for Selection {
    fn default() -> Self {
        Self {
            selected: Vec::new(),
        }
    }
}

/// Holds Z position information. Important for drawing order. Contained within [`Globals`].
#[derive(Clone, Debug)]
pub struct ZPos {
    pub bezier_parent: f32,
    pub anchors: f32,
    pub controls: f32,
    pub middles: f32,
    pub group_parent: f32,
    pub group_bouding_box: f32,
    pub group_middles: f32,
    pub selecting_box: f32,
    pub selection_box: f32,
    pub bounding_box: f32,
    pub road: f32,
    pub fill: f32,
    pub heli: f32,
    pub heli_top: f32,
    pub ui_board: f32,
    pub ui_buttons: f32,
    pub ui_button_icons: f32,
    pub ui_color_board: f32,
    pub ui_color_buttons: f32,
}

impl Default for ZPos {
    fn default() -> Self {
        Self {
            bezier_parent: 0.33,

            anchors: 0.33,
            controls: 0.33,
            middles: 0.33,
            group_parent: 0.33,
            group_bouding_box: 0.33,
            group_middles: 0.33,
            selecting_box: 0.33,
            selection_box: 0.53,
            bounding_box: 0.33,
            road: 0.35,
            fill: 0.33,
            heli: 0.4,
            heli_top: 0.01,
            ui_board: 0.33,
            ui_buttons: 0.33,
            ui_button_icons: 0.33,
            ui_color_board: 0.33,
            ui_color_buttons: 0.33,
        }
    }
}

/// Global parameters for the plugin
#[derive(Clone, Debug)]
pub struct Globals {
    pub do_hide_anchors: bool,
    pub do_hide_bounding_boxes: bool,
    pub num_points_on_curve: usize,
    pub camera_scale: f32,
    pub scale: f32,
    pub picked_color: Option<Color>,
    pub sound_on: bool,
    pub hide_control_points: bool,
    pub group_lut_num_points: u32,
    pub road_width: f32,
    pub anchor_clicking_dist: f32,
    pub z_pos: ZPos,
}

impl Default for Globals {
    fn default() -> Self {
        Self {
            do_hide_bounding_boxes: true,
            do_hide_anchors: false,
            camera_scale: 0.15,
            scale: 1.0,
            picked_color: None,
            sound_on: true,
            hide_control_points: false,
            num_points_on_curve: 25,
            group_lut_num_points: 100,
            road_width: 8.0,
            anchor_clicking_dist: 12.0,
            z_pos: ZPos::default(),
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Hash, Eq, Inspectable)]
pub struct GroupId(pub HandleId);
impl From<HandleId> for GroupId {
    fn from(id: HandleId) -> Self {
        Self(id)
    }
}

impl Default for GroupId {
    fn default() -> Self {
        let mut rng = thread_rng();
        let uuid = Uuid::parse_str("b16f31ff-a594-4fca-a0e3-85e626d3d01a").unwrap();
        Self(HandleId::new(uuid, rng.gen()))
    }
}

#[derive(Component)]
//
pub struct UiBoard {
    pub expanded: bool,
    pub size: Vec2,
    pub position: Vec2,
    pub action: UiAction,
    pub previous_position: Vec2,
}

#[derive(Debug, PartialEq, Eq)]
pub enum UiAction {
    PickingColor,
    PressedUiButton,
    MovingUi,
    None,
}

#[derive(Component)]
pub struct ColorButton {
    pub size: Vec2,
}

pub fn find_connected_curves(
    bezier_id: BezierId,
    bezier_curves: &ResMut<Assets<Bezier>>,
    id_handle_map: &HashMap<BezierId, BezierHandleEntity>,
) -> Vec<Handle<Bezier>> {
    //
    let mut connected_curves: Vec<Handle<Bezier>> = Vec::new();

    let bezier = bezier_curves
        .get(&id_handle_map.get(&bezier_id).unwrap().handle)
        .unwrap();

    if bezier.latches.is_empty() {
        return connected_curves;
    };

    let anchors_temp = vec![AnchorEdge::Start, AnchorEdge::End];
    let anchors = anchors_temp
        .iter()
        .filter(|anchor| bezier.quad_is_latched(anchor))
        .collect::<Vec<&AnchorEdge>>();

    let initial_bezier = bezier.id;

    // for both ends of the curve, find the other curves that are latched to it
    for anchor in anchors.clone() {
        let mut latch = bezier.latches.get(&anchor).unwrap().clone();

        //
        loop {
            //
            // let (partner_id, partners_edge) = (latch.latched_to_id, );
            let next_edge = latch.partners_edge.other();

            let next_curve_handle = id_handle_map
                .get(&latch.latched_to_id)
                .unwrap()
                .handle
                .clone();

            let bezier_next = bezier_curves.get(&next_curve_handle).unwrap();
            connected_curves.push(next_curve_handle);

            if let Some(next_latch) = bezier_next.latches.get(&next_edge) {
                latch = next_latch.clone();
                if latch.latched_to_id == initial_bezier {
                    break;
                }
            } else {
                break;
            }
        }
        // }
    }

    return connected_curves;
}

pub fn update_latched_partner_position(
    bezier_map: &HashMap<BezierId, BezierHandleEntity>,
    bezier_curves: &mut ResMut<Assets<Bezier>>,
    latch_info: LatchInfo,
    // control: Vec2,
    // position: Vec2,
) {
    // let latch_info = self.get_mover_latch_info();
    // change the control point of a latched point
    if let Some((partner_latch, mover_position, opposite_control)) = latch_info {
        //

        if let Some(bezier_handle) = bezier_map.get(&partner_latch.latched_to_id) {
            //
            let bezier_partner = bezier_curves.get_mut(&bezier_handle.handle).unwrap();

            bezier_partner.update_latched_position(
                partner_latch.partners_edge,
                opposite_control,
                mover_position,
            );
        } else {
            // Problems with non-existing ids may occur when using undo, redo and delete
            // TODO: Delete latched anchors that no longer have a partner
            println!(
                "Warning: Could not retrieve handle for Bezier id: {:?}",
                &partner_latch.latched_to_id
            );
        }
    }
}

// leave this public
/// Holds information about the position of each anchor for a given Bezier curve.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, Inspectable, PartialEq)]
pub struct BezierPositions {
    pub start: Vec2,
    pub end: Vec2,
    pub control_start: Vec2,
    pub control_end: Vec2,
}

impl Default for BezierPositions {
    fn default() -> Self {
        Self {
            start: Vec2::ZERO,
            end: Vec2::ZERO,
            control_start: Vec2::ZERO,
            control_end: Vec2::ZERO,
        }
    }
}

impl BezierPositions {
    pub const ZERO: Self = Self {
        start: Vec2::ZERO,
        end: Vec2::ZERO,
        control_start: Vec2::ZERO,
        control_end: Vec2::ZERO,
    };
}

pub struct ButtonMaterials {
    pub normal: Handle<ColorMaterial>,
    pub hovered: Handle<ColorMaterial>,
    pub pressed: Handle<ColorMaterial>,
}

impl FromWorld for ButtonMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        ButtonMaterials {
            normal: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
            hovered: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
            pressed: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
        }
    }
}

pub fn interpolate(p0: f64, p1: f64, rem: f64) -> f64 {
    return p0 + rem * (p1 - p0);
}

pub fn interpolate_vec2(p0: Vec2, p1: Vec2, rem: f32) -> Vec2 {
    return p0 + rem * (p1 - p0);
}

// get the Bezier handle of the closest point to "position" (other than itself
// if other_than_moving is true)
pub fn get_close_anchor(
    max_dist: f32,
    position: Vec2,
    bezier_curves: &ResMut<Assets<Bezier>>,
    query: &Query<(&Handle<Bezier>, &BezierParent)>,
    // mut globals: ResMut<Globals>,
    // scale: f32,
) -> Option<(f32, Anchor, BezierId)> {
    for (bezier_handle, _) in query.iter() {
        if let Some(bezier) = bezier_curves.get(bezier_handle) {
            let ((start_displacement, end_displacement), (_start_rotation, _end_rotation)) =
                bezier.ends_displacement();

            let distance_to_control0 = (bezier.positions.control_start - position).length();
            let distance_to_control1 = (bezier.positions.control_end - position).length();
            let distance_to_start =
                (bezier.positions.start + 2.0 * start_displacement - position).length();
            let distance_to_endpoint =
                (bezier.positions.end + 2.0 * end_displacement - position).length();

            if distance_to_control0 < max_dist {
                return Some((distance_to_control0, Anchor::ControlStart, bezier.id));
            } else if distance_to_control1 < max_dist {
                return Some((distance_to_control1, Anchor::ControlEnd, bezier.id));
            } else if distance_to_start < max_dist {
                return Some((distance_to_start, Anchor::Start, bezier.id));
            } else if distance_to_endpoint < max_dist {
                return Some((distance_to_endpoint, Anchor::End, bezier.id));
            }
        }
    }
    return None;
}

// TODO: scale?
pub fn get_close_mesh(
    max_dist: f32,
    fill_query: &Query<(Entity, &Transform, &Handle<FillMesh2dMaterial>, &PenMesh)>,
    road_query: &Query<(Entity, &Transform, &Handle<RoadMesh2dMaterial>, &PenMesh)>,
    fill_mesh_materials: &mut ResMut<Assets<FillMesh2dMaterial>>,
    road_mesh_materials: &mut ResMut<Assets<RoadMesh2dMaterial>>,
    position: Vec2,
) -> Option<(Entity, MeshId, Vec2)> {
    for (entity, transform, fill_handle, mesh) in fill_query.iter() {
        let dist = (transform.translation.truncate() - position).length();
        let mut fill_mesh_material = fill_mesh_materials.get_mut(fill_handle).unwrap();

        if dist < max_dist {
            fill_mesh_material.show_com = 1.;
            return Some((entity, mesh.id, transform.translation.truncate()));
        } else {
            fill_mesh_material.show_com = 0.;
        }
    }
    for (entity, transform, road_handle, mesh) in road_query.iter() {
        let dist = (transform.translation.truncate() - position).length();
        let mut road_mesh_material = road_mesh_materials.get_mut(road_handle).unwrap();

        if dist < max_dist {
            road_mesh_material.show_com = 1.;
            return Some((entity, mesh.id, transform.translation.truncate()));
        } else {
            road_mesh_material.show_com = 0.;
        }
    }
    return None;
}

pub fn get_close_anchor_entity(
    max_dist: f32,
    position: Vec2,
    bezier_curves: &ResMut<Assets<Bezier>>,
    query: &Query<(Entity, &Handle<Bezier>), With<BezierParent>>,
    // scale: f32,
) -> Option<(f32, Anchor, Entity, Handle<Bezier>)> {
    //
    for (entity, bezier_handle) in query.iter() {
        //
        if let Some(bezier) = bezier_curves.get(bezier_handle) {
            //
            let ((start_displacement, end_displacement), (_start_rotation, _end_rotation)) =
                bezier.ends_displacement();

            let distance_to_control0 = (bezier.positions.control_start - position).length();
            let distance_to_control1 = (bezier.positions.control_end - position).length();
            let distance_to_start =
                (bezier.positions.start + 2.0 * start_displacement - position).length();
            let distance_to_endpoint =
                (bezier.positions.end + 2.0 * end_displacement - position).length();

            if distance_to_control0 < max_dist {
                return Some((
                    distance_to_control0,
                    Anchor::ControlStart,
                    entity,
                    bezier_handle.clone(),
                ));
            } else if distance_to_control1 < max_dist {
                return Some((
                    distance_to_control1,
                    Anchor::ControlEnd,
                    entity,
                    bezier_handle.clone(),
                ));
            } else if distance_to_start < max_dist {
                return Some((
                    distance_to_start,
                    Anchor::Start,
                    entity,
                    bezier_handle.clone(),
                ));
            } else if distance_to_endpoint < max_dist {
                return Some((
                    distance_to_endpoint,
                    Anchor::End,
                    entity,
                    bezier_handle.clone(),
                ));
            }
        }
    }
    return None;
}

pub fn get_close_still_unlatched_anchor(
    max_dist: f32,
    position: Vec2,
    bezier_curves: &ResMut<Assets<Bezier>>,
    query: &Query<(&Handle<Bezier>, &AchorEdgeQuad), Without<MovingAnchor>>,
) -> Option<(f32, AnchorEdge, Handle<Bezier>)> {
    for (bezier_handle, anchor_edge) in query.iter() {
        if let Some(bezier) = bezier_curves.get(bezier_handle) {
            if !bezier.quad_is_latched(&anchor_edge.0) {
                //
                let distance_to_anchor = match anchor_edge.0 {
                    AnchorEdge::Start => (bezier.positions.start - position).length(),
                    AnchorEdge::End => (bezier.positions.end - position).length(),
                };

                if distance_to_anchor < max_dist {
                    return Some((distance_to_anchor, anchor_edge.0, bezier_handle.clone()));
                }
            }
        }
    }
    return None;
}

pub fn get_close_still_anchor(
    max_dist: f32,
    position: Vec2,
    bezier_curves: &ResMut<Assets<Bezier>>,
    query: &Query<(&Handle<Bezier>, &AchorEdgeQuad), Without<MovingAnchor>>,
) -> Option<(f32, AnchorEdge, BezierId, bool)> {
    for (bezier_handle, anchor_edge) in query.iter() {
        if let Some(bezier) = bezier_curves.get(bezier_handle) {
            //
            let distance_to_anchor = match anchor_edge.0 {
                AnchorEdge::Start => (bezier.positions.start - position).length(),
                AnchorEdge::End => (bezier.positions.end - position).length(),
            };

            if distance_to_anchor < max_dist {
                return Some((
                    distance_to_anchor,
                    anchor_edge.0,
                    bezier.id.clone(),
                    bezier.quad_is_latched(&anchor_edge.0),
                ));
            }
        }
    }
    return None;
}

// change the selection mesh according to the bounding box of the selected curves
pub fn adjust_selection_attributes(
    mut my_shader_params: ResMut<Assets<SelectionMat>>,
    mut query: Query<&Mesh2dHandle, With<SelectedBoxQuad>>,
    shader_query: Query<&Handle<SelectionMat>, With<SelectedBoxQuad>>,
    bezier_curves: ResMut<Assets<Bezier>>,
    mut meshes: ResMut<Assets<Mesh>>,
    selection: ResMut<Selection>,
    maps: Res<Maps>,
) {
    let (mut minx, mut maxx, mut miny, mut maxy) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);

    for selected_item in selection.selected.clone().iter() {
        match selected_item {
            SelectionChoice::CurveSet(bezier_set) => {
                // We set the mesh attributes as a function of the bounding box.
                // This could be done by removing the mesh from the mesh asset
                // and adding a brand new  = mesh

                for bezier_id in bezier_set.iter() {
                    if let Some(bezier_entity_handle) = maps.bezier_map.get(bezier_id) {
                        let bezier = bezier_curves.get(&bezier_entity_handle.handle).unwrap();

                        let (bound0, bound1) = bezier.bounding_box();
                        minx = minx.min(bound0.x);
                        maxx = maxx.max(bound1.x);
                        miny = miny.min(bound0.y);
                        maxy = maxy.max(bound1.y);
                    }
                }
            }
            SelectionChoice::Mesh(pen_mesh, translation) => {
                minx = minx.min(pen_mesh.bounding_box.0.x + translation.x);
                maxx = maxx.max(pen_mesh.bounding_box.1.x + translation.x);
                miny = miny.min(pen_mesh.bounding_box.0.y + translation.y);
                maxy = maxy.max(pen_mesh.bounding_box.1.y + translation.y);
            }
            _ => {}
        }
    }
    if selection.selected.clone().iter().count() > 0 {
        let shader_handle = shader_query.single();
        let mut shader_params = my_shader_params.get_mut(shader_handle).unwrap();
        let up_factor = 1.10;
        let x_pos = (maxx + minx) / 2.0;
        let y_pox = (maxy + miny) / 2.0;
        let x_width = (maxx - minx) * up_factor / 2.0;
        let y_width = (maxy - miny) * up_factor / 2.0;

        // send correct width to shader that will adjust the thickness of the box accordingly
        // let scale = globals.scale / 0.5;
        shader_params.size = Vec2::new(x_width, y_width) / 5.0;

        let vertex_positions = vec![
            [x_pos - x_width, y_pox - y_width, 0.0],
            [x_pos - x_width, y_pox + y_width, 0.0],
            [x_pos + x_width, y_pox + y_width, 0.0],
            [x_pos + x_width, y_pox - y_width, 0.0],
        ];

        for mesh_handle in query.iter_mut() {
            let mesh = meshes.get_mut(&mesh_handle.0.clone()).unwrap();
            let v_pos = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION);

            if let Some(array2) = v_pos {
                *array2 =
                    bevy::render::mesh::VertexAttributeValues::Float32x3(vertex_positions.clone());
            }
        }
    }
}

// change the selection mesh according to the bounding box of the selected curves
pub fn adjust_selecting_attributes(
    cursor: ResMut<Cursor>,
    mut my_shader_params: ResMut<Assets<SelectingMat>>,
    mut query: Query<&Mesh2dHandle, (With<SelectingBoxQuad>, With<CurrentlySelecting>)>,
    shader_query: Query<&Handle<SelectingMat>, With<SelectingBoxQuad>>,
    mut meshes: ResMut<Assets<Mesh>>,
    globals: ResMut<Globals>,
) {
    for mesh_handle in query.iter_mut() {
        let mut click_position = cursor.last_click_position;

        let mouse_position = cursor.position * globals.scale;
        click_position *= globals.scale;

        let (minx, maxx, miny, maxy) = (
            mouse_position.x.min(click_position.x),
            mouse_position.x.max(click_position.x),
            mouse_position.y.min(click_position.y),
            mouse_position.y.max(click_position.y),
        );

        let shader_handle = shader_query.single();
        let mut shader_params = my_shader_params.get_mut(shader_handle).unwrap();
        let up_factor = 1.10;
        let x_pos = (maxx + minx) / 2.0;
        let y_pox = (maxy + miny) / 2.0;
        let x_width = (maxx - minx) * up_factor / 2.0;
        let y_width = (maxy - miny) * up_factor / 2.0;

        // send correct width to shader that will adjust the thickness of the box accordingly

        shader_params.size = Vec2::new(x_width, y_width) / 5.0;

        let vertex_positions = vec![
            [x_pos - x_width, y_pox - y_width, 0.0],
            [x_pos - x_width, y_pox + y_width, 0.0],
            [x_pos + x_width, y_pox + y_width, 0.0],
            [x_pos + x_width, y_pox - y_width, 0.0],
        ];
        // println!("will attempt selecting");

        let mesh = meshes.get_mut(&mesh_handle.0.clone()).unwrap();
        let v_pos = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION);

        if let Some(array2) = v_pos {
            // println!("changed selecting");
            *array2 =
                bevy::render::mesh::VertexAttributeValues::Float32x3(vertex_positions.clone());
        }
    }
}

// change the group selection mesh according to the bounding box of the curves inside the group
pub fn adjust_group_attributes(
    mouse_button_input: Res<Input<MouseButton>>,
    mut my_shader_params: ResMut<Assets<SelectionMat>>,
    mut query: Query<&Handle<Mesh>, With<GroupBoxQuad>>,
    groups: ResMut<Assets<Group>>,
    group_query: Query<(&Handle<Group>, &Handle<SelectionMat>)>,
    bezier_curves: ResMut<Assets<Bezier>>,
    mut meshes: ResMut<Assets<Mesh>>,
    // mut group_box_event_reader: EventReader<GroupBoxEvent>,
    // globals: ResMut<Globals>,
) {
    // TODO: make this system run only when necessary

    if mouse_button_input.pressed(MouseButton::Left) {
        for (group_handle, shader_handle) in group_query.iter() {
            if let Some(group) = groups.get(group_handle) {
                let (mut minx, mut maxx, mut miny, mut maxy) =
                    (1000000.0f32, -1000000.0f32, 1000000.0f32, -1000000.0f32);

                // We set the mesh attributes as a function of the bounding box.
                // This could be done by removing the mesh from the mesh asset
                // and adding a brand new mesh
                for (_entity, selected_handle) in group.group.clone() {
                    let bezier = bezier_curves.get(&selected_handle).unwrap();

                    let (bound0, bound1) = bezier.bounding_box();
                    // bound0 *= globals.scale;
                    // bound1 *= globals.scale;

                    minx = minx.min(bound0.x);
                    maxx = maxx.max(bound1.x);
                    miny = miny.min(bound0.y);
                    maxy = maxy.max(bound1.y);
                }

                let mut shader_params = my_shader_params.get_mut(shader_handle).unwrap();
                let scale = 1.10;
                let x_pos = (maxx + minx) / 2.0;
                let y_pox = (maxy + miny) / 2.0;
                let x_width = (maxx - minx) * scale / 2.0;
                let y_width = (maxy - miny) * scale / 2.0;

                // send correct width to shader that will adjust the thickness of the box accordingly
                shader_params.size = Vec2::new(x_width * 2.0, y_width * 2.0);

                let vertex_positions = vec![
                    [x_pos - x_width, y_pox - y_width, 0.0],
                    [x_pos - x_width, y_pox + y_width, 0.0],
                    [x_pos + x_width, y_pox + y_width, 0.0],
                    [x_pos + x_width, y_pox - y_width, 0.0],
                ];

                for mesh_handle in query.iter_mut() {
                    let mesh = meshes.get_mut(mesh_handle).unwrap();
                    let v_pos = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION);

                    if let Some(array2) = v_pos {
                        *array2 = bevy::render::mesh::VertexAttributeValues::Float32x3(
                            vertex_positions.clone(),
                        );
                    }
                }
            }
        }
    }
}

// makes UI and quads bigger or smaller using Ctrl + mousewheel
pub fn rescale(
    mut grandparent_query: Query<
        &mut Transform,
        Or<(With<MainUi>, With<GroupParent>, With<SelectedBoxQuad>)>,
    >,
    // shader_param_query: Query<&Handle<UiMat>>,
    // mut my_shaders: ResMut<Assets<UiMat>>,
    mut globals: ResMut<Globals>,
    mut action_event_reader: EventReader<Action>,
) {
    for action in action_event_reader.iter() {
        //
        let mut pressed_rescale_button = false;
        let mut zoom_direction = 0.0;
        //
        if action == &Action::ScaleUp {
            pressed_rescale_button = true;
            zoom_direction = 1.0;
        } else if action == &Action::ScaleDown {
            pressed_rescale_button = true;
            zoom_direction = -1.0;
        }
        if pressed_rescale_button {
            let zoom_factor = 1.0 + zoom_direction * 0.1;
            globals.scale = globals.scale * zoom_factor;

            // the bounding box, the ends and the control points share the same shader parameters
            for mut transform in grandparent_query.iter_mut() {
                transform.scale = Vec2::new(globals.scale, globals.scale).extend(1.0);
            }

            // // update the shader params for the middle quads (animated quads)
            // for shader_handle in shader_param_query.iter() {
            //     let shader_param = my_shaders.get_mut(shader_handle).unwrap();
            //     shader_param.zoom = 0.15 / globals.scale;
            //     shader_param.size *= 1.0 / zoom_factor;
            // }
        }
    }
}
