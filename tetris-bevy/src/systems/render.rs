use bevy::prelude::*;

use crate::board::{Cell, BOARD_HEIGHT, BOARD_WIDTH};
use crate::components::{ActiveBlock, GhostBlock, LockedBlock};
use crate::game::{TetrisSession, VisualEffectsState};
use crate::tetromino::TetrominoType;

pub const CELL_SIZE: f32 = 28.0;
pub const BOARD_ORIGIN_X: f32 = -250.0;
pub const BOARD_ORIGIN_Y: f32 = -280.0;
pub const SIDE_PANEL_X: f32 = 92.0;
pub const SIDE_PANEL_WIDTH: f32 = 210.0;
pub const SIDE_PANEL_LEFT_PX: f32 = 400.0;
pub const HUD_TOP_PX: f32 = 80.0;
pub const HUD_HEIGHT_PX: f32 = 560.0;
pub const NEXT_CARD_TOP_PX: f32 = HUD_TOP_PX;
pub const PREVIEW_BOX_LEFT_PX: f32 = SIDE_PANEL_LEFT_PX + 20.0;
pub const PREVIEW_BOX_TOP_PX: f32 = NEXT_CARD_TOP_PX + 46.0;
pub const PREVIEW_BOX_WIDTH_PX: f32 = SIDE_PANEL_WIDTH - 40.0;
pub const PREVIEW_BOX_HEIGHT_PX: f32 = 86.0;

pub fn board_pixel_width() -> f32 {
    BOARD_WIDTH as f32 * CELL_SIZE
}

pub fn board_pixel_height() -> f32 {
    BOARD_HEIGHT as f32 * CELL_SIZE
}

pub fn board_cell_center(x: i32, y: i32) -> Vec2 {
    Vec2::new(
        BOARD_ORIGIN_X + x as f32 * CELL_SIZE + CELL_SIZE / 2.0,
        BOARD_ORIGIN_Y + (BOARD_HEIGHT as i32 - 1 - y) as f32 * CELL_SIZE + CELL_SIZE / 2.0,
    )
}

pub fn sync_blocks(
    mut commands: Commands,
    session: Res<TetrisSession>,
    effects: Res<VisualEffectsState>,
    locked_blocks: Query<Entity, With<LockedBlock>>,
    active_blocks: Query<Entity, With<ActiveBlock>>,
    ghost_blocks: Query<Entity, With<GhostBlock>>,
) {
    for entity in locked_blocks.iter() {
        commands.entity(entity).despawn();
    }
    for entity in active_blocks.iter() {
        commands.entity(entity).despawn();
    }
    for entity in ghost_blocks.iter() {
        commands.entity(entity).despawn();
    }

    for y in 0..BOARD_HEIGHT {
        for x in 0..BOARD_WIDTH {
            if let Cell::Filled(kind) = session.game.board.cell(x, y) {
                let base_size = CELL_SIZE - 2.0;
                let (outer_size, inner_size) = if effects.lock_squash_timer > 0.0
                    && effects
                        .locked_blocks
                        .iter()
                        .any(|(bx, by, _)| *bx == x as i32 && *by == y as i32)
                {
                    let progress =
                        effects.lock_squash_timer / VisualEffectsState::LOCK_SQUASH_DURATION;
                    let outer = Vec2::new(
                        base_size * (1.0 + 0.18 * progress),
                        base_size * (1.0 - 0.12 * progress),
                    );
                    let inner = Vec2::new(
                        (base_size - 6.0) * (1.0 + 0.12 * progress),
                        (base_size - 6.0) * (1.0 - 0.08 * progress),
                    );
                    (outer, inner)
                } else {
                    (
                        Vec2::splat(base_size),
                        Vec2::splat((base_size - 6.0).max(1.0)),
                    )
                };
                spawn_block(
                    &mut commands,
                    board_cell_center(x as i32, y as i32),
                    kind,
                    1.0,
                    1.0,
                    outer_size,
                    inner_size,
                    LockedBlock,
                );
            }
        }
    }

    if session.game.level == 1 {
        spawn_piece(
            &mut commands,
            session.game.ghost_piece(),
            0.10,
            2.0,
            Vec2::splat(CELL_SIZE - 1.0),
            Vec2::splat((CELL_SIZE - 7.0).max(1.0)),
            GhostBlock,
        );
    }

    spawn_piece(
        &mut commands,
        session.game.current,
        1.0,
        3.0,
        Vec2::splat(CELL_SIZE - 1.0),
        Vec2::splat((CELL_SIZE - 7.0).max(1.0)),
        ActiveBlock,
    );
}

fn spawn_piece<T: Component + Clone>(
    commands: &mut Commands,
    piece: crate::tetromino::Tetromino,
    alpha: f32,
    z: f32,
    outer_size: Vec2,
    inner_size: Vec2,
    marker: T,
) {
    for block in piece.blocks() {
        if block.y < 0 {
            continue;
        }
        spawn_block(
            commands,
            board_cell_center(block.x, block.y),
            piece.kind,
            alpha,
            z,
            outer_size,
            inner_size,
            marker.clone(),
        );
    }
}

fn spawn_block<T: Component + Clone>(
    commands: &mut Commands,
    position: Vec2,
    kind: TetrominoType,
    alpha: f32,
    z: f32,
    outer_size: Vec2,
    inner_size: Vec2,
    marker: T,
) {
    let mut color = color_for(kind);
    color.set_alpha(alpha);
    let mut shell = shell_color_for(kind);
    shell.set_alpha((alpha * 0.92).clamp(0.0, 1.0));

    commands.spawn((
        Sprite {
            color: shell,
            custom_size: Some(outer_size),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, z),
        marker.clone(),
    ));

    commands.spawn((
        Sprite {
            color,
            custom_size: Some(inner_size),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, z + 0.01),
        marker,
    ));
}

pub(crate) fn color_for(kind: TetrominoType) -> Color {
    match kind {
        TetrominoType::I => Color::srgb(0.31, 0.80, 1.0),
        TetrominoType::O => Color::srgb(1.0, 0.84, 0.04),
        TetrominoType::T => Color::srgb(0.71, 0.47, 1.0),
        TetrominoType::S => Color::srgb(0.36, 0.86, 0.34),
        TetrominoType::Z => Color::srgb(1.0, 0.37, 0.34),
        TetrominoType::J => Color::srgb(0.27, 0.43, 1.0),
        TetrominoType::L => Color::srgb(1.0, 0.63, 0.21),
    }
}

pub(crate) fn shell_color_for(kind: TetrominoType) -> Color {
    match kind {
        TetrominoType::I => Color::srgb(0.10, 0.52, 0.72),
        TetrominoType::O => Color::srgb(0.66, 0.54, 0.06),
        TetrominoType::T => Color::srgb(0.43, 0.23, 0.67),
        TetrominoType::S => Color::srgb(0.18, 0.54, 0.18),
        TetrominoType::Z => Color::srgb(0.68, 0.18, 0.18),
        TetrominoType::J => Color::srgb(0.15, 0.27, 0.63),
        TetrominoType::L => Color::srgb(0.70, 0.36, 0.11),
    }
}
