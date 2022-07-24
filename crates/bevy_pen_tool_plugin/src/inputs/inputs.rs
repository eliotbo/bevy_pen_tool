use super::buttons::{ButtonInteraction, ButtonState, UiButton};
// use crate::cam::Cam;
use crate::util::{
    get_close_anchor, get_close_still_anchor, Anchor, AnchorEdge, Bezier, BoundingBoxQuad,
    ButtonMat, ColorButton, Globals, GrandParent, OfficialLatch, UiAction, UiBoard, UserState,
};

use bevy::render::camera::OrthographicProjection;
use bevy::{input::mouse::MouseWheel, prelude::*, window::CursorMoved};

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

#[derive(PartialEq, Debug)]
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
    SpawnHeli,
    MakeMesh,
    SpawnRoad,
    StartMoveAnchor,
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
    mut button_query: Query<(&ButtonState, &UiButton)>,
) {
    // send Action event upon UI button press
    for ui_button in ui_event_reader.iter() {
        match ui_button {
            UiButton::Redo => action_event_writer.send(Action::Redo),
            UiButton::Undo => action_event_writer.send(Action::Undo),
            UiButton::Load => action_event_writer.send(Action::Load),
            UiButton::Save => action_event_writer.send(Action::Save),
            UiButton::Group => action_event_writer.send(Action::Group),
            UiButton::Hide => action_event_writer.send(Action::HideAnchors),
            UiButton::Sound => action_event_writer.send(Action::ToggleSound),
            UiButton::ScaleUp => action_event_writer.send(Action::ScaleUp),
            UiButton::ScaleDown => action_event_writer.send(Action::ScaleDown),
            UiButton::HideControls => action_event_writer.send(Action::HideControls),
            UiButton::Lut => action_event_writer.send(Action::ComputeLut),
            UiButton::Helicopter => action_event_writer.send(Action::SpawnHeli),
            UiButton::MakeMesh => action_event_writer.send(Action::MakeMesh),
            UiButton::SpawnRoad => action_event_writer.send(Action::SpawnRoad),
            UiButton::Delete => action_event_writer.send(Action::Delete),

            _ => {}
        }
    }

    // continuously send Latch events when latch button is On
    for (button_state, ui_button) in button_query.iter_mut() {
        if ui_button == &UiButton::Latch && button_state == &ButtonState::On {
            action_event_writer.send(Action::Latch)
        }
    }

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
        // (true, false, false) if mouse_just_pressed => action_event_writer.send(Action::SpawnCurve),
        (true, true, false) if mouse_pressed => action_event_writer.send(Action::Latch),
        // (false, false, true) if mouse_just_pressed => action_event_writer.send(Action::Detach),

        // TODO: move to mouseclick event router
        // (false, true, false) if mouse_just_pressed => {
        //     action_event_writer.send(Action::SelectionBox);
        // }
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

pub enum MouseClickEvent {
    OnUiBoard,
    OnColorButton((Color, Handle<ButtonMat>)),
    OnUiButton(UiButton),
    OnAnchor((Anchor, Handle<Bezier>, bool)), // the bool is for unlatching
    OnAnchorEdge((AnchorEdge, Handle<Bezier>)),
    SpawnOnBezier((AnchorEdge, Handle<Bezier>)),
    SpawnOnCanvas,
}

pub fn check_mouseclick_on_objects(
    cursor: ResMut<Cursor>,
    keyboard_input: Res<Input<KeyCode>>,
    my_shader_params: ResMut<Assets<ButtonMat>>,
    globals: ResMut<Globals>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut button_query: Query<(
        &ButtonState,
        &GlobalTransform,
        &Handle<ButtonMat>,
        &mut ButtonInteraction,
        &UiButton,
    )>,
    color_button_query: Query<(&GlobalTransform, &Handle<ButtonMat>, &ColorButton)>,
    mut ui_query: Query<(&Transform, &mut UiBoard), With<GrandParent>>,
    bezier_query: Query<(&Handle<Bezier>, &BoundingBoxQuad)>,
    bezier_curves: ResMut<Assets<Bezier>>,
    mut mouse_event_writer: EventWriter<MouseClickEvent>,
    mut action_event_writer: EventWriter<Action>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        let cam_scale = globals.scale * globals.scale;

        // TODO: too much boilerplate to check if a button is on...
        let mut spawn_button_on = false;
        let mut unlatch_button_on = false;
        let mut selection_button_on = false;
        for (button_state, _button_trans, _shader_handle, _interaction, ui_button) in
            button_query.iter_mut()
        {
            if ui_button == &UiButton::SpawnCurve {
                spawn_button_on = button_state == &ButtonState::On;
            } else if ui_button == &UiButton::Detach {
                unlatch_button_on = button_state == &ButtonState::On;
            } else if ui_button == &UiButton::Selection {
                selection_button_on = button_state == &ButtonState::On;
            }
        }

        // list of priorities from highest to lowest
        // 1. OnUiButton / OnColorButton
        // 2. OnUiBoard
        // 3. OnBezier / OnAnchorEdge

        //
        // check for mouseclick on UI buttons
        //
        // This block is useless other than to return () upon button press (no effects)
        for (_state, button_transform, shader_handle, mut _button_interaction, ui_button) in
            button_query.iter_mut()
        {
            //
            let shader_params = my_shader_params.get(shader_handle).unwrap().clone();
            //
            if cursor.within_rect(
                button_transform.translation().truncate(),
                shader_params.size * 0.95 * cam_scale,
            ) {
                // this sends into nothingness
                mouse_event_writer.send(MouseClickEvent::OnUiButton(ui_button.clone()));

                return ();
            }
        }

        //
        // check for mouseclick on color buttons
        for (transform, shader_param_handle, _color_button) in color_button_query.iter() {
            let shader_params = my_shader_params
                .get(&shader_param_handle.clone())
                .unwrap()
                .clone();

            if cursor.within_rect(
                transform.translation().truncate(),
                shader_params.size * 1.15 * cam_scale,
            ) {
                mouse_event_writer.send(MouseClickEvent::OnColorButton((
                    shader_params.color.clone().into(),
                    shader_param_handle.clone(),
                )));
                return ();
            }
        }

        //
        // check for mouseclick on UI Board
        for (ui_transform, mut ui_board) in ui_query.iter_mut() {
            if
            // ui_board.action == UiAction::None &&
            cursor.within_rect(
                ui_transform.translation.truncate(),
                ui_board.size * globals.scale,
            ) {
                mouse_event_writer.send(MouseClickEvent::OnUiBoard);
                ui_board.action = UiAction::MovingUi;
                return ();
            }
        }

        //
        // check for mouseclick on anchors (including control points)
        let mut anchor_event: Option<MouseClickEvent> = None;
        if let Some((_distance, anchor, handle)) = get_close_anchor(
            3.0 * globals.scale,
            cursor.position,
            &bezier_curves,
            &bezier_query,
            globals.scale,
        ) {
            anchor_event = Some(MouseClickEvent::OnAnchor((anchor, handle, false)));
        }

        //
        // check for mouseclick on anchors (excluding control points)
        let mut anchor_edge_event: Option<MouseClickEvent> = None;
        if let Some((_dist, anchor_edge, handle)) = get_close_still_anchor(
            3.0 * globals.scale,
            cursor.position,
            &bezier_curves,
            &bezier_query,
        ) {
            anchor_edge_event = Some(MouseClickEvent::OnAnchorEdge((anchor_edge, handle)));
        }

        match (
            anchor_event,
            anchor_edge_event,
            keyboard_input.pressed(KeyCode::LShift),
            keyboard_input.pressed(KeyCode::LControl),
            keyboard_input.pressed(KeyCode::Space),
        ) {
            // case of spawning a curve close to an anchor
            (_, Some(MouseClickEvent::OnAnchorEdge(info)), true, false, false) => {
                mouse_event_writer.send(MouseClickEvent::SpawnOnBezier(info));
            }

            // case of spawning a curve close to an anchor (with spawn button)
            (_, Some(MouseClickEvent::OnAnchorEdge(info)), false, false, false)
                if spawn_button_on =>
            {
                mouse_event_writer.send(MouseClickEvent::SpawnOnBezier(info));
            }

            // case of clicking on an anchor (higher priority)
            (Some(event), _, false, false, false)
                if !globals.do_hide_anchors
                    && !globals.hide_control_points
                    && !spawn_button_on
                    && !unlatch_button_on =>
            {
                mouse_event_writer.send(event);
            }

            // case of clicking on a control point (lower priority)
            (Some(_event), Some(MouseClickEvent::OnAnchorEdge(info)), false, false, false)
                if !globals.do_hide_anchors && !spawn_button_on && !unlatch_button_on =>
            {
                mouse_event_writer.send(MouseClickEvent::OnAnchor((
                    info.0.to_anchor(),
                    info.1,
                    false,
                )));
            }

            // case of clicking on an anchor and unlatching
            (_, Some(MouseClickEvent::OnAnchorEdge(info)), false, false, true)
                if !globals.do_hide_anchors =>
            {
                mouse_event_writer.send(MouseClickEvent::OnAnchor((
                    info.0.to_anchor(),
                    info.1,
                    true,
                )));
            }

            // case of clicking on an anchor and unlatching with unlatch button
            (_, Some(MouseClickEvent::OnAnchorEdge(info)), false, false, false)
                if !globals.do_hide_anchors && unlatch_button_on =>
            {
                mouse_event_writer.send(MouseClickEvent::OnAnchor((
                    info.0.to_anchor(),
                    info.1,
                    true,
                )));
            }

            // case of spawning a curve away from any anchor
            (_, None, true, false, false) => {
                // mouse_event_writer.send(MouseClickEvent::OnCanvas(None))
                mouse_event_writer.send(MouseClickEvent::SpawnOnCanvas);
            }

            // case of spawning a curve away from any anchor, with spawn button on
            (_, None, false, false, false) if spawn_button_on => {
                mouse_event_writer.send(MouseClickEvent::SpawnOnCanvas);
                // mouse_event_writer.send(MouseClickEvent::OnCanvas(None));
            }

            (None, None, false, true, false) if !spawn_button_on => {
                action_event_writer.send(Action::SelectionBox);
            }

            (None, None, false, false, false) if !spawn_button_on && selection_button_on => {
                action_event_writer.send(Action::SelectionBox);
            }

            (None, None, false, false, false) if !spawn_button_on => {
                // println!("Debug");
                action_event_writer.send(Action::Unselect);
            }

            _ => {}
        }
    }
}

pub fn pick_color(
    mut my_shader_params: ResMut<Assets<ButtonMat>>,
    query: Query<(&GlobalTransform, &Handle<ButtonMat>, &ColorButton)>,
    mut ui_query: Query<(&Transform, &mut UiBoard), With<GrandParent>>,
    mut globals: ResMut<Globals>,
    mut mouse_event_reader: EventReader<MouseClickEvent>,
) {
    // if mouse_button_input.just_pressed(MouseButton::Left) {

    if let Some(MouseClickEvent::OnColorButton((color, shader_param_handle))) =
        mouse_event_reader.iter().next()
    {
        let (_ui_transform, mut ui_board) = ui_query.single_mut();
        globals.picked_color = Some(color.clone());

        ui_board.action = UiAction::PickingColor;

        // This loops over all colors to deselect them. A more efficient way of deselecting
        // would be to store the color and the handle of the selected color as well
        for (_transform, other_shader_param_handle, _color_button) in query.iter() {
            //
            let mut shader_params = my_shader_params.get_mut(other_shader_param_handle).unwrap();
            shader_params.t = 0.0;
        }
        // send selected color to shaders so that it shows the selected color with a white contour
        let mut shader_params = my_shader_params.get_mut(shader_param_handle).unwrap();
        shader_params.t = 1.0;

        // if ui_board.action == UiAction::None
        //     && cursor.within_rect(
        //         ui_transform.translation.truncate(),
        //         ui_board.size * globals.scale,
        //     )
        // {
        //     ui_board.action = UiAction::MovingUi;
        // }
    }
}

pub fn spawn_curve_order_on_mouseclick(
    mut bezier_curves: ResMut<Assets<Bezier>>,
    mut event_writer: EventWriter<Latch>,
    mut user_state: ResMut<UserState>,
    mut mouse_event_reader: EventReader<MouseClickEvent>,
) {
    let click_event = mouse_event_reader.iter().next();

    match click_event {
        Some(MouseClickEvent::SpawnOnBezier((anchor_edge, handle))) => {
            // this is too stateful, be more functional please. events please.
            let us = user_state.as_mut();
            *us = UserState::SpawningCurve;
            //
            if let Some(bezier) = bezier_curves.get_mut(handle) {
                //
                bezier.send_latch_on_spawn(*anchor_edge, &mut event_writer);
            }
        }
        Some(MouseClickEvent::SpawnOnCanvas) => {
            let us = user_state.as_mut();
            *us = UserState::SpawningCurve;
        }
        _ => {}
    }
}

pub fn check_mouse_on_canvas(
    mut move_event_writer: EventWriter<MoveAnchor>,
    mut user_state: ResMut<UserState>,
    mut mouse_event_reader: EventReader<MouseClickEvent>,
) {
    let click_event = mouse_event_reader.iter().next();

    match click_event {
        Some(MouseClickEvent::OnAnchor((anchor, handle, unlatch))) => {
            let moving_anchor = MoveAnchor {
                handle: handle.clone(),
                anchor: anchor.clone(),
                unlatch: *unlatch,
            };
            // passing anchor data to a MoveAnchor event
            move_event_writer.send(moving_anchor.clone());

            let user_state = user_state.as_mut();
            *user_state = UserState::MovingAnchor;
        }

        _ => {}
    }
}

pub fn mouse_release_actions(
    mouse_button_input: Res<Input<MouseButton>>,
    mut bezier_curves: ResMut<Assets<Bezier>>,
    query: Query<(&Handle<Bezier>, &BoundingBoxQuad)>,
    mut ui_query: Query<(&mut Transform, &mut UiBoard), With<GrandParent>>,
    mut user_state: ResMut<UserState>,
    mut cursor: ResMut<Cursor>,
    mut action_event_writer: EventWriter<Action>,
    mut latch_event_writer: EventWriter<OfficialLatch>,
) {
    if mouse_button_input.just_released(MouseButton::Left) {
        cursor.latch = Vec::new();
        let user_state = user_state.as_mut();

        match user_state {
            UserState::Selected(_) | UserState::Selecting(_) => {}
            _ => {
                *user_state = UserState::Idle;
            }
        }

        // let go of all any moving quad upon mouse button release
        for (bezier_handle, _unused) in query.iter() {
            //
            if let Some(mut bezier) = bezier_curves.get_mut(bezier_handle) {
                //
                if let Some(potential_latch) = bezier.potential_latch.clone() {
                    latch_event_writer.send(OfficialLatch(potential_latch, bezier_handle.clone()));
                }
                bezier.potential_latch = None;
                bezier.move_quad = Anchor::None;
            }
        }

        // let go of UiBoard if moving
        for (transform, mut ui_board) in ui_query.iter_mut() {
            ui_board.action = UiAction::None;
            ui_board.previous_position = transform.translation.truncate();
        }

        action_event_writer.send(Action::Selected)
    }
}
