use super::buttons::{ButtonState, UiButton};
use super::inputs::Cursor;
use crate::spawner::spawn_bezier;
use crate::GroupMiddleQuad;

use crate::util::{
    get_close_anchor, get_close_anchor_entity, get_close_still_anchor, Anchor, AnchorEdge, Bezier,
    BoundingBoxQuad, ColorButton, ControlPointQuad, EndpointQuad, Globals, GrandParent, Group,
    GroupBoxQuad, GroupSaveLoad, LatchData, MiddlePointQuad, MyShader, OfficialLatch,
    SelectionBoxQuad, SoundStruct, UiAction, UiBoard,
};

// use crate::util::*;

use bevy::{
    input::mouse::MouseWheel,
    prelude::*,
    // render::camera::OrthographicProjection
};

// use rand::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;
// use std::ops::DerefMut;

use std::fs::File;
use std::io::Read;
use std::io::Write;

pub fn pick_color(
    cursor: ResMut<Cursor>,
    mut my_shader_params: ResMut<Assets<MyShader>>,
    mouse_button_input: Res<Input<MouseButton>>,
    query: Query<(&GlobalTransform, &Handle<MyShader>, &ColorButton)>,
    mut ui_query: Query<(&Transform, &mut UiBoard), With<GrandParent>>,
    mut globals: ResMut<Globals>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        //
        let mut pressed_button = (false, 0);
        //
        for (ui_transform, mut ui_board) in ui_query.iter_mut() {
            // TODO: fix scales
            let cam_scale = globals.scale * globals.scale;
            for (k, (transform, shader_param_handle, _color_button)) in query.iter().enumerate() {
                let shader_params = my_shader_params.get(shader_param_handle).unwrap().clone();
                // println!("{:?}", cam_scale);
                if cursor.within_rect(
                    transform.translation.truncate(),
                    shader_params.size * 1.15 * cam_scale,
                ) {
                    pressed_button = (true, k);

                    globals.picked_color = Some(shader_params.color);

                    // println!("chose color: {:?}", globals.picked_color);

                    ui_board.action = UiAction::PickingColor;

                    break;
                }
            }

            // send selected color to shaders so that it shows the selected color with a white contour
            if pressed_button.0 {
                //
                for (k, (_transform, shader_param_handle, _color_button)) in
                    query.iter().enumerate()
                {
                    //
                    let mut shader_params = my_shader_params.get_mut(shader_param_handle).unwrap();
                    //
                    if pressed_button.1 == k {
                        shader_params.t = 1.0;
                    } else {
                        shader_params.t = 0.0;
                    }
                }
            }

            if ui_board.action == UiAction::None
                && cursor.within_rect(
                    ui_transform.translation.truncate(),
                    ui_board.size * globals.scale,
                )
            {
                ui_board.action = UiAction::MovingUi;
            }
        }
    }
    // else if mouse_button_input.just_released(MouseButton::Left) {
    //     for (_ui_transform, mut ui_board) in ui_query.iter_mut() {
    //         ui_board.action = UiAction::None;
    //     }
    // }
}

pub fn begin_move_on_mouseclick(
    keyboard_input: Res<Input<KeyCode>>,
    mut cursor: ResMut<Cursor>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    mouse_button_input: Res<Input<MouseButton>>,
    query: Query<&Handle<Bezier>, With<BoundingBoxQuad>>,
    // ui_query: Query<&UiBoard>,
    globals: ResMut<Globals>,
    // mut my_shader_params: ResMut<Assets<MyShader>>,
    button_query: Query<(&ButtonState, &UiButton, &Handle<MyShader>)>,
    audio: Res<Audio>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) && !globals.do_spawn_curve {
        let mut latch_partners: Vec<LatchData> = Vec::new();

        if let Some((_distance, anchor, handle)) = get_close_anchor(
            3.0 * globals.scale,
            cursor.position,
            &bezier_curves,
            &query,
            globals.scale,
        ) {
            let mut bezier = bezier_curves.get_mut(handle.clone()).unwrap();

            // order to move the quad that was clicked on
            if anchor != Anchor::None {
                bezier.move_quad = anchor;
                bezier.do_compute_lut = true;
                bezier.update_previous_pos();

                // retrieve the detach (unlatch) button state
                let mut detach_button_on = false;
                for (button_state, ui_button, _my_shader_handle) in button_query.iter() {
                    if ui_button == &UiButton::Detach {
                        detach_button_on = button_state == &ButtonState::On;
                    }
                }
                // order to unlatch the anchor if the user presses Space
                if !keyboard_input.pressed(KeyCode::LShift)
                    && !keyboard_input.pressed(KeyCode::LControl)
                    && (keyboard_input.pressed(KeyCode::Space) || detach_button_on)
                {
                    match anchor {
                        Anchor::Start => {
                            // keep the latch information in memory to unlatch the anchor's partner below
                            latch_partners = bezier.latches[&AnchorEdge::Start].clone();
                            if let Some(latch) = bezier.latches.get_mut(&AnchorEdge::Start) {
                                *latch = Vec::new();
                            }
                        }
                        Anchor::End => {
                            latch_partners = bezier.latches[&AnchorEdge::End].clone();
                            // bezier.latches[&AnchorEdge::End] = Vec::new();
                            if let Some(latch) = bezier.latches.get_mut(&AnchorEdge::End) {
                                *latch = Vec::new();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // unlatch partner
        if let Some(latch) = latch_partners.get(0) {
            //
            if let Some(handle) = globals.id_handle_map.get(&latch.latched_to_id) {
                //
                let bezier = bezier_curves.get_mut(handle).unwrap();
                //
                if let Some(latch_local) = bezier.latches.get_mut(&latch.partners_edge) {
                    *latch_local = Vec::new();
                    if globals.sound_on {
                        if let Some(sound) = globals.sounds.get("unlatch") {
                            audio.play(sound.clone());
                        }
                    }
                }
            }
        }
    }

    // let go of all any moving quad upon mouse button release
    if mouse_button_input.just_released(MouseButton::Left) {
        //
        for bezier_handle in query.iter() {
            //
            if let Some(bezier) = bezier_curves.get_mut(bezier_handle) {
                //
                cursor.latch = Vec::new();
                bezier.move_quad = Anchor::None;
                // TODO: set the "just_created" field to false elsewhere once the right schedule
                // of systems is in place
                bezier.just_created = false;
            }
        }
    }
}

pub fn selection(
    mut globals: ResMut<Globals>,
    cursor: ResMut<Cursor>,
    mouse_button_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
    bezier_curves: ResMut<Assets<Bezier>>,
    groups: ResMut<Assets<Group>>,
    mut visible_selection_query: Query<&mut Visible, With<SelectionBoxQuad>>,
    group_query: Query<&Handle<Group>>,
    query: Query<(Entity, &Handle<Bezier>), With<BoundingBoxQuad>>,
    button_query: Query<(&ButtonState, &UiButton)>,
    ui_query: Query<&UiBoard>,
) {
    // avoids selection if the click is on the UI
    let mut clicked_on_ui = false;
    for ui_board in ui_query.iter() {
        if ui_board.action != UiAction::None {
            clicked_on_ui = true;
            break;
        }
    }

    if mouse_button_input.just_pressed(MouseButton::Left) && !clicked_on_ui {
        //
        let mut selection_button_on = false;
        for (button_state, ui_button) in button_query.iter() {
            if ui_button == &UiButton::Selection {
                selection_button_on = button_state == &ButtonState::On;
            }
        }

        if !globals.do_spawn_curve
            && !keyboard_input.pressed(KeyCode::LShift)
            && !keyboard_input.pressed(KeyCode::Space)
            && (keyboard_input.pressed(KeyCode::LControl) || selection_button_on)
        {
            if let Some((_distance, _anchor, entity, selected_handle)) = get_close_anchor_entity(
                2.0 * globals.scale,
                cursor.position,
                &bezier_curves,
                &query,
                globals.scale,
            ) {
                // if the selected quad is part of a group, show group selection
                for group_handle in group_query.iter() {
                    let group = groups.get(group_handle).unwrap();
                    //
                    if group.handles.contains(&selected_handle) {
                        globals.selected = group.clone();
                        for mut visible in visible_selection_query.iter_mut() {
                            visible.is_visible = true;
                        }

                        return ();
                    }
                }

                let selected_entity = entity.clone();

                // add the selected quad to the selected group
                globals
                    .selected
                    .group
                    .insert((selected_entity.clone(), selected_handle.clone()));

                globals.selected.handles.insert(selected_handle.clone());

                // these will be computed when a group order has been emitted
                globals.selected.ends = None;
                globals.selected.lut = Vec::new();

                for mut visible in visible_selection_query.iter_mut() {
                    visible.is_visible = true;
                }
                // println!("selectd: {:?}", &globals.selected);
            }
        } else if !globals.do_spawn_curve
            && !keyboard_input.pressed(KeyCode::LControl)
            && !keyboard_input.pressed(KeyCode::LShift)
            && !keyboard_input.pressed(KeyCode::Space)
        {
            globals.selected.group = HashSet::new();
            globals.selected.handles = HashSet::new();
            globals.selected.ends = None;
            globals.selected.lut = Vec::new();

            for mut visible in visible_selection_query.iter_mut() {
                visible.is_visible = false;
            }
        }
    }
}

pub fn groupy(
    mut commands: Commands,
    mut groups: ResMut<Assets<Group>>,
    globals: ResMut<Globals>,
    keyboard_input: Res<Input<KeyCode>>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    query: Query<(Entity, &Handle<Bezier>), With<MiddlePointQuad>>,
    group_query: Query<(Entity, &Handle<Group>), Or<(With<GroupBoxQuad>, With<GroupMiddleQuad>)>>,
    mut ui_event_reader: EventReader<UiButton>,
    mut group_event_reader: EventReader<Group>,
    mut event_writer: EventWriter<Handle<Group>>,
    audio: Res<Audio>,
    // mut query: Query<(Entity, &Handle<Bezier>), With<BoundingBoxQuad>>,
) {
    let mut pressed_group_button = false;
    for ui_button in ui_event_reader.iter() {
        pressed_group_button = ui_button == &UiButton::Group;
        break;
    }

    // TODO: replace globals.selected by the value passed in the event
    for _group in group_event_reader.iter() {
        pressed_group_button = true;
    }

    if !globals.selected.group.is_empty()
        && (pressed_group_button
            || (keyboard_input.pressed(KeyCode::LControl)
                && keyboard_input.just_pressed(KeyCode::G)
                && !keyboard_input.pressed(KeyCode::LShift)
                && !keyboard_input.pressed(KeyCode::Space)))
    {
        let id_handle_map: HashMap<u128, Handle<Bezier>> = globals.id_handle_map.clone();

        let mut selected = globals.selected.clone();

        selected.find_connected_ends(&mut bezier_curves, id_handle_map.clone());
        // println!("connected ends: {:?}, ", selected.ends);

        // abort grouping if the selection is not completely connected with latches
        if selected.ends.is_none() {
            println!("Cannot group. Select multiple latched curves to successfully group");
            return;
        }

        if globals.sound_on {
            if let Some(sound) = globals.sounds.get("group") {
                audio.play(sound.clone());
            }
        }

        selected.group_lut(&mut bezier_curves, id_handle_map.clone());
        selected.compute_standalone_lut(&bezier_curves, globals.group_lut_num_points);

        // get rid of the middle point quads
        for (entity, bezier_handle) in query.iter() {
            if selected.handles.contains(bezier_handle) {
                commands.entity(entity).despawn();
            }
        }

        // get rid of the current group before making a new one
        for (entity, group_handle) in group_query.iter() {
            let group = groups.get(group_handle).unwrap();
            for bezier_handle in group.handles.clone() {
                if selected.handles.contains(&bezier_handle) {
                    commands.entity(entity).despawn();
                    break;
                }
            }
        }

        let group_handle = groups.add(selected);
        // spawn the middle quads and the bounding box
        event_writer.send(group_handle.clone());
    }
}

// pub fn ungroup(
//     mut commands: Commands,
//     mut groups: ResMut<Assets<Group>>,
//     globals: ResMut<Globals>,
//     keyboard_input: Res<Input<KeyCode>>,
//     mut bezier_curves: ResMut<Assets<Bezier>>,
//     query: Query<(Entity, &Handle<Bezier>), With<MiddlePointQuad>>,
//     mut ui_event_reader: EventReader<UiButton>,
//     mut group_event_reader: EventReader<Group>,
//     mut event_writer: EventWriter<Handle<Group>>,
//     audio: Res<Audio>,
// ) {
//     if !globals.selected.group.is_empty()
//         && (keyboard_input.pressed(KeyCode::LControl)
//             && keyboard_input.just_pressed(KeyCode::G)
//             && keyboard_input.pressed(KeyCode::LShift)
//             && !keyboard_input.pressed(KeyCode::Space))
//     {
//         let mut selected = globals.selected.clone();
//         for bezier_handle in selected.handles {
//             for (idx, group) in groups.clone().iter().enumerate() {
//                 for handle_in_group in group.handles {

//                 }
//             }
//         }
//     }
// }

pub fn delete(
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut globals: ResMut<Globals>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    mut groups: ResMut<Assets<Group>>,
    mut visible_query: Query<&mut Visible, With<SelectionBoxQuad>>,
    query: Query<(Entity, &Handle<Bezier>), With<BoundingBoxQuad>>,
    query2: Query<(Entity, &Handle<Group>), With<GroupBoxQuad>>,
) {
    if keyboard_input.pressed(KeyCode::Delete) {
        // println!("{:?}", globals.selected.clone());

        // list of partners that need to be unlatched
        let mut latched_partners: Vec<Vec<LatchData>> = Vec::new();
        for (entity, bezier_handle) in query.iter() {
            //
            for (_entity_selected, handle) in globals.selected.group.clone() {
                //
                let bezier = bezier_curves.get_mut(handle.clone()).unwrap();

                latched_partners.push(bezier.latches[&AnchorEdge::Start].clone());
                latched_partners.push(bezier.latches[&AnchorEdge::End].clone());

                if &handle == bezier_handle {
                    commands.entity(entity).despawn_recursive();
                }
            }
        }

        for (entity, group_handle) in query2.iter() {
            //
            let group = groups.get(group_handle).unwrap();
            for (_entity_selected, bezier_handle) in globals.selected.group.clone() {
                if group.handles.contains(&bezier_handle) {
                    commands.entity(entity).despawn_recursive();
                }
            }
        }

        // unlatch partners of deleted curves
        for latch_vec in latched_partners {
            //
            if let Some(latch) = latch_vec.get(0) {
                //
                if let Some(handle) = globals.id_handle_map.get(&latch.latched_to_id) {
                    //
                    let bezier = bezier_curves.get_mut(handle).unwrap();

                    if let Some(latch_local) = bezier.latches.get_mut(&latch.partners_edge) {
                        // println!("selectd: {:?}", &latch_local);
                        *latch_local = Vec::new();
                        println!("deleted partner latches");
                    }
                }
            }
        }

        // make the group box quad invisible
        for mut visible in visible_query.iter_mut() {
            visible.is_visible = false;
        }

        // reset selection
        globals.selected.group = HashSet::new();
        globals.selected.handles = HashSet::new();
    }
}

pub fn undo(
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut globals: ResMut<Globals>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    mut event_reader: EventReader<UiButton>,
    query: Query<(Entity, &Handle<Bezier>), With<BoundingBoxQuad>>,
) {
    let mut pressed_undo_button = false;
    for ui_button in event_reader.iter() {
        pressed_undo_button = ui_button == &UiButton::Undo;
        break;
    }

    if pressed_undo_button
        || (keyboard_input.pressed(KeyCode::LControl)
            && keyboard_input.just_pressed(KeyCode::Z)
            && !keyboard_input.pressed(KeyCode::LShift))
    {
        // let mut latched_start: Vec<LatchData> = Vec::new();
        // let mut latched_end: Vec<LatchData> = Vec::new();
        let mut latches: Vec<Vec<LatchData>> = Vec::new();

        if let Some((entity, bezier_handle)) = query.iter().last() {
            globals.history.push(bezier_handle.clone());

            let bezier = bezier_curves.get(bezier_handle).unwrap();
            latches.push(bezier.latches[&AnchorEdge::Start].clone());
            latches.push(bezier.latches[&AnchorEdge::End].clone());

            commands.entity(entity).despawn_recursive();
        }

        // This piece of code is shared with delete()
        // unlatch partners of deleted curves
        for latch_vec in latches {
            //
            if let Some(latch) = latch_vec.get(0) {
                //
                if let Some(handle) = globals.id_handle_map.get(&latch.latched_to_id) {
                    //
                    let bezier_partner = bezier_curves.get_mut(handle).unwrap();
                    //
                    if let Some(latch_local) = bezier_partner.latches.get_mut(&latch.partners_edge)
                    {
                        *latch_local = Vec::new();
                        println!("deleted partner's latch {:?}", latch.partners_edge);
                    }
                }
            }
        }
    }
}

// Warning: undo followed by redo does not preserve the latch data
// spawn_bezier() does not allow the end point to be latched
pub fn redo(
    keyboard_input: Res<Input<KeyCode>>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    // mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut my_shader_params: ResMut<Assets<MyShader>>,
    clearcolor_struct: Res<ClearColor>,
    mut globals: ResMut<Globals>,
    mut event_reader: EventReader<UiButton>,
) {
    let mut pressed_redo_button = false;
    for ui_button in event_reader.iter() {
        pressed_redo_button = ui_button == &UiButton::Redo;
        break;
    }

    if pressed_redo_button
        || (keyboard_input.pressed(KeyCode::LControl)
            && keyboard_input.just_pressed(KeyCode::Z)
            && keyboard_input.pressed(KeyCode::LShift))
    {
        let clearcolor = clearcolor_struct.0;
        let length = globals.history.len();
        let mut should_remove_last_from_history = false;
        if let Some(bezier_handle) = globals.history.last() {
            should_remove_last_from_history = true;
            let mut bezier = bezier_curves.get_mut(bezier_handle).unwrap().clone();
            bezier_curves.remove(bezier_handle);
            globals.do_spawn_curve = false;
            // println!("{:?}", bezier.color);

            spawn_bezier(
                &mut bezier,
                &mut bezier_curves,
                &mut commands,
                &mut meshes,
                // &mut pipelines,
                &mut my_shader_params,
                clearcolor,
                &mut globals,
            );
        }

        if should_remove_last_from_history {
            globals.history.swap_remove(length - 1);
        }
    }
}

pub fn hide_anchors(
    keyboard_input: Res<Input<KeyCode>>,
    mut globals: ResMut<Globals>,
    mut query: Query<&mut Visible, Or<(With<ControlPointQuad>, With<EndpointQuad>)>>,
    mut event_reader: EventReader<UiButton>,
) {
    let mut pressed_hide_button = false;
    for ui_button in event_reader.iter() {
        pressed_hide_button = ui_button == &UiButton::Hide;
        break;
    }
    if keyboard_input.just_pressed(KeyCode::H) || pressed_hide_button {
        globals.do_hide_anchors = !globals.do_hide_anchors;
        for mut visible in query.iter_mut() {
            visible.is_visible = !globals.do_hide_anchors;
        }
    }
}

pub fn toggle_sound(
    // asset_server: Res<AssetServer>,
    mut globals: ResMut<Globals>,
    // mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<(&mut Handle<ColorMaterial>, &mut SoundStruct)>,
    mut event_reader: EventReader<UiButton>,
) {
    for ui_button in event_reader.iter() {
        //
        if ui_button == &UiButton::Sound {
            //
            globals.sound_on = !globals.sound_on;
            //
            for (mut material_handle, mut soundstruct) in query.iter_mut() {
                // toggle sprite
                use std::ops::DerefMut;
                let other_material = soundstruct.material.clone();
                let current_material = material_handle.clone();
                let mat = material_handle.deref_mut();
                *mat = other_material.clone();
                soundstruct.material = current_material;
            }
        }
    }
}

pub fn save(
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<&Handle<Bezier>, With<BoundingBoxQuad>>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    group_query: Query<&Handle<Group>, With<GroupBoxQuad>>,
    mut groups: ResMut<Assets<Group>>,
    mut event_reader: EventReader<UiButton>,
    // mut event_writer: EventWriter<Handle<Group>>,
    // mut query: Query<(Entity, &Handle<Bezier>), With<BoundingBoxQuad>>,
) {
    let mut pressed_save_button = false;
    for ui_button in event_reader.iter() {
        pressed_save_button = ui_button == &UiButton::Save;
        break;
    }

    if pressed_save_button
        || (keyboard_input.pressed(KeyCode::LControl)
            && keyboard_input.just_released(KeyCode::S)
            && !keyboard_input.pressed(KeyCode::LShift))
    {
        let mut vec: Vec<Bezier> = Vec::new();
        for bezier_handle in query.iter() {
            let bezier = bezier_curves.get(bezier_handle).unwrap();
            let mut bezier_clone = bezier.clone();
            bezier_clone.lut = Vec::new();
            vec.push(bezier_clone);
        }

        let serialized = serde_json::to_string_pretty(&vec).unwrap();

        let path = "curves.txt";
        let mut output = File::create(path).unwrap();
        let _result = output.write(serialized.as_bytes());

        println!("{:?}", "saved individual Bezier curves");

        let mut group_vec = Vec::new();
        for group_handle in group_query.iter() {
            let group = groups.get_mut(group_handle).unwrap();

            group.compute_standalone_lut(&mut bezier_curves, 100);
            let lut_serialized = serde_json::to_string_pretty(&group.standalone_lut).unwrap();
            let lut_path = "group_lut.txt";
            let mut lut_output = File::create(lut_path).unwrap();
            let _lut_result = lut_output.write(lut_serialized.as_bytes());

            group_vec.push(group.into_group_save(&mut bezier_curves).clone());
        }

        let serialized = serde_json::to_string_pretty(&group_vec).unwrap();

        let path = "curve_groups.txt";
        let mut output = File::create(path).unwrap();
        let _result = output.write(serialized.as_bytes());

        println!("{:?}", "saved groups");
    }
}

pub fn load(
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<Entity, Or<(With<BoundingBoxQuad>, With<GroupBoxQuad>)>>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    mut groups: ResMut<Assets<Group>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    // mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut my_shader_params: ResMut<Assets<MyShader>>,
    clearcolor_struct: Res<ClearColor>,
    mut globals: ResMut<Globals>,
    mut event_reader: EventReader<UiButton>,
    mut event_writer: EventWriter<Group>,
    // mut query: Query<(Entity, &Handle<Bezier>), With<BoundingBoxQuad>>,
) {
    let mut pressed_load_button = false;
    for ui_button in event_reader.iter() {
        pressed_load_button = ui_button == &UiButton::Load;
        break;
    }

    if pressed_load_button
        || (keyboard_input.pressed(KeyCode::LControl)
            && keyboard_input.just_released(KeyCode::S)
            && keyboard_input.pressed(KeyCode::LShift))
    {
        let clearcolor = clearcolor_struct.0;

        let path = "bezier.txt";
        let mut file = std::fs::File::open(path).unwrap();

        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let loaded_bezier_vec: Vec<Bezier> = serde_json::from_str(&contents).unwrap();

        // delete all current groups and curves before spawning the saved ones
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        globals.history = Vec::new();
        globals.do_hide_anchors = false;
        globals.do_hide_bounding_boxes = true;

        // for mut bezier in loaded_bezier_vec {
        //     spawn_bezier(
        //         &mut bezier,
        //         &mut bezier_curves,
        //         &mut commands,
        //         &mut meshes,
        //         // &mut pipelines,
        //         &mut my_shader_params,
        //         clearcolor,
        //         &mut globals,
        //     )
        // }
        // println!("{:?}", "loaded Bezier curves");

        let path = "curve_groups.txt";
        let mut file = std::fs::File::open(path).unwrap();

        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let loaded_groups_vec: Vec<GroupSaveLoad> = serde_json::from_str(&contents).unwrap();

        let mut group = Group {
            group: HashSet::new(),
            handles: HashSet::new(),
            lut: Vec::new(),
            ends: None,
            standalone_lut: (0.0, Vec::new()),
        };

        println!("group_load_save length: {:?}", loaded_groups_vec.len());

        for group_load_save in loaded_groups_vec {
            for (_anchor, mut bezier) in group_load_save.curves {
                let entity_and_handle = spawn_bezier(
                    &mut bezier,
                    &mut bezier_curves,
                    &mut commands,
                    &mut meshes,
                    // &mut pipelines,
                    &mut my_shader_params,
                    clearcolor,
                    &mut globals,
                );
                group.group.insert(entity_and_handle.clone());
                group.handles.insert(entity_and_handle.1);
            }
        }
        globals.selected = group.clone();
        event_writer.send(group);
        // to create a group: select all the curves programmatically, and send a UiButton::Group event

        println!("{:?}", "loaded groups");
    }
}

pub fn latch2(
    keyboard_input: Res<Input<KeyCode>>,
    cursor: ResMut<Cursor>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    mut query: QuerySet<(
        QueryState<(&Handle<Bezier>, &BoundingBoxQuad)>,
        QueryState<(&Handle<Bezier>, &BoundingBoxQuad)>,
    )>,
    globals: ResMut<Globals>,
    mut event_writer: EventWriter<OfficialLatch>,
    // mut event_reader: EventReader<UiButton>,
    button_query: Query<(&ButtonState, &UiButton)>,
) {
    let mut latch_button_on = false;
    for (button_state, ui_button) in button_query.iter() {
        if ui_button == &UiButton::Latch {
            latch_button_on = button_state == &ButtonState::On;
        }
    }

    if !globals.do_spawn_curve
        && (latch_button_on
            || keyboard_input.pressed(KeyCode::LShift)
                && keyboard_input.pressed(KeyCode::LControl)
                && !keyboard_input.pressed(KeyCode::Space)
                && mouse_button_input.pressed(MouseButton::Left))
    {
        let latching_distance = 5.0;

        let mut potential_mover: Option<(Vec2, u128, AnchorEdge, Handle<Bezier>)> = None;
        let mut potential_partner: Option<(
            u128,
            AnchorEdge,
            AnchorEdge,
            Handle<Bezier>,
            Handle<Bezier>,
        )> = None;

        // find moving quad and store its parameters
        for (bezier_handle, _bb) in query.q0().iter() {
            let bezier = bezier_curves.get(bezier_handle).unwrap().clone();
            if bezier.edge_is_moving() {
                // a latched point does not latch to an additional point
                let moving_anchor = bezier.get_mover_edge();
                if bezier.quad_is_latched(moving_anchor) {
                    return;
                }

                let mover_pos = cursor.position;
                potential_mover = Some((
                    mover_pos,
                    bezier.id,
                    bezier.get_mover_edge(),
                    bezier_handle.clone(),
                ));

                break;
            }
        }

        // find quad within latching_distance. Upon success, setup a latch and store the
        // paramters of the latchee (partner)
        if let Some((pos, id, mover_edge, mover_handle)) = potential_mover {
            if let Some((_dist, anchor_edge, partner_handle)) = get_close_still_anchor(
                latching_distance * globals.scale,
                pos,
                &bezier_curves,
                &query.q0(),
            ) {
                let partner_bezier = bezier_curves.get_mut(partner_handle.clone()).unwrap();

                // if the potential partner is free, continue
                if partner_bezier.quad_is_latched(anchor_edge) {
                    return;
                }

                potential_partner = Some((
                    partner_bezier.id,
                    mover_edge,
                    anchor_edge,
                    mover_handle,
                    partner_handle.clone(),
                ));

                let partner_latch_data = LatchData {
                    latched_to_id: id,
                    self_edge: anchor_edge,
                    partners_edge: mover_edge,
                };

                event_writer.send(OfficialLatch(partner_latch_data, partner_handle.clone()));
            }
        }

        // setup the latcher if a partner has been found
        if let Some((partner_id, mover_anchor, pa_edge, mover_handle, partner_handle)) =
            potential_partner
        {
            let partner_bezier = bezier_curves.get(partner_handle).unwrap().clone();
            let bezier = bezier_curves.get_mut(mover_handle.clone()).unwrap();

            let mover_latch_data = LatchData {
                latched_to_id: partner_id,
                self_edge: mover_anchor,
                partners_edge: pa_edge,
            };

            // set the position of the latched moving quad and its control point
            if mover_anchor == AnchorEdge::Start {
                bezier.positions.start = partner_bezier.get_position(pa_edge.to_anchor());
                bezier.positions.control_start = partner_bezier.get_opposite_control(pa_edge)
            } else if mover_anchor == AnchorEdge::End {
                bezier.positions.end = partner_bezier.get_position(pa_edge.to_anchor());
                bezier.positions.control_end = partner_bezier.get_opposite_control(pa_edge)
            }

            event_writer.send(OfficialLatch(mover_latch_data, mover_handle.clone()));
        }
    }
}

pub fn officiate_latch_partnership(
    mouse_button_input: Res<Input<MouseButton>>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    mut latch_event_reader: EventReader<OfficialLatch>,
    globals: ResMut<Globals>,
    audio: Res<Audio>,
) {
    if mouse_button_input.just_released(MouseButton::Left) {
        for OfficialLatch(latch, bezier_handle) in latch_event_reader.iter() {
            let bezier = bezier_curves.get_mut(bezier_handle).unwrap();
            bezier.set_latch(latch.clone());
            println!("latched, {:?}", bezier.latches);

            if globals.sound_on {
                if let Some(sound) = globals.sounds.get("latch") {
                    audio.play(sound.clone());
                }
            }
        }
    }
}

//
pub fn rescale(
    mut grandparent_query: Query<&mut Transform, With<GrandParent>>,
    shader_param_query: Query<&Handle<MyShader>>,
    mut my_shaders: ResMut<Assets<MyShader>>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut globals: ResMut<Globals>,
    keyboard_input: Res<Input<KeyCode>>,
    mut event_reader: EventReader<UiButton>,
) {
    for ui_button in event_reader.iter() {
        //
        //
        let mut pressed_rescale_button = false;
        let mut zoom_direction = 0.0;
        //
        if ui_button == &UiButton::ScaleUp {
            pressed_rescale_button = true;
            zoom_direction = 1.0;
        } else if ui_button == &UiButton::ScaleDown {
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

            // update the shader params for the middle quads (animated quads)
            for shader_handle in shader_param_query.iter() {
                let shader_param = my_shaders.get_mut(shader_handle).unwrap();
                shader_param.zoom = 0.15 / globals.scale;
                shader_param.size *= 1.0 / zoom_factor;
            }
        }
    }

    for event in mouse_wheel_events.iter() {
        if keyboard_input.pressed(KeyCode::LControl) {
            let zoom_factor = 1.0 + event.y * 0.1;
            globals.scale = globals.scale * zoom_factor;
            // the bounding box, the ends and the control points share the same shader parameters
            for mut transform in grandparent_query.iter_mut() {
                transform.scale = Vec2::new(globals.scale, globals.scale).extend(1.0);
            }

            // update the shader params for the middle quads (animated quads)
            for shader_handle in shader_param_query.iter() {
                let shader_param = my_shaders.get_mut(shader_handle).unwrap();
                shader_param.zoom = 0.15 / globals.scale;
                shader_param.size *= 1.0 / zoom_factor;
            }
        }
    }
}