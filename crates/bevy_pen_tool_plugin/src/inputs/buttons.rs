use crate::util::{Globals, MyShader, OnOffMaterial};

use bevy::prelude::*;

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
}

pub fn button_system(
    mut my_shader_params: ResMut<Assets<MyShader>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut interaction_query: Query<
        (
            &ButtonInteraction,
            &Handle<MyShader>,
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
        let mut shader_params = my_shader_params.get_mut(shader_handle).unwrap();

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

        // TODO: send events, replacing the chunky if statements in actions.rs
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
                    } else {
                        shader_params.t = 0.0;
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
                    let mut shader_params = my_shader_params.get_mut(shader_handle).unwrap();
                    shader_params.t = 0.0;
                }
            }
        }
    }
}

pub fn toggle_ui_button(
    // asset_server: Res<AssetServer>,
    mut globals: ResMut<Globals>,
    // mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<(&mut Handle<ColorMaterial>, &mut OnOffMaterial, &UiButton)>,
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
                use std::ops::DerefMut;
                let other_material = on_off_mat.material.clone();
                let current_material = material_handle.clone();
                let mat = material_handle.deref_mut();
                *mat = other_material.clone();
                on_off_mat.material = current_material;
            }
        }
    }
}
