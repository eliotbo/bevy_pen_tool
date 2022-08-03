use crate::inputs::*;
use crate::moves::*;
use crate::spawner::*;

use crate::util::*;

use bevy::{prelude::*, sprite::Material2dPlugin};

pub struct PenPlugin;

impl Plugin for PenPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<Bezier>()
            .add_asset::<Group>()
            .add_event::<MouseClickEvent>()
            .add_event::<Group>()
            .add_event::<OfficialLatch>()
            .add_event::<MoveAnchor>()
            .add_event::<Latch>()
            .add_event::<Loaded>()
            .add_event::<Action>()
            .add_event::<UiButton>()
            .add_event::<Handle<Group>>()
            .add_event::<SpawnMids>()
            // .add_plugin(Material2dPlugin::<BezierMat>::default())
            .add_plugin(ColoredMesh2dPlugin) // mesh making
            .add_plugin(RoadMesh2dPlugin) // mesh making
            .add_plugin(Material2dPlugin::<SelectionMat>::default())
            .add_plugin(Material2dPlugin::<SelectingMat>::default())
            .add_plugin(Material2dPlugin::<ButtonMat>::default())
            .add_plugin(Material2dPlugin::<UiMat>::default())
            .add_plugin(Material2dPlugin::<BezierEndsMat>::default())
            .add_plugin(Material2dPlugin::<BezierControlsMat>::default())
            .add_plugin(Material2dPlugin::<BezierMidMat>::default())
            .add_plugin(Material2dPlugin::<FillMat>::default())
            .add_state("ModelViewController")
            .insert_resource(ClearColor(Color::hex("6e7f80").unwrap()))
            .insert_resource(Cursor::default())
            .insert_resource(Globals::default())
            .insert_resource(Selection::default())
            .insert_resource(Maps::default())
            .insert_resource(UserState::default())
            .add_startup_system(setup.exclusive_system().at_start()) //.label("setup"))
            .add_startup_system(spawn_selection_bounding_box) //.after("setup"))
            .add_startup_system(spawn_ui) //.after("setup"))
            .add_startup_system(spawn_selecting_bounding_box) //.after("setup"))
            //
            // Update controller
            .add_system_set(
                SystemSet::on_update("ModelViewController")
                    .with_system(record_mouse_events_system.exclusive_system().at_start())
                    .with_system(check_mouseclick_on_objects)
                    .with_system(rescale)
                    .with_system(check_mouse_on_ui)
                    .with_system(pick_color)
                    .with_system(check_mouse_on_canvas)
                    .with_system(spawn_curve_order_on_mouseclick)
                    .with_system(spawn_middle_quads)
                    .with_system(button_system)
                    .with_system(toggle_ui_button)
                    .with_system(send_action.exclusive_system().at_end())
                    .label("controller"),
            )
            //
            // Update model
            .add_system_set(
                SystemSet::on_update("ModelViewController")
                    .with_system(groupy.label("group"))
                    .with_system(load.after("group"))
                    .with_system(recompute_lut.label("recompute_lut"))
                    .with_system(save.after("recompute_lut"))
                    .with_system(change_ends_and_controls_params.exclusive_system().at_end())
                    .with_system(latchy)
                    .with_system(update_lut)
                    .with_system(officiate_latch_partnership)
                    .with_system(selection_box_init)
                    .with_system(selection_final)
                    .with_system(hide_anchors)
                    .with_system(delete)
                    .with_system(hide_control_points)
                    .with_system(spawn_heli)
                    .with_system(make_mesh)
                    .with_system(make_road)
                    .with_system(unselect)
                    .with_system(debug)
                    .with_system(ungroup)
                    .label("model")
                    .after("controller"),
            )
            //
            // Update view
            .add_system_set(
                SystemSet::on_update("ModelViewController")
                    // TODO:
                    // mouse_release_actions should be in the controller,
                    // but there is a bug with the position of new latches when it's there
                    .with_system(mouse_release_actions)
                    //
                    .with_system(begin_move_on_mouseclick)
                    .with_system(move_end_quads)
                    .with_system(move_middle_quads)
                    .with_system(move_group_middle_quads)
                    .with_system(move_control_quads)
                    .with_system(move_bb_quads)
                    .with_system(move_ui)
                    .with_system(turn_round_animation)
                    .with_system(follow_bezier_group)
                    .with_system(adjust_selection_attributes)
                    .with_system(adjust_selecting_attributes)
                    .with_system(adjust_group_attributes)
                    .with_system(spawn_bezier_system)
                    // .with_system(spawn_group_middle_quads)
                    .with_system(spawn_group_entities)
                    .label("view")
                    .after("model"),
            );
    }
}

//////////////////////////// Debugging ////////////////////////////
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Clone, Debug)]
struct BezierPrint {
    pub positions: BezierPositions,
    pub previous_positions: BezierPositions,
    pub move_quad: Anchor,
    pub color: Option<Color>,
    pub do_compute_lut: bool,
    pub id: u128,
    pub latches: HashMap<AnchorEdge, LatchData>,
    pub potential_latch: Option<LatchData>,
    pub grouped: bool,
}

impl BezierPrint {
    #[allow(dead_code)]
    pub fn from_bezier(bezier: &Bezier) -> Self {
        Self {
            positions: bezier.positions.clone(),
            previous_positions: bezier.positions.clone(),
            move_quad: bezier.move_quad,
            color: None,
            do_compute_lut: false,
            id: bezier.id,
            latches: bezier.latches.clone(),
            potential_latch: None,
            grouped: false,
        }
    }
}

use std::collections::HashSet;
#[derive(Clone, Debug)]
#[allow(dead_code)]
struct GroupPrint {
    // TODO: rid Group of redundancy
    pub group: HashSet<(Entity, Handle<Bezier>)>,
    pub bezier_handles: HashSet<Handle<Bezier>>,
    //
    // Attempts to store the start and end points of a group.
    // Fails if curves are not connected
    pub ends: Option<Vec<(Handle<Bezier>, AnchorEdge)>>,
}

impl GroupPrint {
    #[allow(dead_code)]
    pub fn from_group(group: &Group) -> Self {
        Self {
            group: group.group.clone(),
            bezier_handles: group.bezier_handles.clone(),
            ends: group.ends.clone(),
        }
    }
}

fn debug(
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<&Handle<Bezier>, With<BezierParent>>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    groups: Res<Assets<Group>>,
) {
    if keyboard_input.just_pressed(KeyCode::B)
        && !keyboard_input.pressed(KeyCode::LShift)
        && !keyboard_input.pressed(KeyCode::LControl)
    {
        // println!("'B' currently pressed");
        for handle in query.iter() {
            let _bezier = bezier_curves.get_mut(handle).unwrap();

            // println!("group id: {:?}", bezier.group);
            // println!("latches: {:#?}", BezierPrint::from_bezier(bezier));
            // println!("");
        }
    }

    if keyboard_input.just_pressed(KeyCode::G) {
        // println!("'B' currently pressed");
        for (_, _group) in groups.iter() {
            // let bezier = bezier_curves.get_mut(handle).unwrap();

            // println!("group: {:#?}", GroupPrint::from_group(group));
            // println!("");
        }
    }
}
//////////////////////////// Debugging ////////////////////////////

fn setup(
    // mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut maps: ResMut<Maps>,
    // audio: Res<Audio>,
) {
    asset_server.watch_for_changes().unwrap();

    let latch_sound: Handle<AudioSource> = asset_server.load("sounds/latch.ogg");
    let unlatch_sound: Handle<AudioSource> = asset_server.load("sounds/unlatch.ogg");
    let group_sound: Handle<AudioSource> = asset_server.load("sounds/group.ogg");

    maps.sounds.insert("latch", latch_sound);
    maps.sounds.insert("unlatch", unlatch_sound);
    maps.sounds.insert("group", group_sound);

    // let hundred_millis = time::Duration::from_millis(100);
    // thread::sleep(hundred_millis);

    let color_ui_size = Vec2::new(200.0, 375.0);
    let button_ui_size = Vec2::new(200.0, 225.0);

    let button_width = 40.0;
    let button_size = Vec2::new(button_width, button_width);
    let icon_size = Vec2::new(button_width / 4.0, button_width / 2.0);

    let mesh_handle_color_ui =
        bevy::sprite::Mesh2dHandle(meshes.add(Mesh::from(shape::Quad::new(color_ui_size))));

    let mesh_handle_button_ui =
        bevy::sprite::Mesh2dHandle(meshes.add(Mesh::from(shape::Quad::new(button_ui_size))));
    let mesh_handle_button =
        bevy::sprite::Mesh2dHandle(meshes.add(Mesh::from(shape::Quad::new(button_size))));

    let ends_controls_mesh_handle =
        bevy::sprite::Mesh2dHandle(meshes.add(Mesh::from(shape::Quad::new(Vec2::new(20.0, 20.0)))));

    let ends_mesh_handle =
        bevy::sprite::Mesh2dHandle(meshes.add(Mesh::from(shape::Quad::new(Vec2::new(10.0, 20.0)))));

    let middle_mesh_handle =
        bevy::sprite::Mesh2dHandle(meshes.add(Mesh::from(shape::Quad::new(Vec2::new(7.5, 7.5)))));

    let mesh_handle_icon =
        bevy::sprite::Mesh2dHandle(meshes.add(Mesh::from(shape::Quad::new(icon_size))));

    maps.mesh_handles.insert("middles", middle_mesh_handle);

    maps.mesh_handles.insert("ends", ends_mesh_handle);

    maps.mesh_handles
        .insert("ends_controls", ends_controls_mesh_handle);

    maps.mesh_handles.insert("button", mesh_handle_button);

    maps.mesh_handles
        .insert("color_ui", mesh_handle_color_ui.clone());

    maps.mesh_handles
        .insert("button_ui", mesh_handle_button_ui.clone());

    maps.mesh_handles.insert("icon", mesh_handle_icon.clone());

    let road_texture_handle: Handle<Image> = asset_server.load("textures/single_lane_road.png");
    // print!("road_texture_handle: {:?}", road_texture_handle);

    maps.textures
        .insert("single_lane_road", road_texture_handle);

    // thread::sleep(hundred_millis);
}

// fn setup(
//     // mut commands: Commands,
//     asset_server: Res<AssetServer>,
//     mut meshes: ResMut<Assets<Mesh>>,
//     // mut pipelines: ResMut<Assets<PipelineDescriptor>>,
//     // mut render_graph: ResMut<RenderGraph>,
//     // mut maps: ResMut<Maps>,
//     mut commands: Commands,
//     // mut meshes: ResMut<Assets<Mesh>>,
//     // mut materials: ResMut<Assets<MyShader>>,
// ) {
//     let size = Vec2::new(300.0, 300.0);

//     // let material = materials.add(MyShader::default());

//     // // quad
//     // commands.spawn().insert_bundle(MaterialMesh2dBundle {
//     //     mesh: Mesh2dHandle(meshes.add(Mesh::from(shape::Quad::new(size)))),
//     //     material,
//     //     ..Default::default()
//     // });

//     // commands.spawn_bundle(OrthographicCameraBundle::new_2d());

//     // asset_server.watch_for_changes().unwrap();
// }

// #[derive(Debug, Clone, TypeUuid, AsStd140)]
// #[uuid = "da63852d-f82b-459d-9790-3e652f92eaf7"]
// pub struct MyShader {
//     pub color: Vec4,
// }
