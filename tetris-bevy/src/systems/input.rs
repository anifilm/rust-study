use bevy::prelude::*;

use crate::game::{SoftDropState, TetrisSession, VisualEffectsState};
use crate::rotation::RotationDirection;

#[derive(Resource)]
pub struct DasState {
    left_hold: f32,
    right_hold: f32,
}

impl Default for DasState {
    fn default() -> Self {
        Self::new()
    }
}

impl DasState {
    const INITIAL_DELAY: f32 = 0.14;
    const REPEAT_RATE: f32 = 0.05;

    pub fn new() -> Self {
        Self {
            left_hold: 0.0,
            right_hold: 0.0,
        }
    }

    fn frame_input(&mut self, keyboard: &ButtonInput<KeyCode>, dt: f32) -> FrameInput {
        let left = keyboard.pressed(KeyCode::ArrowLeft);
        let right = keyboard.pressed(KeyCode::ArrowRight);

        let move_left =
            Self::repeat_action(left, &mut self.left_hold, dt) && !(right && self.right_hold > 0.0);
        let move_right =
            Self::repeat_action(right, &mut self.right_hold, dt) && !(left && self.left_hold > 0.0);

        FrameInput {
            move_left,
            move_right,
            soft_drop: keyboard.pressed(KeyCode::ArrowDown),
            hard_drop: keyboard.just_pressed(KeyCode::Space),
            rotate_cw: keyboard.just_pressed(KeyCode::KeyX),
            rotate_ccw: keyboard.just_pressed(KeyCode::ArrowUp)
                || keyboard.just_pressed(KeyCode::KeyZ),
            toggle_pause: keyboard.just_pressed(KeyCode::KeyP),
            restart: keyboard.just_pressed(KeyCode::KeyR),
        }
    }

    fn repeat_action(held: bool, timer: &mut f32, dt: f32) -> bool {
        if !held {
            *timer = 0.0;
            return false;
        }

        if *timer == 0.0 {
            *timer = dt;
            return true;
        }

        *timer += dt;

        if *timer >= Self::INITIAL_DELAY {
            let elapsed = *timer - Self::INITIAL_DELAY;
            let previous = elapsed - dt;
            let current_ticks = (elapsed / Self::REPEAT_RATE).floor() as i32;
            let previous_ticks = previous.max(0.0) / Self::REPEAT_RATE;
            return current_ticks > previous_ticks.floor() as i32;
        }

        false
    }
}

struct FrameInput {
    move_left: bool,
    move_right: bool,
    soft_drop: bool,
    hard_drop: bool,
    rotate_cw: bool,
    rotate_ccw: bool,
    toggle_pause: bool,
    restart: bool,
}

pub fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut das: ResMut<DasState>,
    mut soft_drop_state: ResMut<SoftDropState>,
    mut session: ResMut<TetrisSession>,
    mut effects: ResMut<VisualEffectsState>,
) {
    let input = das.frame_input(&keyboard, time.delta_secs());
    soft_drop_state.0 = input.soft_drop;

    if input.restart && session.game.phase == crate::game::GamePhase::GameOver {
        session.game = crate::game::Game::new();
        return;
    }

    if input.toggle_pause {
        session.game.toggle_pause();
        if session.game.phase == crate::game::GamePhase::Paused {
            effects.trigger_paused_popup();
        }
    }

    if input.rotate_cw {
        session.game.rotate(RotationDirection::Clockwise);
    }

    if input.rotate_ccw {
        session.game.rotate(RotationDirection::CounterClockwise);
    }

    if input.move_left {
        session.game.move_horizontal(-1);
    }

    if input.move_right {
        session.game.move_horizontal(1);
    }

    if input.hard_drop {
        session.game.hard_drop();
    } else if input.soft_drop {
        session.game.soft_drop_once();
    }
}
