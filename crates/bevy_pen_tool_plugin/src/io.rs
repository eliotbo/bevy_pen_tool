use bevy_pen_tool_model::inputs::Action;
use bevy_pen_tool_model::materials::*;

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use bevy_pen_tool_model::mesh::*;
use bevy_pen_tool_model::model::*;
use bevy_pen_tool_model::spawn_bezier;

use serde::Deserialize;
use serde::Serialize;

use std::collections::HashMap;
use std::collections::HashSet;

use std::fs::File;
use std::io::Read;
use std::io::Write;

use rand::{thread_rng, Rng};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct MeshMeta {
    center_of_mass: Vec2,
    position: Vec2,
    bounding_box: (Vec2, Vec2),
    color: Vec4,
}

pub fn save(
    bezier_curves: Res<Assets<Bezier>>,
    // group_query: Query<&Handle<Group>, With<GroupParent>>,
    selection: Res<Selection>,
    mesh_query: Query<(&Mesh2dHandle, &Handle<FillMesh2dMaterial>)>,
    fill_mats: Res<Assets<FillMesh2dMaterial>>,
    // road_mesh_query: Query<(&Handle<Mesh>, &RoadMesh)>,
    mut groups: ResMut<Assets<Group>>,
    meshes: Res<Assets<Mesh>>,
    globals: ResMut<Globals>,
    mut action_event_reader: EventReader<Action>,
    maps: Res<Maps>,
) {
    if action_event_reader.iter().any(|x| x == &Action::Save) {
        //

        //
        // ////////////// start.  Save individual Bezier curves
        // let mut vec: Vec<Bezier> = Vec::new();
        // for bezier_handle in query.iter() {
        //     let bezier = bezier_curves.get(bezier_handle).unwrap();
        //     let mut bezier_clone = bezier.clone();
        //     bezier_clone.lut = Vec::new();
        //     vec.push(bezier_clone);
        // }

        // let serialized = serde_json::to_string_pretty(&vec).unwrap();

        // let path = "curves.txt";
        // let mut output = File::create(path).unwrap();
        // let _result = output.write(serialized.as_bytes());
        // ////////////// end.  Save individual Bezier curves
        //

        for selected in selection.selected.iter() {
            match selected {
                SelectionChoice::CurveSet(bezier_ids) => {
                    ////////////// start. Save Group and save Group look-up table

                    // collect all the different groups
                    let mut group_ids: HashSet<GroupId> = HashSet::new();
                    for bezier_id in bezier_ids.iter() {
                        let handle = maps.bezier_map.get(bezier_id).unwrap();
                        let bezier = bezier_curves.get(&handle.handle).unwrap();
                        group_ids.insert(bezier.group);
                    }

                    for group_id in group_ids.iter() {
                        let group_handle = maps.group_map.get(group_id).unwrap();
                        let group = groups.get_mut(&group_handle).unwrap();
                        let mut group_vec = Vec::new();

                        // for group_handle in group_query.iter() {

                        // let group = groups.get_mut(group_handle).unwrap();
                        //
                        ////////////// start. Save Group look-up table
                        let lut_dialog_result =
                            open_file_dialog("my_group", "look_up_tables", ".lut");
                        if let Some(lut_path) = lut_dialog_result {
                            let bezier_assets =
                                bezier_curves
                                    .iter()
                                    .collect::<HashMap<bevy::asset::HandleId, &Bezier>>();

                            group.compute_standalone_lut(
                                &bezier_assets,
                                globals.group_lut_num_points,
                            );
                            let lut_serialized =
                                serde_json::to_string_pretty(&group.standalone_lut).unwrap();
                            // let lut_path = "assets/lut/my_group_lut.txt";
                            let mut lut_output = File::create(&lut_path).unwrap();
                            let _lut_write_result = lut_output.write(lut_serialized.as_bytes());
                        }

                        ////////////// start. Save Group
                        let group_dialog_result = open_file_dialog("my_group", "groups", ".group");
                        if let Some(group_path) = group_dialog_result {
                            group_vec.push(group.into_group_save(&bezier_curves).clone());
                            // }

                            let serialized = serde_json::to_string_pretty(&group_vec).unwrap();

                            // let path = "curve_groups.txt";
                            let mut output = File::create(group_path).unwrap();
                            let _group_write_result = output.write(serialized.as_bytes());
                        }
                    }
                }
                SelectionChoice::Mesh(PenMesh { id, bounding_box }, position) => {
                    //
                    let mesh_entity = maps.mesh_map.get(id).unwrap();
                    let (mesh_handle, fill_material_handle) = mesh_query.get(*mesh_entity).unwrap();
                    let fill_mat = fill_mats.get(fill_material_handle).unwrap();

                    let mut default_path = std::env::current_dir().unwrap();
                    default_path.push("assets");
                    // default_path.push("meshes");

                    println!("saving messhh");

                    if let Some(path) = open_file_dialog("my_mesh", "meshes", ".obj") {
                        save_mesh(&mesh_handle.0, &meshes, path.clone());

                        let mesh_info = MeshMeta {
                            center_of_mass: fill_mat.center_of_mass,
                            position: *position,
                            bounding_box: *bounding_box,
                            color: fill_mat.color,
                        };

                        let serialized = serde_json::to_string_pretty(&mesh_info).unwrap();
                        let position_file_path = path.with_extension("meta");
                        let mut output = File::create(position_file_path).unwrap();
                        let _group_write_result = output.write(serialized.as_bytes());
                    }
                }
                _ => (),
            }
        }
        ////////////// end. Save group and look-up table
        //

        // ////////////// start. Save mesh in obj format
        // if let Some((mesh_handle, GroupMesh(_color))) = mesh_query.iter().next() {
        //     let mesh_dialog_result = open_file_dialog("my_mesh", "meshes", ".obj");
        //     save_mesh(mesh_handle, &meshes, mesh_dialog_result);

        //     ////////////// end. Save mesh in obj format
        // }

        // ////////////// start. Save road in obj format
        // if let Some((road_mesh_handle, RoadMesh(_color))) = road_mesh_query.iter().next() {
        //     let road_dialog_result = open_file_dialog("my_road", "meshes", ".obj");
        //     save_mesh(road_mesh_handle, &meshes, road_dialog_result);

        //     ////////////// end. Save road in obj format
        // }
    }
}

// only loads groups
pub fn load_mesh(
    mut commands: Commands,
    // mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    // globals: Res<Globals>,
    mut fill_materials: ResMut<Assets<FillMesh2dMaterial>>,
    mut maps: ResMut<Maps>,
    mut action_event_reader: EventReader<Action>,
) {
    if action_event_reader.iter().any(|x| x == &Action::Load) {
        let mut default_path = std::env::current_dir().unwrap();
        default_path.push("assets");
        default_path.push("meshes");

        if let Some(res) = rfd::FileDialog::new()
            .add_filter("text", &["obj"])
            .set_directory(&default_path)
            .pick_files()
        {
            if let Some(path) = res.get(0) {
                let mesh_handle: Handle<Mesh> = asset_server.load(path.to_str().unwrap());
                let mut rng = thread_rng();
                let id = rng.gen::<u64>();

                // get mesh info using the .meta extension
                let meta_path = path.with_extension("meta");
                let mut file = std::fs::File::open(meta_path).unwrap();
                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();
                let loaded_mesh_params: MeshMeta = serde_json::from_str(&contents).unwrap();

                // Useless at the moment, but here for future use
                let mat_handle = fill_materials.add(FillMesh2dMaterial {
                    color: loaded_mesh_params.color.into(),
                    center_of_mass: loaded_mesh_params.center_of_mass, // center_of_mass, // is this Ok?
                    show_com: 0.0,                                     // show center of mass
                });

                let entity = commands
                    .spawn_bundle(MaterialMesh2dBundle {
                        mesh: Mesh2dHandle(mesh_handle),
                        material: mat_handle,
                        transform: Transform::default(),
                        ..default()
                    })
                    .insert(PenMesh {
                        id,
                        bounding_box: loaded_mesh_params.bounding_box, // bounding box relative to center of mass
                    })
                    .id();

                maps.mesh_map.insert(id, entity);

                // meshes.add(mesh_handle);
            }
            // let mesh_handle = asset_server.load("example.obj");
        }
    }
}

pub fn load(
    query: Query<Entity, Or<(With<BezierParent>, With<GroupParent>)>>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    // mut groups: ResMut<Assets<Group>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    // mut my_shader_params: ResMut<Assets<BezierMat>>,
    clearcolor_struct: Res<ClearColor>,
    mut globals: ResMut<Globals>,
    mut selection: ResMut<Selection>,
    mut maps: ResMut<Maps>,
    mut action_event_reader: EventReader<Action>,
    mut loaded_event_writer: EventWriter<Loaded>,
    mut selection_params: ResMut<Assets<SelectionMat>>,
    mut controls_params: ResMut<Assets<BezierControlsMat>>,
    mut ends_params: ResMut<Assets<BezierEndsMat>>,
    // mut mid_params: ResMut<Assets<BezierMidMat>>,
    mut add_to_history_event_writer: EventWriter<HistoryAction>,
) {
    if action_event_reader.iter().any(|x| x == &Action::Load) {
        let mut default_path = std::env::current_dir().unwrap();
        default_path.push("saved");
        default_path.push("groups");

        let res = rfd::FileDialog::new()
            .add_filter("text", &["group"])
            .set_directory(&default_path)
            .pick_files();

        // cancel loading if user cancelled the file dialog
        let path: std::path::PathBuf;
        if let Some(chosen_path) = res.clone() {
            let path_some = chosen_path.get(0);
            if let Some(path_local) = path_some {
                path = path_local.clone();
            } else {
                return ();
            }
        } else {
            return ();
        }

        let clearcolor = clearcolor_struct.0;

        // delete all current groups and curves before spawning the saved ones
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        globals.do_hide_anchors = false;
        globals.do_hide_bounding_boxes = true;

        let mut file = std::fs::File::open(path).unwrap();

        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        let loaded_groups_vec: Vec<GroupSaveLoad> = serde_json::from_str(&contents).unwrap();

        let id: GroupId = GroupId::default();

        let mut group = Group {
            group: HashSet::new(),
            bezier_handles: HashSet::new(),
            lut: Vec::new(),
            ends: None,
            standalone_lut: StandaloneLut {
                path_length: 0.0,
                lut: Vec::new(),
            },
            id,
            entity: None,
        };

        let mut curve_set: HashSet<BezierId> = HashSet::new();

        for group_load_save in loaded_groups_vec {
            for (mut bezier, anchor, t_ends, local_lut) in group_load_save.lut {
                let (entity, handle) = spawn_bezier(
                    &mut bezier,
                    &mut bezier_curves,
                    &mut commands,
                    &mut meshes,
                    &mut selection_params,
                    &mut controls_params,
                    &mut ends_params,
                    // &mut mid_params,
                    clearcolor,
                    &mut globals,
                    &mut maps,
                    &mut add_to_history_event_writer,
                    &None, // does not have handle information
                    true,  // do send to history
                    false, // do not follow mouse
                );
                group.group.insert((entity.clone(), handle.clone()));
                group.bezier_handles.insert(handle.clone());
                group.standalone_lut = group_load_save.standalone_lut.clone();
                group.lut.push((handle.clone(), anchor, t_ends, local_lut));

                curve_set.insert(handle.id.into());
            }
        }
        selection.selected = vec![SelectionChoice::CurveSet(curve_set)];

        // to create a group: select all the curves programmatically, and send a UiButton::Group event
        loaded_event_writer.send(Loaded(group));
        println!("{:?}", "loaded groups");
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

pub fn save_mesh(mesh_handle: &Handle<Mesh>, meshes: &Res<Assets<Mesh>>, path: PathBuf) {
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
            // let olen = obj_vertices.len();
            let olen = 1;

            // let tvert =

            let set = obj_exporter::ObjSet {
                material_library: None,
                objects: vec![obj_exporter::Object {
                    name: "My_mesh".to_owned(),
                    vertices: obj_vertices,
                    tex_vertices: vec![(0.0, 0.0); olen]
                        .into_iter()
                        .map(|(u, v)| obj_exporter::TVertex { u, v, w: 0.0 })
                        .collect(),
                    normals,
                    geometry: vec![obj_exporter::Geometry {
                        material_name: None,
                        shapes: obj_inds_vecs
                            .into_iter()
                            .map(|(x, y, z)| obj_exporter::Shape {
                                primitive: obj_exporter::Primitive::Triangle(
                                    (x, None, None),
                                    (y, None, None),
                                    (z, None, None),
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

// use obj::{Geometry, ObjSet, Object, Primitive, Shape, TVertex, Vertex};

// pub fn example_obj() {
//     let set = ObjSet {
//         material_library: None,
//         objects: vec![Object {
//             name: "Square".to_owned(),
//             vertices: vec![
//                 (-1.0, -1.0, 0.0),
//                 (1.0, -1.0, 0.0),
//                 (1.0, 1.0, 0.0),
//                 (-1.0, 1.0, 0.0),
//             ]
//             .into_iter()
//             .map(|(x, y, z)| Vertex { x, y, z })
//             .collect(),
//             tex_vertices: vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]
//                 .into_iter()
//                 .map(|(u, v)| TVertex { u, v, w: 0.0 })
//                 .collect(),
//             normals: vec![Vertex {
//                 x: 0.0,
//                 y: 0.0,
//                 z: -1.0,
//             }],
//             geometry: vec![Geometry {
//                 material_name: None,
//                 shapes: vec![(0, 1, 2), (0, 2, 3)]
//                     .into_iter()
//                     .map(|(x, y, z)| Shape {
//                         primitive: Primitive::Triangle(
//                             (x, Some(x), Some(0)),
//                             (y, Some(y), Some(0)),
//                             (z, Some(z), Some(0)),
//                         ),
//                         groups: vec![],
//                         smoothing_groups: vec![],
//                     })
//                     .collect(),
//             }],
//         }],
//     };

//     obj::export_to_file(&set, "output_single.obj").unwrap();
// }
