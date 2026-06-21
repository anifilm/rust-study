use bevy::prelude::*;

use crate::{
    constants::*,
    game::{assets::GameAssets, logic::Board},
    state::GameEntity,
};

// ─── 보드 스폰 (프레임 + 격자 + 별점) ───────────────────────────────────────
pub fn spawn_board(mut commands: Commands, _assets: Res<GameAssets>) {
    let frame_size = BOARD_PIXEL + 28.0;
    let frame_inner = BOARD_PIXEL + 10.0;

    // 외곽 프레임 (어두운 나무색)
    commands.spawn((
        Sprite {
            color: COLOR_FRAME,
            custom_size: Some(Vec2::splat(frame_size)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -0.5),
        GameEntity,
    ));

    // 내측 프레임 (밝은 나무색)
    commands.spawn((
        Sprite {
            color: COLOR_FRAME_INNER,
            custom_size: Some(Vec2::splat(frame_inner)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -0.2),
        GameEntity,
    ));

    // 보드 배경
    commands.spawn((
        Sprite {
            color: COLOR_BOARD,
            custom_size: Some(Vec2::splat(BOARD_PIXEL)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        GameEntity,
    ));

    // 체크보드 패턴 (교대 셀)
    for row in 0..BOARD_SIZE {
        for col in 0..BOARD_SIZE {
            if (row + col) % 2 == 0 {
                let pos = cell_to_world(row, col);
                commands.spawn((
                    Sprite {
                        color: COLOR_BOARD_CELL_ALT,
                        custom_size: Some(Vec2::splat(CELL_SIZE)),
                        ..default()
                    },
                    Transform::from_xyz(pos.x, pos.y, 0.1),
                    GameEntity,
                ));
            }
        }
    }

    // 격자선
    let line_len = BOARD_PIXEL;
    for i in 0..=BOARD_SIZE {
        let offset = -BOARD_PIXEL / 2.0 + i as f32 * CELL_SIZE;
        let thickness = if i == 0 || i == BOARD_SIZE { 2.0 } else { 1.2 };

        // 가로선
        commands.spawn((
            Sprite {
                color: COLOR_GRID_LINE,
                custom_size: Some(Vec2::new(line_len, thickness)),
                ..default()
            },
            Transform::from_xyz(0.0, offset, 0.5),
            GameEntity,
        ));
        // 세로선
        commands.spawn((
            Sprite {
                color: COLOR_GRID_LINE,
                custom_size: Some(Vec2::new(thickness, line_len)),
                ..default()
            },
            Transform::from_xyz(offset, 0.0, 0.5),
            GameEntity,
        ));
    }

}

/// 보드 (row, col) → 월드 좌표
pub fn cell_to_world(row: usize, col: usize) -> Vec2 {
    let x = col as f32 * CELL_SIZE - BOARD_OFFSET;
    let y = -(row as f32 * CELL_SIZE - BOARD_OFFSET);
    Vec2::new(x, y)
}

/// 월드 좌표 → 보드 (row, col)
pub fn world_to_cell(world: Vec2) -> Option<(usize, usize)> {
    let half = BOARD_PIXEL / 2.0;
    if world.x < -half || world.x > half || world.y < -half || world.y > half {
        return None;
    }
    let col = ((world.x + half) / CELL_SIZE) as usize;
    let row = ((-world.y + half) / CELL_SIZE) as usize;
    if col < BOARD_SIZE && row < BOARD_SIZE {
        Some((row, col))
    } else {
        None
    }
}

// ─── 보드 정리 ───────────────────────────────────────────────────────────────
pub fn despawn_board(mut commands: Commands, query: Query<Entity, With<GameEntity>>) {
    for e in &query {
        commands.entity(e).despawn_recursive();
    }
}

// ─── Board resource 초기화 ───────────────────────────────────────────────────
pub fn init_board_resource(mut commands: Commands) {
    commands.insert_resource(Board::new());
}
