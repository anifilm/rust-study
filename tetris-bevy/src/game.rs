use bevy::prelude::*;
use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::board::{Board, BOARD_HEIGHT};
use crate::rotation::RotationDirection;
use crate::systems::{effects, gameplay, input, render, setup, ui};
use crate::tetromino::{all_bag_pieces, Tetromino, TetrominoType};

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
    pub last_cleared_rows: Vec<usize>,
    pub last_locked_blocks: Vec<(i32, i32, TetrominoType)>,
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
            last_cleared_rows: Vec::new(),
            last_locked_blocks: Vec::new(),
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

    #[allow(dead_code)]
    pub fn ghost_piece(&self) -> Tetromino {
        self.board.hard_drop_position(&self.current)
    }

    fn gravity_interval(&self) -> f32 {
        let level = self.level.saturating_sub(1) as f32;
        (0.8 - level * 0.07).max(0.08)
    }

    fn lock_piece(&mut self) {
        self.last_locked_blocks = self
            .current
            .blocks()
            .into_iter()
            .filter(|block| block.y >= 0)
            .map(|block| (block.x, block.y, self.current.kind))
            .collect();
        self.board.lock_piece(&self.current);
        self.last_cleared_rows = self.board.full_rows();
        let cleared = self.last_cleared_rows.len();
        self.board.clear_full_rows();
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

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Resource, Default)]
pub struct TetrisSession {
    pub game: Game,
}

#[derive(Resource, Default)]
pub struct SoftDropState(pub bool);

#[derive(Resource, Default)]
pub struct VisualEffectsState {
    pub cleared_row_flash_timer: f32,
    pub cleared_rows: Vec<usize>,
    pub shake_timer: f32,
    pub clear_badge_timer: f32,
    pub clear_badge_text: String,
    pub preview_transition_timer: f32,
    pub displayed_next: Option<TetrominoType>,
    pub lock_squash_timer: f32,
    pub locked_blocks: Vec<(i32, i32, TetrominoType)>,
    pub game_over_popup_timer: f32,
    pub paused_popup_timer: f32,
    pub previous_phase: Option<GamePhase>,
}

impl VisualEffectsState {
    pub const ROW_FLASH_DURATION: f32 = 0.18;
    pub const SHAKE_DURATION: f32 = 0.14;
    pub const CLEAR_BADGE_DURATION: f32 = 0.55;
    pub const PREVIEW_TRANSITION_DURATION: f32 = 0.24;
    pub const LOCK_SQUASH_DURATION: f32 = 0.16;
    pub const GAME_OVER_POPUP_DURATION: f32 = 0.24;
    pub const PAUSED_POPUP_DURATION: f32 = 0.24;

    pub fn trigger_cleared_rows_flash(&mut self, rows: &[usize]) {
        self.cleared_rows.clear();
        self.cleared_rows.extend_from_slice(rows);
        self.cleared_row_flash_timer = Self::ROW_FLASH_DURATION;
    }

    pub fn trigger_clear_badge(&mut self, cleared: usize) {
        self.clear_badge_text = match cleared {
            1 => "",
            2 => "DOUBLE",
            3 => "TRIPLE",
            4 => "TETRIS",
            _ => "",
        }
        .to_string();

        if !self.clear_badge_text.is_empty() {
            self.clear_badge_timer = Self::CLEAR_BADGE_DURATION;
            self.shake_timer = Self::SHAKE_DURATION;
        }
    }

    pub fn trigger_preview_transition(&mut self, next: TetrominoType) {
        self.displayed_next = Some(next);
        self.preview_transition_timer = Self::PREVIEW_TRANSITION_DURATION;
    }

    pub fn trigger_lock_squash(&mut self, locked_blocks: &[(i32, i32, TetrominoType)]) {
        self.locked_blocks.clear();
        self.locked_blocks.extend_from_slice(locked_blocks);
        self.lock_squash_timer = Self::LOCK_SQUASH_DURATION;
    }

    pub fn trigger_game_over_popup(&mut self) {
        self.game_over_popup_timer = Self::GAME_OVER_POPUP_DURATION;
    }

    pub fn trigger_paused_popup(&mut self) {
        self.paused_popup_timer = Self::PAUSED_POPUP_DURATION;
    }
}

pub struct TetrisPlugin;

impl Plugin for TetrisPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TetrisSession>()
            .init_resource::<SoftDropState>()
            .init_resource::<VisualEffectsState>()
            .init_resource::<input::DasState>()
            .add_systems(
                Startup,
                (setup::setup_camera, setup::setup_board, ui::setup_ui),
            )
            .add_systems(
                Update,
                (
                    input::handle_input,
                    gameplay::update_game.after(input::handle_input),
                    effects::update_visual_effects.after(gameplay::update_game),
                    render::sync_blocks.after(gameplay::update_game),
                    ui::sync_next_preview.after(render::sync_blocks),
                    ui::update_ui.after(ui::sync_next_preview),
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hard_drop_increases_score() {
        let mut game = Game::new();
        game.hard_drop();
        assert!(game.score > 0);
    }
}
