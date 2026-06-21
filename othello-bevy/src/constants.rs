use bevy::prelude::Color;

// 보드 크기
pub const BOARD_SIZE: usize = 8;
pub const CELL_SIZE: f32 = 72.0;
pub const BOARD_PIXEL: f32 = CELL_SIZE * BOARD_SIZE as f32; // 576.0
pub const BOARD_OFFSET: f32 = BOARD_PIXEL / 2.0 - CELL_SIZE / 2.0;

// 보드 색상
pub const COLOR_BACKGROUND: Color = Color::srgb(0.10, 0.10, 0.12);
pub const COLOR_BOARD: Color = Color::srgb(0.12, 0.52, 0.22);
pub const COLOR_BOARD_CELL_ALT: Color = Color::srgb(0.14, 0.54, 0.24);
pub const COLOR_GRID_LINE: Color = Color::srgb(0.06, 0.30, 0.10);
pub const COLOR_FRAME: Color = Color::srgb(0.22, 0.14, 0.06);
pub const COLOR_FRAME_INNER: Color = Color::srgb(0.30, 0.20, 0.08);
pub const COLOR_STAR_POINT: Color = Color::srgb(0.06, 0.30, 0.10);

// 돌 색상
pub const COLOR_BLACK_PIECE: Color = Color::srgb(0.08, 0.08, 0.10);
pub const COLOR_BLACK_HIGHLIGHT: Color = Color::srgb(0.28, 0.28, 0.32);
pub const COLOR_WHITE_PIECE: Color = Color::srgb(0.92, 0.92, 0.90);
pub const COLOR_WHITE_HIGHLIGHT: Color = Color::srgb(1.0, 1.0, 1.0);
pub const COLOR_PIECE_SHADOW: Color = Color::srgba(0.0, 0.0, 0.0, 0.30);

// 유효수 힌트
pub const COLOR_VALID_HINT: Color = Color::srgba(1.0, 1.0, 1.0, 0.18);

// 돌 크기
pub const PIECE_RADIUS: f32 = 28.0;
pub const PIECE_HIGHLIGHT_RADIUS: f32 = 14.0;
pub const PIECE_SHADOW_RADIUS: f32 = 29.0;
pub const VALID_HINT_RADIUS: f32 = 8.0;
pub const STAR_POINT_RADIUS: f32 = 4.0;

// UI 색상
pub const COLOR_BTN_NORMAL: Color = Color::srgb(0.18, 0.18, 0.18);
pub const COLOR_BTN_HOVER: Color = Color::srgb(0.30, 0.30, 0.30);
pub const COLOR_BTN_PRESS: Color = Color::srgb(0.10, 0.10, 0.10);
pub const COLOR_TEXT: Color = Color::srgb(0.95, 0.95, 0.95);
pub const COLOR_TITLE: Color = Color::srgb(0.13, 0.85, 0.13);

// HUD
pub const COLOR_HUD_BG: Color = Color::srgba(0.06, 0.06, 0.08, 0.85);
pub const COLOR_HUD_BORDER: Color = Color::srgb(0.22, 0.22, 0.26);
pub const HUD_PANEL_WIDTH: f32 = 180.0;

// AI
pub const AI_THINK_DELAY: f32 = 0.5;
#[allow(dead_code)]
pub const AI_DEPTH: u8 = 6;

// 윈도우
pub const WINDOW_WIDTH: f32 = 960.0;
pub const WINDOW_HEIGHT: f32 = 700.0;
