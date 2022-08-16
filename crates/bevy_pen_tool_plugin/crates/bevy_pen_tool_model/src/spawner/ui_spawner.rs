use crate::inputs::{ButtonInteraction, ButtonState, UiButton};
// use crate::util::materials::*;
use crate::materials::{BezierEndsMat, ButtonMat, UiMat};
use crate::model::{ColorButton, Globals, Icon, MainUi, Maps, OnOffMaterial, UiAction, UiBoard};

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

pub fn spawn_ui(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    // mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    // mut my_shader_params: ResMut<Assets<MyShader>>,
    mut ui_materials: ResMut<Assets<UiMat>>,
    mut button_materials: ResMut<Assets<ButtonMat>>,
    mut ends_materials: ResMut<Assets<BezierEndsMat>>,
    // mut materials: ResMut<Assets<ColorMaterial>>,
    mut globals: ResMut<Globals>,
    mut maps: ResMut<Maps>,
) {
    let colors_str = vec![
        vec!["eee3e7", "ead5dc", "eec9d2", "f4b6c2", "f6abb6"],
        vec!["a8e6cf", "dcedc1", "ffd3b6", "ffaaa5", "ff8b94"],
        vec!["fe4a49", "2ab7ca", "fed766", "e6e6ea", "f4f4f8"],
        vec!["edc951", "eb6841", "cc2a36", "4f372d", "00a0b0"],
        vec!["011f4b", "03396c", "005b96", "6497b1", "b3cde0"],
        vec!["051e3e", "251e3e", "451e3e", "651e3e", "851e3e"],
        vec!["4a4e4d", "0e9aa7", "3da4ab", "f6cd61", "fe8a71"],
        vec!["4b3832", "854442", "fff4e6", "3c2f2f", "be9b7b"],
        vec!["2e003e", "3d2352", "3d1e6d", "8874a3", "e4dcf1"],
    ];

    let num_rows = colors_str.len();

    let colors: Vec<Vec<Color>> = colors_str
        .iter()
        .map(|x| x.iter().map(|y| Color::hex(y).unwrap()).collect())
        .collect();

    let color_ui_size = Vec2::new(200.0, 375.0);
    let button_ui_size = Vec2::new(200.0, 225.0);
    let button_width = 40.0;
    let button_size = Vec2::new(button_width, button_width);
    let icon_size = Vec2::new(button_width / 4.0, button_width / 2.0);

    let mesh_handle_button_ui = maps.mesh_handles.get("button_ui").unwrap();
    let mesh_handle_color_ui = maps.mesh_handles.get("color_ui").unwrap();
    let mesh_handle_button = maps.mesh_handles.get("button").unwrap();
    let mesh_handle_icon = maps.mesh_handles.get("icon").unwrap();

    globals.picked_color = Some(colors[0][0]);

    let visible_ui = bevy::render::view::Visibility { is_visible: true };

    /////////////////////// buttons ui ////////////////////////////

    let button_ui_position = Vec3::new(-350.0, 187.5, globals.z_pos.ui_buttons);
    let ui_transform = Transform::from_translation(button_ui_position);

    let ui_mat = UiMat::default();

    // ui_mat.size = button_ui_size;

    let ui_material = ui_materials.add(ui_mat);

    let main_ui = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle_button_ui.clone(),
            transform: ui_transform.clone(),
            material: ui_material,
            ..Default::default()
        })
        .insert(MainUi)
        .insert(UiBoard {
            expanded: true,
            size: button_ui_size,
            position: button_ui_position.truncate(),
            previous_position: button_ui_position.truncate(),
            action: UiAction::None,
        })
        .id();
    /////////////////////// buttons ui ////////////////////////////
    //
    //
    //
    //
    ///////////////////// latch button /////////////////////

    let shader_params_latch = button_materials.add(ButtonMat {
        color: Color::hex("4a4e4d").unwrap().into(),
        size: button_size,
        ..Default::default()
    });

    let button = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle_button.clone(),
            visibility: visible_ui.clone(),
            transform: Transform::from_translation(Vec3::new(
                button_width / 2.0,
                -button_width,
                globals.z_pos.ui_buttons,
            )),
            material: shader_params_latch,
            ..Default::default()
        })
        .insert(ButtonInteraction::None)
        .insert(UiButton::Latch)
        .insert(ButtonState::Off)
        .id();

    commands.entity(main_ui).push_children(&[button]);

    let shader_params_icon1 = ends_materials.add(BezierEndsMat {
        color: Color::hex("f6abb6").unwrap().into(),
        size: icon_size,
        ..Default::default()
    });
    let icon1 = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle_icon.clone(),
            visibility: visible_ui.clone(),
            transform: Transform::from_translation(Vec3::new(
                -button_width / 8.0,
                0.0,
                globals.z_pos.ui_button_icons,
            )),
            material: shader_params_icon1,
            ..Default::default()
        })
        // .insert(ButtonInteraction::None)
        // .insert(shader_params_icon1)
        .insert(Icon)
        .id();

    let shader_params_icon2 = ends_materials.add(BezierEndsMat {
        color: Color::hex("3da4ab").unwrap().into(),
        size: icon_size,
        ..Default::default()
    });
    let mut icon2_transform = Transform::from_translation(Vec3::new(
        button_width / 8.0,
        0.0,
        globals.z_pos.ui_button_icons,
    ));
    icon2_transform.rotation = Quat::from_rotation_z(std::f32::consts::PI);
    let icon2 = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle_icon.clone(),
            visibility: visible_ui.clone(),
            transform: icon2_transform,
            material: shader_params_icon2,
            ..Default::default()
        })
        // .insert(ButtonInteraction::None)
        // .insert(shader_params_icon2)
        .insert(Icon)
        .id();

    commands.entity(button).push_children(&[icon1, icon2]);

    //
    //
    //
    ///////////////////// detach button /////////////////////

    let shader_params_button_detach = button_materials.add(ButtonMat {
        color: Color::hex("4a4e4d").unwrap().into(),
        size: button_size,
        ..Default::default()
    });
    let button = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle_button.clone(),
            transform: Transform::from_translation(Vec3::new(
                button_width / 2.0,
                0.0,
                globals.z_pos.ui_buttons,
            )),
            material: shader_params_button_detach,
            ..Default::default()
        })
        .insert(ButtonInteraction::None)
        .insert(UiButton::Detach)
        .insert(ButtonState::Off)
        // .insert(DetachButton)
        .id();

    commands.entity(main_ui).push_children(&[button]);

    let shader_params_icon1 = ends_materials.add(BezierEndsMat {
        color: Color::hex("f6abb6").unwrap().into(),
        size: icon_size,
        ..Default::default()
    });

    let icon1 = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle_icon.clone(),
            transform: Transform::from_translation(Vec3::new(
                -button_width / 5.0,
                0.0,
                globals.z_pos.ui_button_icons,
            )),
            material: shader_params_icon1.clone(),
            ..Default::default()
        })
        // .insert(ButtonInteraction::None)
        .insert(Icon)
        .id();

    let shader_params_icon2 = ends_materials.add(BezierEndsMat {
        color: Color::hex("3da4ab").unwrap().into(),
        size: icon_size,
        ..Default::default()
    });

    let mut icon2_transform = Transform::from_translation(Vec3::new(
        button_width / 5.0,
        0.0,
        globals.z_pos.ui_button_icons,
    ));
    icon2_transform.rotation = Quat::from_rotation_z(std::f32::consts::PI);
    let icon2 = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle_icon.clone(),
            transform: icon2_transform,
            material: shader_params_icon2,
            ..Default::default()
        })
        // .insert(ButtonInteraction::None)
        .insert(Icon)
        .id();

    commands.entity(button).push_children(&[icon1, icon2]);

    //     //
    //     //
    //     //
    //     ///////////////////// Spawn Curve button /////////////////////

    let shader_params_spawn = button_materials.add(ButtonMat {
        color: Color::hex("4a4e4d").unwrap().into(),
        size: button_size,
        ..Default::default()
    });

    let button = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle_button.clone(),
            material: shader_params_spawn,

            transform: Transform::from_translation(Vec3::new(
                button_width * 1.5,
                0.0,
                globals.z_pos.ui_buttons,
            )),
            ..Default::default()
        })
        .insert(ButtonInteraction::None)
        // .insert(shader_params_spawn.clone())
        .insert(UiButton::SpawnCurve)
        .insert(ButtonState::Off)
        .id();

    commands.entity(main_ui).push_children(&[button]);

    let mut icon1_transform = Transform::from_translation(Vec3::new(
        -button_width / 5.0,
        0.0,
        globals.z_pos.ui_button_icons,
    ));
    icon1_transform.rotation = Quat::from_rotation_z(std::f32::consts::PI);
    let icon1 = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle_icon.clone(),
            material: shader_params_icon1.clone(),

            transform: icon1_transform,
            ..Default::default()
        })
        // .insert(ButtonInteraction::None)
        .insert(Icon)
        .id();

    let icon2_transform = Transform::from_translation(Vec3::new(
        button_width / 5.0,
        0.0,
        globals.z_pos.ui_button_icons,
    ));
    // icon2_transform.rotation = Quat::from_rotation_z(std::f32::consts::PI);
    let icon2 = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle_icon.clone(),
            material: shader_params_icon1.clone(),
            transform: icon2_transform,
            ..Default::default()
        })
        // .insert(ButtonInteraction::None)
        // .insert(shader_params_icon1.clone())
        .insert(Icon)
        .id();

    commands.entity(button).push_children(&[icon1, icon2]);

    //     // //
    //     // //
    //     // //
    //     // ///////////////////// Selection button /////////////////////
    let shader_params_selection = button_materials.add(ButtonMat {
        color: Color::hex("4a4e4d").unwrap().into(),
        size: button_size,
        ..Default::default()
    });

    let selection_button = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle_button.clone(),

            material: shader_params_selection,
            transform: Transform::from_translation(Vec3::new(
                button_width * 1.5,
                -1.0 * button_width,
                globals.z_pos.ui_buttons,
            )),
            ..Default::default()
        })
        .insert(ButtonInteraction::None)
        // .insert(shader_params_selection.clone())
        .insert(UiButton::Selection)
        .insert(ButtonState::Off)
        .id();

    commands.entity(main_ui).push_children(&[selection_button]);

    let texture_handle = asset_server.load("textures/selection.png");
    let selection_sprite = commands
        .spawn_bundle(SpriteBundle {
            texture: texture_handle,

            transform: Transform::from_translation(Vec3::new(
                0.0,
                0.0,
                globals.z_pos.ui_button_icons,
            )),
            sprite: Sprite {
                custom_size: Some(button_size / 1.3),
                ..Default::default()
            },
            ..Default::default()
        })
        .id();

    commands
        .entity(selection_button)
        .push_children(&[selection_sprite]);

    //     //
    //     //
    //     //
    //     ///////////////////// Group button /////////////////////
    let shader_params_group = button_materials.add(ButtonMat {
        color: Color::hex("4a4e4d").unwrap().into(),
        size: button_size,
        ..Default::default()
    });

    let group_button = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle_button.clone(),
            transform: Transform::from_translation(Vec3::new(
                button_width * 1.5,
                -2.0 * button_width,
                globals.z_pos.ui_buttons,
            )),
            material: shader_params_group,
            ..Default::default()
        })
        .insert(ButtonInteraction::None)
        // .insert(shader_params_group.clone())
        .insert(UiButton::Group)
        .id();

    commands.entity(main_ui).push_children(&[group_button]);

    let texture_handle = asset_server.load("textures/selection.png");
    let selection_sprite = commands
        .spawn_bundle(SpriteBundle {
            texture: texture_handle,
            // mesh: mesh_handle_button.clone(),
            transform: Transform::from_translation(Vec3::new(
                0.0,
                0.0,
                globals.z_pos.ui_button_icons,
            )),
            // sprite: Sprite::new(button_size / 1.3),
            sprite: Sprite {
                custom_size: Some(button_size / 1.3),
                ..Default::default()
            },
            ..Default::default()
        })
        .id();

    let icon1_transform = Transform::from_translation(Vec3::new(
        -button_width / 8.0,
        0.0,
        globals.z_pos.ui_button_icons + 0.01,
    ));
    // icon1_transform.rotation = Quat::from_rotation_z(std::f32::consts::PI);
    let icon1 = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle_icon.clone(),
            material: shader_params_icon1.clone(),
            transform: icon1_transform,
            ..Default::default()
        })
        // .insert(ButtonInteraction::None)
        // .insert(shader_params_icon1.clone())
        .insert(Icon)
        .id();

    let mut icon2_transform = Transform::from_translation(Vec3::new(
        button_width / 8.0,
        0.0,
        globals.z_pos.ui_button_icons + 0.01,
    ));
    icon2_transform.rotation = Quat::from_rotation_z(std::f32::consts::PI);
    let icon2 = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle_icon.clone(),
            material: shader_params_icon1.clone(),
            transform: icon2_transform,
            ..Default::default()
        })
        // .insert(ButtonInteraction::None)
        // .insert(shader_params_icon2.clone())
        .insert(Icon)
        .id();

    // commands.entity(button).push_children(&[icon1, icon2]);
    commands
        .entity(group_button)
        .push_children(&[selection_sprite, icon1, icon2]);

    //     //
    //     //
    //     //
    //     ///////////////////// Save button /////////////////////
    let shader_params_save = button_materials.add(ButtonMat {
        color: Color::hex("4a4e4d").unwrap().into(),
        size: button_size,
        ..Default::default()
    });
    let save_button = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle_button.clone(),

            material: shader_params_save,
            transform: Transform::from_translation(Vec3::new(
                -button_width * 1.5,
                button_width,
                globals.z_pos.ui_buttons,
            )),
            ..Default::default()
        })
        .insert(ButtonInteraction::None)
        // .insert(shader_params_save.clone())
        .insert(UiButton::Save)
        .id();

    commands.entity(main_ui).push_children(&[save_button]);

    let texture_handle = asset_server.load("textures/save.png");
    let save_sprite = commands
        .spawn_bundle(SpriteBundle {
            texture: texture_handle,
            // mesh: mesh_handle_button.clone(),
            transform: Transform::from_translation(Vec3::new(
                0.0,
                0.0,
                globals.z_pos.ui_button_icons,
            )),
            sprite: Sprite {
                custom_size: Some(button_size / 1.3),
                ..Default::default()
            },
            ..Default::default()
        })
        .id();

    commands.entity(save_button).push_children(&[save_sprite]);

    //     //
    //     //
    //     //
    //     ///////////////////// Load button /////////////////////
    let shader_params_load = button_materials.add(ButtonMat {
        color: Color::hex("4a4e4d").unwrap().into(),
        size: button_size,
        ..Default::default()
    });
    let load_button = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle_button.clone(),
            material: shader_params_load,
            transform: Transform::from_translation(Vec3::new(
                -button_width * 0.5,
                button_width,
                globals.z_pos.ui_buttons,
            )),
            ..Default::default()
        })
        .insert(ButtonInteraction::None)
        // .insert(shader_params_load.clone())
        .insert(UiButton::Load)
        .id();

    commands.entity(main_ui).push_children(&[load_button]);

    let texture_handle = asset_server.load("textures/load.png");
    let load_sprite = commands
        .spawn_bundle(SpriteBundle {
            texture: texture_handle,
            // mesh: mesh_handle_button.clone(),
            transform: Transform::from_translation(Vec3::new(
                0.0,
                0.0,
                globals.z_pos.ui_button_icons,
            )),
            sprite: Sprite {
                custom_size: Some(button_size / 1.3),
                ..Default::default()
            },
            ..Default::default()
        })
        .id();

    commands.entity(load_button).push_children(&[load_sprite]);

    //     //
    //     //
    //     //
    //     //
    //     ///////////////////// lut button /////////////////////
    let shader_params_lut = button_materials.add(ButtonMat {
        color: Color::hex("4a4e4d").unwrap().into(),
        size: button_size,
        ..Default::default()
    });
    let lut_button = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle_button.clone(),
            material: shader_params_lut,
            transform: Transform::from_translation(Vec3::new(
                button_width * 1.5,
                button_width,
                globals.z_pos.ui_buttons,
            )),
            ..Default::default()
        })
        .insert(ButtonInteraction::None)
        // .insert(shader_params_lut.clone())
        .insert(UiButton::Lut)
        .id();

    commands.entity(main_ui).push_children(&[lut_button]);

    let texture_handle = asset_server.load("textures/lut.png");
    let lut_sprite = commands
        .spawn_bundle(SpriteBundle {
            texture: texture_handle,
            // mesh: mesh_handle_button.clone(),
            transform: Transform::from_translation(Vec3::new(
                0.0,
                0.0,
                globals.z_pos.ui_button_icons,
            )),
            sprite: Sprite {
                custom_size: Some(button_size / 1.3),
                ..Default::default()
            },
            ..Default::default()
        })
        .id();

    commands.entity(lut_button).push_children(&[lut_sprite]);

    //     //
    //     //
    //     //
    //     ///////////////////// undo button /////////////////////
    // let shader_params_undo = my_shader_params.add(MyShader {
    //     color: Color::hex("4a4e4d").unwrap(),
    //     size: button_size,
    //     ..Default::default()
    // });
    // let undo_button = commands
    //     .spawn_bundle(MaterialMesh2dBundle {
    //         mesh: mesh_handle_button.clone(),
    //         visible: visible_ui.clone(),
    //         render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
    //             button_pipeline_handle.clone(),
    //         )]),
    //         transform: Transform::from_translation(Vec3::new(
    //             -button_width * 1.5,
    //             -button_width,
    //             globals.z_pos.ui_buttons,
    //         )),
    //         ..Default::default()
    //     })
    //     .insert(ButtonInteraction::None)
    //     .insert(shader_params_undo.clone())
    //     .insert(UiButton::Undo)
    //     .id();

    // commands.entity(main_ui).push_children(&[undo_button]);

    // let texture_handle = asset_server.load("textures/undo.png");
    // let undo_sprite = commands
    //     .spawn_bundle(SpriteBundle {
    //         material: materials.add(texture_handle.into()),
    //         // mesh: mesh_handle_button.clone(),
    //         transform: Transform::from_translation(Vec3::new(0.0, 0.0, globals.z_pos.ui_button_icons)),
    //         sprite: Sprite::new(button_size / 1.3),
    //         ..Default::default()
    //     })
    //     .id();

    // commands.entity(undo_button).push_children(&[undo_sprite]);

    //     //
    //     //
    //     //
    //     ///////////////////// redo button /////////////////////
    // let shader_params_redo = my_shader_params.add(MyShader {
    //     color: Color::hex("4a4e4d").unwrap(),
    //     size: button_size,
    //     ..Default::default()
    // });
    // let redo_button = commands
    //     .spawn_bundle(MaterialMesh2dBundle {
    //         mesh: mesh_handle_button.clone(),
    //         visible: visible_ui.clone(),
    //         render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
    //             button_pipeline_handle.clone(),
    //         )]),
    //         transform: Transform::from_translation(Vec3::new(
    //             -button_width * 0.5,
    //             -button_width,
    //             globals.z_pos.ui_buttons,
    //         )),
    //         ..Default::default()
    //     })
    //     .insert(ButtonInteraction::None)
    //     .insert(shader_params_redo.clone())
    //     .insert(UiButton::Redo)
    //     .id();

    // commands.entity(main_ui).push_children(&[redo_button]);

    // let texture_handle = asset_server.load("textures/redo.png");
    // let redo_sprite = commands
    //     .spawn_bundle(SpriteBundle {
    //         material: materials.add(texture_handle.into()),
    //         // mesh: mesh_handle_button.clone(),
    //         transform: Transform::from_translation(Vec3::new(0.0, 0.0, globals.z_pos.ui_button_icons)),
    //         sprite: Sprite::new(button_size / 1.3),
    //         ..Default::default()
    //     })
    //     .id();

    // commands.entity(redo_button).push_children(&[redo_sprite]);

    //
    //
    //
    //     ///////////////////// hide button /////////////////////
    let shader_params_hide = button_materials.add(ButtonMat {
        color: Color::hex("4a4e4d").unwrap().into(),
        size: button_size,
        ..Default::default()
    });
    let on_material_show = asset_server.load("textures/show_anchors.png");
    let off_material_hide = asset_server.load("textures/hide.png");
    let hide_button = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle_button.clone(),
            material: shader_params_hide,
            transform: Transform::from_translation(Vec3::new(
                -button_width * 0.5,
                -button_width,
                globals.z_pos.ui_buttons,
            )),
            ..Default::default()
        })
        .insert(ButtonInteraction::None)
        // .insert(shader_params_hide.clone())
        .insert(UiButton::Hide)
        .id();

    commands.entity(main_ui).push_children(&[hide_button]);

    // let texture_handle = asset_server.load("textures/hide.png");
    let hide_sprite = commands
        .spawn_bundle(SpriteBundle {
            texture: on_material_show,
            // mesh: mesh_handle_button.clone(),
            transform: Transform::from_translation(Vec3::new(
                0.0,
                0.0,
                globals.z_pos.ui_button_icons,
            )),
            sprite: Sprite {
                custom_size: Some(button_size / 1.3),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(OnOffMaterial {
            material: off_material_hide,
        })
        .insert(UiButton::Hide)
        .id();

    commands.entity(hide_button).push_children(&[hide_sprite]);

    //     //
    //     //
    //     //
    //     ///////////////////// controls button /////////////////////
    let shader_params_controls = button_materials.add(ButtonMat {
        color: Color::hex("4a4e4d").unwrap().into(),
        size: button_size,
        ..Default::default()
    });
    let controls_button = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle_button.clone(),
            material: shader_params_controls,
            transform: Transform::from_translation(Vec3::new(
                -button_width * 1.5,
                -button_width,
                globals.z_pos.ui_buttons,
            )),
            ..Default::default()
        })
        .insert(ButtonInteraction::None)
        // .insert(shader_params_controls.clone())
        .insert(UiButton::HideControls)
        .id();

    commands.entity(main_ui).push_children(&[controls_button]);

    let on_material = asset_server.load("textures/controls_on.png");
    let off_material = asset_server.load("textures/controls_off.png");
    let controls_sprite = commands
        .spawn_bundle(SpriteBundle {
            texture: on_material,

            transform: Transform::from_translation(Vec3::new(
                0.0,
                0.0,
                globals.z_pos.ui_button_icons,
            )),
            sprite: Sprite {
                custom_size: Some(button_size / 1.3),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(UiButton::HideControls)
        .insert(OnOffMaterial {
            material: off_material,
        })
        .id();

    commands
        .entity(controls_button)
        .push_children(&[controls_sprite]);

    //
    //
    //
    //     ///////////////////// sound button /////////////////////
    let shader_params_sound = button_materials.add(ButtonMat {
        color: Color::hex("4a4e4d").unwrap().into(),
        size: button_size,
        ..Default::default()
    });
    let sound_button = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle_button.clone(),
            material: shader_params_sound,
            transform: Transform::from_translation(Vec3::new(
                button_width * -1.5,
                2.0 * button_width,
                globals.z_pos.ui_buttons,
            )),
            ..Default::default()
        })
        .insert(ButtonInteraction::None)
        // .insert(shader_params_sound.clone())
        .insert(UiButton::Sound)
        .id();

    commands.entity(main_ui).push_children(&[sound_button]);

    let on_material = asset_server.load("textures/sound_on.png");
    let off_material = asset_server.load("textures/sound_off.png");
    let sound_sprite = commands
        .spawn_bundle(SpriteBundle {
            texture: on_material,
            transform: Transform::from_translation(Vec3::new(
                0.0,
                0.0,
                globals.z_pos.ui_button_icons,
            )),
            sprite: Sprite {
                custom_size: Some(button_size / 1.3),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(UiButton::Sound)
        .insert(OnOffMaterial {
            material: off_material,
        })
        .id();

    commands.entity(sound_button).push_children(&[sound_sprite]);

    //     //
    //     //
    //     //
    //     ///////////////////// scale up button /////////////////////
    let shader_params_scale_up = button_materials.add(ButtonMat {
        color: Color::hex("4a4e4d").unwrap().into(),
        size: button_size,
        ..Default::default()
    });
    let scale_up_button = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle_button.clone(),
            material: shader_params_scale_up,
            transform: Transform::from_translation(Vec3::new(
                -button_width * 1.5,
                0.0,
                globals.z_pos.ui_buttons,
            )),
            ..Default::default()
        })
        .insert(ButtonInteraction::None)
        // .insert(shader_params_scale_up.clone())
        .insert(UiButton::ScaleUp)
        .id();

    commands.entity(main_ui).push_children(&[scale_up_button]);

    let scale_up_material = asset_server.load("textures/scale_up.png");
    let scale_up_sprite = commands
        .spawn_bundle(SpriteBundle {
            texture: scale_up_material,
            transform: Transform::from_translation(Vec3::new(
                0.0,
                0.0,
                globals.z_pos.ui_button_icons,
            )),
            sprite: Sprite {
                custom_size: Some(button_size / 1.3),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(UiButton::ScaleUp)
        .id();

    commands
        .entity(scale_up_button)
        .push_children(&[scale_up_sprite]);

    //
    //
    //
    ///////////////////// scale down button /////////////////////
    let shader_params_scale_down = button_materials.add(ButtonMat {
        color: Color::hex("4a4e4d").unwrap().into(),
        size: button_size,
        ..Default::default()
    });
    let scale_down_button = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle_button.clone(),
            material: shader_params_scale_down,
            transform: Transform::from_translation(Vec3::new(
                -button_width * 0.5,
                0.0,
                globals.z_pos.ui_buttons,
            )),
            ..Default::default()
        })
        .insert(ButtonInteraction::None)
        // .insert(shader_params_scale_down.clone())
        .insert(UiButton::ScaleDown)
        .id();

    commands.entity(main_ui).push_children(&[scale_down_button]);

    let scale_down_material = asset_server.load("textures/scale_down.png");
    let scale_down_sprite = commands
        .spawn_bundle(SpriteBundle {
            texture: scale_down_material,
            // mesh: mesh_handle_button.clone(),
            transform: Transform::from_translation(Vec3::new(
                0.0,
                0.0,
                globals.z_pos.ui_button_icons,
            )),
            sprite: Sprite {
                custom_size: Some(button_size / 1.3),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(UiButton::ScaleDown)
        .id();

    commands
        .entity(scale_down_button)
        .push_children(&[scale_down_sprite]);

    //     //
    //     //
    //     //
    //     ///////////////////// spawn delete button /////////////////////
    let shader_params_delete = button_materials.add(ButtonMat {
        color: Color::hex("4a4e4d").unwrap().into(),
        size: button_size,
        ..Default::default()
    });
    let delete_button = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle_button.clone(),
            material: shader_params_delete,
            transform: Transform::from_translation(Vec3::new(
                button_width * 0.5,
                button_width,
                globals.z_pos.ui_buttons,
            )),
            ..Default::default()
        })
        .insert(ButtonInteraction::None)
        // .insert(shader_params_delete.clone())
        .insert(UiButton::Delete)
        .id();

    commands.entity(main_ui).push_children(&[delete_button]);

    let delete_material = asset_server.load("textures/bin.png");
    let delete_sprite = commands
        .spawn_bundle(SpriteBundle {
            texture: delete_material,
            // mesh: mesh_handle_button.clone(),
            transform: Transform::from_translation(Vec3::new(
                0.0,
                0.0,
                globals.z_pos.ui_button_icons,
            )),
            sprite: Sprite {
                custom_size: Some(button_size / 1.3),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(UiButton::Delete)
        .id();

    commands
        .entity(delete_button)
        .push_children(&[delete_sprite]);

    //
    //
    //
    ///////////////////// make mesh button /////////////////////
    let shader_params_mesh = button_materials.add(ButtonMat {
        color: Color::hex("4a4e4d").unwrap().into(),
        size: button_size,
        ..Default::default()
    });
    let mesh_button = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle_button.clone(),
            material: shader_params_mesh,
            transform: Transform::from_translation(Vec3::new(
                -button_width * 1.5,
                -button_width * 2.0,
                globals.z_pos.ui_buttons,
            )),
            ..Default::default()
        })
        .insert(ButtonInteraction::None)
        // .insert(shader_params_mesh.clone())
        .insert(UiButton::MakeMesh)
        .id();

    commands.entity(main_ui).push_children(&[mesh_button]);

    let mesh_material = asset_server.load("textures/mesh.png");
    let mesh_sprite = commands
        .spawn_bundle(SpriteBundle {
            texture: mesh_material,
            // mesh: mesh_handle_button.clone(),
            transform: Transform::from_translation(Vec3::new(
                0.0,
                0.0,
                globals.z_pos.ui_button_icons,
            )),
            sprite: Sprite {
                custom_size: Some(button_size / 1.3),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(UiButton::MakeMesh)
        .id();

    commands.entity(mesh_button).push_children(&[mesh_sprite]);

    //     //
    //     //
    //     //
    //     ///////////////////// spawn heli button /////////////////////
    let shader_params_heli = button_materials.add(ButtonMat {
        color: Color::hex("4a4e4d").unwrap().into(),
        size: button_size,
        ..Default::default()
    });
    let heli_button = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle_button.clone(),
            material: shader_params_heli,
            transform: Transform::from_translation(Vec3::new(
                button_width * 0.5,
                -button_width * 2.0,
                globals.z_pos.ui_buttons,
            )),
            ..Default::default()
        })
        .insert(ButtonInteraction::None)
        // .insert(shader_params_heli.clone())
        .insert(UiButton::Helicopter)
        .id();

    commands.entity(main_ui).push_children(&[heli_button]);

    let heli_material = asset_server.load("textures/heli_button.png");
    let heli_sprite = commands
        .spawn_bundle(SpriteBundle {
            texture: heli_material,
            // mesh: mesh_handle_button.clone(),
            transform: Transform::from_translation(Vec3::new(
                0.0,
                0.0,
                globals.z_pos.ui_button_icons,
            )),
            sprite: Sprite {
                custom_size: Some(button_size / 1.3),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(UiButton::Helicopter)
        .id();

    commands.entity(heli_button).push_children(&[heli_sprite]);

    //
    //
    //
    ///////////////////// spawn road button /////////////////////
    let shader_params_road = button_materials.add(ButtonMat {
        color: Color::hex("4a4e4d").unwrap().into(),
        size: button_size,
        ..Default::default()
    });
    let road_button = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle_button.clone(),
            material: shader_params_road,
            transform: Transform::from_translation(Vec3::new(
                button_width * -0.5,
                -button_width * 2.0,
                globals.z_pos.ui_buttons,
            )),
            ..Default::default()
        })
        .insert(ButtonInteraction::None)
        // .insert(shader_params_road.clone())
        .insert(UiButton::SpawnRoad)
        .id();

    commands.entity(main_ui).push_children(&[road_button]);

    let road_material = asset_server.load("textures/road_icon.png");
    let road_sprite = commands
        .spawn_bundle(SpriteBundle {
            texture: road_material,
            // mesh: mesh_handle_button.clone(),
            transform: Transform::from_translation(Vec3::new(
                0.0,
                0.0,
                globals.z_pos.ui_button_icons,
            )),
            sprite: Sprite {
                custom_size: Some(button_size / 1.3),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(UiButton::SpawnRoad)
        .id();

    commands.entity(road_button).push_children(&[road_sprite]);

    //     /////////////////////// buttons ui ////////////////////////////

    //     //
    //     //
    //     //
    //     //
    //     //
    //     //////////////////////////// color ui /////////////////////////

    let color_ui_position = Vec3::new(0.0, -325.0 * globals.scale, globals.z_pos.ui_color_board);

    let shader_params_color_ui = ui_materials.add(UiMat {
        color: Color::hex("131B23").unwrap().into(),
        size: color_ui_size,
        ..Default::default()
    });
    let ui_transform = Transform::from_translation(color_ui_position);
    let parent = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: mesh_handle_color_ui.clone(),
            material: shader_params_color_ui,
            transform: ui_transform.clone(),
            ..Default::default()
        })
        // .insert(shader_params_color_ui)
        .insert(UiBoard {
            expanded: true,
            size: color_ui_size,
            position: color_ui_position.truncate(),
            previous_position: color_ui_position.truncate(),
            action: UiAction::None,
        })
        .id();

    commands.entity(main_ui).push_children(&[parent]);

    let color_button_size = Vec2::new(30. * globals.scale, 30. * globals.scale);
    let mesh_handle_color_button =
        bevy::sprite::Mesh2dHandle(meshes.add(Mesh::from(shape::Quad {
            size: color_button_size,
            flip: false,
        })));
    maps.mesh_handles
        .insert("color_button", mesh_handle_color_button.clone());

    for (j, color_vec) in colors.iter().enumerate() {
        for (k, color) in color_vec.iter().enumerate() {
            let mut t = 0.0;
            if k == 0 && j == 0 {
                t = 1.0;
            }

            let shader_params_handle_button = button_materials.add(ButtonMat {
                color: color.clone().into(),
                size: color_button_size,
                t,
                ..Default::default()
            });

            let button_transform = Transform::from_translation(Vec3::new(
                (k as f32 - 2.0) * 30.5,
                (j as f32 + 0.5 - (num_rows as f32) / 2.0) * 35.0,
                globals.z_pos.ui_color_buttons,
            ));

            let child = commands
                .spawn_bundle(MaterialMesh2dBundle {
                    mesh: mesh_handle_color_button.clone(),
                    material: shader_params_handle_button,
                    transform: button_transform.clone(),
                    ..Default::default()
                })
                // .insert(shader_params_handle_button)
                // TODO: remove this size field as the size is known in shader_params
                .insert(ColorButton { size: button_size })
                .id();

            commands.entity(parent).push_children(&[child]);
        }
    }
    //////////////////////////// color ui /////////////////////////
}
