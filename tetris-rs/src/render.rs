use macroquad::prelude::*;

use crate::board::{Cell, BOARD_HEIGHT, BOARD_WIDTH};
use crate::game::{Game, GamePhase};
use crate::tetromino::{Tetromino, TetrominoType};

pub const CELL_SIZE: f32 = 30.0;
const BOARD_X: f32 = 60.0;
const BOARD_Y: f32 = 40.0;
const SIDE_PANEL_X: f32 = BOARD_X + BOARD_WIDTH as f32 * CELL_SIZE + 55.0;

pub fn window_conf() -> Conf {
    Conf {
        window_title: "Tetris RS".to_owned(),
        window_width: 620,
        window_height: 720,
        high_dpi: true,
        ..Default::default()
    }
}

pub fn draw(game: &Game) {
    clear_background(Color::from_rgba(12, 16, 28, 255));
    draw_board_frame();
    draw_locked_cells(game);
    if game.level == 1 {
        draw_ghost_piece(game.ghost_piece());
    }
    draw_piece(game.current, 1.0);
    draw_sidebar(game);
    draw_overlay(game.phase);
}

fn draw_board_frame() {
    let width = BOARD_WIDTH as f32 * CELL_SIZE;
    let height = BOARD_HEIGHT as f32 * CELL_SIZE;

    draw_rectangle_lines(
        BOARD_X - 2.0,
        BOARD_Y - 2.0,
        width + 4.0,
        height + 4.0,
        3.0,
        WHITE,
    );

    for row in 0..BOARD_HEIGHT {
        for col in 0..BOARD_WIDTH {
            draw_rectangle_lines(
                BOARD_X + col as f32 * CELL_SIZE,
                BOARD_Y + row as f32 * CELL_SIZE,
                CELL_SIZE,
                CELL_SIZE,
                1.0,
                Color::from_rgba(44, 54, 80, 255),
            );
        }
    }
}

fn draw_locked_cells(game: &Game) {
    for y in 0..BOARD_HEIGHT {
        for x in 0..BOARD_WIDTH {
            if let Cell::Filled(kind) = game.board.cell(x, y) {
                draw_block(x as i32, y as i32, color_for(kind), 1.0);
            }
        }
    }
}

fn draw_piece(piece: Tetromino, alpha: f32) {
    for block in piece.blocks() {
        if block.y >= 0 {
            draw_block(block.x, block.y, color_for(piece.kind), alpha);
        }
    }
}

fn draw_ghost_piece(piece: Tetromino) {
    draw_piece(piece, 0.2);
}

fn draw_block(x: i32, y: i32, color: Color, alpha: f32) {
    let px = BOARD_X + x as f32 * CELL_SIZE;
    let py = BOARD_Y + y as f32 * CELL_SIZE;
    let tint = Color { a: alpha, ..color };
    let border = Color { a: alpha, ..WHITE };

    draw_rectangle(px + 1.0, py + 1.0, CELL_SIZE - 2.0, CELL_SIZE - 2.0, tint);
    draw_rectangle_lines(
        px + 1.0,
        py + 1.0,
        CELL_SIZE - 2.0,
        CELL_SIZE - 2.0,
        2.0,
        border,
    );
}

fn draw_sidebar(game: &Game) {
    draw_text("NEXT", SIDE_PANEL_X, 80.0, 32.0, WHITE);
    draw_preview(game.next, SIDE_PANEL_X, 110.0);

    draw_text(
        &format!("SCORE\n{}", game.score),
        SIDE_PANEL_X,
        250.0,
        32.0,
        WHITE,
    );
    draw_text(
        &format!("LINES\n{}", game.lines),
        SIDE_PANEL_X,
        340.0,
        32.0,
        WHITE,
    );
    draw_text(
        &format!("LEVEL\n{}", game.level),
        SIDE_PANEL_X,
        430.0,
        32.0,
        WHITE,
    );

    draw_text("CONTROLS", SIDE_PANEL_X, 540.0, 28.0, WHITE);
    draw_multiline_text(
        &[
            "Arrows: Move",
            "Up/Z: Rotate Left",
            "X: Rotate Right",
            "Space: Drop",
            "P: Pause",
            "R: Restart",
        ],
        SIDE_PANEL_X,
        575.0,
        22.0,
        24.0,
        GRAY,
    );
}

fn draw_multiline_text(
    lines: &[&str],
    x: f32,
    y: f32,
    font_size: f32,
    line_height: f32,
    color: Color,
) {
    for (index, line) in lines.iter().enumerate() {
        draw_text(line, x, y + index as f32 * line_height, font_size, color);
    }
}

fn draw_preview(kind: TetrominoType, x: f32, y: f32) {
    let preview = Tetromino::new(kind);
    for block in preview.blocks() {
        let px = x + block.x as f32 * 18.0 - 54.0;
        let py = y + block.y as f32 * 18.0;
        draw_rectangle(px, py, 16.0, 16.0, color_for(kind));
        draw_rectangle_lines(px, py, 16.0, 16.0, 1.0, WHITE);
    }
}

fn draw_overlay(phase: GamePhase) {
    let text = match phase {
        GamePhase::Playing => return,
        GamePhase::Paused => "PAUSED\nPress P to resume",
        GamePhase::GameOver => "GAME OVER\nPress R to restart",
    };

    draw_rectangle(
        BOARD_X,
        BOARD_Y + 220.0,
        BOARD_WIDTH as f32 * CELL_SIZE,
        120.0,
        Color::from_rgba(0, 0, 0, 180),
    );
    draw_text(text, BOARD_X + 28.0, BOARD_Y + 275.0, 38.0, WHITE);
}

fn color_for(kind: TetrominoType) -> Color {
    match kind {
        TetrominoType::I => Color::from_rgba(78, 205, 255, 255),
        TetrominoType::O => Color::from_rgba(255, 214, 10, 255),
        TetrominoType::T => Color::from_rgba(180, 120, 255, 255),
        TetrominoType::S => Color::from_rgba(91, 219, 87, 255),
        TetrominoType::Z => Color::from_rgba(255, 94, 87, 255),
        TetrominoType::J => Color::from_rgba(70, 110, 255, 255),
        TetrominoType::L => Color::from_rgba(255, 161, 53, 255),
    }
}
