use macroquad::prelude::{get_frame_time, is_key_down, is_key_pressed, KeyCode};

pub struct InputState {
    pub move_left: bool,
    pub move_right: bool,
    pub soft_drop: bool,
    pub hard_drop: bool,
    pub rotate_cw: bool,
    pub rotate_ccw: bool,
    pub toggle_pause: bool,
    pub restart: bool,
}

pub struct DasState {
    left_hold: f32,
    right_hold: f32,
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

    pub fn frame_input(&mut self) -> InputState {
        let dt = get_frame_time();
        let left = is_key_down(KeyCode::Left);
        let right = is_key_down(KeyCode::Right);

        let move_left =
            Self::repeat_action(left, &mut self.left_hold, dt) && !(right && self.right_hold > 0.0);
        let move_right =
            Self::repeat_action(right, &mut self.right_hold, dt) && !(left && self.left_hold > 0.0);

        InputState {
            move_left,
            move_right,
            soft_drop: is_key_down(KeyCode::Down),
            hard_drop: is_key_pressed(KeyCode::Space),
            rotate_cw: is_key_pressed(KeyCode::X),
            rotate_ccw: is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::Z),
            toggle_pause: is_key_pressed(KeyCode::P),
            restart: is_key_pressed(KeyCode::R),
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
