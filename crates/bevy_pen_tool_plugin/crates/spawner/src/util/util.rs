use crate::inputs::*;
use crate::util::materials::*;

use bevy::{asset::HandleId, prelude::*, reflect::TypeUuid, sprite::Mesh2dHandle, utils::Uuid};

use serde::{Deserialize, Serialize};

use rand::prelude::*;

use std::collections::HashMap;
use std::collections::HashSet;

use flo_curves::bezier::BezierCurve;
use flo_curves::bezier::Curve;
use flo_curves::*;

use bevy_inspector_egui::Inspectable;

#[derive(Clone)]
pub struct CurveVec(pub Vec<BezierId>);

#[derive(Copy, Clone)]
pub struct CurveIdEdge {
    pub id: BezierId,
    pub anchor_edge: AnchorEdge,
}

pub enum PenCommand {
    Spawn {
        positions: BezierPositions,
        id: BezierId,
    },
    Move(MoveCommand),
    Latch {
        l1: CurveIdEdge,
        l2: CurveIdEdge,
    },
    Unlatch {
        l1: CurveIdEdge,
        l2: CurveIdEdge,
    },
    Delete {
        id: BezierId,
    },
}

#[derive(Copy, Clone)]
pub struct MoveCommand {
    pub anchor: Anchor,
    pub id: BezierId,
    pub new_position: Vec2,
}

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
    // API
    // TODO: make most of the structs public(crate) and
    // leave only the API public
    //
    // Programmatically generate a Bezier Curve (see examples)
    pub fn new(positions: BezierPositions, id: u64) -> Self {
        // generate a random id

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

#[derive(Debug)]
pub enum UserState {
    Idle,
    Selecting(Vec2),
    Selected(Group),
    // SpawningCurve {
    //     bezier_hist: Option<BezierHist>,
    //     maybe_bezier_id: Option<BezierId>,
    // },
    // MovingAnchor,
    MovingWholeCurve,
}

impl Default for UserState {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Component)]
pub struct BezierParent;

#[derive(Component)]
pub struct GroupParent;

pub struct Loaded;

#[derive(Component)]
pub struct GroupMesh(pub Color);

#[derive(Component)]
pub struct RoadMesh(pub Color);

#[derive(Component)]
pub struct BezierGrandParent;

#[derive(Component)]
pub struct Icon;

#[derive(Component)]
pub struct OnOffMaterial {
    pub material: Handle<Image>,
}

#[derive(Component)]
pub struct AchorEdgeQuad(pub AnchorEdge);

#[derive(Component)]
pub struct ControlPointQuad(pub AnchorEdge);

#[derive(Component)]
pub struct MiddlePointQuad;

#[derive(Component)]
pub struct GroupMiddleQuad(pub usize);

#[derive(Debug, Component)]
pub struct BoundingBoxQuad;

#[derive(Component)]
pub struct SelectedBoxQuad;

#[derive(Component)]
pub struct SelectingBoxQuad;

#[derive(Component)]
pub struct GroupBoxQuad;

#[derive(Debug)]
pub struct OfficialLatch(pub LatchData, pub Handle<Bezier>);

pub struct GroupBoxEvent;

#[derive(Debug)]
pub struct SpawnMids {
    pub color: Color,
    pub bezier_handle: Handle<Bezier>,
    pub parent_entity: Entity,
}

// helicopter animation
#[derive(Component)]
pub struct TurnRoundAnimation;

#[derive(Component)]
pub struct FollowBezierAnimation {
    pub animation_offset: f64,
    pub initial_direction: Vec3,
}

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

// TODO: change all instances of LutDistance to LutPosition
//
// look-up tables (LUT):
//
// map from t-values (between 0 and 1) to distance on Bezier curve.
// A t-values is converted to an index in the LUT
type LutDistance = Vec<f64>;
// map from t-values (between 0 and 1) to point on Bezier curve
type LutPosition = Vec<Vec2>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LutSaveLoad {
    pub lut: Vec<((f64, f64), LutDistance)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StandaloneLut {
    pub path_length: f32,
    pub lut: LutPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSaveLoad {
    // the AnchorEdge corresponds to first anchor encountered when traversing the group
    pub lut: Vec<(Bezier, AnchorEdge, (f64, f64), LutDistance)>,
    // pub standalone_lut: (f32, LutPosition),
    pub standalone_lut: StandaloneLut,
}

#[derive(Debug, Clone, TypeUuid, PartialEq)]
#[uuid = "b16f31ff-a594-4fca-a0e3-85e626d3d01a"] // do not change this uuid without changing the Default impl for GroupId
pub struct Group {
    // TODO: rid Group of redundancy
    pub group: HashSet<(Entity, Handle<Bezier>)>,
    pub bezier_handles: HashSet<Handle<Bezier>>,
    //
    // Attempts to store the start and end points of a group.
    // Fails if curves are not connected or if the curves form a loop
    pub ends: Option<Vec<(Handle<Bezier>, AnchorEdge)>>,
    //
    // vec of each curve's look-up table
    // the tuple (f64, f64) represents (t_min, t_max), the min and max t-values for
    // the curve
    pub lut: Vec<(Handle<Bezier>, AnchorEdge, (f64, f64), LutDistance)>,
    pub standalone_lut: StandaloneLut,
    pub group_id: GroupId,
}

impl Default for Group {
    fn default() -> Self {
        // let mut rng = thread_rng();
        Group {
            group: HashSet::new(),
            bezier_handles: HashSet::new(),
            lut: Vec::new(),
            ends: None,
            standalone_lut: StandaloneLut {
                path_length: 0.0,
                lut: Vec::new(),
            },
            group_id: GroupId::default(),
            // ..Default::default() // group_id: HandleId::default(),
        }
    }
}

impl Group {
    pub fn into_group_save(&self, bezier_curves: &mut ResMut<Assets<Bezier>>) -> GroupSaveLoad {
        let mut lut = Vec::new();
        for (handle, anchor, t_ends, local_lut) in self.lut.iter() {
            let mut bezier = bezier_curves.get(&handle.clone()).unwrap().clone();
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
        id_handle_map: HashMap<BezierId, BezierHandleEntity>,
    ) {
        //
        if self.bezier_handles.len() == 0 {
            return ();
        };

        let mut handles = self.bezier_handles.clone();
        let num_curves = handles.len();
        let handle = handles.iter().next().unwrap().clone();
        handles.remove(&handle);

        // TODO: is this really a good way of clone assets?
        let bezier_curve_hack = bezier_curves
            .iter()
            .map(|(s, x)| (s.clone(), x.clone()))
            .collect::<HashMap<HandleId, Bezier>>();

        let initial_bezier = bezier_curves.get_mut(&handle.clone()).unwrap();

        let anchors_temp = vec![AnchorEdge::Start, AnchorEdge::End];
        let anchors = anchors_temp
            .iter()
            .filter(|anchor| initial_bezier.quad_is_latched(anchor))
            .collect::<Vec<&AnchorEdge>>();

        let mut ends: Vec<(Handle<Bezier>, AnchorEdge)> = Vec::new();

        // if a curve is completely disconnected form other curves, a group cannot be created
        if anchors.len() == 0 && handles.len() > 1 {
            self.ends = None;
            return ();
        } else if anchors.len() == 1 {
            // println!("Anchors len : 1");
            ends.push((handle.clone(), anchors[0].clone().other()));
        }

        let mut num_con = 0;

        // TODO: only consider curves that are selected
        for anchor in anchors.clone() {
            num_con += 1;
            // let mut latch = initial_bezier.latches[&anchor].get(0).unwrap();
            let mut latch = initial_bezier.latches.get(&anchor).unwrap().clone();
            // if let Some(latch) = initial_bezier.latches.get_mut(&anchor) {
            //
            while num_con <= num_curves {
                //
                // let (partner_id, partners_edge) = (latch.latched_to_id, );
                let next_edge = latch.partners_edge.other();

                let next_curve_handle = id_handle_map
                    .get(&latch.latched_to_id.into())
                    .unwrap()
                    .clone();

                let bezier_next = bezier_curve_hack.get(&next_curve_handle.handle.id).unwrap();

                if let Some(next_latch) = bezier_next.latches.get(&next_edge) {
                    latch = next_latch.clone();
                    num_con += 1;
                } else {
                    ends.push((next_curve_handle.handle, next_edge));
                    break;
                }
            }
            // }
        }

        if num_con + 2 > num_curves {
            self.ends = Some(ends.clone());
        }
    }

    pub fn group_lut(
        &mut self,
        bezier_curves: &mut ResMut<Assets<Bezier>>,
        id_handle_map: HashMap<BezierId, BezierHandleEntity>,
    ) {
        // if the group is connected with latches, then go ahead and group
        if let Some(ends) = self.ends.clone() {
            let (starting_handle, starting_anchor) = if let Some((handle, anchor)) = ends.get(0) {
                (handle.clone(), anchor.clone())
            } else {
                (
                    self.bezier_handles.iter().next().unwrap().clone(),
                    AnchorEdge::Start,
                )
            };

            let mut luts: Vec<(LutDistance, AnchorEdge, f32, Handle<Bezier>)> = Vec::new();

            let mut sorted_handles: Vec<Handle<Bezier>> = vec![starting_handle.clone()];

            let initial_bezier = bezier_curves.get(&starting_handle.clone()).unwrap();
            //
            luts.push((
                initial_bezier.lut.clone(),
                starting_anchor.other(),
                initial_bezier.length(),
                starting_handle.clone(),
            ));

            if let Some(mut latch) = initial_bezier.latches.get(&starting_anchor.other()) {
                //
                let mut found_connection = true;

                // traverse a latched selection
                // return None if traversal cannot be done through all curves
                while found_connection {
                    //&& !returned_to_initial_latch {
                    //
                    let next_edge = latch.partners_edge.other();

                    let next_curve_handle = id_handle_map
                        .get(&latch.latched_to_id)
                        .unwrap()
                        .clone()
                        .handle;

                    if next_curve_handle == starting_handle {
                        // returned to initial latch -> true
                        break;
                    }

                    let bezier_next = bezier_curves.get(&next_curve_handle.clone()).unwrap();
                    sorted_handles.push(next_curve_handle.clone());
                    luts.push((
                        bezier_next.lut.clone(),
                        next_edge.clone(),
                        bezier_next.length(),
                        next_curve_handle.clone(),
                    ));

                    if let Some(next_latch) = bezier_next.latches.get(&next_edge) {
                        if self
                            .bezier_handles
                            .contains(&id_handle_map.get(&next_latch.latched_to_id).unwrap().handle)
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
            let mut group_lut: Vec<(Handle<Bezier>, AnchorEdge, (f64, f64), LutDistance)> =
                Vec::new();
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
        let mut pos: Vec2 = Vec2::ZERO;
        //
        for (_handle, _anchor, (t_min, t_max), _lut) in &self.lut {
            // println!("t: {}, t_min: {}, t_max: {}, ", t, t_min, t_max);
            if &t >= t_min && &t <= &(t_max + 0.000001) {
                break;
            } else {
                curve_index += 1;
            }
        }
        //
        if let Some((handle, anchor, (t_min, t_max), lut)) = self.lut.get(curve_index) {
            //
            let bezier = bezier_curves.get(&handle.clone()).unwrap();

            // some of this code is shared with move_middle_quads()
            let curve = bezier.to_curve();
            let mut t_0_1 = (t as f64 - t_min) / (t_max - t_min);

            if anchor == &AnchorEdge::Start {
                t_0_1 = 1.0 - t_0_1;
            }

            t_0_1 = t_0_1.clamp(0.00000000001, 0.9999);

            let idx_f64 = t_0_1 * (lut.len() - 1) as f64;
            let p1 = lut[(idx_f64 as usize)];
            let p2 = lut[idx_f64 as usize + 1];

            let rem = idx_f64 % 1.0;
            let t_distance = interpolate(p1, p2, rem);
            let pos_coord2 = curve.point_at_pos(t_distance);

            pos = Vec2::new(pos_coord2.0 as f32, pos_coord2.1 as f32);
        } else {
            println!("couldn't get a curve at index: {}. ", curve_index);
        }

        return pos;
    }

    pub fn compute_normal_with_bezier(
        &self,
        bezier_curves: &ResMut<Assets<Bezier>>,
        t: f64,
    ) -> Vec2 {
        let mut curve_index = 0;

        #[allow(unused_assignments)]
        let mut normal = Vec2::ZERO;
        for (_handle, _anchor, (t_min, t_max), _lut) in &self.lut {
            // println!("t: {}, t_min: {}, t_max: {}, ", t, t_min, t_max);
            if &t >= t_min && &t <= &(t_max + 0.000001) {
                break;
            } else {
                curve_index += 1;
            }
        }
        if let Some((handle, anchor, (t_min, t_max), lut)) = self.lut.get(curve_index) {
            let bezier = bezier_curves.get(&handle.clone()).unwrap();

            // some of this code is shared with move_middle_quads()
            let curve = bezier.to_curve();
            let mut t_0_1 = (t as f64 - t_min) / (t_max - t_min);

            if anchor == &AnchorEdge::Start {
                t_0_1 = 1.0 - t_0_1;
            }

            t_0_1 = t_0_1.clamp(0.00000000001, 0.9999);

            let idx_f64 = t_0_1 * (lut.len() - 1) as f64;
            let p1 = lut[(idx_f64 as usize)];
            let p2 = lut[idx_f64 as usize + 1];

            let rem = idx_f64 % 1.0;
            let t_distance = interpolate(p1, p2, rem);

            use flo_curves::bezier::NormalCurve;

            let normal_coord2 = curve.normal_at_pos(t_distance).to_unit_vector();

            normal = Vec2::new(normal_coord2.x() as f32, normal_coord2.y() as f32);
        } else {
            panic!("couldn't get a curve at index: {}. ", curve_index);
        }

        return normal;
    }

    pub fn compute_standalone_lut(
        &mut self,
        bezier_curves: &ResMut<Assets<Bezier>>,
        num_points: u32,
    ) {
        let mut total_length: f32 = 0.0;
        for lut in self.lut.clone() {
            let bezier = bezier_curves.get(&lut.0).unwrap();
            total_length += bezier.length();
        }

        let t_range: Vec<f64> = (0..num_points)
            .map(|x| ((x) as f64) / (num_points as f64 - 1.0))
            .collect();

        let mut standalone_lut: StandaloneLut = StandaloneLut {
            path_length: total_length,
            lut: Vec::new(),
        };
        for t in t_range {
            let val = self.compute_position_with_bezier(bezier_curves, t);

            standalone_lut.lut.push(val);
        }

        self.standalone_lut = standalone_lut;
    }

    // this is now used inside the plugin, but this would be the function used in
    // an application where the look-up table (lut) would be loaded
    pub fn compute_position_with_lut(&self, t: f32) -> Vec2 {
        let lut = self.standalone_lut.lut.clone();
        let idx_f64 = t * (lut.len() - 1) as f32;
        let p1 = lut[(idx_f64 as usize)];
        let p2 = lut[idx_f64 as usize + 1];
        let rem = idx_f64 % 1.0;
        let position = interpolate_vec2(p1, p2, rem);
        return position;
    }
}

pub type AnchorEntities = HashMap<Anchor, Entity>;

#[derive(Clone, Debug)]
pub struct BezierHandleEntity {
    pub handle: Handle<Bezier>,
    pub entity: Entity,
    pub anchor_entities: AnchorEntities,
}

#[derive(Debug)]
pub struct Maps {
    pub mesh_handles: HashMap<&'static str, Mesh2dHandle>,
    // pub pipeline_handles: HashMap<&'static str, Handle<PipelineDescriptor>>,
    pub bezier_map: HashMap<BezierId, BezierHandleEntity>,
    pub group_map: HashMap<GroupId, Handle<Group>>,
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
            bezier_map: HashMap::new(),
            group_map: HashMap::new(),
            sounds: HashMap::new(),
            textures: HashMap::new(),
        }
    }
}

// pub struct History {
//     pub history: Vec<Handle<Bezier>>,
// }

pub struct Selection {
    pub selected: Group,
}

impl Default for Selection {
    fn default() -> Self {
        Self {
            selected: Group::default(),
        }
    }
}

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
            selection_box: 0.33,
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

#[derive(Debug, Clone, Serialize, PartialEq, Deserialize, Inspectable)]
pub struct LatchData {
    // #[inspectable(min = 0, max = 999999999999999999)]
    pub latched_to_id: BezierId,
    pub self_edge: AnchorEdge,
    pub partners_edge: AnchorEdge,
    // pub latch_position: Vec2,
}

impl Default for LatchData {
    fn default() -> Self {
        Self {
            latched_to_id: BezierId::default(),
            self_edge: AnchorEdge::default(),
            partners_edge: AnchorEdge::default(),
            // latch_position: Vec2::default(),
        }
    }
}

pub type LatchInfo = Option<(LatchData, Vec2, Vec2)>;

pub struct BezierCoord2 {
    pub start: Coord2,
    pub end: Coord2,
    pub control_points: (Coord2, Coord2),
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Hash, Eq, Inspectable)]
pub struct GroupId(HandleId);
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
    pub group: Option<GroupId>,
}

impl Default for Bezier {
    fn default() -> Self {
        Bezier {
            do_compute_lut: true,
            latches: HashMap::new(),
            id: BezierId::default(),
            positions: BezierPositions::default(),
            previous_positions: BezierPositions::default(),
            // move_quad: Anchor::default(),
            color: None,
            lut: LutDistance::default(),
            potential_latch: None,
            group: None,
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

        return bezier::Curve::from_points(
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
            },
            AnchorEdge::End => Latch {
                position: self.positions.end,
                control_point: 2.0 * self.positions.end - self.positions.control_end,
                latchee_id: self.id,
                latcher_id: BezierId::default(),
                latchee_edge: AnchorEdge::End,
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

        info!("moving anchor {:?}", anchor);

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
    // mouse_button_input: Res<Input<MouseButton>>,
    mut my_shader_params: ResMut<Assets<SelectionMat>>,
    mut query: Query<&Mesh2dHandle, With<SelectedBoxQuad>>,
    shader_query: Query<&Handle<SelectionMat>, With<SelectedBoxQuad>>,
    bezier_curves: ResMut<Assets<Bezier>>,
    mut meshes: ResMut<Assets<Mesh>>,
    // globals: ResMut<Globals>,
    selection: ResMut<Selection>,
    user_state: Res<UserState>,
    mut group_box_event_reader: EventReader<GroupBoxEvent>,
) {
    let mut do_adjust = false;

    // if let UserState::MovingAnchor = user_state.as_ref() {
    //     do_adjust = true;
    // }

    let us = user_state.as_ref();
    if let UserState::Selected(_) = us {
        do_adjust = true;
    }

    if do_adjust || group_box_event_reader.iter().next().is_some() {
        // println!("adjusting selection!!!");
        let (mut minx, mut maxx, mut miny, mut maxy) =
            (1000000.0f32, -1000000.0f32, 1000000.0f32, -1000000.0f32);

        // We set the mesh attributes as a function of the bounding box.
        // This could be done by removing the mesh from the mesh asset
        // and adding a brand new mesh
        for (_entity, selected_handle) in selection.selected.group.clone() {
            let bezier = bezier_curves.get(&selected_handle).unwrap();

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
    user_state: ResMut<UserState>,
    cursor: ResMut<Cursor>,
    mut my_shader_params: ResMut<Assets<SelectingMat>>,
    mut query: Query<&Mesh2dHandle, With<SelectingBoxQuad>>,
    shader_query: Query<&Handle<SelectingMat>, With<SelectingBoxQuad>>,
    mut meshes: ResMut<Assets<Mesh>>,
    globals: ResMut<Globals>,
) {
    // TODO: make this system run only when necessary
    if let UserState::Selecting(mut click_position) = user_state.as_ref() {
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

        for mesh_handle in query.iter_mut() {
            let mesh = meshes.get_mut(&mesh_handle.0.clone()).unwrap();
            let v_pos = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION);

            if let Some(array2) = v_pos {
                // println!("changed selecting");
                *array2 =
                    bevy::render::mesh::VertexAttributeValues::Float32x3(vertex_positions.clone());
            }
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
            let group = groups.get(group_handle).unwrap();
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

pub fn move_anchor(
    commands: &mut Commands,
    move_command: MoveCommand,
    mut bezier_curves: &mut ResMut<Assets<Bezier>>,
    maps: &ResMut<Maps>,
) {
    // println!("undo: MovedAnchor");
    let anchor = move_command.anchor;
    let handle_entities = maps.bezier_map[&move_command.id.into()].clone();
    let bezier = bezier_curves.get_mut(&handle_entities.handle).unwrap();

    bezier.set_position(anchor, move_command.new_position);

    // attaches MovingAnchor component to the entity
    bezier.move_anchor(
        commands,
        true,  // one move for a single frame
        false, // do not follow mouse
        anchor,
        maps.as_ref(),
    );

    let anchor_edge = anchor.to_edge_with_controls();
    if let Some(_) = bezier.latches.get(&anchor_edge) {
        let latch_info = bezier.get_anchor_latch_info(anchor);

        update_latched_partner_position(&maps.bezier_map, &mut bezier_curves, latch_info);
    }
}

use std::path::PathBuf;
pub fn open_file_dialog(save_name: &str, folder: &str, extension: &str) -> Option<PathBuf> {
    let mut k = 0;

    let mut default_path = std::env::current_dir().unwrap();
    default_path.push("saved");
    default_path.push(folder.to_string());
    let mut default_name: String;

    loop {
        default_name = save_name.to_string();
        default_name.push_str(&(k.to_string()));
        default_name.push_str(extension);

        default_path.push(&default_name);

        if !default_path.exists() {
            break;
        }
        default_path.pop();

        k += 1;
    }

    let res = rfd::FileDialog::new()
        .set_file_name(&default_name)
        .set_directory(&default_path)
        .save_file();
    println!("The user chose: {:#?}", &res);

    return res;
}

pub fn save_mesh(
    mesh_handle: &Handle<Mesh>,
    meshes: &Res<Assets<Mesh>>,
    dialog_result: Option<PathBuf>,
) {
    if let Some(path) = dialog_result {
        let mesh = meshes.get(mesh_handle).unwrap();
        let vertex_attributes = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
        let indices_u32 = mesh.indices().unwrap();

        match (vertex_attributes, indices_u32) {
            (
                bevy::render::mesh::VertexAttributeValues::Float32x3(vertices),
                bevy::render::mesh::Indices::U32(indices),
            ) => {
                let obj_vertices = vertices
                    .clone()
                    .iter()
                    .map(|arr| obj_exporter::Vertex {
                        x: arr[0] as f64,
                        y: arr[1] as f64,
                        z: arr[2] as f64,
                    })
                    .collect::<Vec<obj_exporter::Vertex>>();

                // let mut obj_inds_vecs: Vec<Vec<u32>> =
                // indices.chunks(3).map(|x| x.to_vec()).collect();
                let obj_inds_vecs: Vec<(usize, usize, usize)> = indices
                    .chunks_exact(3)
                    .map(|z| {
                        let mut x = z.iter();
                        return (
                            *x.next().unwrap() as usize,
                            *x.next().unwrap() as usize,
                            *x.next().unwrap() as usize,
                        );
                    })
                    .collect();

                let normals = vec![obj_exporter::Vertex {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                }];

                let set = obj_exporter::ObjSet {
                    material_library: None,
                    objects: vec![obj_exporter::Object {
                        name: "My_mesh".to_owned(),
                        vertices: obj_vertices,
                        tex_vertices: vec![],
                        normals,
                        geometry: vec![obj_exporter::Geometry {
                            material_name: None,
                            shapes: obj_inds_vecs
                                .into_iter()
                                .map(|(x, y, z)| obj_exporter::Shape {
                                    primitive: obj_exporter::Primitive::Triangle(
                                        (x, Some(x), Some(0)),
                                        (y, Some(y), Some(0)),
                                        (z, Some(z), Some(0)),
                                    ),
                                    groups: vec![],
                                    smoothing_groups: vec![],
                                })
                                .collect(),
                        }],
                    }],
                };

                obj_exporter::export_to_file(&set, path).unwrap();
            }
            _ => {}
        }
    }
}
