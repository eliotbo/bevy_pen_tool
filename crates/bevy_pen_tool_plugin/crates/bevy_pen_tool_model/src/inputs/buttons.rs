use crate::inputs::{Cursor, MouseClickEvent};
use crate::materials::ButtonMat;
use crate::mesh::{FillMesh, FillMesh2dMaterial, RoadMesh, RoadMesh2dMaterial, StartMovingMesh};
use crate::model::{get_close_mesh, Globals, MainUi, OnOffMaterial, UiAction, UiBoard};

use bevy::prelude::*;

use std::ops::DerefMut;

#[derive(Clone, Copy, Debug, Component)]
pub enum ButtonInteraction {
    Clicked,
    Pressed,
    Hovered,
    Released,
    None,
}

// type for buttons that keep their state
#[derive(PartialEq, Component)]
pub enum ButtonState {
    On,
    Off,
}

#[derive(Clone, Copy, Debug, PartialEq, Component)]
pub enum UiButton {
    Latch,
    Redo,
    Undo,
    Load,
    Save,
    Group,
    Selection,
    Detach,
    SpawnCurve,
    Hide,
    Sound,
    ScaleUp,
    ScaleDown,
    HideControls,
    Lut,
    MakeMesh,
    Helicopter,
    SpawnRoad,
    Delete,
}

pub fn check_mouse_on_meshes(
    mut commands: Commands,
    cursor: ResMut<Cursor>,
    mouse_button_input: Res<Input<MouseButton>>,
    fill_query: Query<(Entity, &Transform, &Handle<FillMesh2dMaterial>, &FillMesh)>,
    road_query: Query<(Entity, &Transform, &Handle<RoadMesh2dMaterial>, &RoadMesh)>,
    mut fill_mesh_materials: ResMut<Assets<FillMesh2dMaterial>>,
    mut road_mesh_materials: ResMut<Assets<RoadMesh2dMaterial>>,
    mut start_moving_mesh_event: EventWriter<StartMovingMesh>,
) {
    //
    let max_dist = 20.;
    let maybe_mesh_id_translation = get_close_mesh(
        max_dist,
        &fill_query,
        &road_query,
        &mut fill_mesh_materials,
        &mut road_mesh_materials,
        cursor.position,
    );

    if let Some((entity, mesh_id, translation)) = maybe_mesh_id_translation {
        if mouse_button_input.just_pressed(MouseButton::Left) {
            // start_moving_mesh_event.send(StartMovingMesh {
            //     id: mesh_id,
            //     start_position: translation,
            //     entity: entity,
            // });
            commands.entity(entity).insert(StartMovingMesh {
                start_position: translation,
            });
            //     id: mesh_id,
            //     start_position: translation,
            //     entity: entity,
            // })
        }
    }
    // if mouse_button_input.just_pressed(MouseButton::Left) {

    //     start_moving_mesh_event.send(StartMovingMesh { id: mesh_id, start_position:  });
    // }
}

// tODO: move to moves
// system that moves the fill mesh
pub fn move_fill_mesh(
    cursor: ResMut<Cursor>,
    // mut start_moving_mesh_event_reader: EventReader<StartMovingMesh>,
    mut fill_query: Query<(
        &mut Transform,
        &StartMovingMesh,
        &Handle<FillMesh2dMaterial>,
    )>,
    mut fill_mesh_materials: ResMut<Assets<FillMesh2dMaterial>>,
    // mut road_query: Query<(&mut Transform, &RoadMesh)>,
    // mut fill_mesh_materials: ResMut<Assets<FillMesh2dMaterial>>,
    // mut road_mesh_materials: ResMut<Assets<RoadMesh2dMaterial>>,
) {
    for (mut transform, start_move, mesh_mat_handle) in fill_query.iter_mut() {
        //
        let new_pos = cursor.pos_relative_to_click + start_move.start_position;

        transform.translation = new_pos.extend(transform.translation.z);

        let mut mesh_mat = fill_mesh_materials.get_mut(mesh_mat_handle).unwrap();
        mesh_mat.center_of_mass = new_pos;
    }
}

pub fn check_mouse_on_ui(
    cursor: ResMut<Cursor>,
    button_shader_params: ResMut<Assets<ButtonMat>>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut mouse_event_writer: EventWriter<MouseClickEvent>,
    mut query: Query<(
        &GlobalTransform,
        &Handle<ButtonMat>,
        &mut ButtonInteraction,
        &UiButton,
    )>,
    mut ui_query: Query<(&Transform, &mut UiBoard), With<MainUi>>,

    globals: ResMut<Globals>,
) {
    for (ui_transform, mut ui_board) in ui_query.iter_mut() {
        //
        // if mouseclick is within the ui_board, check if it's on a button
        if cursor.within_rect(
            ui_transform.translation.truncate() / globals.scale,
            ui_board.size,
        ) {
            for (button_transform, shader_handle, mut button_interaction, _ui_button) in
                query.iter_mut()
            {
                let shader_params = button_shader_params.get(shader_handle).unwrap().clone();

                if cursor.within_rect(
                    button_transform.translation().truncate() / globals.scale,
                    shader_params.size * 0.95,
                ) {
                    let bi = button_interaction.deref_mut();
                    *bi = ButtonInteraction::Hovered;

                    // Disallow the UI board to be dragged upon click on button
                    if mouse_button_input.just_pressed(MouseButton::Left) {
                        *bi = ButtonInteraction::Clicked;
                        ui_board.action = UiAction::PressedUiButton;

                        return;
                    }

                    if mouse_button_input.pressed(MouseButton::Left) {
                        *bi = ButtonInteraction::Pressed;
                        return;
                    }

                    if mouse_button_input.just_released(MouseButton::Left) {
                        *bi = ButtonInteraction::Released;
                        ui_board.action = UiAction::None;
                        return;
                    }
                } else {
                    let bi = button_interaction.deref_mut();
                    *bi = ButtonInteraction::None;
                }
            }

            // // if there is a left mouseclick and none of the buttons have been pressed,
            // // send order to move the UI board
            // if mouse_button_input.just_pressed(MouseButton::Left) {
            //     mouse_event_writer.send(MouseClickEvent::OnUiBoard);
            //     ui_board.action = UiAction::MovingUi;
            // }
        }
    }
}

pub fn button_system(
    mut button_shader_params: ResMut<Assets<ButtonMat>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut interaction_query: Query<
        (
            &ButtonInteraction,
            &Handle<ButtonMat>,
            &UiButton,
            Option<&mut ButtonState>,
        ),
        (Changed<ButtonInteraction>,),
    >,
    mut event_writer: EventWriter<UiButton>,
) {
    let mut turn_other_buttons_off = false;

    // dummy value
    let mut ui_button_that_was_turned_on = UiButton::Undo;
    for (interaction, shader_handle, ui_button, button_state_option) in interaction_query.iter_mut()
    {
        let mut shader_params = button_shader_params.get_mut(shader_handle).unwrap();

        match *interaction {
            ButtonInteraction::Released => {
                if let Some(mut button_state_mut) = button_state_option {
                    let button_state = button_state_mut.as_mut();
                    // toggle button
                    if button_state == &ButtonState::On {
                        *button_state = ButtonState::Off;
                        shader_params.t = 0.0;
                    } else {
                        *button_state = ButtonState::On;
                        ui_button_that_was_turned_on = ui_button.clone();
                        turn_other_buttons_off = true;
                        shader_params.t = 1.0;
                    }

                    // two UI buttons cannot be clicked simultaneously
                    break;
                } else {
                    // send event to actions.rs
                    event_writer.send(*ui_button);
                }
            }
            ButtonInteraction::Pressed => {
                // send pressed event to shader
                shader_params.hovered = 0.8;
            }
            ButtonInteraction::Hovered => {
                shader_params.hovered = 1.0;
            }
            ButtonInteraction::Clicked => {}
            ButtonInteraction::None => {
                shader_params.hovered = 0.0;
            }
        }

        // TODO: receive MouseClickEvent
        if let Some(mut button_state_mut) = button_state_option {
            let button_state = button_state_mut.as_mut();
            match (
                keyboard_input.pressed(KeyCode::LShift),
                keyboard_input.pressed(KeyCode::LControl),
                keyboard_input.pressed(KeyCode::Space),
            ) {
                (true, false, false) => {
                    if ui_button == &UiButton::SpawnCurve {
                        shader_params.t = 1.0;
                        // shader_params.hovered = 0.95;
                    } else {
                        shader_params.t = 0.0;
                        // shader_params.hovered = 0.0;
                        *button_state = ButtonState::Off;
                    }
                }
                (true, true, false) => {
                    if ui_button == &UiButton::Latch {
                        shader_params.t = 1.0;
                    } else {
                        shader_params.t = 0.0;
                        *button_state = ButtonState::Off;
                    }
                }
                (false, false, true) => {
                    if ui_button == &UiButton::Detach {
                        shader_params.t = 1.0;
                    } else {
                        shader_params.t = 0.0;
                        *button_state = ButtonState::Off;
                        // }
                    }
                }
                (false, true, false) => {
                    if ui_button == &UiButton::Selection {
                        shader_params.t = 1.0;
                    } else {
                        shader_params.t = 0.0;
                        *button_state = ButtonState::Off;
                    }
                }
                _ => {}
            };

            if keyboard_input.just_released(KeyCode::LShift) {
                if ui_button == &UiButton::SpawnCurve || ui_button == &UiButton::Latch {
                    shader_params.t = 0.0;
                    *button_state = ButtonState::Off;
                }
            }
            if keyboard_input.just_released(KeyCode::LControl) {
                if ui_button == &UiButton::Selection || ui_button == &UiButton::Latch {
                    shader_params.t = 0.0;
                    *button_state = ButtonState::Off;
                }
            }
            if keyboard_input.just_released(KeyCode::Space) {
                if ui_button == &UiButton::Detach {
                    shader_params.t = 0.0;
                    *button_state = ButtonState::Off;
                }
            }
        }
    }

    if turn_other_buttons_off {
        for (_interaction, shader_handle, ui_button, button_state_option) in
            interaction_query.iter_mut()
        {
            if ui_button != &ui_button_that_was_turned_on {
                if let Some(mut button_state_mut) = button_state_option {
                    let button_state = button_state_mut.as_mut();
                    *button_state = ButtonState::Off;
                    let mut shader_params = button_shader_params.get_mut(shader_handle).unwrap();
                    shader_params.t = 0.0;
                }
            }
        }
    }
}

pub fn toggle_ui_button(
    mut globals: ResMut<Globals>,
    mut query: Query<(&mut Handle<Image>, &mut OnOffMaterial, &UiButton)>,
    mut event_reader: EventReader<UiButton>,
) {
    for ui_button in event_reader.iter() {
        //
        match ui_button {
            // TODO : move to actions
            &UiButton::Sound => {
                //
                globals.sound_on = !globals.sound_on;
                //
            }
            &UiButton::HideControls => {
                // globals.hide_control_points = !globals.hide_control_points;
            }
            _ => {}
        }
        for (mut material_handle, mut on_off_mat, ui_button_queried) in query.iter_mut() {
            // toggle sprite
            if ui_button == ui_button_queried {
                let other_material = on_off_mat.material.clone();
                let current_material = material_handle.clone();
                let mat = material_handle.deref_mut();
                *mat = other_material.clone();
                on_off_mat.material = current_material;
            }
        }
    }
}
