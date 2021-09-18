use crate::util::MyShader;

use bevy::prelude::*;

#[derive(Clone, Copy, Debug)]
pub enum ButtonInteraction {
    Clicked,
    Pressed,
    Hovered,
    Released,
    None,
}

// type for buttons that keep their state
#[derive(PartialEq)]
pub enum ButtonState {
    On,
    Off,
}

#[derive(Clone, Copy, Debug, PartialEq)]
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

    let mut ui_button_that_was_turned_on = UiButton::Undo;
    for (interaction, shader_handle, ui_button, button_state_option) in interaction_query.iter_mut()
    {
        let mut shader_params = my_shader_params.get_mut(shader_handle).unwrap();

        match *interaction {
            ButtonInteraction::Released => {
                if let Some(mut button_state_mut) = button_state_option {
                    let button_state = button_state_mut.as_mut();
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
                    if let Some(mut button_state_mut) = button_state_option {
                        let button_state = button_state_mut.as_mut();
                        *button_state = ButtonState::Off;
                    }
                }
            }
            (true, true, false) => {
                if ui_button == &UiButton::Latch {
                    shader_params.t = 1.0;
                } else {
                    shader_params.t = 0.0;
                    if let Some(mut button_state_mut) = button_state_option {
                        let button_state = button_state_mut.as_mut();
                        *button_state = ButtonState::Off;
                    }
                }
            }
            (false, false, true) => {
                if ui_button == &UiButton::Detach {
                    shader_params.t = 1.0;
                } else {
                    shader_params.t = 0.0;
                    if let Some(mut button_state_mut) = button_state_option {
                        let button_state = button_state_mut.as_mut();
                        *button_state = ButtonState::Off;
                    }
                }
            }
            (false, true, false) => {
                if ui_button == &UiButton::Selection {
                    shader_params.t = 1.0;
                } else {
                    shader_params.t = 0.0;
                    if let Some(mut button_state_mut) = button_state_option {
                        let button_state = button_state_mut.as_mut();
                        *button_state = ButtonState::Off;
                    }
                }
            }
            _ => {}
        };

        if keyboard_input.just_released(KeyCode::LShift) {
            if ui_button == &UiButton::SpawnCurve || ui_button == &UiButton::Latch {
                shader_params.t = 0.0;
            }
        }
        if keyboard_input.just_released(KeyCode::LControl) {
            if ui_button == &UiButton::Selection || ui_button == &UiButton::Latch {
                shader_params.t = 0.0;
            }
        }
        if keyboard_input.just_released(KeyCode::Space) {
            if ui_button == &UiButton::Detach {
                shader_params.t = 0.0;
            }
        }
    }

    if turn_other_buttons_off {
        for (interaction, shader_handle, ui_button, button_state_option) in
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
