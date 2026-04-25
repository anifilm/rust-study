use bevy::prelude::Color;

// Window
pub const WINDOW_WIDTH: f32 = 800.0;
pub const WINDOW_HEIGHT: f32 = 800.0;

// Maze
pub const MAZE_WIDTH: usize = 21;  // Must be odd
pub const MAZE_HEIGHT: usize = 21; // Must be odd
pub const TILE_SIZE: f32 = 32.0;

// Derived
pub const GRID_WIDTH: f32 = MAZE_WIDTH as f32 * TILE_SIZE;
pub const GRID_HEIGHT: f32 = MAZE_HEIGHT as f32 * TILE_SIZE;

// Colors
pub const COLOR_WALL: Color = Color::srgb(0.1, 0.1, 0.15);
pub const COLOR_PATH: Color = Color::srgb(0.95, 0.95, 0.98);
pub const COLOR_PLAYER: Color = Color::srgb(0.2, 0.4, 0.9);
pub const COLOR_TREASURE: Color = Color::srgb(1.0, 0.8, 0.0);
pub const COLOR_EXIT: Color = Color::srgb(0.2, 0.8, 0.2);
pub const COLOR_HUD_TEXT: Color = Color::srgb(1.0, 1.0, 1.0);

// Player
pub const PLAYER_MOVE_SPEED: f32 = 6.0; // tiles per second for smooth movement
pub const PLAYER_RADIUS: f32 = TILE_SIZE * 0.35;

// Treasure
pub const TREASURE_RADIUS: f32 = TILE_SIZE * 0.25;
pub const TREASURE_SCORE: u32 = 100;
pub const TREASURE_RATIO: f32 = 0.1; // 10% of path cells

// Starting position (in maze grid coordinates)
pub const START_POS: (usize, usize) = (1, 1);
// Exit position (in maze grid coordinates)
pub const EXIT_POS: (usize, usize) = (MAZE_WIDTH - 2, MAZE_HEIGHT - 2);
