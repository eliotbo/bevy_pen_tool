use bevy_pen_tool_model::inputs::Action;
use bevy_pen_tool_model::materials::*;

use bevy_pen_tool_model::model::*;
use bevy_pen_tool_model::spawn_bezier;

use bevy::prelude::*;

use std::collections::HashSet;

use std::fs::File;
use std::io::Read;
use std::io::Write;

pub fn save(
    mut bezier_curves: ResMut<Assets<Bezier>>,
    group_query: Query<&Handle<Group>, With<GroupParent>>,
    // // mesh_query: Query<(&Handle<Mesh>, &GroupMesh)>,
    // road_mesh_query: Query<(&Handle<Mesh>, &RoadMesh)>,
    mut groups: ResMut<Assets<Group>>,
    // meshes: Res<Assets<Mesh>>,
    globals: ResMut<Globals>,
    mut action_event_reader: EventReader<Action>,
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

        ////////////// start. Save Group and save Group look-up table
        if let Some(group_handle) = group_query.iter().next() {
            let mut group_vec = Vec::new();
            // for group_handle in group_query.iter() {
            let group = groups.get_mut(group_handle).unwrap();
            //
            ////////////// start. Save Group look-up table
            let lut_dialog_result = open_file_dialog("my_group", "look_up_tables", ".lut");
            if let Some(lut_path) = lut_dialog_result {
                group.compute_standalone_lut(&mut bezier_curves, globals.group_lut_num_points);
                let lut_serialized = serde_json::to_string_pretty(&group.standalone_lut).unwrap();
                // let lut_path = "assets/lut/my_group_lut.txt";
                let mut lut_output = File::create(&lut_path).unwrap();
                let _lut_write_result = lut_output.write(lut_serialized.as_bytes());
            }

            ////////////// start. Save Group
            let group_dialog_result = open_file_dialog("my_group", "groups", ".group");
            if let Some(group_path) = group_dialog_result {
                group_vec.push(group.into_group_save(&mut bezier_curves).clone());
                // }

                let serialized = serde_json::to_string_pretty(&group_vec).unwrap();

                // let path = "curve_groups.txt";
                let mut output = File::create(group_path).unwrap();
                let _group_write_result = output.write(serialized.as_bytes());
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
    mut mid_params: ResMut<Assets<BezierMidMat>>,
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
            group_id: id,
        };

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
                    &mut mid_params,
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
            }
        }
        selection.selected = vec![SelectionChoice::Group(group.clone())];

        // to create a group: select all the curves programmatically, and send a UiButton::Group event
        loaded_event_writer.send(Loaded);
        println!("{:?}", "loaded groups");
    }
}
