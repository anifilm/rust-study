use bevy::prelude::*;

use crate::components::*;
use crate::game::GameState;
use crate::systems::animation::{AnimationConfig, MoveAnimation, SpawnAnimation};

const CELL_SIZE: f32 = 140.0;
const GAP: f32 = 10.0;
const BOARD_SIZE: f32 = CELL_SIZE * 4.0 + GAP * 5.0;

/// 카메라 스폰
pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// 그리드 배경 스폰
pub fn setup_grid(mut commands: Commands) {
    let bg_color = Color::srgb(0.73, 0.68, 0.63);
    let cell_color = Color::srgb(0.80, 0.75, 0.70);

    // 전체 보드 배경
    commands.spawn((
        Sprite {
            color: bg_color,
            custom_size: Some(Vec2::new(BOARD_SIZE, BOARD_SIZE)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        GridCell,
    ));

    // 개별 셀
    for row in 0..4 {
        for col in 0..4 {
            let pos = GridPosition::new(row, col);
            let world_pos = pos.to_world_pos();
            commands.spawn((
                Sprite {
                    color: cell_color,
                    custom_size: Some(Vec2::new(CELL_SIZE, CELL_SIZE)),
                    ..default()
                },
                Transform::from_xyz(world_pos.x, world_pos.y, 1.0),
                GridCell,
            ));
        }
    }
}

/// 초기 타일 2개 스폰
pub fn spawn_initial_tiles(
    mut commands: Commands,
    animation_config: Res<AnimationConfig>,
    mut game_state: ResMut<GameState>,
) {
    for _ in 0..2 {
        if let Some((row, col, value)) = game_state.spawn_random_tile() {
            spawn_tile(&mut commands, row, col, value, animation_config.spawn_duration);
        }
    }
}

/// 타일 엔티티 생성 (숨김 상태, 병합 결과용)
pub fn spawn_tile_hidden(commands: &mut Commands, row: usize, col: usize, value: u32) {
    let tile_value = TileValue(value);
    let pos = GridPosition::new(row, col);
    let world_pos = pos.to_world_pos();

    commands.spawn((
        Sprite {
            color: tile_value.color(),
            custom_size: Some(Vec2::new(CELL_SIZE, CELL_SIZE)),
            ..default()
        },
        Transform::from_xyz(world_pos.x, world_pos.y, 1.5).with_scale(Vec3::splat(0.0)), // 더 낮은 Z 좌표
        Tile,
        tile_value,
        pos,
        MergeTarget,
    )).with_children(|parent| {
        parent.spawn((
            Text2d(value.to_string()),
            TextFont {
                font_size: if value >= 1000 { 40.0 } else if value >= 100 { 50.0 } else { 60.0 },
                ..default()
            },
            TextColor(tile_value.text_color()),
            Transform::from_xyz(0.0, 0.0, 1.0),
        ));
    });
}

/// 타일 엔티티 생성
pub fn spawn_tile(commands: &mut Commands, row: usize, col: usize, value: u32, spawn_duration: f32) {
    let tile_value = TileValue(value);
    let pos = GridPosition::new(row, col);
    let world_pos = pos.to_world_pos();

    commands.spawn((
        Sprite {
            color: tile_value.color(),
            custom_size: Some(Vec2::new(CELL_SIZE, CELL_SIZE)),
            ..default()
        },
        Transform::from_xyz(world_pos.x, world_pos.y, 2.0).with_scale(Vec3::splat(0.0)),
        Tile,
        tile_value,
        pos,
        SpawnAnimation {
            timer: Timer::from_seconds(spawn_duration, TimerMode::Once),
        },
    )).with_children(|parent| {
        parent.spawn((
            Text2d(value.to_string()),
            TextFont {
                font_size: if value >= 1000 { 40.0 } else if value >= 100 { 50.0 } else { 60.0 },
                ..default()
            },
            TextColor(tile_value.text_color()),
            Transform::from_xyz(0.0, 0.0, 1.0),
        ));
    });
}

/// 그리드 상태와 엔티티 동기화
pub fn sync_tiles(
    mut commands: Commands,
    animation_config: Res<AnimationConfig>,
    mut game_state: ResMut<GameState>,
    tile_query: Query<(Entity, &GridPosition, &TileValue, &Transform), With<Tile>>,
) {
    if !game_state.needs_sync {
        return;
    }
    game_state.needs_sync = false;

    let Some(ref move_result) = game_state.last_move else {
        return;
    };

    // 타일 엔티티를 그리드 위치로 매핑
    let mut tile_map: std::collections::HashMap<(usize, usize), (Entity, u32, Vec2)> =
        std::collections::HashMap::new();
    for (entity, pos, val, transform) in tile_query.iter() {
        tile_map.insert(
            (pos.row, pos.col),
            (entity, val.0, transform.translation.truncate()),
        );
    }

    // 이동한 타일에 MoveAnimation 추가
    for &(from_row, from_col, to_row, to_col, _value) in &move_result.moved {
        if let Some(&(entity, _, start)) = tile_map.get(&(from_row, from_col)) {
            let end = GridPosition::new(to_row, to_col).to_world_pos();
            commands.entity(entity).insert(MoveAnimation {
                start,
                end,
                timer: Timer::from_seconds(animation_config.move_duration, TimerMode::Once),
            });
            commands.entity(entity).insert(GridPosition::new(to_row, to_col));
        }
    }

    // 병합된 타일: 두 소스 타일을 목적지로 이동시킨 후 제거, 새 타일 생성
    for &(src1_row, src1_col, src2_row, src2_col, to_row, to_col, new_value) in &move_result.merged {
        // 소스1 타일 이동
        if let Some(&(entity, _, start)) = tile_map.get(&(src1_row, src1_col)) {
            let end = GridPosition::new(to_row, to_col).to_world_pos();
            commands.entity(entity).insert(MoveAnimation {
                start,
                end,
                timer: Timer::from_seconds(animation_config.move_duration, TimerMode::Once),
            });
            // 위치 업데이트 및 애니메이션 후 제거를 위해 마커 추가
            commands.entity(entity).insert(GridPosition::new(to_row, to_col));
            commands.entity(entity).insert(MergeSource);
        }
        // 소스2 타일 이동
        if let Some(&(entity, _, start)) = tile_map.get(&(src2_row, src2_col)) {
            let end = GridPosition::new(to_row, to_col).to_world_pos();
            commands.entity(entity).insert(MoveAnimation {
                start,
                end,
                timer: Timer::from_seconds(animation_config.move_duration, TimerMode::Once),
            });
            commands.entity(entity).insert(GridPosition::new(to_row, to_col));
            commands.entity(entity).insert(MergeSource);
        }
        // 병합 결과 타일 생성 (처음에는 숨김)
        spawn_tile_hidden(&mut commands, to_row, to_col, new_value);
    }

    // 새로 생성된 타일
    for &(row, col, value) in &move_result.spawned {
        spawn_tile(&mut commands, row, col, value, animation_config.spawn_duration);
    }
}
