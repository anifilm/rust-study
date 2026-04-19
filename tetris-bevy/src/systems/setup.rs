use bevy::prelude::*;

use crate::board::{BOARD_HEIGHT, BOARD_WIDTH};
use crate::components::{BoardFrame, ClearedRowFlash};
use crate::systems::render::{
    board_cell_center, board_pixel_height, board_pixel_width, BOARD_ORIGIN_X, BOARD_ORIGIN_Y,
    CELL_SIZE, SIDE_PANEL_WIDTH, SIDE_PANEL_X,
};

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub fn setup_board(mut commands: Commands) {
    commands.spawn((
        Sprite {
            color: Color::srgb(0.03, 0.04, 0.08),
            custom_size: Some(Vec2::new(1600.0, 1200.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -30.0),
    ));

    commands.spawn((
        Sprite {
            color: Color::srgba(0.09, 0.20, 0.38, 0.35),
            custom_size: Some(Vec2::new(
                board_pixel_width() + 160.0,
                board_pixel_height() + 200.0,
            )),
            ..default()
        },
        Transform::from_xyz(
            BOARD_ORIGIN_X + board_pixel_width() / 2.0 - 16.0,
            BOARD_ORIGIN_Y + board_pixel_height() / 2.0,
            -24.0,
        ),
    ));

    commands.spawn((
        Sprite {
            color: Color::srgba(0.33, 0.13, 0.46, 0.18),
            custom_size: Some(Vec2::new(SIDE_PANEL_WIDTH + 120.0, 560.0)),
            ..default()
        },
        Transform::from_xyz(SIDE_PANEL_X + SIDE_PANEL_WIDTH / 2.0, 10.0, -24.0),
    ));

    commands.spawn((
        Sprite {
            color: Color::srgb(0.12, 0.18, 0.31),
            custom_size: Some(Vec2::new(
                board_pixel_width() + 28.0,
                board_pixel_height() + 28.0,
            )),
            ..default()
        },
        Transform::from_xyz(
            BOARD_ORIGIN_X + board_pixel_width() / 2.0,
            BOARD_ORIGIN_Y + board_pixel_height() / 2.0,
            -10.0,
        ),
    ));

    commands.spawn((
        Sprite {
            color: Color::srgb(0.36, 0.56, 0.95),
            custom_size: Some(Vec2::new(
                board_pixel_width() + 12.0,
                board_pixel_height() + 12.0,
            )),
            ..default()
        },
        Transform::from_xyz(
            BOARD_ORIGIN_X + board_pixel_width() / 2.0,
            BOARD_ORIGIN_Y + board_pixel_height() / 2.0,
            -9.0,
        ),
        BoardFrame,
    ));

    commands.spawn((
        Sprite {
            color: Color::srgb(0.05, 0.07, 0.13),
            custom_size: Some(Vec2::new(
                board_pixel_width() - 4.0,
                board_pixel_height() - 4.0,
            )),
            ..default()
        },
        Transform::from_xyz(
            BOARD_ORIGIN_X + board_pixel_width() / 2.0,
            BOARD_ORIGIN_Y + board_pixel_height() / 2.0,
            -8.0,
        ),
    ));

    for y in 0..BOARD_HEIGHT {
        commands.spawn((
            Sprite {
                color: Color::srgba(0.82, 0.91, 1.0, 0.0),
                custom_size: Some(Vec2::new(board_pixel_width() - 4.0, CELL_SIZE - 2.0)),
                ..default()
            },
            Transform::from_xyz(
                BOARD_ORIGIN_X + board_pixel_width() / 2.0,
                board_cell_center(0, y as i32).y,
                2.5,
            ),
            ClearedRowFlash { row: y },
        ));
    }

    for y in 0..BOARD_HEIGHT {
        for x in 0..BOARD_WIDTH {
            commands.spawn((
                Sprite {
                    color: Color::srgb(0.10, 0.13, 0.22),
                    custom_size: Some(Vec2::splat(CELL_SIZE - 3.0)),
                    ..default()
                },
                Transform::from_xyz(
                    board_cell_center(x as i32, y as i32).x,
                    board_cell_center(x as i32, y as i32).y,
                    -7.0,
                ),
            ));
        }
    }
}
