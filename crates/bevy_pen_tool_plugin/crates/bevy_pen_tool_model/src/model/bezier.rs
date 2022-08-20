use crate::inputs::*;
// use crate::util::materials::*;
use crate::model::*;

use bevy::{asset::HandleId, prelude::*, reflect::TypeUuid, utils::Uuid};

use serde::{Deserialize, Serialize};

use rand::prelude::*;

use std::collections::HashMap;
use std::collections::HashSet;

use flo_curves::bezier::BezierCurve;
use flo_curves::bezier::Curve;
use flo_curves::*;

use bevy_inspector_egui::Inspectable;

pub type BezierAssets<'a> = HashMap<bevy::asset::HandleId, &'a Bezier>;

pub struct SpawnCurve {
    pub positions: BezierPositions,
}

pub struct SpawningCurve {
    pub bezier_hist: Option<BezierHist>,
    pub maybe_bezier_id: Option<BezierId>,
    pub follow_mouse: bool,
}

pub type BezierHistId = u64;

#[derive(Debug, Clone, Default, Inspectable)]
pub struct BezierHist {
    pub positions: BezierPositions,
    pub color: Option<Color>,
    pub latches: HashMap<AnchorEdge, LatchData>,
    pub id: BezierHistId,
    pub do_send_to_history: bool,
}

impl From<&Bezier> for BezierHist {
    fn from(bezier: &Bezier) -> Self {
        Self {
            positions: bezier.positions.clone(),
            color: None,
            latches: bezier.latches.clone(),
            id: bezier.id.into(),
            do_send_to_history: false,
        }
    }
}

impl BezierHist {
    pub fn new(positions: BezierPositions, id: u64) -> Self {
        Self {
            positions,
            color: None,
            latches: HashMap::new(),
            id,
            do_send_to_history: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MoveAnchorEvent {
    pub bezier_id: BezierId,
    pub anchor: Anchor,
    pub unlatch: bool,
    pub once: bool, // if true, MovingQuad will be removed after a single frame
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnlatchEvent {
    pub bezier_id: BezierId,
    pub anchor: Anchor,
}

pub struct RemoveMovingQuadEvent(Anchor);

pub struct RedoDelete {
    pub bezier_id: BezierId,
}
pub struct ComputeLut;

#[derive(Debug, Clone, Default)]
pub struct GroupHist {
    pub bezier_handles: HashSet<Handle<Bezier>>,
    pub ends: Option<Vec<(Handle<Bezier>, AnchorEdge)>>,
}

impl From<&Group> for GroupHist {
    fn from(group: &Group) -> Self {
        Self {
            bezier_handles: group.bezier_handles.clone(),
            ends: group.ends.clone(),
        }
    }
}

#[derive(Debug, Clone, Inspectable)]
pub enum HistoryAction {
    MovedAnchor {
        bezier_id: BezierHistId,
        previous_position: Vec2,
        new_position: Vec2,
        anchor: Anchor,
    },

    SpawnedCurve {
        bezier_id: BezierHistId,
        bezier_hist: BezierHist,
    },

    DeletedCurve {
        bezier: BezierHist,
        bezier_id: BezierHistId,
    },

    Latched {
        self_id: BezierHistId,
        self_anchor: AnchorEdge,
        partner_id: BezierHistId,
        partner_anchor: AnchorEdge,
    },

    Unlatched {
        self_id: BezierHistId,
        partner_id: BezierHistId,
        self_anchor: AnchorEdge,
        partner_anchor: AnchorEdge,
    },

    // MovedGroup {
    //     // group_handle: Handle<Group>,
    //     group_id: GroupId,
    //     previous_position: Vec2,
    //     new_position: Vec2,
    // },
    // DeletedGroup {
    //     group: GroupHist,
    //     bezier_hists: Vec<BezierHist>,
    // },
    // Grouped {
    //     group_handle: Handle<Group>,
    // },
    // UnGrouped {
    //     bezier_handles: Vec<Handle<Bezier>>,
    // },
    None,
}

impl Default for HistoryAction {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Inspectable)]
pub struct History {
    pub actions: Vec<HistoryAction>,
    pub index: i32,
}

impl Default for History {
    fn default() -> Self {
        Self {
            actions: vec![],
            index: -1,
        }
    }
}

/// Either the start point or the end point of a Bezier curve.
#[derive(PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize, Hash, Inspectable)]
pub enum AnchorEdge {
    Start,
    End,
}

impl Default for AnchorEdge {
    fn default() -> Self {
        Self::Start
    }
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

#[derive(Component)]
pub struct BezierParent;

#[derive(Component)]
pub struct GroupParent;

pub struct Loaded(pub Group);

#[derive(Component)]
pub struct AchorEdgeQuad(pub AnchorEdge);

#[derive(Component)]
pub struct ControlPointQuad(pub AnchorEdge);

#[derive(Component)]
pub struct MiddlePointQuad;

#[derive(Debug)]
pub struct OfficialLatch(pub LatchData, pub Handle<Bezier>);

#[derive(Debug)]
pub struct SpawnMids {
    pub color: Color,
    pub bezier_handle: Handle<Bezier>,
    pub parent_entity: Entity,
}

/// A Bezier curve is defined by four points: the start and end points (also called anchor edges throughout the crate)
/// and two control points. The [`Anchor::All`] variant is used to refer to all four points.
#[derive(
    PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Copy, Inspectable, Hash, Component,
)]
pub enum Anchor {
    Start,
    End,
    ControlStart,
    ControlEnd,
    All, // used to move the whole curve
    None,
}

impl Default for Anchor {
    fn default() -> Self {
        Self::None
    }
}

impl Anchor {
    pub fn to_edge(&self) -> AnchorEdge {
        match self {
            Self::Start => AnchorEdge::Start,
            Self::End => AnchorEdge::End,
            _ => {
                panic!("Failure to convert Anchor to AnchorEdge!");
            }
        }
    }

    pub fn to_edge_with_controls(&self) -> AnchorEdge {
        match self {
            Self::Start | Self::ControlStart => AnchorEdge::Start,
            Self::End | Self::ControlEnd => AnchorEdge::End,
            _ => {
                panic!("Failure to convert Anchor::None to AnchorEdge!");
            }
        }
    }

    pub fn adjoint(&self) -> Anchor {
        match self {
            Self::Start => Self::ControlStart,
            Self::End => Self::ControlEnd,
            Self::ControlStart => Self::Start,
            Self::ControlEnd => Self::End,
            _ => {
                panic!("Anchor::None has no adjoint!");
            }
        }
    }
    pub fn is_edge(&self) -> bool {
        match self {
            Self::Start | Self::End => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Deserialize, Inspectable)]
pub struct LatchData {
    pub latched_to_id: BezierId,
    pub self_edge: AnchorEdge,
    pub partners_edge: AnchorEdge,
}

impl Default for LatchData {
    fn default() -> Self {
        Self {
            latched_to_id: BezierId::default(),
            self_edge: AnchorEdge::default(),
            partners_edge: AnchorEdge::default(),
        }
    }
}

pub type LatchInfo = Option<(LatchData, Vec2, Vec2)>;

pub struct BezierCoord2 {
    pub start: Coord2,
    pub end: Coord2,
    pub control_points: (Coord2, Coord2),
}

/// Identifier for a Bezier curve. Collisions are possible but very unlikely.
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Hash, Eq, Inspectable)]
pub struct BezierId(pub HandleId);

impl From<HandleId> for BezierId {
    fn from(id: HandleId) -> Self {
        Self(id)
    }
}

impl From<BezierHistId> for BezierId {
    fn from(id: BezierHistId) -> Self {
        let uuid = Uuid::parse_str("8cb22c5d-5ab0-4912-8833-ab46062b7d38").unwrap();
        Self(HandleId::new(uuid, id))
    }
}

impl Into<BezierHistId> for BezierId {
    fn into(self) -> BezierHistId {
        if let HandleId::Id(_, id) = self.0 {
            id
        } else {
            panic!("BezierId is not an Id");
        }
    }
}

impl Default for BezierId {
    fn default() -> Self {
        let mut rng = thread_rng();
        let uuid = Uuid::parse_str("8cb22c5d-5ab0-4912-8833-ab46062b7d38").unwrap();
        Self(HandleId::new(uuid, rng.gen()))
    }
}

use core::fmt::Debug;
impl Debug for BezierId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        if let HandleId::Id(_, id) = self.0 {
            write!(f, "BezierId({})", id)
        } else {
            write!(f, "none")
        }
    }
}

use core::fmt;

impl fmt::Display for BezierId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        if let HandleId::Id(_, id) = self.0 {
            write!(f, "BezierId({})", id)
        } else {
            write!(f, "none")
        }
    }
}

#[derive(Component)]
pub struct MovingAnchor {
    pub once: bool,         // if true, the anchor will move for only one frame
    pub follow_mouse: bool, // if false, the anchor is the adjoint of actively moving anchor
}

#[derive(Debug, Clone, TypeUuid, Serialize, Deserialize)]
#[uuid = "8cb22c5d-5ab0-4912-8833-ab46062b7d38"] // do not change this uuid without changing the Default impl for BezierId
pub struct Bezier {
    pub positions: BezierPositions,
    pub previous_positions: BezierPositions, // was useful for an undo functionality
    // pub move_quad: Anchor,
    pub color: Option<Color>,
    pub do_compute_lut: bool,
    pub lut: LutDistance,
    pub id: BezierId,
    pub latches: HashMap<AnchorEdge, LatchData>,
    pub potential_latch: Option<LatchData>,
    pub group: GroupId,
    pub entity: Option<Entity>,
}

impl Default for Bezier {
    fn default() -> Self {
        Bezier {
            do_compute_lut: true,
            // move_quad: Anchor::default(),
            color: None,
            potential_latch: None,
            group: GroupId::default(),
            lut: LutDistance::default(),
            latches: HashMap::new(),
            id: BezierId::default(),
            positions: BezierPositions::default(),
            previous_positions: BezierPositions::default(),
            entity: None,
            // ..Default::default()
        }
    }
}

impl Bezier {
    pub fn to_coord2(&self) -> BezierCoord2 {
        let pos = self.positions.clone();
        BezierCoord2 {
            start: Coord2(pos.start.x as f64, pos.start.y as f64),
            end: Coord2(pos.end.x as f64, pos.end.y as f64),
            control_points: (
                Coord2(pos.control_start.x as f64, pos.control_start.y as f64),
                Coord2(pos.control_end.x as f64, pos.control_end.y as f64),
            ),
        }
    }

    pub fn to_curve(&self) -> Curve<Coord2> {
        let bezier_c0 = self.to_coord2();

        return flo_curves::bezier::Curve::from_points(
            bezier_c0.start,
            bezier_c0.control_points,
            bezier_c0.end,
        );
    }

    // Computes a better look-up table using the walking algorithm from flo_curve
    // pub fn compute_lut_walk(curve: Curve<Coord2>, num_sections: usize) -> LutDistance {
    pub fn compute_lut_walk(&mut self, num_sections: usize) {
        let bezier_c = self.to_coord2();

        let curve = flo_curves::bezier::Curve::from_points(
            bezier_c.start,
            bezier_c.control_points,
            bezier_c.end,
        );

        let whole_distance = curve.estimate_length();

        let mut look_up_table: LutDistance = Vec::new();

        flo_curves::bezier::walk_curve_evenly(&curve, whole_distance / num_sections as f64, 0.001)
            .for_each(|section| {
                let (_t_min, t_max) = section.original_curve_t_values();
                look_up_table.push(t_max);
            });

        self.lut = look_up_table;
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

    pub fn set_position(&mut self, anchor: Anchor, pos: Vec2) {
        match anchor {
            Anchor::Start => {
                let delta = self.positions.control_start - self.positions.start;
                self.positions.start = pos;
                self.positions.control_start = pos + delta;
            }
            Anchor::End => {
                let delta = self.positions.control_end - self.positions.end;
                self.positions.end = pos;
                self.positions.control_end = pos + delta;
            }
            Anchor::ControlEnd => {
                self.positions.control_end = pos;
            }
            Anchor::ControlStart => {
                self.positions.control_start = pos;
            }
            _ => {}
        }
    }

    pub fn set_previous_pos(&mut self, anchor: Anchor, pos: Vec2) {
        match anchor {
            Anchor::Start => {
                self.previous_positions.start = pos;
            }
            Anchor::End => {
                self.previous_positions.end = pos;
            }
            Anchor::ControlEnd => {
                self.previous_positions.control_end = pos;
            }
            Anchor::ControlStart => {
                self.previous_positions.control_start = pos;
            }
            _ => {}
        }
    }

    pub fn get_previous_position(&self, anchor: Anchor) -> Vec2 {
        match anchor {
            Anchor::Start => self.previous_positions.start,
            Anchor::End => self.previous_positions.end,
            Anchor::ControlEnd => self.previous_positions.control_end,
            Anchor::ControlStart => self.previous_positions.control_start,
            _ => Vec2::new(0.0, 0.0),
        }
    }

    pub fn get_opposite_control(&self, anchor: AnchorEdge) -> Vec2 {
        match anchor {
            AnchorEdge::Start => 2.0 * self.positions.start - self.positions.control_start,
            AnchorEdge::End => 2.0 * self.positions.end - self.positions.control_end,
        }
    }

    pub fn set_latch(&mut self, latch: LatchData) {
        self.latches.insert(latch.self_edge, latch.clone());
    }

    pub fn quad_is_latched(&self, anchor_edge: &AnchorEdge) -> bool {
        // !self.latches[&anchor_edge].is_empty()
        self.latches.contains_key(&anchor_edge)
    }

    // computes the desired anchor quad positions
    // they should be slighty off the anchor positions, towards the curve center
    pub fn ends_displacement(&self) -> ((Vec2, Vec2), (Quat, Quat)) {
        let quad_width = 5.0;
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

    // compute anchor positions, given cursor position relative to the last clicked position,
    // taking scale into account
    pub fn update_positions_cursor(&mut self, cursor: &Res<Cursor>, anchor: Anchor) {
        match anchor {
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

            Anchor::All => {
                self.positions.start = self.previous_positions.start + cursor.pos_relative_to_click;
                self.positions.end = self.previous_positions.end + cursor.pos_relative_to_click;
                self.positions.control_start =
                    self.previous_positions.control_start + cursor.pos_relative_to_click;
                self.positions.control_end =
                    self.previous_positions.control_end + cursor.pos_relative_to_click;
            }
        }
    }

    // gives the LatchData of the anchor that is attached to the moving anchor
    pub fn get_anchor_latch_info(&self, anchor: Anchor) -> Option<(LatchData, Vec2, Vec2)> {
        match anchor {
            Anchor::Start | Anchor::ControlStart => {
                if let Some(latch_start) = self.latches.get(&AnchorEdge::Start) {
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
                if let Some(latch_end) = self.latches.get(&AnchorEdge::End) {
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
        // let mut rng = thread_rng();
        match anchor_edge {
            AnchorEdge::Start => Latch {
                position: self.positions.start,
                control_point: 2.0 * self.positions.start - self.positions.control_start,
                latchee_id: self.id,
                latcher_id: BezierId::default(),
                latchee_edge: AnchorEdge::Start,
                group_id: self.group,
            },
            AnchorEdge::End => Latch {
                position: self.positions.end,
                control_point: 2.0 * self.positions.end - self.positions.control_end,
                latchee_id: self.id,
                latcher_id: BezierId::default(),
                latchee_edge: AnchorEdge::End,
                group_id: self.group,
            },
        }
    }

    pub fn send_latch_on_spawn(
        &mut self,
        anchor_edge: AnchorEdge,
        event_writer: &mut EventWriter<Latch>,
    ) {
        let latch = self.generate_start_latch_on_spawn(anchor_edge);

        let latch_start = LatchData {
            latched_to_id: latch.latcher_id,
            self_edge: anchor_edge,
            partners_edge: AnchorEdge::Start,
        };

        self.latches.insert(anchor_edge, latch_start);

        event_writer.send(latch);
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

    pub fn move_anchor(
        &mut self,
        commands: &mut Commands,
        once: bool,
        follow_mouse: bool,
        anchor: Anchor,
        maps: &Maps,
    ) {
        self.do_compute_lut = true;

        let id = self.id;

        let handle_entities = maps.bezier_map[&id.into()].clone();

        // info!("moving anchor {:?}", anchor);

        let anchor_entity = handle_entities.anchor_entities[&anchor];

        commands.entity(anchor_entity).insert(MovingAnchor {
            once,
            // the main anchor follows the mouse, unless it's a one-frame move
            follow_mouse: follow_mouse && !once,
        });

        let adjoint_anchor_entity = handle_entities.anchor_entities[&anchor.adjoint()];

        commands.entity(adjoint_anchor_entity).insert(MovingAnchor {
            once,
            follow_mouse: false,
        });

        // move_anchor_event_writer.send(moving_partner_anchor_event);
        let anchor_edge = anchor.to_edge_with_controls();
        if let Some(latch) = self.latches.get(&anchor_edge) {
            let partner_handle_entity = maps.bezier_map[&latch.latched_to_id.into()].clone();

            commands
                .entity(partner_handle_entity.anchor_entities[&latch.partners_edge.to_anchor()])
                .insert(MovingAnchor {
                    once,
                    follow_mouse: false,
                });

            commands
                .entity(
                    partner_handle_entity.anchor_entities
                        [&latch.partners_edge.to_anchor().adjoint()],
                )
                .insert(MovingAnchor {
                    once,
                    follow_mouse: false,
                });
        }
    }

    pub fn bounding_box(&self) -> (Vec2, Vec2) {
        let bezier_coord = self.to_coord2();

        let curve0 = flo_curves::bezier::Curve::from_points(
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

// let mut visited = HashSet::new();
// let mut queue = VecDeque::new();
// queue.push_back(self.id);
// while let Some(id) = queue.pop_front() {
//     if visited.contains(&id) {
//         continue;
//     }
//     visited.insert(id);
//     let curve = &bezier_curves[id];
//     for edge in [AnchorEdge::Start, AnchorEdge::End].iter() {
//         if let Some(latch) = curve.latches.get(edge) {
//             queue.push_back(latch.latched_to_id);
//         }
//     }
// }
// self.chained_curves = visited;

pub type AnchorEntities = HashMap<Anchor, Entity>;

#[derive(Clone, Debug)]
pub struct BezierHandleEntity {
    pub handle: Handle<Bezier>,
    pub entity: Entity,
    pub anchor_entities: AnchorEntities,
}

pub fn find_onesided_chained_curves(
    first_bezier: &Bezier,
    bezier_curves: &BezierAssets,
    id_handle_map: HashMap<BezierId, BezierHandleEntity>,
    mut next_anchor_edge: AnchorEdge,
) -> Vec<BezierHandleEntity> {
    //
    let self_handle_entity = id_handle_map[&first_bezier.id].clone();
    let mut chained_curves = vec![self_handle_entity];
    let mut current_curve = first_bezier.clone();
    //
    loop {
        let maybe_latch = current_curve.latches.get(&next_anchor_edge);

        if let Some(latch) = maybe_latch {
            // the next anchor edge is the anchor edge opposite to known latched edge
            next_anchor_edge = latch.partners_edge.other();

            let next_handle_entity = id_handle_map[&latch.latched_to_id].clone();

            // find the next curve
            let next_curve = bezier_curves
                .get(&next_handle_entity.handle.id)
                .unwrap()
                .clone();

            // add next curve to the list
            chained_curves.push(next_handle_entity);

            current_curve = next_curve.clone();
        } else {
            break;
        }
    }

    return chained_curves;
}

// pub struct LatchData {
//     pub latched_to_id: BezierId,
//     pub self_edge: AnchorEdge,
//     pub partners_edge: AnchorEdge,
// }
