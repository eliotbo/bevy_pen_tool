use crate::inputs::*;
use crate::moves::*;
use crate::spawner::*;
use crate::util::*;

use bevy::{
    prelude::*,
    render::{
        pipeline::PipelineDescriptor,
        render_graph::{base, AssetRenderResourcesNode, RenderGraph},
        shader::ShaderStages,
    },
};

use std::{thread, time};

pub struct PenPlugin;

impl Plugin for PenPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<MyShader>()
            .add_asset::<Bezier>()
            .add_asset::<Group>()
            .add_asset::<MyArrayTexture>()
            .add_event::<MouseClickEvent>()
            .add_event::<Group>()
            .add_event::<OfficialLatch>()
            .add_event::<MoveAnchor>()
            .add_event::<Latch>()
            .add_event::<Loaded>()
            .add_event::<Action>()
            .add_event::<UiButton>()
            .add_event::<Handle<Group>>()
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
                    .with_system(button_system)
                    .with_system(toggle_ui_button)
                    .with_system(send_action.exclusive_system().at_end())
                    .label("controller"),
            )
            //
            // Update model
            .add_system_set(
                SystemSet::on_update("ModelViewController")
                    .with_system(groupy)
                    .with_system(change_ends_and_controls_params.exclusive_system().at_end())
                    .with_system(latch2)
                    .with_system(officiate_latch_partnership)
                    .with_system(recompute_lut)
                    .with_system(load)
                    .with_system(save)
                    .with_system(selection_box_init)
                    .with_system(selection_final)
                    .with_system(hide_anchors)
                    .with_system(delete)
                    .with_system(hide_control_points)
                    .with_system(spawn_heli)
                    .with_system(make_mesh)
                    .with_system(make_road)
                    .label("model")
                    .after("controller"),
            )
            //
            // Update view
            .add_system_set(
                SystemSet::on_update("ModelViewController")
                    // TODO:
                    // mouse_release_actions should be in the controller,
                    // but there is a visual bug with new latches when it's there
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
                    .with_system(spawn_group_middle_quads)
                    .with_system(spawn_group_bounding_box)
                    .label("view")
                    .after("model"),
            );

        //
        //

        // .add_system(
        //     move_end_quads

        //         .label("move_ends")
        //         .after("mouse_color"),
        // )
        // .add_system(
        //     spawn_curve_order_on_mouseclick

        //         .label("spawn_curve")
        //         .after("move_ends"),
        // )
        // .add_system(
        //     begin_move_on_mouseclick
        //         .label("move_curve")
        //         .after("spawn_curve"),
        // )
        // .add_system(
        //     check_mouse_on_canvas
        //         .label("check_move")
        //         .after("spawn_curve"),
        // )
        // .add_system(
        //     spawn_bezier_system
        //   .label("spawn_bezier")
        //         .after("check_move"),
        // )
        // .add_system(
        //     spawn_group_middle_quads
        //        .label("group_mid")
        //         .after("spawn_bezier"),
        // )
        // .add_system(
        //     spawn_group_bounding_box
        //         .label("spawn_group")
        //         .after("group_mid"),
        // )
        // .add_system(groupy.after("spawn_group").label("groupy"))
        // .add_system(load.after("groupy"))
        // .add_system(
        //     change_ends_and_controls_params
        //         .label("update_params")
        //         .after("spawn_bezier"),
        // )
        // .add_system(latch2.label("latch").after("update_params"))
        // .add_system(
        //     officiate_latch_partnership
        //         .system()
        //         .label("offi")
        //         .after("latch"),
        // )
        // .add_system(move_middle_quads.after("move_ends"))
        // .add_system(move_group_middle_quads.after("move_ends"))
        // .add_system(move_control_quads.after("move_ends"))
        // .add_system(move_bb_quads)
        // .add_system(recompute_lut)
        // .add_system(undo)
        // .add_system(redo)
        // .add_system(selection.label("selection"))
        // .add_system(selection_box_init.label("selection_box"))
        // .add_system(selection_final.label("select_final").after("selection_box"))
        // .add_system(unselect)
        // .add_system(adjust_selection_attributes.system())
        // .add_system(adjust_selecting_attributes)
        // .add_system(adjust_group_attributes.system())
        // .add_system(hide_anchors.system())
        // .add_system(do_long_lut.system().label("long_lut"))
        // .add_system(save)
        // .add_system(delete.label("delete"))
        // .add_system(button_system.after("mouse_color"))
        // .add_system(move_ui.system().label("move_ui"))
        // .add_system(toggle_ui_button.system())
        // .add_system(hide_control_points)
        // .add_system(send_action)
        // .add_system(spawn_heli)
        // .add_system(make_mesh)
        // .add_system(turn_round_animation)
        // .add_system(follow_bezier_group)
        // .add_system(make_road)
        // .add_system(mouse_release_actions);
    }
}

fn setup(
    // mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut render_graph: ResMut<RenderGraph>,
    mut maps: ResMut<Maps>,
) {
    asset_server.watch_for_changes().unwrap();

    let latch_sound: Handle<AudioSource> = asset_server.load("sounds/latch.mp3");
    let unlatch_sound: Handle<AudioSource> = asset_server.load("sounds/unlatch.mp3");
    let group_sound: Handle<AudioSource> = asset_server.load("sounds/group.mp3");

    maps.sounds.insert("latch", latch_sound);
    maps.sounds.insert("unlatch", unlatch_sound);
    maps.sounds.insert("group", group_sound);

    let frag = asset_server.load::<Shader, _>("shaders/bezier.frag");
    let vert = asset_server.load::<Shader, _>("shaders/bezier.vert");
    let ends = asset_server.load::<Shader, _>("shaders/ends.frag");
    let button = asset_server.load::<Shader, _>("shaders/button.frag");
    let frag_bb = asset_server.load::<Shader, _>("shaders/bounding_box.frag");
    let selecting = asset_server.load::<Shader, _>("shaders/selecting.frag");
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

    let selecting_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: vert.clone(),
        fragment: Some(selecting),
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

    maps.pipeline_handles.insert("ends", ends_pipeline_handle);
    maps.pipeline_handles
        .insert("controls", controls_pipeline_handle);
    maps.pipeline_handles.insert("mids", mids_pipeline_handle);
    maps.pipeline_handles
        .insert("button", button_pipeline_handle);

    maps.pipeline_handles
        .insert("bounding_box", bb_pipeline_handle);

    maps.pipeline_handles.insert("selecting", selecting_handle);

    maps.mesh_handles.insert("middles", mesh_handle_middle);

    maps.mesh_handles.insert("ends", mesh_handle_ends);

    maps.mesh_handles
        .insert("ends_controls", mesh_handle_ends_controls);

    maps.mesh_handles.insert("button", mesh_handle_button);

    thread::sleep(hundred_millis);
}
