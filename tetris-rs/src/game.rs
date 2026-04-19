use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::board::{BOARD_HEIGHT, Board};
use crate::rotation::RotationDirection;
use crate::tetromino::{Tetromino, TetrominoType, all_bag_pieces};

pub const LOCK_DELAY: f32 = 0.5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GamePhase {
    Playing,
    Paused,
    GameOver,
}

pub struct Game {
    pub board: Board,
    pub current: Tetromino,
    pub next: TetrominoType,
    pub score: u32,
    pub lines: u32,
    pub level: u32,
    pub phase: GamePhase,
    drop_timer: f32,
    lock_timer: f32,
    bag: Vec<TetrominoType>,
}

impl Game {
    pub fn new() -> Self {
        let mut game = Self {
            board: Board::new(),
            current: Tetromino::new(TetrominoType::I),
            next: TetrominoType::O,
            score: 0,
            lines: 0,
            level: 1,
            phase: GamePhase::Playing,
            drop_timer: 0.0,
            lock_timer: 0.0,
            bag: Vec::new(),
        };

        let first = game.draw_piece();
        let next = game.draw_piece();
        game.current = Tetromino::new(first);
        game.next = next;
        game
    }

    pub fn restart(&mut self) {
        *self = Self::new();
    }

    pub fn toggle_pause(&mut self) {
        self.phase = match self.phase {
            GamePhase::Playing => GamePhase::Paused,
            GamePhase::Paused => GamePhase::Playing,
            GamePhase::GameOver => GamePhase::GameOver,
        };
    }

    pub fn update(&mut self, dt: f32, soft_drop: bool) {
        if self.phase != GamePhase::Playing {
            return;
        }

        let gravity = if soft_drop {
            0.03
        } else {
            self.gravity_interval()
        };

        self.drop_timer += dt;

        if self.drop_timer >= gravity {
            self.drop_timer = 0.0;
            if let Some(next) = self.board.try_move(&self.current, 0, 1) {
                self.current = next;
                self.lock_timer = 0.0;
            }
        }

        if self.board.try_move(&self.current, 0, 1).is_none() {
            self.lock_timer += dt;
            if self.lock_timer >= LOCK_DELAY {
                self.lock_piece();
            }
        } else {
            self.lock_timer = 0.0;
        }
    }

    pub fn move_horizontal(&mut self, direction: i32) {
        if self.phase != GamePhase::Playing {
            return;
        }

        if let Some(next) = self.board.try_move(&self.current, direction, 0) {
            self.current = next;
            self.lock_timer = 0.0;
        }
    }

    pub fn soft_drop_once(&mut self) {
        if self.phase != GamePhase::Playing {
            return;
        }

        if let Some(next) = self.board.try_move(&self.current, 0, 1) {
            self.current = next;
            self.score += 1;
        }
    }

    pub fn hard_drop(&mut self) {
        if self.phase != GamePhase::Playing {
            return;
        }

        let dropped = self.board.hard_drop_position(&self.current);
        let distance = (dropped.y - self.current.y).max(0) as u32;
        self.current = dropped;
        self.score += distance * 2;
        self.lock_piece();
    }

    pub fn rotate(&mut self, direction: RotationDirection) {
        if self.phase != GamePhase::Playing {
            return;
        }

        if let Some(next) = self.board.try_rotate(&self.current, direction) {
            self.current = next;
            self.lock_timer = 0.0;
        }
    }

    pub fn ghost_piece(&self) -> Tetromino {
        self.board.hard_drop_position(&self.current)
    }

    pub fn is_game_over(&self) -> bool {
        self.phase == GamePhase::GameOver
    }

    fn gravity_interval(&self) -> f32 {
        let level = self.level.saturating_sub(1) as f32;
        (0.8 - level * 0.07).max(0.08)
    }

    fn lock_piece(&mut self) {
        self.board.lock_piece(&self.current);
        let cleared = self.board.clear_lines();
        self.apply_line_score(cleared);
        self.spawn_next_piece();
        self.drop_timer = 0.0;
        self.lock_timer = 0.0;
    }

    fn apply_line_score(&mut self, cleared: usize) {
        if cleared == 0 {
            return;
        }

        self.lines += cleared as u32;
        self.level = (self.lines / 10) + 1;

        let base = match cleared {
            1 => 100,
            2 => 300,
            3 => 500,
            4 => 800,
            _ => 0,
        };

        self.score += base * self.level;
    }

    fn spawn_next_piece(&mut self) {
        self.current = Tetromino::new(self.next);
        self.next = self.draw_piece();
        if self.board.collides(&self.current) || self.current.y >= BOARD_HEIGHT as i32 {
            self.phase = GamePhase::GameOver;
        }
    }

    fn draw_piece(&mut self) -> TetrominoType {
        if self.bag.is_empty() {
            let mut pieces = all_bag_pieces();
            pieces.shuffle(&mut thread_rng());
            self.bag.extend(pieces);
        }

        self.bag.pop().expect("bag should always contain pieces")
    }
}
