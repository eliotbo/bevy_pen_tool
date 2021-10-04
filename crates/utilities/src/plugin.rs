use crate::cam::Cam;

use crate::inputs::{
    begin_move_on_mouseclick,
    button_system,
    check_mouse_on_canvas,
    check_mouse_on_ui,
    delete,
    groupy,
    hide_anchors,
    hide_control_points,
    latch2,
    load,
    officiate_latch_partnership,
    pick_color,
    recompute_lut,
    record_mouse_events_system,
    rescale,
    save,
    selection,
    send_action,
    toggle_ui_button,
    unselect,
    // redo, undo,
    Action,
    Cursor,
    Latch,
    MoveAnchor,

    UiButton,
};
use crate::moves::{
    move_bb_quads,
    move_control_quads,
    move_end_quads,
    move_group_middle_quads,
    move_middle_quads,
    move_ui,
    //
};
use crate::spawner::{
    spawn_bezier_system,
    spawn_curve_order_on_mouseclick,
    spawn_group_bounding_box,
    spawn_group_middle_quads,
    spawn_selection_bounding_box,
    spawn_ui,
    //
};
use crate::util::*;

use bevy::{
    prelude::*,
    render::{
        // camera::OrthographicProjection,
        pipeline::PipelineDescriptor,
        render_graph::{base, AssetRenderResourcesNode, RenderGraph},
        shader::ShaderStages,
    },
};

use std::{thread, time};

pub struct PenPlugin;
//
//
//

impl Plugin for PenPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<MyShader>()
            .add_asset::<Bezier>()
            .add_asset::<Group>()
            .add_event::<Group>()
            .add_event::<OfficialLatch>()
            .add_event::<MoveAnchor>()
            .add_event::<Latch>()
            .add_event::<Loaded>()
            .add_event::<Action>()
            .add_event::<UiButton>()
            .add_event::<Handle<Group>>()
            .insert_resource(ClearColor(Color::hex("6e7f80").unwrap()))
            .insert_resource(Cursor::default())
            .insert_resource(Globals::default())
            .add_startup_system(setup.system().label("setup"))
            .add_startup_system(spawn_selection_bounding_box.system().after("setup"))
            .add_startup_system(spawn_ui.system().after("setup"))
            .add_system(record_mouse_events_system.system().label("input"))
            .add_system(rescale.system().before("mouse_ui"))
            .add_system(check_mouse_on_ui.system().label("mouse_ui").after("input"))
            .add_system(pick_color.system().label("mouse_color").after("mouse_ui"))
            .add_system(
                move_end_quads
                    .system()
                    .label("move_ends")
                    .after("mouse_color"),
            )
            .add_system(
                spawn_curve_order_on_mouseclick
                    .system()
                    .label("spawn_curve")
                    .after("move_ends"),
            )
            .add_system(
                begin_move_on_mouseclick
                    .system()
                    .label("move_curve")
                    .after("spawn_curve"),
            )
            .add_system(
                spawn_bezier_system
                    .system()
                    .label("spawn_bezier")
                    .after("move_curve"),
            )
            .add_system(
                spawn_group_middle_quads
                    .system()
                    .label("group_mid")
                    .after("spawn_bezier"),
            )
            .add_system(
                spawn_group_bounding_box
                    .system()
                    .label("spawn_group")
                    .after("group_mid"),
            )
            .add_system(groupy.system().after("spawn_group").label("groupy"))
            .add_system(load.system().after("groupy"))
            .add_system(
                change_ends_and_controls_params
                    .system()
                    .label("update_params")
                    .after("spawn_bezier"),
            )
            .add_system(latch2.system().label("latch").after("update_params"))
            .add_system(
                officiate_latch_partnership
                    .system()
                    .label("offi")
                    .after("latch"),
            )
            .add_system(move_middle_quads.system().after("move_ends"))
            .add_system(move_group_middle_quads.system().after("move_ends"))
            .add_system(move_control_quads.system().after("move_ends"))
            .add_system(move_bb_quads.system())
            .add_system(recompute_lut.system())
            // .add_system(undo.system())
            // .add_system(redo.system())
            .add_system(selection.system().label("selection"))
            .add_system(unselect)
            .add_system(adjust_selection_attributes.system())
            .add_system(adjust_group_attributes.system())
            .add_system(hide_anchors.system())
            // .add_system(do_long_lut.system().label("long_lut"))
            .add_system(save.system().after("long_lut"))
            .add_system(delete.system().label("delete"))
            .add_system(button_system.after("mouse_color"))
            .add_system(move_ui.system().label("move_ui").after("selection"))
            .add_system(toggle_ui_button.system())
            .add_system(hide_control_points)
            .add_system(send_action)
            .add_system(check_mouse_on_canvas.before("move_curve"));
    }
}

fn setup(
    // mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut render_graph: ResMut<RenderGraph>,
    mut globals: ResMut<Globals>,
) {
    asset_server.watch_for_changes().unwrap();

    let latch_sound: Handle<AudioSource> = asset_server.load("sounds/latch.mp3");
    let unlatch_sound: Handle<AudioSource> = asset_server.load("sounds/unlatch.mp3");
    let group_sound: Handle<AudioSource> = asset_server.load("sounds/group.mp3");

    globals.sounds.insert("latch", latch_sound);
    globals.sounds.insert("unlatch", unlatch_sound);
    globals.sounds.insert("group", group_sound);

    let frag = asset_server.load::<Shader, _>("shaders/bezier.frag");
    let vert = asset_server.load::<Shader, _>("shaders/bezier.vert");
    let ends = asset_server.load::<Shader, _>("shaders/ends.frag");
    let button = asset_server.load::<Shader, _>("shaders/button.frag");
    let frag_bb = asset_server.load::<Shader, _>("shaders/bounding_box.frag");
    let controls_frag = asset_server.load::<Shader, _>("shaders/controls.frag");

    let hundred_millis = time::Duration::from_millis(100);
    thread::sleep(hundred_millis);

    render_graph.add_system_node(
        "my_shader_params",
        AssetRenderResourcesNode::<MyShader>::new(true),
    );
    render_graph
        .add_node_edge("my_shader_params", base::node::MAIN_PASS)
        .unwrap();

    let ends_pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: vert.clone(),
        fragment: Some(ends.clone()),
    }));

    let mids_pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: vert.clone(),
        fragment: Some(frag.clone()),
    }));

    let controls_pipeline_handle =
        pipelines.add(PipelineDescriptor::default_config(ShaderStages {
            vertex: vert.clone(),
            fragment: Some(controls_frag.clone()),
        }));

    let bb_pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: vert.clone(),
        fragment: Some(frag_bb),
    }));

    let button_pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: vert.clone(),
        fragment: Some(button),
    }));

    let mesh_handle_ends_controls = meshes.add(Mesh::from(shape::Quad {
        size: Vec2::new(4.0, 4.0),
        flip: false,
    }));

    let mesh_handle_ends = meshes.add(Mesh::from(shape::Quad {
        size: Vec2::new(2.0, 4.0),
        flip: false,
    }));

    let mesh_handle_middle = meshes.add(Mesh::from(shape::Quad {
        size: Vec2::new(1.5, 1.5),
        flip: false,
    }));

    let mesh_handle_button = meshes.add(Mesh::from(shape::Quad {
        size: Vec2::new(3.0, 3.0),
        flip: false,
    }));

    globals
        .pipeline_handles
        .insert("ends", ends_pipeline_handle);
    globals
        .pipeline_handles
        .insert("controls", controls_pipeline_handle);
    globals
        .pipeline_handles
        .insert("mids", mids_pipeline_handle);
    globals
        .pipeline_handles
        .insert("button", button_pipeline_handle);

    globals
        .pipeline_handles
        .insert("bounding_box", bb_pipeline_handle);

    globals.mesh_handles.insert("middles", mesh_handle_middle);

    globals.mesh_handles.insert("ends", mesh_handle_ends);

    globals
        .mesh_handles
        .insert("ends_controls", mesh_handle_ends_controls);

    globals.mesh_handles.insert("button", mesh_handle_button);
}
