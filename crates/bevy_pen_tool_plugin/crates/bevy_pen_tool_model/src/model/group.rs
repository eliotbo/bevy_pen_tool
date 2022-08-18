use crate::model::*;

use bevy::{asset::HandleId, prelude::*, reflect::TypeUuid};

use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::collections::HashSet;

use flo_curves::bezier::BezierCurve;

use flo_curves::*;

// TODO: change all instances of LutDistance to LutPosition
//
// look-up tables (LUT):
//
// map from t-values (between 0 and 1) to distance on Bezier curve.
// A t-values is converted to an index in the LUT
pub type LutDistance = Vec<f64>;
// map from t-values (between 0 and 1) to point on Bezier curve
type LutPosition = Vec<Vec2>;

pub struct ComputeGroupLut(pub GroupId);

#[derive(Component)]
pub struct GroupMiddleQuad(pub usize);

pub struct GroupBoxEvent;

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
    // the AnchorEdge is the starting
    pub lut: Vec<(Handle<Bezier>, AnchorEdge, (f64, f64), LutDistance)>,
    pub standalone_lut: StandaloneLut,
    pub id: GroupId,
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
            id: GroupId::default(),
            // ..Default::default() // group_id: HandleId::default(),
        }
    }
}

impl Group {
    pub fn add_curve(&mut self, curve_entity: Entity, curve_handle: Handle<Bezier>) {
        self.group.insert((curve_entity, curve_handle.clone()));
        self.bezier_handles.insert(curve_handle.clone());
    }

    pub fn remove_curve(
        &mut self,
        bezier_handle_entity: &BezierHandleEntity,
        bezier_curves: &BezierAssets, //&Res<Assets<Bezier>>,
        id_handle_map: HashMap<BezierId, BezierHandleEntity>,
    ) {
        self.bezier_handles.remove(&bezier_handle_entity.handle);
        self.group.remove(&(
            bezier_handle_entity.entity,
            bezier_handle_entity.handle.clone(),
        ));
        self.find_connected_ends(bezier_curves, id_handle_map.clone());
        self.group_lut(bezier_curves, id_handle_map.clone());
        self.compute_standalone_lut(bezier_curves, 1000);
    }

    pub fn into_group_save(&self, bezier_curves: &Res<Assets<Bezier>>) -> GroupSaveLoad {
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
        bezier_curves: &BezierAssets, //&Res<Assets<Bezier>>,
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
        let bezier_curve_hack = bezier_curves.clone();
        // bezier_curves
        //     .iter()
        //     .map(|(s, x)| (s.clone(), x.clone()))
        //     .collect::<HashMap<HandleId, Bezier>>();

        let initial_bezier = bezier_curves.get(&handle.id).unwrap();

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
        // bezier_curves: HashMap<bevy::asset::HandleId, &Bezier>,
        bezier_curves: &BezierAssets,
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

            let initial_bezier = bezier_curves.get(&starting_handle.id).unwrap();
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

                    let bezier_next = bezier_curves.get(&next_curve_handle.id).unwrap();
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

    pub fn compute_position_with_bezier(&self, bezier_curves: &BezierAssets, t: f64) -> Vec2 {
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
            let bezier = bezier_curves.get(&handle.id).unwrap();

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

    pub fn compute_normal_with_bezier(&self, bezier_curves: &BezierAssets, t: f64) -> Vec2 {
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
        let mut s = 1.0;
        if let Some((handle, anchor, (t_min, t_max), lut)) = self.lut.get(curve_index) {
            let bezier = bezier_curves.get(&handle.id).unwrap();

            // some of this code is shared with move_middle_quads()
            let curve = bezier.to_curve();
            let mut t_0_1 = (t as f64 - t_min) / (t_max - t_min);

            if anchor == &AnchorEdge::Start {
                t_0_1 = 1.0 - t_0_1;

                // this sign is important for road mesh generation
                s = -1.0;
            }

            t_0_1 = t_0_1.clamp(0.00000000001, 0.9999);

            let idx_f64 = t_0_1 * (lut.len() - 1) as f64;
            let p1 = lut[(idx_f64 as usize)];
            let p2 = lut[idx_f64 as usize + 1];

            let rem = idx_f64 % 1.0;
            let t_distance = interpolate(p1, p2, rem);

            use flo_curves::bezier::NormalCurve;

            let normal_coord2 = curve.normal_at_pos(t_distance).to_unit_vector();

            normal = Vec2::new(normal_coord2.x() as f32, normal_coord2.y() as f32) * s;
        } else {
            panic!("couldn't get a curve at index: {}. ", curve_index);
        }

        return normal;
    }

    pub fn compute_standalone_lut(&mut self, bezier_curves: &BezierAssets, num_points: u32) {
        let mut total_length: f32 = 0.0;
        for lut in self.lut.clone() {
            let bezier = bezier_curves.get(&lut.0.id).unwrap();
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

    // compute the average position of the anchors making up the group
    pub fn center_of_mass(&self, bezier_curves: &BezierAssets) -> Vec2 {
        let mut center_of_mass = Vec2::ZERO;
        for (handle, anchor, _t_range, _lut) in &self.lut {
            let bezier = bezier_curves.get(&handle.id).unwrap();
            // center_of_mass += bezier.center_of_mass();
            let pos = match anchor {
                AnchorEdge::Start => bezier.positions.start,
                AnchorEdge::End => bezier.positions.end,
            };
            center_of_mass += pos;
        }
        center_of_mass /= self.lut.len() as f32;
        return center_of_mass;
    }
}
