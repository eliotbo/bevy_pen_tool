use super::buttons::{ButtonInteraction, ButtonState, UiButton};
// use crate::cam::Cam;
use crate::util::{
    get_close_anchor, Anchor, AnchorEdge, Bezier, BoundingBoxQuad, ColorButton, Globals,
    GrandParent, MyShader, UiAction, UiBoard, UserState,
};

use bevy::render::camera::OrthographicProjection;
use bevy::{input::mouse::MouseWheel, prelude::*, window::CursorMoved};

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

#[derive(PartialEq)]
pub enum Action {
    Latch,
    Redo,
    Undo,
    Load,
    Save,
    Group,
    Select,
    Unselect,
    Detach,
    SpawnCurve,
    HideAnchors,
    ToggleSound,
    ScaleUp,
    ScaleDown,
    HideControls,
    ComputeLut,
    Delete,
    SelectionBox,
    Selected,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MoveAnchor {
    pub handle: Handle<Bezier>,
    pub anchor: Anchor,
    pub unlatch: bool,
}

pub fn send_action(
    mut ui_event_reader: EventReader<UiButton>,
    mut action_event_writer: EventWriter<Action>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
) {
    for ui_button in ui_event_reader.iter() {
        match ui_button {
            UiButton::Latch => action_event_writer.send(Action::Latch),
            UiButton::Redo => action_event_writer.send(Action::Redo),
            UiButton::Undo => action_event_writer.send(Action::Undo),
            UiButton::Load => action_event_writer.send(Action::Load),
            UiButton::Save => action_event_writer.send(Action::Save),
            UiButton::Group => action_event_writer.send(Action::Group),
            // is this correct?
            UiButton::Selection => {
                action_event_writer.send(Action::Select);
                // action_event_writer.send(Action::SelectionBox);
            }

            UiButton::Detach => action_event_writer.send(Action::Detach),
            // UiButton::SpawnCurve => action_event_writer.send(Action::SpawnCurve),
            UiButton::Hide => action_event_writer.send(Action::HideAnchors),
            UiButton::Sound => action_event_writer.send(Action::ToggleSound),
            UiButton::ScaleUp => action_event_writer.send(Action::ScaleUp),
            UiButton::ScaleDown => action_event_writer.send(Action::ScaleDown),
            UiButton::HideControls => action_event_writer.send(Action::HideControls),
            UiButton::Lut => action_event_writer.send(Action::ComputeLut),
            _ => {}
        }
    }

    let mouse_just_pressed = mouse_button_input.just_pressed(MouseButton::Left);
    let mouse_just_released = mouse_button_input.just_released(MouseButton::Left);
    let mouse_pressed = mouse_button_input.pressed(MouseButton::Left);
    let mut mouse_wheel_up = false;
    let mut mouse_wheel_down = false;
    if let Some(mouse_wheel) = mouse_wheel_events.iter().next() {
        if mouse_wheel.y > 0.5 {
            mouse_wheel_up = true;
        }
        if mouse_wheel.y < -0.5 {
            mouse_wheel_down = true;
        }
    }

    // only used for pattern matching
    // let _control_only = (false, true, false);
    let _pressed_g = keyboard_input.just_pressed(KeyCode::G);
    let _pressed_h = keyboard_input.just_pressed(KeyCode::H);
    let _pressed_s = keyboard_input.just_pressed(KeyCode::S);
    let _pressed_l = keyboard_input.just_pressed(KeyCode::L);
    let _pressed_z = keyboard_input.just_pressed(KeyCode::Z);
    let _pressed_t = keyboard_input.just_pressed(KeyCode::T);
    let _pressed_delete = keyboard_input.just_pressed(KeyCode::Delete);

    // match keys / mouse buttons / mouse wheel combination and send event to corresponding action
    match (
        keyboard_input.pressed(KeyCode::LShift),
        keyboard_input.pressed(KeyCode::LControl),
        keyboard_input.pressed(KeyCode::Space),
    ) {
        (true, false, false) if mouse_just_pressed => action_event_writer.send(Action::SpawnCurve),
        (true, true, false) if mouse_pressed => action_event_writer.send(Action::Latch),
        (false, false, true) if mouse_just_pressed => action_event_writer.send(Action::Detach),
        (false, true, false) if mouse_just_pressed => {
            // action_event_writer.send(Action::Select);
            action_event_writer.send(Action::SelectionBox);
        }
        (false, true, false) if mouse_just_released => action_event_writer.send(Action::Selected),
        (false, true, false) if _pressed_g => action_event_writer.send(Action::Group),
        (false, true, false) if _pressed_h => action_event_writer.send(Action::HideAnchors),
        (true, true, false) if _pressed_h => action_event_writer.send(Action::HideControls),
        (false, true, false) if _pressed_s => action_event_writer.send(Action::Save),
        (false, true, false) if _pressed_l => action_event_writer.send(Action::Load),
        (false, true, false) if _pressed_z => action_event_writer.send(Action::Undo),
        (true, true, false) if _pressed_z => action_event_writer.send(Action::Redo),
        (false, true, false) if mouse_wheel_up => action_event_writer.send(Action::ScaleUp),
        (false, true, false) if mouse_wheel_down => action_event_writer.send(Action::ScaleDown),

        (false, false, false) if _pressed_delete => action_event_writer.send(Action::Delete),
        (true, false, false) if _pressed_t => action_event_writer.send(Action::ComputeLut),
        _ => {}
    }
}

pub fn record_mouse_events_system(
    mut cursor_moved_events: EventReader<CursorMoved>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut cursor_res: ResMut<Cursor>,
    mut windows: ResMut<Windows>,
    cam_transform_query: Query<&Transform, With<OrthographicProjection>>,
    cam_ortho_query: Query<&OrthographicProjection>,
) {
    for event in cursor_moved_events.iter() {
        let cursor_in_pixels = event.position; // lower left is origin
        let window_size = Vec2::new(
            windows.get_primary_mut().unwrap().width(),
            windows.get_primary_mut().unwrap().height(),
        );

        let screen_position = cursor_in_pixels - window_size / 2.0;

        let cam_transform = cam_transform_query.iter().next().unwrap();

        // this variable currently has no effect
        let mut scale = 1.0;

        for ortho in cam_ortho_query.iter() {
            scale = ortho.scale;
        }

        let cursor_vec4: Vec4 = cam_transform.compute_matrix()
            * screen_position.extend(0.0).extend(1.0 / (scale))
            * scale;

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
    my_shader_params: ResMut<Assets<MyShader>>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut query: Query<(
        &GlobalTransform,
        &Handle<MyShader>,
        &mut ButtonInteraction,
        &UiButton,
    )>,
    mut ui_query: Query<(&Transform, &mut UiBoard)>,
    globals: ResMut<Globals>,
) {
    for (global_transform, shader_handle, mut button_interaction, _ui_button) in query.iter_mut() {
        let shader_params = my_shader_params.get(shader_handle).unwrap().clone();

        // this looks incorrect, but it is due to buttons being children of the UI board
        let cam_scale = globals.scale * globals.scale;
        if cursor.within_rect(
            global_transform.translation.truncate(),
            shader_params.size * 0.95 * cam_scale,
        ) {
            let bi = button_interaction.deref_mut();
            *bi = ButtonInteraction::Hovered;

            // TODO: change to a match statement

            // Disallow the UI board to be dragged upon click
            if mouse_button_input.just_pressed(MouseButton::Left) {
                *bi = ButtonInteraction::Clicked;
                for (_t, mut ui_board) in ui_query.iter_mut() {
                    ui_board.action = UiAction::PressedUiButton;
                }
            }

            if mouse_button_input.pressed(MouseButton::Left) {
                *bi = ButtonInteraction::Pressed;
                for (_t, mut ui_board) in ui_query.iter_mut() {
                    ui_board.action = UiAction::PressedUiButton;
                }
            }

            if mouse_button_input.just_released(MouseButton::Left) {
                *bi = ButtonInteraction::Released;

                // button_interaction.set_changed(); // probably not necessary
                for (_t, mut ui_board) in ui_query.iter_mut() {
                    ui_board.action = UiAction::None;
                }
            }
        } else {
            let bi = button_interaction.deref_mut();
            *bi = ButtonInteraction::None;
        }
    }
}

// This is an action. It triggers upon left mouseclick
pub fn spawn_curve_order_on_mouseclick(
    keyboard_input: Res<Input<KeyCode>>,
    mut cursor: ResMut<Cursor>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    mouse_button_input: Res<Input<MouseButton>>,
    query: Query<&Handle<Bezier>, With<BoundingBoxQuad>>,
    ui_query: Query<&UiBoard>,
    globals: ResMut<Globals>,
    mut event_writer: EventWriter<Latch>,
    button_query: Query<(&ButtonState, &UiButton)>,
    mut user_state: ResMut<UserState>,
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

        // println!("ui_action: {:?}", ui_action);

        if !ui_action
            && ((keyboard_input.pressed(KeyCode::LShift)
                && !keyboard_input.pressed(KeyCode::LControl)
                && !keyboard_input.pressed(KeyCode::Space))
                || spawn_button_on)
        {
            //TODO: use event instead
            // globals.do_spawn_curve = true;

            let us = user_state.as_mut();
            *us = UserState::SpawningCurve;

            // Check for latching on nearby curve endings
            for bezier_handle in query.iter() {
                //
                if let Some(bezier) = bezier_curves.get_mut(bezier_handle) {
                    //
                    let max_click_distance = 5.0 * globals.scale;

                    let start_close_enough =
                        (bezier.positions.start - cursor.position).length() < max_click_distance;
                    let end_close_enough =
                        (bezier.positions.end - cursor.position).length() < max_click_distance;

                    if start_close_enough && !bezier.quad_is_latched(AnchorEdge::Start) {
                        //
                        bezier.send_latch_on_spawn(AnchorEdge::Start, &mut event_writer);
                        // println!("latched on start point");
                        break;
                    } else if end_close_enough && !bezier.quad_is_latched(AnchorEdge::End) {
                        //
                        bezier.send_latch_on_spawn(AnchorEdge::End, &mut event_writer);
                        // println!("latched on end point");
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

pub fn check_mouse_on_canvas(
    keyboard_input: Res<Input<KeyCode>>,
    mut cursor: ResMut<Cursor>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    mouse_button_input: Res<Input<MouseButton>>,
    query: Query<&Handle<Bezier>, With<BoundingBoxQuad>>,
    globals: ResMut<Globals>,
    mut move_event_writer: EventWriter<MoveAnchor>,
    mut action_event_writer: EventWriter<Action>,
    mut user_state: ResMut<UserState>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left)
        && !keyboard_input.pressed(KeyCode::LShift)
        && !keyboard_input.pressed(KeyCode::LControl)
        && !globals.do_hide_anchors
        && !(user_state.as_ref() == &UserState::SpawningCurve)
    {
        if let Some((_distance, anchor, handle)) = get_close_anchor(
            3.0 * globals.scale,
            cursor.position,
            &bezier_curves,
            &query,
            globals.scale,
        ) {
            let unlatch = !keyboard_input.pressed(KeyCode::LShift)
                && !keyboard_input.pressed(KeyCode::LControl)
                && keyboard_input.pressed(KeyCode::Space);

            let moving_anchor = MoveAnchor {
                handle,
                anchor,
                unlatch,
            };
            move_event_writer.send(moving_anchor.clone());

            let user_state = user_state.as_mut();
            *user_state = UserState::MovingAnchor(moving_anchor);
        } else {
            action_event_writer.send(Action::Unselect);
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
            }
        }

        // let user_state = user_state.as_mut();
        // *user_state = UserState::Idle;
    }
}

// TODO: refactor
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
}
