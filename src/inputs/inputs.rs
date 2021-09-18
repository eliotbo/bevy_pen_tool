use super::buttons::{ButtonInteraction, ButtonState, UiButton};
use crate::cam::Cam;
use crate::util::{AnchorEdge, Bezier, BoundingBoxQuad, Globals, MyShader, UiAction, UiBoard};

use bevy::{prelude::*, window::CursorMoved};

use std::ops::DerefMut;

pub struct Cursor {
    pub position: Vec2,
    pub pos_relative_to_click: Vec2,
    pub last_click_position: Vec2,
    pub latch: Vec<Latch>,
}

impl Default for Cursor {
    fn default() -> Self {
        Cursor {
            position: Vec2::ZERO,
            pos_relative_to_click: Vec2::ZERO,
            last_click_position: Vec2::ZERO,
            latch: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Latch {
    pub position: Vec2,
    pub control_point: Vec2,
    pub latchee_id: u128,
    pub latcher_id: u128,
    pub latchee_edge: AnchorEdge,
}

impl Cursor {
    pub fn within_rect(&self, position: Vec2, size: Vec2) -> bool {
        if self.position.x < position.x + size.x / 2.0
            && self.position.x > position.x - size.x / 2.0
            && self.position.y < position.y + size.y / 2.0
            && self.position.y > position.y - size.y / 2.0
        {
            return true;
        }
        return false;
    }
}

pub fn record_mouse_events_system(
    mut cursor_moved_events: EventReader<CursorMoved>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut cursor_res: ResMut<Cursor>,
    mut windows: ResMut<Windows>,
    // mut cam_query: Query<(&Cam, &Transform)>,
    cam_transform_query: Query<&Transform, With<Cam>>,
    globals: Res<Globals>,
) {
    for event in cursor_moved_events.iter() {
        let cursor_in_pixels = event.position; // lower left is origin
        let window_size = Vec2::new(
            windows.get_primary_mut().unwrap().width(),
            windows.get_primary_mut().unwrap().height(),
        );

        let screen_position = cursor_in_pixels - window_size / 2.0;

        let cam_transform = cam_transform_query.iter().next().unwrap();
        let cursor_vec4: Vec4 = cam_transform.compute_matrix()
            * screen_position
                .extend(0.0)
                .extend(1.0 / globals.camera_scale)
            * globals.camera_scale;

        let cursor_pos = Vec2::new(cursor_vec4.x, cursor_vec4.y);
        cursor_res.position = cursor_pos;
        cursor_res.pos_relative_to_click = cursor_res.position - cursor_res.last_click_position;
    }

    if mouse_button_input.just_pressed(MouseButton::Left) {
        cursor_res.last_click_position = cursor_res.position;
        cursor_res.pos_relative_to_click = Vec2::ZERO;
    }
}

pub fn check_mouse_on_ui(
    cursor: ResMut<Cursor>,
    mut my_shader_params: ResMut<Assets<MyShader>>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut query: Query<(
        &GlobalTransform,
        &Handle<MyShader>,
        &mut ButtonInteraction,
        &UiButton,
    )>,
    mut ui_query: Query<(&Transform, &mut UiBoard)>,
    mut globals: ResMut<Globals>,
) {
    for (global_transform, shader_handle, mut button_interaction, ui_button) in query.iter_mut() {
        let shader_params = my_shader_params.get(shader_handle).unwrap().clone();
        // TODO: fix scales
        let cam_scale = globals.camera_scale / 0.15;
        if cursor.within_rect(
            global_transform.translation.truncate(),
            shader_params.size * 0.95 * cam_scale,
        ) {
            let bi = button_interaction.deref_mut();
            *bi = ButtonInteraction::Hovered;

            // Disallow the UI board to be dragged upon click
            if mouse_button_input.just_pressed(MouseButton::Left) {
                for (_t, mut ui_board) in ui_query.iter_mut() {
                    ui_board.action = UiAction::PressedUiButton;
                }
            }

            if mouse_button_input.just_released(MouseButton::Left) {
                *bi = ButtonInteraction::Clicked;

                button_interaction.set_changed(); // probably not necessary
            }
        } else {
            let bi = button_interaction.deref_mut();
            *bi = ButtonInteraction::None;
        }
    }
}

pub fn spawn_curve_order_on_mouseclick(
    keyboard_input: Res<Input<KeyCode>>,
    mut cursor: ResMut<Cursor>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    mouse_button_input: Res<Input<MouseButton>>,
    query: Query<&Handle<Bezier>, With<BoundingBoxQuad>>,
    ui_query: Query<&UiBoard>,
    mut globals: ResMut<Globals>,
    mut event_writer: EventWriter<Latch>,
    button_query: Query<(&ButtonState, &UiButton)>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        let mut ui_action = false;
        for ui_board in ui_query.iter() {
            if ui_board.action != UiAction::None {
                ui_action = true;
                break;
            }
        }

        let mut spawn_button_on = false;
        for (button_state, ui_button) in button_query.iter() {
            if ui_button == &UiButton::SpawnCurve {
                spawn_button_on = button_state == &ButtonState::On;
            }
        }

        if !ui_action
            && ((keyboard_input.pressed(KeyCode::LShift)
                && !keyboard_input.pressed(KeyCode::LControl)
                && !keyboard_input.pressed(KeyCode::Space))
                || spawn_button_on)
        {
            //TODO: use event instead
            globals.do_spawn_curve = true;

            // Check for latching on nearby curve endings
            for bezier_handle in query.iter() {
                //
                if let Some(bezier) = bezier_curves.get_mut(bezier_handle) {
                    //
                    let max_click_distance = 5.0;

                    let start_close_enough =
                        (bezier.positions.start - cursor.position).length() < max_click_distance;
                    let end_close_enough =
                        (bezier.positions.end - cursor.position).length() < max_click_distance;

                    if start_close_enough && !bezier.quad_is_latched(AnchorEdge::Start) {
                        // bezier.send_latch_on_spawn(AnchorEdge::Start, &mut cursor);
                        bezier.send_latch_on_spawn(AnchorEdge::Start, &mut event_writer);
                        println!("latched on start point");
                        break;
                    } else if end_close_enough && !bezier.quad_is_latched(AnchorEdge::End) {
                        // bezier.send_latch_on_spawn(AnchorEdge::End, &mut cursor);
                        bezier.send_latch_on_spawn(AnchorEdge::End, &mut event_writer);
                        println!("latched on end point");
                        break;
                    }
                }
            }
        }
    }

    if mouse_button_input.just_released(MouseButton::Left) {
        cursor.latch = Vec::new();
    }
}
