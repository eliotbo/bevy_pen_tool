use crate::inputs::{ButtonInteraction, ButtonState, UiButton};
use crate::util::{ColorButton, Globals, Icon, MyShader, SoundStruct, UiAction, UiBoard};

use bevy::{
    prelude::*,
    render::{
        pipeline::{PipelineDescriptor, RenderPipeline, RenderPipelines},
        shader::ShaderStages,
    },
};

use std::{thread, time};

pub fn spawn_ui(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut my_shader_params: ResMut<Assets<MyShader>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut globals: ResMut<Globals>,
    // button_materials: Res<ButtonMaterials>,
    // mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let colors_str = vec![
        vec!["eee3e7", "ead5dc", "eec9d2", "f4b6c2", "f6abb6"],
        vec!["a8e6cf", "dcedc1", "ffd3b6", "ffaaa5", "ff8b94"],
        vec!["fe4a49", "2ab7ca", "fed766", "e6e6ea", "f4f4f8"],
        vec!["edc951", "eb6841", "cc2a36", "4f372d", "00a0b0"],
        vec!["011f4b", "03396c", "005b96", "6497b1", "b3cde0"],
        vec!["051e3e", "251e3e", "451e3e", "651e3e", "851e3e"],
        vec!["4a4e4d", "0e9aa7", "3da4ab", "f6cd61", "fe8a71"],
        // vec!["6e7f80", "536872", "708090", "536878", "36454f"],
        vec!["4b3832", "854442", "fff4e6", "3c2f2f", "be9b7b"],
        vec!["2e003e", "3d2352", "3d1e6d", "8874a3", "e4dcf1"],
    ];

    let num_rows = colors_str.len();

    let colors: Vec<Vec<Color>> = colors_str
        .iter()
        .map(|x| x.iter().map(|y| Color::hex(y).unwrap()).collect())
        .collect();

    // let colors: Vec<Color> = color_list.iter().map(|x| Color::hex(x).unwrap()).collect();

    asset_server.watch_for_changes().unwrap();
    let hundred_millis = time::Duration::from_millis(100);
    thread::sleep(hundred_millis);

    let vert = asset_server.load::<Shader, _>("shaders/bezier.vert"); // duplicate
    let button_frag = asset_server.load::<Shader, _>("shaders/button.frag");
    let ui_frag = asset_server.load::<Shader, _>("shaders/ui.frag");

    let button_pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: vert.clone(),
        fragment: Some(button_frag.clone()),
    }));

    let ui_pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: vert.clone(),
        fragment: Some(ui_frag.clone()),
    }));

    globals
        .pipeline_handles
        .insert("button", button_pipeline_handle.clone());

    globals
        .pipeline_handles
        .insert("ui", ui_pipeline_handle.clone());

    let color_ui_size = Vec2::new(40.0, 75.0);
    let mesh_handle_color_ui = meshes.add(Mesh::from(shape::Quad {
        size: color_ui_size,
        flip: false,
    }));

    let button_ui_size = Vec2::new(40.0, 35.0);
    let mesh_handle_button_ui = meshes.add(Mesh::from(shape::Quad {
        size: button_ui_size,
        flip: false,
    }));

    globals
        .mesh_handles
        .insert("color_ui", mesh_handle_color_ui.clone());

    globals
        .mesh_handles
        .insert("button_ui", mesh_handle_button_ui.clone());

    globals.picked_color = Some(colors[0][0]);

    let visible_ui = Visible {
        is_visible: true,
        is_transparent: true,
    };

    let shader_params_button_ui = my_shader_params.add(MyShader {
        color: Color::hex("131B23").unwrap(),
        size: button_ui_size,
        ..Default::default()
    });

    /////////////////////// buttons ui ////////////////////////////
    // let mut trans = Transform::from_translation(Vec3::new(00.0, 0.0, -100.0));
    // trans.rotation = Quat::from_rotation_y(-std::f32::consts::PI / 4.0);

    let button_ui_position = Vec3::new(-70.0, 37.5, -550.0);
    let ui_transform = Transform::from_translation(button_ui_position);

    let main_ui = commands
        .spawn_bundle(MeshBundle {
            mesh: mesh_handle_button_ui,
            visible: visible_ui.clone(),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                ui_pipeline_handle.clone(),
            )]),
            transform: ui_transform.clone(),
            ..Default::default()
        })
        .insert(shader_params_button_ui)
        .insert(UiBoard {
            expanded: true,
            size: button_ui_size,
            position: button_ui_position.truncate(),
            previous_position: button_ui_position.truncate(),
            action: UiAction::None,
        })
        .id();

    //
    //
    //
    //
    ///////////////////// latch button /////////////////////
    let button_width = 8.0;
    let button_size = Vec2::new(button_width, button_width);
    let mesh_handle_button = meshes.add(Mesh::from(shape::Quad {
        size: button_size,
        flip: false,
    }));

    globals
        .mesh_handles
        .insert("button", mesh_handle_button.clone());
    let shader_params_latch = my_shader_params.add(MyShader {
        color: Color::hex("4a4e4d").unwrap(),
        size: button_size,
        ..Default::default()
    });
    let button = commands
        .spawn_bundle(MeshBundle {
            mesh: mesh_handle_button.clone(),
            visible: visible_ui.clone(),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                button_pipeline_handle.clone(),
            )]),
            transform: Transform::from_translation(Vec3::new(
                button_width / 2.0,
                -button_width,
                -400.0,
            )),
            ..Default::default()
        })
        .insert(ButtonInteraction::None)
        .insert(UiButton::Latch)
        .insert(ButtonState::Off)
        .insert(shader_params_latch.clone())
        .id();

    commands.entity(main_ui).push_children(&[button]);

    let ends_pipeline_handle = globals.pipeline_handles["ends"].clone();

    let icon_size = Vec2::new(button_width / 4.0, button_width / 2.0);
    let mesh_handle_icon = meshes.add(Mesh::from(shape::Quad {
        size: icon_size,
        flip: false,
    }));

    globals
        .mesh_handles
        .insert("latch_button_icons", mesh_handle_icon.clone());

    let shader_params_icon1 = my_shader_params.add(MyShader {
        color: Color::hex("f6abb6").unwrap(),
        size: icon_size,
        ..Default::default()
    });
    let icon1 = commands
        .spawn_bundle(MeshBundle {
            mesh: mesh_handle_icon.clone(),
            visible: visible_ui.clone(),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                ends_pipeline_handle.clone(),
            )]),
            transform: Transform::from_translation(Vec3::new(-button_width / 8.0, 0.0, 10.1)),
            ..Default::default()
        })
        // .insert(ButtonInteraction::None)
        .insert(shader_params_icon1)
        .insert(Icon)
        .id();

    let shader_params_icon2 = my_shader_params.add(MyShader {
        color: Color::hex("3da4ab").unwrap(),
        size: icon_size,
        ..Default::default()
    });
    let mut icon2_transform = Transform::from_translation(Vec3::new(button_width / 8.0, 0.0, 20.1));
    icon2_transform.rotation = Quat::from_rotation_z(std::f32::consts::PI);
    let icon2 = commands
        .spawn_bundle(MeshBundle {
            mesh: mesh_handle_icon.clone(),
            visible: visible_ui.clone(),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                ends_pipeline_handle.clone(),
            )]),
            transform: icon2_transform,
            ..Default::default()
        })
        // .insert(ButtonInteraction::None)
        .insert(shader_params_icon2)
        .insert(Icon)
        .id();

    commands.entity(button).push_children(&[icon1, icon2]);
    //
    //
    //
    ///////////////////// detach button /////////////////////

    let shader_params_button_detach = my_shader_params.add(MyShader {
        color: Color::hex("4a4e4d").unwrap(),
        size: button_size,
        ..Default::default()
    });
    let button = commands
        .spawn_bundle(MeshBundle {
            mesh: mesh_handle_button.clone(),
            visible: visible_ui.clone(),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                button_pipeline_handle.clone(),
            )]),
            transform: Transform::from_translation(Vec3::new(button_width / 2.0, 0.0, -410.0)),
            ..Default::default()
        })
        .insert(ButtonInteraction::None)
        .insert(UiButton::Detach)
        .insert(ButtonState::Off)
        .insert(shader_params_button_detach.clone())
        // .insert(DetachButton)
        .id();

    commands.entity(main_ui).push_children(&[button]);

    let shader_params_icon1 = my_shader_params.add(MyShader {
        color: Color::hex("f6abb6").unwrap(),
        size: icon_size,
        ..Default::default()
    });
    let icon1 = commands
        .spawn_bundle(MeshBundle {
            mesh: mesh_handle_icon.clone(),
            visible: visible_ui.clone(),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                ends_pipeline_handle.clone(),
            )]),
            transform: Transform::from_translation(Vec3::new(-button_width / 5.0, 0.0, 10.1)),
            ..Default::default()
        })
        // .insert(ButtonInteraction::None)
        .insert(shader_params_icon1.clone())
        .insert(Icon)
        .id();

    let shader_params_icon2 = my_shader_params.add(MyShader {
        color: Color::hex("3da4ab").unwrap(),
        size: icon_size,
        ..Default::default()
    });
    let mut icon2_transform = Transform::from_translation(Vec3::new(button_width / 5.0, 0.0, 20.1));
    icon2_transform.rotation = Quat::from_rotation_z(std::f32::consts::PI);
    let icon2 = commands
        .spawn_bundle(MeshBundle {
            mesh: mesh_handle_icon.clone(),
            visible: visible_ui.clone(),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                ends_pipeline_handle.clone(),
            )]),
            transform: icon2_transform,
            ..Default::default()
        })
        // .insert(ButtonInteraction::None)
        .insert(shader_params_icon2.clone())
        .insert(Icon)
        .id();

    commands.entity(button).push_children(&[icon1, icon2]);

    //
    //
    //
    ///////////////////// Spawn Curve button /////////////////////

    let shader_params_spawn = my_shader_params.add(MyShader {
        color: Color::hex("4a4e4d").unwrap(),
        size: button_size,
        ..Default::default()
    });
    let button = commands
        .spawn_bundle(MeshBundle {
            mesh: mesh_handle_button.clone(),
            visible: visible_ui.clone(),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                button_pipeline_handle.clone(),
            )]),
            transform: Transform::from_translation(Vec3::new(button_width * 1.5, 0.0, -410.0)),
            ..Default::default()
        })
        .insert(ButtonInteraction::None)
        .insert(shader_params_spawn.clone())
        .insert(UiButton::SpawnCurve)
        .insert(ButtonState::Off)
        .id();

    commands.entity(main_ui).push_children(&[button]);

    // let shader_params_icon1 = my_shader_params.add(MyShader {
    //     color: Color::hex("f6abb6").unwrap(),
    //     size: icon_size,
    //     ..Default::default()
    // });
    let mut icon1_transform =
        Transform::from_translation(Vec3::new(-button_width / 5.0, 0.0, 20.1));
    icon1_transform.rotation = Quat::from_rotation_z(std::f32::consts::PI);
    let icon1 = commands
        .spawn_bundle(MeshBundle {
            mesh: mesh_handle_icon.clone(),
            visible: visible_ui.clone(),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                ends_pipeline_handle.clone(),
            )]),
            transform: icon1_transform,
            ..Default::default()
        })
        // .insert(ButtonInteraction::None)
        .insert(shader_params_icon1.clone())
        .insert(Icon)
        .id();

    let icon2_transform = Transform::from_translation(Vec3::new(button_width / 5.0, 0.0, 20.1));
    // icon2_transform.rotation = Quat::from_rotation_z(std::f32::consts::PI);
    let icon2 = commands
        .spawn_bundle(MeshBundle {
            mesh: mesh_handle_icon.clone(),
            visible: visible_ui.clone(),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                ends_pipeline_handle.clone(),
            )]),
            transform: icon2_transform,
            ..Default::default()
        })
        // .insert(ButtonInteraction::None)
        .insert(shader_params_icon1.clone())
        .insert(Icon)
        .id();

    commands.entity(button).push_children(&[icon1, icon2]);

    //
    //
    //
    ///////////////////// Selection button /////////////////////
    let shader_params_selection = my_shader_params.add(MyShader {
        color: Color::hex("4a4e4d").unwrap(),
        size: button_size,
        ..Default::default()
    });

    let selection_button = commands
        .spawn_bundle(MeshBundle {
            mesh: mesh_handle_button.clone(),
            visible: visible_ui.clone(),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                button_pipeline_handle.clone(),
            )]),
            transform: Transform::from_translation(Vec3::new(
                button_width * 1.5,
                -button_width,
                -420.0,
            )),
            ..Default::default()
        })
        .insert(ButtonInteraction::None)
        .insert(shader_params_selection.clone())
        .insert(UiButton::Selection)
        .insert(ButtonState::Off)
        .id();

    commands.entity(main_ui).push_children(&[selection_button]);

    let texture_handle = asset_server.load("textures/selection.png");
    let selection_sprite = commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(texture_handle.into()),
            // mesh: mesh_handle_button.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 11.0)),
            sprite: Sprite::new(button_size / 1.3),
            ..Default::default()
        })
        .id();

    commands
        .entity(selection_button)
        .push_children(&[selection_sprite]);

    //
    //
    //
    ///////////////////// Group button /////////////////////
    let shader_params_group = my_shader_params.add(MyShader {
        color: Color::hex("4a4e4d").unwrap(),
        size: button_size,
        ..Default::default()
    });

    let group_button = commands
        .spawn_bundle(MeshBundle {
            mesh: mesh_handle_button.clone(),
            visible: visible_ui.clone(),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                button_pipeline_handle.clone(),
            )]),
            transform: Transform::from_translation(Vec3::new(
                button_width * 1.5,
                button_width,
                -420.0,
            )),
            ..Default::default()
        })
        .insert(ButtonInteraction::None)
        .insert(shader_params_group.clone())
        .insert(UiButton::Group)
        .id();

    commands.entity(main_ui).push_children(&[group_button]);

    let texture_handle = asset_server.load("textures/selection.png");
    let selection_sprite = commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(texture_handle.into()),
            // mesh: mesh_handle_button.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 11.0)),
            sprite: Sprite::new(button_size / 1.3),
            ..Default::default()
        })
        .id();

    let mut icon1_transform =
        Transform::from_translation(Vec3::new(-button_width / 8.0, 0.0, 20.1));
    // icon1_transform.rotation = Quat::from_rotation_z(std::f32::consts::PI);
    let icon1 = commands
        .spawn_bundle(MeshBundle {
            mesh: mesh_handle_icon.clone(),
            visible: visible_ui.clone(),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                ends_pipeline_handle.clone(),
            )]),
            transform: icon1_transform,
            ..Default::default()
        })
        // .insert(ButtonInteraction::None)
        .insert(shader_params_icon1.clone())
        .insert(Icon)
        .id();

    let mut icon2_transform = Transform::from_translation(Vec3::new(button_width / 8.0, 0.0, 20.1));
    icon2_transform.rotation = Quat::from_rotation_z(std::f32::consts::PI);
    let icon2 = commands
        .spawn_bundle(MeshBundle {
            mesh: mesh_handle_icon.clone(),
            visible: visible_ui.clone(),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                ends_pipeline_handle.clone(),
            )]),
            transform: icon2_transform,
            ..Default::default()
        })
        // .insert(ButtonInteraction::None)
        .insert(shader_params_icon2.clone())
        .insert(Icon)
        .id();

    // commands.entity(button).push_children(&[icon1, icon2]);
    commands
        .entity(group_button)
        .push_children(&[selection_sprite, icon1, icon2]);

    //
    //
    //
    ///////////////////// Save button /////////////////////
    let shader_params_save = my_shader_params.add(MyShader {
        color: Color::hex("4a4e4d").unwrap(),
        size: button_size,
        ..Default::default()
    });
    let save_button = commands
        .spawn_bundle(MeshBundle {
            mesh: mesh_handle_button.clone(),
            visible: visible_ui.clone(),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                button_pipeline_handle.clone(),
            )]),
            transform: Transform::from_translation(Vec3::new(
                -button_width * 0.5,
                button_width,
                -430.0,
            )),
            ..Default::default()
        })
        .insert(ButtonInteraction::None)
        .insert(shader_params_save.clone())
        .insert(UiButton::Save)
        .id();

    commands.entity(main_ui).push_children(&[save_button]);

    let texture_handle = asset_server.load("textures/save.png");
    let save_sprite = commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(texture_handle.into()),
            // mesh: mesh_handle_button.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 11.0)),
            sprite: Sprite::new(button_size / 1.3),
            ..Default::default()
        })
        .id();

    commands.entity(save_button).push_children(&[save_sprite]);

    //
    //
    //
    ///////////////////// Load button /////////////////////
    let shader_params_load = my_shader_params.add(MyShader {
        color: Color::hex("4a4e4d").unwrap(),
        size: button_size,
        ..Default::default()
    });
    let load_button = commands
        .spawn_bundle(MeshBundle {
            mesh: mesh_handle_button.clone(),
            visible: visible_ui.clone(),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                button_pipeline_handle.clone(),
            )]),
            transform: Transform::from_translation(Vec3::new(
                -button_width * 1.5,
                button_width,
                -430.0,
            )),
            ..Default::default()
        })
        .insert(ButtonInteraction::None)
        .insert(shader_params_load.clone())
        .insert(UiButton::Load)
        .id();

    commands.entity(main_ui).push_children(&[load_button]);

    let texture_handle = asset_server.load("textures/load.png");
    let load_sprite = commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(texture_handle.into()),
            // mesh: mesh_handle_button.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 11.0)),
            sprite: Sprite::new(button_size / 1.3),
            ..Default::default()
        })
        .id();

    commands.entity(load_button).push_children(&[load_sprite]);

    //
    //
    //
    ///////////////////// undo button /////////////////////
    let shader_params_undo = my_shader_params.add(MyShader {
        color: Color::hex("4a4e4d").unwrap(),
        size: button_size,
        ..Default::default()
    });
    let undo_button = commands
        .spawn_bundle(MeshBundle {
            mesh: mesh_handle_button.clone(),
            visible: visible_ui.clone(),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                button_pipeline_handle.clone(),
            )]),
            transform: Transform::from_translation(Vec3::new(
                -button_width * 1.5,
                -button_width,
                -430.0,
            )),
            ..Default::default()
        })
        .insert(ButtonInteraction::None)
        .insert(shader_params_undo.clone())
        .insert(UiButton::Undo)
        .id();

    commands.entity(main_ui).push_children(&[undo_button]);

    let texture_handle = asset_server.load("textures/undo.png");
    let undo_sprite = commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(texture_handle.into()),
            // mesh: mesh_handle_button.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 11.0)),
            sprite: Sprite::new(button_size / 1.3),
            ..Default::default()
        })
        .id();

    commands.entity(undo_button).push_children(&[undo_sprite]);

    //
    //
    //
    ///////////////////// redo button /////////////////////
    let shader_params_redo = my_shader_params.add(MyShader {
        color: Color::hex("4a4e4d").unwrap(),
        size: button_size,
        ..Default::default()
    });
    let redo_button = commands
        .spawn_bundle(MeshBundle {
            mesh: mesh_handle_button.clone(),
            visible: visible_ui.clone(),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                button_pipeline_handle.clone(),
            )]),
            transform: Transform::from_translation(Vec3::new(
                -button_width * 0.5,
                -button_width,
                -430.0,
            )),
            ..Default::default()
        })
        .insert(ButtonInteraction::None)
        .insert(shader_params_redo.clone())
        .insert(UiButton::Redo)
        .id();

    commands.entity(main_ui).push_children(&[redo_button]);

    let texture_handle = asset_server.load("textures/redo.png");
    let redo_sprite = commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(texture_handle.into()),
            // mesh: mesh_handle_button.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 11.0)),
            sprite: Sprite::new(button_size / 1.3),
            ..Default::default()
        })
        .id();

    commands.entity(redo_button).push_children(&[redo_sprite]);

    //
    //
    //
    ///////////////////// hide button /////////////////////
    let shader_params_hide = my_shader_params.add(MyShader {
        color: Color::hex("4a4e4d").unwrap(),
        size: button_size,
        ..Default::default()
    });
    let hide_button = commands
        .spawn_bundle(MeshBundle {
            mesh: mesh_handle_button.clone(),
            visible: visible_ui.clone(),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                button_pipeline_handle.clone(),
            )]),
            transform: Transform::from_translation(Vec3::new(
                button_width * 0.5,
                button_width,
                -430.0,
            )),
            ..Default::default()
        })
        .insert(ButtonInteraction::None)
        .insert(shader_params_hide.clone())
        .insert(UiButton::Hide)
        .id();

    commands.entity(main_ui).push_children(&[hide_button]);

    let texture_handle = asset_server.load("textures/hide.png");
    let hide_sprite = commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(texture_handle.into()),
            // mesh: mesh_handle_button.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 11.0)),
            sprite: Sprite::new(button_size / 1.3),
            ..Default::default()
        })
        .id();

    commands.entity(hide_button).push_children(&[hide_sprite]);

    //
    //
    //
    ///////////////////// sound button /////////////////////
    let shader_params_sound = my_shader_params.add(MyShader {
        color: Color::hex("4a4e4d").unwrap(),
        size: button_size,
        ..Default::default()
    });
    let sound_button = commands
        .spawn_bundle(MeshBundle {
            mesh: mesh_handle_button.clone(),
            visible: visible_ui.clone(),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                button_pipeline_handle.clone(),
            )]),
            transform: Transform::from_translation(Vec3::new(-button_width * 0.5, 0.0, -430.0)),
            ..Default::default()
        })
        .insert(ButtonInteraction::None)
        .insert(shader_params_sound.clone())
        .insert(UiButton::Sound)
        .id();

    commands.entity(main_ui).push_children(&[sound_button]);

    let on_material = asset_server.load("textures/sound_on.png");
    let off_material = asset_server.load("textures/sound_off.png");
    let sound_sprite = commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(on_material.into()),
            // mesh: mesh_handle_button.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 11.0)),
            sprite: Sprite::new(button_size / 1.3),
            ..Default::default()
        })
        .insert(UiButton::Sound)
        .insert(SoundStruct {
            material: materials.add(off_material.into()),
        })
        .id();

    commands.entity(sound_button).push_children(&[sound_sprite]);
    /////////////////////// buttons ui ////////////////////////////
    /////////////////////// buttons ui ////////////////////////////
    //
    //
    //
    //
    //
    //////////////////////////// color ui /////////////////////////

    let color_ui_position = Vec3::new(-70.0, -15.0, -500.0);

    let shader_params_color_ui = my_shader_params.add(MyShader {
        color: Color::hex("131B23").unwrap(),
        size: color_ui_size,
        ..Default::default()
    });
    let ui_transform = Transform::from_translation(color_ui_position);
    let parent = commands
        .spawn_bundle(MeshBundle {
            mesh: mesh_handle_color_ui,
            visible: visible_ui.clone(),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                ui_pipeline_handle,
            )]),
            transform: ui_transform.clone(),
            ..Default::default()
        })
        .insert(shader_params_color_ui)
        .insert(UiBoard {
            expanded: true,
            size: color_ui_size,
            position: color_ui_position.truncate(),
            previous_position: color_ui_position.truncate(),
            action: UiAction::None,
        })
        .id();

    let color_button_size = Vec2::new(6., 6.);
    let mesh_handle_color_button = meshes.add(Mesh::from(shape::Quad {
        size: color_button_size,
        flip: false,
    }));
    globals
        .mesh_handles
        .insert("color_button", mesh_handle_color_button.clone());

    for (j, color_vec) in colors.iter().enumerate() {
        for (k, color) in color_vec.iter().enumerate() {
            let mut t = 0.0;
            if k == 0 && j == 0 {
                t = 1.0;
            }

            let shader_params_handle_button = my_shader_params.add(MyShader {
                color: color.clone(),
                size: color_button_size,
                t,
                ..Default::default()
            });

            let button_transform = Transform::from_translation(Vec3::new(
                (k as f32 - 2.0) * 6.1,
                (j as f32 + 0.5 - (num_rows as f32) / 2.0) * 7.0,
                10.0,
            ));

            let child = commands
                .spawn_bundle(MeshBundle {
                    mesh: mesh_handle_color_button.clone(),
                    visible: visible_ui.clone(),
                    render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                        button_pipeline_handle.clone(),
                    )]),
                    transform: button_transform.clone(),
                    ..Default::default()
                })
                .insert(shader_params_handle_button)
                // TODO: remove this size field as the size is known in shader_params
                .insert(ColorButton { size: button_size })
                .id();

            commands.entity(parent).push_children(&[child]);
        }
    }
    //////////////////////////// color ui /////////////////////////
}
