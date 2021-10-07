use crate::inputs::*;

use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        // camera::OrthographicProjection,
        mesh::VertexAttributeValues::Float32x3,
        pipeline::PipelineDescriptor,
        renderer::RenderResources,
    },
};

use serde::{Deserialize, Serialize};

use rand::prelude::*;

use std::collections::HashMap;
use std::collections::HashSet;

use flo_curves::bezier::BezierCurve;
use flo_curves::bezier::Curve;
use flo_curves::*;

use plotlib::page::Page;
use plotlib::repr::Plot;
use plotlib::style::LineStyle;
use plotlib::view::ContinuousView;

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Copy, Hash)]
pub enum AnchorEdge {
    Start,
    End,
}

impl AnchorEdge {
    pub fn to_anchor(&self) -> Anchor {
        match self {
            Self::Start => Anchor::Start,
            Self::End => Anchor::End,
        }
    }

    pub fn other(&self) -> AnchorEdge {
        match self {
            Self::Start => Self::End,
            Self::End => Self::Start,
        }
    }
}

#[derive(Debug)]
pub enum UserState {
    Idle,
    Selecting(Vec2),
    Selected(Group),
    SpawningCurve,
    MovingAnchor(MoveAnchor),
}

impl Default for UserState {
    fn default() -> Self {
        Self::Idle
    }
}

pub struct Loaded;
pub struct GrandParent;
pub struct Icon;
pub struct OnOffMaterial {
    pub material: Handle<ColorMaterial>,
}
pub struct EndpointQuad(pub AnchorEdge);

pub struct ControlPointQuad(pub AnchorEdge);

pub struct MiddlePointQuad;

pub struct GroupMiddleQuad(pub usize);

#[derive(Debug)]
pub struct BoundingBoxQuad;

pub struct SelectedBoxQuad;
pub struct SelectingBoxQuad;

pub struct GroupBoxQuad;

#[derive(Debug)]
pub struct OfficialLatch(pub LatchData, pub Handle<Bezier>);

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Copy)]
pub enum Anchor {
    Start,
    End,
    ControlStart,
    ControlEnd,
    None,
}

// impl Anchor {
//     pub fn to_edge(&self) -> AnchorEdge {
//         match self {
//             Self::Start => AnchorEdge::Start,
//             Self::End => AnchorEdge::End,
//             _ => {
//                 println!("Failure to convert Anchor to AnchorEdge!");
//                 return AnchorEdge::Start;
//             }
//         }
//     }
// }

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

pub struct ColorButton {
    pub size: Vec2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LutSaveLoad {
    pub lut: Vec<((f64, f64), Vec<f64>)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct GroupSaveLoad {
//     // the AnchorEdge corresponds to first anchor encountered when traversing the group
//     pub curves: Vec<(AnchorEdge, Bezier)>,
//     pub standalone_lut: (f32, Vec<Vec2>),
//     pub lut: Vec<(AnchorEdge, (f64, f64), Vec<f64>)>,
// }

pub struct GroupSaveLoad {
    // the AnchorEdge corresponds to first anchor encountered when traversing the group
    // pub curves: Vec<(AnchorEdge, Bezier)>,
    pub lut: Vec<(Bezier, AnchorEdge, (f64, f64), Vec<f64>)>,
    pub standalone_lut: (f32, Vec<Vec2>),
}

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "1e08866c-0b8a-484e-8bce-31333b21137e"]
pub struct Group {
    pub group: HashSet<(Entity, Handle<Bezier>)>,
    pub handles: HashSet<Handle<Bezier>>,
    // Attempts to store the start and end points of a group. Fails if not fully connected
    pub ends: Option<Vec<(Handle<Bezier>, AnchorEdge)>>,
    // the tuple (f64, f64) represents (t_min, t_max), the min and max t-values for the curve
    pub lut: Vec<(Handle<Bezier>, AnchorEdge, (f64, f64), Vec<f64>)>,
    pub standalone_lut: (f32, Vec<Vec2>),
}

impl Default for Group {
    fn default() -> Self {
        Group {
            group: HashSet::new(),
            handles: HashSet::new(),
            lut: Vec::new(),
            ends: None,
            standalone_lut: (0.0, Vec::new()),
        }
    }
}

impl Group {
    pub fn into_group_save(&self, bezier_curves: &mut ResMut<Assets<Bezier>>) -> GroupSaveLoad {
        let mut lut = Vec::new();
        for (handle, anchor, t_ends, local_lut) in self.lut.iter() {
            let mut bezier = bezier_curves.get(handle.clone()).unwrap().clone();
            bezier.lut = Vec::new();
            lut.push((
                bezier.clone(),
                anchor.clone(),
                t_ends.clone(),
                local_lut.clone(),
            ));
        }
        GroupSaveLoad {
            lut,
            standalone_lut: self.standalone_lut.clone(),
        }
    }

    pub fn find_connected_ends(
        &mut self,
        bezier_curves: &mut ResMut<Assets<Bezier>>,
        id_handle_map: HashMap<u128, Handle<Bezier>>,
    ) {
        //
        if self.handles.len() == 0 {
            return ();
        };

        let mut handles = self.handles.clone();
        let num_curves = handles.len();
        let handle = handles.iter().next().unwrap().clone();
        handles.remove(&handle);

        let initial_bezier = bezier_curves.get(handle.clone()).unwrap();

        let anchors_temp = vec![AnchorEdge::Start, AnchorEdge::End];
        let anchors = anchors_temp
            .iter()
            .filter(|anchor| initial_bezier.latches[anchor].get(0).is_some())
            .collect::<Vec<&AnchorEdge>>();

        let mut ends: Vec<(Handle<Bezier>, AnchorEdge)> = Vec::new();

        // if a curve is completely disconnected form other curves, a group cannot be created
        if anchors.len() == 0 && handles.len() > 1 {
            self.ends = None;
            return ();
        }
        // // if a curve is by itself, return the sole curve as a group
        // else if anchors.len() == 0 && handles.len() == 1 {
        //     ends.push((handle.clone(), AnchorEdge::Start));
        //     ends.push((handle.clone(), AnchorEdge::End));
        //     // println!("Anchors len : 0");
        //     self.ends = Some(ends.clone());
        //     return ();
        // }
        else if anchors.len() == 1 {
            // println!("Anchors len : 1");
            ends.push((handle.clone(), anchors[0].clone().other()));
        }

        let mut num_con = 0;

        // TODO: only consider curves that are selected
        for anchor in anchors.clone() {
            num_con += 1;
            let mut latch = initial_bezier.latches[&anchor].get(0).unwrap();
            //
            while num_con <= num_curves {
                //
                // let (partner_id, partners_edge) = (latch.latched_to_id, );
                let next_edge = latch.partners_edge.other();

                let next_curve_handle = id_handle_map.get(&latch.latched_to_id).unwrap().clone();

                let bezier_next = bezier_curves.get(next_curve_handle.clone()).unwrap();
                if let Some(next_latch) = bezier_next.latches[&next_edge].get(0) {
                    latch = next_latch;
                    num_con += 1;
                } else {
                    ends.push((next_curve_handle, next_edge));
                    break;
                }
            }
        }

        println!("num con : {}", num_con);
        println!("num curves: {}", num_curves);

        if num_con + 2 > num_curves {
            self.ends = Some(ends.clone());
        }
    }

    pub fn group_lut(
        &mut self,
        // ends: Vec<(Handle<Bezier>, AnchorEdge)>,
        bezier_curves: &mut ResMut<Assets<Bezier>>,
        // globals: &mut ResMut<Globals>,
        id_handle_map: HashMap<u128, Handle<Bezier>>,
    ) {
        // println!("got ends:  {:?}", self.ends);
        if let Some(ends) = self.ends.clone() {
            let (starting_handle, starting_anchor) = if let Some((handle, anchor)) = ends.get(0) {
                (handle.clone(), anchor.clone())
            } else {
                (
                    self.handles.iter().next().unwrap().clone(),
                    AnchorEdge::Start,
                )
            };

            let mut luts: Vec<(Vec<f64>, AnchorEdge, f32, Handle<Bezier>)> = Vec::new();

            let mut sorted_handles: Vec<Handle<Bezier>> = vec![starting_handle.clone()];

            let initial_bezier = bezier_curves.get(starting_handle.clone()).unwrap();
            // let mut total_length =  initial_bezier.length()
            luts.push((
                initial_bezier.lut.clone(),
                starting_anchor.other(),
                initial_bezier.length(),
                starting_handle.clone(),
            ));

            if let Some(mut latch) = initial_bezier.latches[&starting_anchor.other()].get(0) {
                //
                let mut found_connection = true;
                // let mut returned_to_initial_latch = false;

                // traverse a latched selection
                // return None if traversal cannot be done through all curves
                while found_connection {
                    //&& !returned_to_initial_latch {
                    //
                    let next_edge = latch.partners_edge.other();

                    let next_curve_handle =
                        id_handle_map.get(&latch.latched_to_id).unwrap().clone();

                    if next_curve_handle == starting_handle {
                        // unused value -> just for readability
                        // returned_to_initial_latch = true;
                        break;
                    }

                    let bezier_next = bezier_curves.get(next_curve_handle.clone()).unwrap();
                    sorted_handles.push(next_curve_handle.clone());
                    luts.push((
                        bezier_next.lut.clone(),
                        next_edge.clone(),
                        bezier_next.length(),
                        next_curve_handle.clone(),
                    ));

                    if let Some(next_latch) = bezier_next.latches[&next_edge].get(0) {
                        if self
                            .handles
                            .contains(id_handle_map.get(&next_latch.latched_to_id).unwrap())
                        {
                            latch = next_latch;
                        } else {
                            found_connection = false;
                        }
                    } else {
                        found_connection = false;
                    }
                }
            }

            let total_length = luts
                .iter()
                .fold(0.0, |acc, (_lut, _anchor, len, _handle)| acc + len);
            let mut min_t = 0.0;
            let mut group_lut: Vec<(Handle<Bezier>, AnchorEdge, (f64, f64), Vec<f64>)> = Vec::new();
            // println!("luts : {:?}", luts);
            for (lut, anchor, length, handle) in luts.clone() {
                let max_t = min_t + length / total_length;

                let t_m = (min_t as f64, max_t as f64);
                group_lut.push((handle, anchor, t_m, lut));
                min_t = max_t;
            }

            // update the look-up table
            self.lut = group_lut.clone();
        }
    }

    pub fn compute_position_with_bezier(
        &self,
        bezier_curves: &ResMut<Assets<Bezier>>,
        t: f64,
    ) -> Vec2 {
        let mut curve_index = 0;
        let mut pos = Vec2::ZERO;
        for (_handle, _anchor, (t_min, t_max), _lut) in &self.lut {
            if &t >= t_min && &t <= &(t_max + 0.00000000001) {
                break;
            } else {
                curve_index += 1;
            }
        }
        if let Some((handle, anchor, (t_min, t_max), lut)) = self.lut.get(curve_index) {
            let bezier = bezier_curves.get(handle.clone()).unwrap();

            // some of this code is shared with move_middle_quads()
            let curve = bezier.to_curve();
            let mut t_0_1 = (t as f64 - t_min) / (t_max - t_min);

            if anchor == &AnchorEdge::Start {
                t_0_1 = 1.0 - t_0_1;
            }

            t_0_1 = t_0_1.clamp(0.00000000001, 0.99999999999);

            let idx_f64 = t_0_1 * (lut.len() - 1) as f64;
            let p1 = lut[(idx_f64 as usize)];
            let p2 = lut[idx_f64 as usize + 1];

            // TODO: is the minus one useful here?
            let rem = idx_f64 % 1.0;
            let t_distance = interpolate(p1, p2, rem);
            let pos_coord2 = curve.point_at_pos(t_distance);
            pos = Vec2::new(pos_coord2.0 as f32, pos_coord2.1 as f32);
        }

        return pos;
    }

    pub fn compute_standalone_lut(
        &mut self,
        bezier_curves: &ResMut<Assets<Bezier>>,
        num_points: u32,
    ) {
        let mut total_length: f32 = 0.0;
        for lut in self.lut.clone() {
            let bezier = bezier_curves.get(lut.0).unwrap();
            total_length += bezier.length();
        }

        let vrange: Vec<f32> = (0..num_points)
            .map(|x| ((x) as f32) / (num_points as f32 - 1.0))
            .collect();

        let mut standalone_lut: (f32, Vec<Vec2>) = (total_length, Vec::new());
        for t in vrange {
            standalone_lut
                .1
                .push(self.compute_position_with_bezier(bezier_curves, t as f64));
        }

        self.standalone_lut = standalone_lut;
    }

    // this is now used inside the plugin, but this would be the function used in
    // an application where the look-up table (lut) would be loaded
    pub fn compute_position_with_lut(&self, t: f32) -> Vec2 {
        let lut = self.standalone_lut.1.clone();
        let idx_f64 = t * (lut.len() - 1) as f32;
        let p1 = lut[(idx_f64 as usize)];
        let p2 = lut[idx_f64 as usize + 1];
        let rem = idx_f64 % 1.0;
        let position = interpolate_vec2(p1, p2, rem);
        return position;
    }
}

pub struct Maps {
    pub mesh_handles: HashMap<&'static str, Handle<Mesh>>,
    pub pipeline_handles: HashMap<&'static str, Handle<PipelineDescriptor>>,
    pub id_handle_map: HashMap<u128, Handle<Bezier>>,
    pub sounds: HashMap<&'static str, Handle<AudioSource>>,
}

impl Default for Maps {
    fn default() -> Self {
        Maps {
            mesh_handles: HashMap::new(),
            pipeline_handles: HashMap::new(),
            id_handle_map: HashMap::new(),
            sounds: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Globals {
    pub do_hide_anchors: bool,
    pub do_hide_bounding_boxes: bool,
    pub do_spawn_curve: bool,
    pub num_points: usize,
    pub camera_scale: f32,
    pub scale: f32,
    pub picked_color: Option<Color>,
    pub history: Vec<Handle<Bezier>>,
    // pub bezier_handles: Vec<Handle<Bezier>>,
    // pub mesh_handles: HashMap<&'static str, Handle<Mesh>>,
    // pub pipeline_handles: HashMap<&'static str, Handle<PipelineDescriptor>>,
    // pub id_handle_map: HashMap<u128, Handle<Bezier>>,
    pub selected: Group,
    // pub sounds: HashMap<&'static str, Handle<AudioSource>>,
    pub sound_on: bool,
    pub hide_control_points: bool,
    pub group_lut_num_points: u32,
    // pub groups: Vec<Group>,
}

impl Default for Globals {
    fn default() -> Self {
        Self {
            do_hide_bounding_boxes: true,
            do_hide_anchors: false,
            do_spawn_curve: false,
            num_points: 25,
            camera_scale: 0.15,
            scale: 1.0,
            picked_color: None,
            history: Vec::new(),
            // mesh_handles: HashMap::new(),
            // pipeline_handles: HashMap::new(),
            // id_handle_map: HashMap::new(),
            selected: Group::default(),
            // sounds: HashMap::new(),
            sound_on: true,
            hide_control_points: false,
            group_lut_num_points: 100,
            // groups: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatchData {
    pub latched_to_id: u128,
    pub self_edge: AnchorEdge,
    pub partners_edge: AnchorEdge,
}

pub struct BezierCoord2 {
    pub start: Coord2,
    pub end: Coord2,
    pub control_points: (Coord2, Coord2),
    // control_start: Coord2,
    // control_end: Coord2,
}

#[derive(TypeUuid, Debug, Clone, RenderResources)]
#[uuid = "1e08866c-0b8a-437e-8bce-37733b21137e"]
#[allow(non_snake_case)]
pub struct MyShader {
    pub color: Color,
    pub clearcolor: Color,
    pub t: f32, // Bezier t-value for MiddleQuads, but is used for other purposes elsewhere
    pub zoom: f32,
    pub size: Vec2,
    pub hovered: f32,
}

impl Default for MyShader {
    fn default() -> Self {
        Self {
            color: Color::hex("F87575").unwrap(),
            t: 0.5,
            zoom: 0.15,
            size: Vec2::new(1.0, 1.0),
            clearcolor: Color::hex("6e7f80").unwrap(),
            hovered: 0.0,
        }
    }
}

// #[derive(RenderResources, Default, TypeUuid, Debug, Clone)]
#[derive(Debug, Clone, TypeUuid, Serialize, Deserialize)]
#[uuid = "1e08866c-0b8a-437e-8bce-37733b21957e"]
pub struct Bezier {
    pub positions: BezierPositions,
    pub previous_positions: BezierPositions,
    pub move_quad: Anchor, //MoveBezierElement,
    pub color: Option<Color>,
    pub do_compute_lut: bool,
    pub lut: Vec<f64>,
    pub id: u128,
    pub just_created: bool,
    // pub latch_start: Option<LatchData>, // the u8 is 0 for control_start and 1 for control_end
    // pub latch_end: Option<LatchData>,
    pub latches: HashMap<AnchorEdge, Vec<LatchData>>,
    pub grouped: bool,
}

impl Default for Bezier {
    fn default() -> Self {
        let mut rng = thread_rng();
        let mut latches = HashMap::new();
        latches.insert(AnchorEdge::Start, Vec::new());
        latches.insert(AnchorEdge::End, Vec::new());

        Bezier {
            positions: BezierPositions::default(),
            previous_positions: BezierPositions::default(),
            move_quad: Anchor::None,
            color: None,
            // lut: look-up table for linearizing the distance on a Bezier curve as a function of the t-value
            do_compute_lut: true,
            lut: Vec::new(),
            just_created: true,
            id: rng.gen(),
            latches,
            grouped: false,
            // latch_start: None, // id of the latch partner if applicable
            // latch_end: None,
        }
    }
}

impl Bezier {
    pub fn to_coord2(&self) -> BezierCoord2 {
        let pos = self.positions.clone();
        BezierCoord2 {
            start: Coord2(pos.start.x as f64, pos.start.y as f64),
            end: Coord2(pos.end.x as f64, pos.end.y as f64),
            // control_start: Coord2(self.control_start.x as f64, self.control_start.y as f64),
            // control_end: Coord2(self.control_end.x as f64, self.control_end.y as f64),
            control_points: (
                Coord2(pos.control_start.x as f64, pos.control_start.y as f64),
                Coord2(pos.control_end.x as f64, pos.control_end.y as f64),
            ),
        }
    }

    // pub fn reset_latches(&mut self) {
    //     self.latches = HashMap::new();
    //     self.latches.insert(AnchorEdge::Start, Vec::new());
    //     self.latches.insert(AnchorEdge::End, Vec::new());
    // }

    pub fn to_curve(&self) -> Curve<Coord2> {
        let bezier_c0 = self.to_coord2();

        return bezier::Curve::from_points(
            bezier_c0.start,
            bezier_c0.control_points,
            bezier_c0.end,
        );
    }

    pub fn length(&self) -> f32 {
        self.to_curve().estimate_length() as f32
    }

    pub fn update_previous_pos(&mut self) {
        self.previous_positions = self.positions.clone();
    }

    // pub fn is_moving(&self) -> bool {
    //     return self.move_quad != Anchor::None;
    // }

    pub fn get_position(&self, anchor: Anchor) -> Vec2 {
        match anchor {
            Anchor::Start => self.positions.start,
            Anchor::End => self.positions.end,
            Anchor::ControlEnd => self.positions.control_end,
            Anchor::ControlStart => self.positions.control_start,
            _ => Vec2::new(0.0, 0.0),
        }
    }

    pub fn get_opposite_control(&self, anchor: AnchorEdge) -> Vec2 {
        match anchor {
            AnchorEdge::Start => 2.0 * self.positions.start - self.positions.control_start,
            AnchorEdge::End => 2.0 * self.positions.end - self.positions.control_end,
        }
    }

    pub fn get_mover_edge(&self) -> AnchorEdge {
        match self.move_quad {
            Anchor::Start => AnchorEdge::Start,
            Anchor::End => AnchorEdge::End,
            _ => {
                println!("Warning: could not get mover edge, returning AnchorEdge::End by default");
                return AnchorEdge::End;
            }
        }
    }

    pub fn edge_is_moving(&self) -> bool {
        if self.move_quad == Anchor::Start || self.move_quad == Anchor::End {
            return true;
        }
        return false;
    }

    pub fn set_latch(&mut self, latch: LatchData) {
        // self.latches[&latch.self_edge] = vec![latch.clone()];
        if let Some(latch_local) = self.latches.get_mut(&latch.self_edge) {
            *latch_local = vec![latch.clone()];
        }
    }

    pub fn quad_is_latched(&self, anchor_edge: AnchorEdge) -> bool {
        !self.latches[&anchor_edge].is_empty()
    }

    pub fn ends_displacement(&self, scale: f32) -> ((Vec2, Vec2), (Quat, Quat)) {
        let quad_width = 1.0 * scale;
        let mut angles_vec = Vec::new();

        for anchor in vec![AnchorEdge::Start, AnchorEdge::End] {
            let control_point: Vec2;
            let anchor_point: Vec2;
            let constant_angle: f32;

            if anchor == AnchorEdge::Start {
                control_point = self.positions.control_start;
                anchor_point = self.positions.start;
                constant_angle = std::f32::consts::PI;
            } else {
                control_point = self.positions.control_end;
                anchor_point = self.positions.end;
                constant_angle = -std::f32::consts::PI;
            }

            let relative_position: Vec2 = control_point - anchor_point;
            let bezier_angle: f32 = relative_position.y.atan2(relative_position.x);

            let bezier_angle_90: f32 = bezier_angle + constant_angle;
            angles_vec.push(bezier_angle_90);
        }
        let angles = (angles_vec[0], angles_vec[1]);

        let start_displacement: Vec2 =
            -Vec2::new(quad_width * angles.0.cos(), quad_width * angles.0.sin());
        let end_displacement: Vec2 =
            -Vec2::new(quad_width * angles.1.cos(), quad_width * angles.1.sin());

        let start_rotation = Quat::from_rotation_z(angles.0);
        let end_rotation = Quat::from_rotation_z(angles.1);

        return (
            (start_displacement, end_displacement),
            (start_rotation, end_rotation),
        );
    }

    // pub fn get_mover_position_and_latch(&self) -> Option<(Vec2, Vec<LatchData>)> {
    //     let info = match self.move_quad {
    //         Anchor::Start => Some((
    //             self.positions.start,
    //             self.latches[&AnchorEdge::Start].clone(),
    //         )),
    //         Anchor::End => Some((self.positions.end, self.latches[&AnchorEdge::End].clone())),
    //         _ => None,
    //     };
    //     return info;
    // }

    pub fn update_positions_cursor(&mut self, cursor: &Res<Cursor>) {
        match self.move_quad {
            Anchor::None => {}

            Anchor::Start => {
                self.positions.start = self.previous_positions.start + cursor.pos_relative_to_click;
                self.positions.control_start =
                    self.previous_positions.control_start + cursor.pos_relative_to_click;
            }
            Anchor::End => {
                self.positions.end = self.previous_positions.end + cursor.pos_relative_to_click;
                self.positions.control_end =
                    self.previous_positions.control_end + cursor.pos_relative_to_click;
            }

            Anchor::ControlStart => {
                self.positions.control_start =
                    self.previous_positions.control_start + cursor.pos_relative_to_click;
            }

            Anchor::ControlEnd => {
                self.positions.control_end =
                    self.previous_positions.control_end + cursor.pos_relative_to_click;
            }
        }
    }

    pub fn get_mover_latch_info(&self) -> Option<(LatchData, Vec2, Vec2)> {
        match self.move_quad {
            Anchor::Start | Anchor::ControlStart => {
                if let Some(latch_start) =
                    self.latches.get(&AnchorEdge::Start).unwrap().clone().get(0)
                {
                    let latch_partner_id = latch_start.clone();
                    let partner_position = self.positions.start;

                    // The control points of latched edges are facing each other
                    let opposite_control =
                        2.0 * self.positions.start - self.positions.control_start;
                    return Some((latch_partner_id, partner_position, opposite_control));
                }
                return None;
            }
            Anchor::End | Anchor::ControlEnd => {
                if let Some(latch_end) = self.latches.get(&AnchorEdge::End).unwrap().clone().get(0)
                {
                    let latch_partner_id = latch_end.clone();
                    let partner_position = self.positions.end;

                    let opposite_control = 2.0 * self.positions.end - self.positions.control_end;
                    return Some((latch_partner_id, partner_position, opposite_control));
                }
                return None;
            }
            _ => None,
        }
    }

    pub fn generate_start_latch_on_spawn(&self, anchor_edge: AnchorEdge) -> Latch {
        let mut rng = thread_rng();
        match anchor_edge {
            AnchorEdge::Start => Latch {
                position: self.positions.start,
                control_point: 2.0 * self.positions.start - self.positions.control_start,
                latchee_id: self.id,
                latcher_id: rng.gen(),
                latchee_edge: AnchorEdge::Start,
            },
            AnchorEdge::End => Latch {
                position: self.positions.end,
                control_point: 2.0 * self.positions.end - self.positions.control_end,
                latchee_id: self.id,
                latcher_id: rng.gen(),
                latchee_edge: AnchorEdge::End,
            },
        }
    }

    pub fn send_latch_on_spawn(
        &mut self,
        anchor_edge: AnchorEdge,
        event_writer: &mut EventWriter<Latch>,
    ) {
        // pub fn send_latch_on_spawn(&mut self, anchor_edge: AnchorEdge, cursor: &mut ResMut<Cursor>) {
        let latch = self.generate_start_latch_on_spawn(anchor_edge);
        if let Some(latch_start) = self.latches.get_mut(&anchor_edge) {
            *latch_start = vec![LatchData {
                latched_to_id: latch.latcher_id,
                self_edge: anchor_edge,
                partners_edge: AnchorEdge::Start,
            }];

            // TODO: replace this by an event
            event_writer.send(latch);
            // cursor.latch = vec![latch];
        }
    }

    pub fn update_latched_position(
        &mut self,
        anchor_edge: AnchorEdge,
        control: Vec2,
        position: Vec2,
    ) {
        match anchor_edge {
            AnchorEdge::Start => {
                self.positions.control_start = control;
                self.positions.start = position;
            }
            AnchorEdge::End => {
                self.positions.control_end = control;
                self.positions.end = position;
            }
        }
    }

    pub fn bounding_box(&self) -> (Vec2, Vec2) {
        let bezier_coord = self.to_coord2();

        let curve0 = bezier::Curve::from_points(
            bezier_coord.start,
            bezier_coord.control_points,
            bezier_coord.end,
        );

        let bb: Bounds<Coord2> = curve0.bounding_box();

        let Coord2(ax, ay) = bb.min();
        let Coord2(bx, by) = bb.max();
        let bound0 = Vec2::new(ax as f32, ay as f32);
        let bound1 = Vec2::new(bx as f32, by as f32);
        return (bound0, bound1);
    }

    pub fn compute_real_distance(&self, t: f64) -> f64 {
        let idx_f64 = t * (self.lut.len() - 1) as f64;
        let p1 = self.lut[(idx_f64 as usize)];
        let p2 = self.lut[idx_f64 as usize + 1];
        //
        // TODO: is the minus one useful here?
        let rem = (idx_f64 - 1.0) % 1.0;
        let t_distance = interpolate(p1, p2, rem);
        return t_distance;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

// // Could use these functions to make the plugin flo_curve independent
// pub fn lerp(p0: Coord2, p1: Coord2, t: f64) -> Coord2 {
//     return p0 + t * (p1 - p0);
// }

// pub fn de_casteljau(curve: Curve<Coord2>, t: f64) -> Coord2 {
//     let a = lerp(p0, p1, t);
//     let b = lerp(p1, p2, t);
//     let c = lerp(p2, p3, t);
//     let d = lerp(a, b, t);
//     let e = lerp(b, c, t);
//     let p = lerp(d, e, t);
//     return p;
// }

// get the Bezier handle of the closest point to "position" (other than itself
// if other_than_moving is true)
pub fn get_close_anchor(
    max_dist: f32,
    position: Vec2,
    bezier_curves: &ResMut<Assets<Bezier>>,
    query: &Query<&Handle<Bezier>, With<BoundingBoxQuad>>,
    // mut globals: ResMut<Globals>,
    scale: f32,
) -> Option<(f32, Anchor, Handle<Bezier>)> {
    for bezier_handle in query.iter() {
        if let Some(bezier) = bezier_curves.get(bezier_handle) {
            let ((start_displacement, end_displacement), (_start_rotation, _end_rotation)) =
                bezier.ends_displacement(scale);

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
                    bezier_handle.clone(),
                ));
            } else if distance_to_control1 < max_dist {
                return Some((
                    distance_to_control1,
                    Anchor::ControlEnd,
                    bezier_handle.clone(),
                ));
            } else if distance_to_start < max_dist {
                return Some((distance_to_start, Anchor::Start, bezier_handle.clone()));
            } else if distance_to_endpoint < max_dist {
                return Some((distance_to_endpoint, Anchor::End, bezier_handle.clone()));
            }
        }
    }
    return None;
}

pub fn get_close_anchor_entity(
    max_dist: f32,
    position: Vec2,
    bezier_curves: &ResMut<Assets<Bezier>>,
    query: &Query<(Entity, &Handle<Bezier>), With<BoundingBoxQuad>>,
    scale: f32,
) -> Option<(f32, Anchor, Entity, Handle<Bezier>)> {
    //
    for (entity, bezier_handle) in query.iter() {
        //
        if let Some(bezier) = bezier_curves.get(bezier_handle) {
            //
            let ((start_displacement, end_displacement), (_start_rotation, _end_rotation)) =
                bezier.ends_displacement(scale);

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

pub fn get_close_still_anchor(
    max_dist: f32,
    position: Vec2,
    bezier_curves: &ResMut<Assets<Bezier>>,
    query: &Query<(&Handle<Bezier>, &BoundingBoxQuad)>,
) -> Option<(f32, AnchorEdge, Handle<Bezier>)> {
    for (bezier_handle, _bb) in query.iter() {
        if let Some(bezier) = bezier_curves.get(bezier_handle) {
            let distance_to_start = (bezier.positions.start - position).length();
            let distance_to_endpoint = (bezier.positions.end - position).length();

            if distance_to_start < max_dist && (bezier.move_quad != Anchor::Start) {
                return Some((distance_to_start, AnchorEdge::Start, bezier_handle.clone()));
            } else if distance_to_endpoint < max_dist && (bezier.move_quad != Anchor::End) {
                return Some((distance_to_endpoint, AnchorEdge::End, bezier_handle.clone()));
            }
        }
    }
    return None;
}

// change the selection mesh according to the bounding box of the selected curves
pub fn adjust_selection_attributes(
    // mouse_button_input: Res<Input<MouseButton>>,
    mut my_shader_params: ResMut<Assets<MyShader>>,
    mut query: Query<&Handle<Mesh>, With<SelectedBoxQuad>>,
    shader_query: Query<&Handle<MyShader>, With<SelectedBoxQuad>>,
    bezier_curves: ResMut<Assets<Bezier>>,
    mut meshes: ResMut<Assets<Mesh>>,
    globals: ResMut<Globals>,
    mut action_event_reader: EventReader<Action>,
    user_state: Res<UserState>,
) {
    // TODO: make this system run only when necessary
    let mut do_adjust = false;
    // if mouse_button_input.pressed(MouseButton::Left) {
    //     do_adjust = true;
    // }
    if let UserState::MovingAnchor(_moving_handle) = user_state.as_ref() {
        do_adjust = true;
    }
    if let Some(Action::Selected) = action_event_reader.iter().next() {
        do_adjust = true;
    }

    if do_adjust {
        let (mut minx, mut maxx, mut miny, mut maxy) =
            (1000000.0f32, -1000000.0f32, 1000000.0f32, -1000000.0f32);

        // We set the mesh attributes as a function of the bounding box.
        // This could be done by removing the mesh from the mesh asset
        // and adding a brand new mesh
        for (_entity, selected_handle) in globals.selected.group.clone() {
            let bezier = bezier_curves.get(selected_handle).unwrap();

            let (bound0, bound1) = bezier.bounding_box();
            minx = minx.min(bound0.x);
            maxx = maxx.max(bound1.x);
            miny = miny.min(bound0.y);
            maxy = maxy.max(bound1.y);
        }
        let shader_handle = shader_query.single();
        let mut shader_params = my_shader_params.get_mut(shader_handle).unwrap();
        let up_factor = 1.10;
        let x_pos = (maxx + minx) / 2.0;
        let y_pox = (maxy + miny) / 2.0;
        let x_width = (maxx - minx) * up_factor / 2.0;
        let y_width = (maxy - miny) * up_factor / 2.0;

        // send correct width to shader that will adjust the thickness of the box accordingly
        let scale = globals.scale / 0.5;
        shader_params.size = Vec2::new(x_width * 2.0 / scale, y_width * 2.0 / scale);

        let vertex_positions = vec![
            [x_pos - x_width, y_pox - y_width, 0.0],
            [x_pos - x_width, y_pox + y_width, 0.0],
            [x_pos + x_width, y_pox + y_width, 0.0],
            [x_pos + x_width, y_pox - y_width, 0.0],
        ];

        for mesh_handle in query.iter_mut() {
            let mesh = meshes.get_mut(mesh_handle).unwrap();
            let v_pos = mesh.attribute_mut("Vertex_Position");

            if let Some(array2) = v_pos {
                *array2 = Float32x3(vertex_positions.clone());
            }
        }
    }
}

// change the selection mesh according to the bounding box of the selected curves
pub fn adjust_selecting_attributes(
    user_state: ResMut<UserState>,
    cursor: ResMut<Cursor>,
    mut my_shader_params: ResMut<Assets<MyShader>>,
    mut query: Query<&Handle<Mesh>, With<SelectingBoxQuad>>,
    shader_query: Query<&Handle<MyShader>, With<SelectingBoxQuad>>,
    mut meshes: ResMut<Assets<Mesh>>,
    globals: ResMut<Globals>,
) {
    // TODO: make this system run only when necessary
    if let UserState::Selecting(click_position) = user_state.as_ref() {
        let mouse_position = cursor.position;

        let (mut minx, mut maxx, mut miny, mut maxy) = (
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
        let scale = globals.scale / 0.5;
        shader_params.size = Vec2::new(x_width * 2.0 / scale, y_width * 2.0 / scale);

        let vertex_positions = vec![
            [x_pos - x_width, y_pox - y_width, 0.0],
            [x_pos - x_width, y_pox + y_width, 0.0],
            [x_pos + x_width, y_pox + y_width, 0.0],
            [x_pos + x_width, y_pox - y_width, 0.0],
        ];

        for mesh_handle in query.iter_mut() {
            let mesh = meshes.get_mut(mesh_handle).unwrap();
            let v_pos = mesh.attribute_mut("Vertex_Position");

            if let Some(array2) = v_pos {
                *array2 = Float32x3(vertex_positions.clone());
            }
        }
    }
}

// change the group selection mesh according to the bounding box of the curves inside the group
pub fn adjust_group_attributes(
    mouse_button_input: Res<Input<MouseButton>>,
    mut my_shader_params: ResMut<Assets<MyShader>>,
    mut query: Query<&Handle<Mesh>, With<GroupBoxQuad>>,
    groups: ResMut<Assets<Group>>,
    group_query: Query<(&Handle<Group>, &Handle<MyShader>)>,
    bezier_curves: ResMut<Assets<Bezier>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // TODO: make this system run only when necessary
    if mouse_button_input.pressed(MouseButton::Left) {
        for (group_handle, shader_handle) in group_query.iter() {
            let group = groups.get(group_handle).unwrap();
            let (mut minx, mut maxx, mut miny, mut maxy) =
                (1000000.0f32, -1000000.0f32, 1000000.0f32, -1000000.0f32);

            // We set the mesh attributes as a function of the bounding box.
            // This could be done by removing the mesh from the mesh asset
            // and adding a brand new mesh
            for (_entity, selected_handle) in group.group.clone() {
                let bezier = bezier_curves.get(selected_handle).unwrap();

                let (bound0, bound1) = bezier.bounding_box();
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
                let v_pos = mesh.attribute_mut("Vertex_Position");

                if let Some(array2) = v_pos {
                    *array2 = Float32x3(vertex_positions.clone());
                }
            }
        }
    }
}

// // Before saving the curves, we compute a lower-memory-and-higher-accuracy look-up table using accelerated gradient
// // descent
// pub fn do_long_lut(
//     keyboard_input: Res<Input<KeyCode>>,
//     query: Query<&Handle<Bezier>, With<BoundingBoxQuad>>,
//     mut bezier_curves: ResMut<Assets<Bezier>>,
//     time: Res<Time>,
// ) {
//     if keyboard_input.pressed(KeyCode::LControl) && keyboard_input.just_pressed(KeyCode::S) {
//         // let last_handle_option: Option<Handle<Bezier>> = None;
//         for handle in query.iter() {
//             let bezier = bezier_curves.get_mut(handle).unwrap();
//             let curve = bezier.to_curve();
//             if let Some(lut_gradient_descent) = compute_lut_long(curve, 100, time.clone()) {
//                 bezier.lut = lut_gradient_descent;
//                 // println!("computed LUT with accelerated  gradient descent");
//             } else {
//                 println!("failed to find look-up table using accelerated gradient descent");
//             }
//         }
//     }
// }

pub fn compute_lut(curve: Curve<Coord2>, num_sections: usize) -> Vec<f64> {
    let mut section_lengths: Vec<f64> = Vec::new();
    let whole_distance = curve.estimate_length();
    for k in 0..(num_sections) {
        let t_min = k as f64 / (num_sections + 1) as f64;
        let t_max = (k + 1) as f64 / (num_sections + 1) as f64;
        section_lengths.push(curve.section(t_min, t_max).estimate_length())
    }

    let vrange: Vec<f64> = (0..num_sections)
        .map(|x| (whole_distance * (x) as f64) / (num_sections as f64 - 1.0))
        .collect();

    // TODO: get rid of redundant length_so_far computations
    let mut look_up_table: Vec<f64> = Vec::new();
    for distance in vrange {
        let mut length_so_far = 0.0;
        for (idx, sl) in section_lengths.iter().enumerate() {
            if distance - 0.0001 <= length_so_far {
                let t = idx as f64 / num_sections as f64;
                look_up_table.push(t);
                break;
            }
            length_so_far += sl;
        }
    }
    return look_up_table;
}

fn derivative(curve: Curve<Coord2>, t: f64, dist: f64) -> f64 {
    let delta_t = 0.0000001;
    let d = 2.0
        * (curve.section(0.0, t).estimate_length() - dist)
        * (curve.section(0.0, t + delta_t / 2.0).estimate_length()
            - curve.section(0.0, t - delta_t / 2.0).estimate_length())
        / delta_t;

    return d;
}

// Computes a better look-up table using gradient descent with Nesterov acceleration
pub fn compute_lut_long(
    curve: Curve<Coord2>,
    num_sections: usize,
    mut time: Time,
) -> Option<Vec<f64>> {
    // let time_at_start = time.seconds_since_startup();
    time.update();
    let mut total_time = 0.0;
    // let mut time_now;

    // let section_lengths: Vec<f64> = Vec::new();
    let whole_distance = curve.estimate_length();
    // Curved sectioned uniformly
    let vrange: Vec<f64> = (0..num_sections)
        .map(|x| (whole_distance * (x) as f64) / (num_sections as f64 - 1.0))
        .collect();

    // // generate plot
    // let f1 = Plot::from_function(|x| curve.section(0.0, x).estimate_length(), 0., 1.)
    //     .line_style(LineStyle::new().colour("burlywood"));

    // let v = ContinuousView::new().add(f1);

    // Page::single(&v).save("function.svg").expect("saving svg");
    // println!("saving svg");

    let eta0 = 0.00001;
    let mut eta;
    let gamma = 0.8;
    let mut look_up_table: Vec<f64> = Vec::new();
    let mut t = 0.0;
    let mut dist_at_t = 0.0;
    let mut cost;
    let mut df;
    let mut momentum;
    let target_cost = (0.01 * whole_distance / 100.0).max(0.01);

    let mut rng = thread_rng();
    // println!("{:?}", target_cost);
    // for (dist_idx, distance) in vrange.clone().iter().enumerate() {
    for distance in vrange.iter() {
        cost = (dist_at_t - distance) * (dist_at_t - distance);
        momentum = 0.0;
        let mut k = 0.0;
        // println!("{:?}", cost);
        // t = distance / whole_distance;
        while cost > target_cost {
            df = derivative(curve, t - gamma * momentum, distance.clone());

            // put a upper bound on the derivative to avoid an explosion of the t-value
            if df.abs() > 50.0 {
                df = 50.0 * df / df.abs();
            }

            eta = eta0 * (1.0 + (rng.gen::<f64>() + 0.5) * 0.1);
            momentum = gamma * momentum + eta * df;
            t = t - momentum;

            // The estimate_length() function for Bezier curves accepts negative t-values,
            // and we correct this behavior by forcing t to be positive.
            if t < 0.0 {
                momentum = -momentum;
                t = t.abs();
            }

            dist_at_t = curve.section(0.0, t).estimate_length();
            cost = (dist_at_t - distance) * (dist_at_t - distance);

            k = k + 1.0;

            let delta_time = time.delta_seconds();
            total_time += delta_time;
            // println!("{:?}", total_time);
            time.update();
            if total_time > 0.2 {
                return None;
            }

            // println!(
            //     "{:?}, dist: {:?}, t: {:?}, df: {:?}, dist_t: {:?}, cost: {:?}, ",
            //     idx, distance, t, df, dist_at_t, cost
            // );
        }

        look_up_table.push(t);
    }

    return Some(look_up_table);
}

// TODO: refactor
pub fn change_ends_and_controls_params(
    mut bezier_curves: ResMut<Assets<Bezier>>,
    mut query: Query<&Handle<Bezier>, With<BoundingBoxQuad>>,
    cursor: Res<Cursor>,
    globals: ResMut<Globals>,
    mut maps: ResMut<Maps>,
) {
    if cursor.latch.is_empty() {
        let mut latch_info: Option<(LatchData, Vec2, Vec2)> = None;

        // TODO: use an event here instead of scanning for a moving quad
        for bezier_handle in query.iter_mut() {
            if let Some(bezier) = bezier_curves.get_mut(bezier_handle) {
                bezier.update_positions_cursor(&cursor);
                latch_info = bezier.get_mover_latch_info();
                if let Some(_) = latch_info {
                    break;
                }
            }
        }

        // change the control point of a latched point
        if let Some((partner_latch, mover_position, opposite_control)) = latch_info {
            // println!("{:?}", mover_position);
            // println!("moving latch");
            if let Some(bezier_handle) = maps.id_handle_map.get(&partner_latch.latched_to_id) {
                let bezier = bezier_curves.get_mut(bezier_handle).unwrap();
                bezier.update_latched_position(
                    partner_latch.partners_edge,
                    opposite_control,
                    mover_position,
                );
            } else {
                // Problems with non-existing ids may occur when using undo, redo and delete
                // TODO: Delete latched anchors that no longer have a partner
                println!(
                    "Warning: Could not retrieve handle for Bezier id: {}",
                    &partner_latch.latched_to_id
                );
            }
        }
    }
}
