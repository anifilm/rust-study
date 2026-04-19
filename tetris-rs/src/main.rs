mod board;
mod game;
mod input;
mod render;
mod rotation;
mod tetromino;

use macroquad::prelude::*;

use crate::game::{Game, GamePhase};
use crate::input::DasState;
use crate::render::window_conf;
use crate::rotation::RotationDirection;

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new();
    let mut das = DasState::new();

    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        let input = das.frame_input();

        if input.restart && game.is_game_over() {
            game.restart();
        }

        if input.toggle_pause && game.phase != GamePhase::GameOver {
            game.toggle_pause();
        }

        if input.rotate_cw {
            game.rotate(RotationDirection::Clockwise);
        }

        if input.rotate_ccw {
            game.rotate(RotationDirection::CounterClockwise);
        }

        if input.move_left {
            game.move_horizontal(-1);
        }

        if input.move_right {
            game.move_horizontal(1);
        }

        if input.hard_drop {
            game.hard_drop();
        } else if input.soft_drop {
            game.soft_drop_once();
        }

        game.update(get_frame_time(), input.soft_drop);
        render::draw(&game);

        next_frame().await;
    }
}
