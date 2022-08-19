use crate::inputs::Action;
use crate::model::*;
use crate::{FillMesh2dMaterial, RoadMesh2dMaterial};

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use lyon::tessellation::geometry_builder::simple_builder;
use lyon::tessellation::math::{point, Point};
use lyon::tessellation::path::Path;
use lyon::tessellation::{FillOptions, FillTessellator, VertexBuffers};

use rand::{thread_rng, Rng};

use std::collections::HashMap;
use std::collections::HashSet;
//
//
//
//

pub type MeshId = u64;

#[derive(Component, Clone, Debug)]
pub struct PenMesh {
    pub id: MeshId,
    pub bounding_box: (Vec2, Vec2),
}

pub struct MinsMaxes {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl Default for MinsMaxes {
    fn default() -> Self {
        MinsMaxes {
            min_x: f32::MAX,
            min_y: f32::MAX,
            max_x: f32::MIN,
            max_y: f32::MIN,
        }
    }
}

impl MinsMaxes {
    pub fn update(&mut self, pos: Vec2) {
        self.min_x = self.min_x.min(pos.x);
        self.min_y = self.min_y.min(pos.y);
        self.max_x = self.max_x.max(pos.x);
        self.max_y = self.max_y.max(pos.y);
    }

    pub fn to_vec2_pair(&self) -> (Vec2, Vec2) {
        (
            Vec2::new(self.min_x, self.min_y),
            Vec2::new(self.max_x, self.max_y),
        )
    }
}

// #[derive(Component, Clone, Debug)]
// pub struct FillMesh {
//     pub id: MeshId,
//     pub bounding_box: (Vec2, Vec2),
// }

#[derive(Component, Clone, Debug)]
pub struct StartMovingMesh {
    pub start_position: Vec2,
}

// spawn a road along the selected group
//
//
pub fn make_road(
    mut action_event_reader: EventReader<Action>,
    mut commands: Commands,
    curves: Res<Assets<Bezier>>,
    globals: Res<Globals>,
    selection: ResMut<Selection>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut groups: ResMut<Assets<Group>>,

    mut road_materials: ResMut<Assets<RoadMesh2dMaterial>>,
    mut maps: ResMut<Maps>,
) {
    if action_event_reader.iter().any(|x| x == &Action::SpawnRoad) {
        if selection.selected.iter().count() == 1 {
            let temp = selection.selected.clone();
            let selected = temp.iter().next().unwrap().clone();
            if let SelectionChoice::CurveSet(curve_set) = selected {
                //
                // check whether the curve set is part of the same group
                let mut group_id_set = HashSet::new();

                for curve in &curve_set {
                    if let Some(handle_entity) = maps.bezier_map.get(&curve) {
                        //
                        let bezier = curves.get(&handle_entity.handle).unwrap();
                        group_id_set.insert(bezier.group);
                    }
                }
                if group_id_set.iter().count() != 1 {
                    info!("cannot spawn road from curves in different groups");
                    return;
                }
                if let Some(group_handle) = maps.group_map.get(group_id_set.iter().next().unwrap())
                // unwrap never fails
                {
                    let group = groups.get_mut(&group_handle).unwrap();

                    let bezier_assets = curves
                        .iter()
                        .collect::<HashMap<bevy::asset::HandleId, &Bezier>>();

                    group.find_connected_ends(&bezier_assets, maps.bezier_map.clone());

                    group.group_lut(&bezier_assets, maps.bezier_map.clone());
                    group.compute_standalone_lut(&bezier_assets, globals.group_lut_num_points);

                    let center_of_mass = group.center_of_mass(&bezier_assets);

                    let num_points = globals.group_lut_num_points;

                    let crop = 0.000001;
                    let t_range: Vec<f32> = (0..num_points)
                        .map(|x| {
                            (x as f32) / (num_points as f32 - 0.99999) / (1.0 + 2.0 * crop) + crop
                        })
                        .collect();

                    let mut mesh_contour: Vec<Vec3> = Vec::new();

                    for t in t_range {
                        let position = group.compute_position_with_lut(t) - center_of_mass;
                        let normal = group
                            .compute_normal_with_bezier(&bezier_assets, t as f64)
                            .normalize();

                        let v1 = Vec3::new(
                            (position.x + normal.x * globals.road_width) as f32,
                            (position.y + normal.y * globals.road_width) as f32,
                            globals.z_pos.road,
                        );

                        let v2 = Vec3::new(
                            (position.x - normal.x * globals.road_width) as f32,
                            (position.y - normal.y * globals.road_width) as f32,
                            globals.z_pos.road,
                        );

                        mesh_contour.push(v1);
                        mesh_contour.push(v2);
                    }

                    mesh_contour.push(mesh_contour[0]);
                    mesh_contour.push(mesh_contour[1]);

                    // indices
                    let mut new_indices: Vec<u32> = Vec::new();
                    // for kk in 0..(num_points - 1) {
                    for kk in 0..(num_points) {
                        let k = kk * 2;
                        let mut local_inds = vec![k, (k + 1), (k + 2), (k + 1), (k + 3), (k + 2)];
                        new_indices.append(&mut local_inds);
                    }

                    // uvs
                    let path_length = group.standalone_lut.path_length;
                    let num_repeats = path_length / 100.0;
                    let mut mesh_attr_uvs: Vec<[f32; 2]> = Vec::new();
                    for k in 0..(num_points + 1) * 2 {
                        // let (pos_x, pos_y) = (pos[0], pos[1]);
                        let v = k as f32 / (num_points as f32 / num_repeats);
                        mesh_attr_uvs.push([v % 1.0, (k as f32) % 2.0]);
                    }

                    let mut mesh_pos_attributes: Vec<[f32; 3]> = Vec::new();

                    // show points from look-up table
                    let color = globals.picked_color.unwrap();
                    let mut colors = Vec::new();
                    let mut normals = Vec::new();

                    let mut mins_maxes = MinsMaxes::default();

                    for position in mesh_contour {
                        mesh_pos_attributes.push([position.x, position.y, 0.0]);

                        colors.push([color.r(), color.g(), color.b(), 1.0]);
                        normals.push([0.0, 0.0, 1.0]);

                        mins_maxes.update(Vec2::new(position.x, position.y));
                    }

                    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

                    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_pos_attributes.clone());

                    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);

                    mesh.set_indices(Some(Indices::U32(new_indices)));

                    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh_attr_uvs);

                    let texture_handle = maps.textures.get("single_lane_road").unwrap();

                    let mat_handle = road_materials.add(RoadMesh2dMaterial {
                        road_texture: texture_handle.clone(),
                        center_of_mass: center_of_mass,
                        show_com: 0.0,
                    });

                    let mut road_transform = Transform::from_translation(Vec3::new(
                        center_of_mass.x,
                        center_of_mass.y,
                        globals.z_pos.road,
                    ));
                    road_transform.scale = Vec3::new(globals.scale, globals.scale, 1.0);

                    let mut rng = thread_rng();
                    let id = rng.gen::<u64>();

                    let entity = commands
                        .spawn_bundle(MaterialMesh2dBundle {
                            mesh: Mesh2dHandle(meshes.add(mesh)),
                            material: mat_handle,
                            transform: road_transform,
                            ..default()
                        })
                        .insert(PenMesh {
                            id,
                            bounding_box: mins_maxes.to_vec2_pair(),
                        })
                        .id();

                    maps.mesh_map.insert(id, entity);
                }
            }
        } else {
            info!("Select a single bezier chain to spawn a road");
        }
    }
}

// generate a fill mesh inside of the group
//
//
//
pub fn make_fill_mesh(
    mut action_event_reader: EventReader<Action>,
    mut commands: Commands,
    globals: Res<Globals>,
    curves: Res<Assets<Bezier>>,
    mut fill_materials: ResMut<Assets<FillMesh2dMaterial>>,
    selection: ResMut<Selection>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut groups: ResMut<Assets<Group>>,
    mut maps: ResMut<Maps>,
) {
    if action_event_reader.iter().any(|x| x == &Action::MakeMesh) {
        if selection.selected.iter().count() == 1 {
            let temp = selection.selected.clone();
            let selected = temp.iter().next().unwrap().clone();
            if let SelectionChoice::CurveSet(curve_set) = selected {
                //
                // check whether the curve set is part of the same group
                let mut group_id_set = HashSet::new();

                for curve in &curve_set {
                    if let Some(handle_entity) = maps.bezier_map.get(&curve) {
                        //
                        let bezier = curves.get(&handle_entity.handle).unwrap();
                        group_id_set.insert(bezier.group);
                    }
                }
                if group_id_set.iter().count() != 1 {
                    info!("cannot spawn road from curves in different groups");
                    return;
                }
                if let Some(group_handle) = maps.group_map.get(group_id_set.iter().next().unwrap())
                // unwrap never fails
                {
                    let group = groups.get_mut(&group_handle).unwrap();

                    let bezier_assets = curves
                        .iter()
                        .collect::<HashMap<bevy::asset::HandleId, &Bezier>>();

                    group.find_connected_ends(&bezier_assets, maps.bezier_map.clone());

                    group.group_lut(&bezier_assets, maps.bezier_map.clone());
                    group.compute_standalone_lut(&bezier_assets, globals.group_lut_num_points);

                    let center_of_mass = group.center_of_mass(&bezier_assets);

                    let mut path_builder = Path::builder();

                    let lut = group.standalone_lut.lut.clone();

                    let first = lut[0];
                    path_builder.begin(point(first.x, first.y));

                    let resto: Vec<Vec2> = lut[1..].to_vec();

                    for e in resto.iter() {
                        path_builder.line_to(point(e.x, e.y));
                    }

                    path_builder.end(true);
                    let path = path_builder.build();

                    // Create the destination vertex and index buffers.
                    let mut buffers: VertexBuffers<Point, u16> = VertexBuffers::new();

                    {
                        let mut vertex_builder = simple_builder(&mut buffers);

                        // Create the tessellator.
                        let mut tessellator = FillTessellator::new();

                        // Compute the tessellation.
                        let result = tessellator.tessellate_path(
                            &path,
                            &FillOptions::default(),
                            &mut vertex_builder,
                        );
                        assert!(result.is_ok());
                    }

                    let mut mesh_pos_attributes: Vec<[f32; 3]> = Vec::new();
                    let mut mesh_attr_uvs: Vec<[f32; 2]> = Vec::new();
                    let mut new_indices: Vec<u32> = Vec::new();

                    // show points from look-up table
                    let color = globals.picked_color.unwrap();
                    let mut colors = Vec::new();

                    let mut mins_maxes = MinsMaxes::default();

                    for position in buffers.vertices[..].iter() {
                        let pos_x = position.x - center_of_mass.x;
                        let pos_y = position.y - center_of_mass.y;
                        mesh_pos_attributes.push([pos_x, pos_y, 0.0]);

                        colors.push([color.r(), color.g(), color.b(), 1.0]);

                        mins_maxes.update(Vec2::new(pos_x, pos_y));
                    }

                    //////////////////////////// uvs ///////////////////////////////
                    let xs: Vec<f32> = mesh_pos_attributes.iter().map(|v| v[0]).collect();
                    let ys: Vec<f32> = mesh_pos_attributes.iter().map(|v| v[1]).collect();

                    use std::cmp::Ordering;

                    fn bounds(v: &Vec<f32>) -> (f32, f32) {
                        let max_v: &f32 = v
                            .iter()
                            .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
                            .unwrap();

                        let min_v: &f32 = v
                            .iter()
                            .min_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
                            .unwrap();

                        return (*min_v, *max_v);
                    }

                    let bounds_x = bounds(&xs);
                    let size_x = bounds_x.1 - bounds_x.0;
                    let bounds_y = bounds(&ys);
                    let size_y = bounds_y.1 - bounds_y.0;

                    let mut normals = Vec::new();
                    for pos in &mesh_pos_attributes {
                        let (pos_x, pos_y) = (pos[0], pos[1]);

                        mesh_attr_uvs.push([
                            1.0 * (pos_x - bounds_x.0) / size_x,
                            1.0 * (pos_y - bounds_y.0) / size_y,
                        ]);

                        normals.push([0.0, 0.0, 1.0]);
                    }

                    for ind in buffers.indices[..].iter().rev() {
                        new_indices.push(ind.clone() as u32);
                    }

                    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

                    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_pos_attributes.clone());
                    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
                    mesh.set_indices(Some(Indices::U32(new_indices)));
                    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh_attr_uvs);

                    let mut fill_transform =
                        Transform::from_translation(center_of_mass.extend(globals.z_pos.fill));

                    fill_transform.scale = Vec3::new(globals.scale, globals.scale, 1.0);

                    let mut rng = thread_rng();

                    // Useless at the moment, but here for future use
                    let mat_handle = fill_materials.add(FillMesh2dMaterial {
                        color: color.into(),
                        center_of_mass: center_of_mass,
                        show_com: 0.0, // show center of mass
                    });

                    let id = rng.gen::<u64>();
                    let entity = commands
                        .spawn_bundle(MaterialMesh2dBundle {
                            mesh: Mesh2dHandle(meshes.add(mesh)),
                            material: mat_handle,
                            transform: fill_transform,
                            ..default()
                        })
                        .insert(PenMesh {
                            id,
                            bounding_box: mins_maxes.to_vec2_pair(), // bounding box relative to center of mass
                        })
                        .id();

                    maps.mesh_map.insert(id, entity);
                }
            } else {
                info!("Select a single bezier chain to spawn a fill mesh");
            }
        }
    }
}
