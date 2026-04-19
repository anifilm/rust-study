use bevy::prelude::*;

use crate::components::*;
use crate::game::{GameOverEvent, GameState, RestartEvent};
use crate::systems::render::spawn_tile;
use crate::systems::animation::AnimationConfig;

/// 점수 UI 초기 스폰 (Startup 시스템에서 호출)
pub fn setup_score_ui(mut commands: Commands) {
    // 점수 라벨
    commands.spawn((
        Text2d("Score".to_string()),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::srgb(0.47, 0.43, 0.40)),
        Transform::from_xyz(-150.0, 420.0, 10.0),
    ));

    // 점수 값
    commands.spawn((
        Text2d("0".to_string()),
        TextFont {
            font_size: 36.0,
            ..default()
        },
        TextColor(Color::srgb(0.47, 0.43, 0.40)),
        Transform::from_xyz(-150.0, 390.0, 10.0),
        ScoreText,
    ));

    // 조작 안내
    commands.spawn((
        Text2d("Arrow keys to move | R to restart".to_string()),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::srgb(0.60, 0.56, 0.52)),
        Transform::from_xyz(0.0, -420.0, 10.0),
    ));
}

/// 점수 업데이트
pub fn update_score(
    game_state: Res<GameState>,
    mut score_query: Query<&mut Text2d, With<ScoreText>>,
) {
    if !game_state.is_changed() {
        return;
    }
    for mut text in score_query.iter_mut() {
        **text = game_state.score.to_string();
    }
}

/// 게임오버 처리
pub fn handle_game_over(
    mut commands: Commands,
    mut events: EventReader<GameOverEvent>,
    overlay_query: Query<Entity, With<GameOverOverlay>>,
) {
    for _ in events.read() {
        // 이미 오버레이가 있으면 무시
        if !overlay_query.is_empty() {
            continue;
        }

        // 반투명 배경
        commands.spawn((
            Sprite {
                color: Color::srgba(0.98, 0.97, 0.94, 0.85),
                custom_size: Some(Vec2::new(800.0, 900.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 50.0),
            GameOverOverlay,
        ));

        // 게임오버 텍스트
        commands.spawn((
            Text2d("Game Over!".to_string()),
            TextFont {
                font_size: 60.0,
                ..default()
            },
            TextColor(Color::srgb(0.47, 0.43, 0.40)),
            Transform::from_xyz(0.0, 50.0, 51.0),
            GameOverOverlay,
        ));

        // 리스타트 안내
        commands.spawn((
            Text2d("Press R to restart".to_string()),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::srgb(0.60, 0.56, 0.52)),
            Transform::from_xyz(0.0, -20.0, 51.0),
            GameOverOverlay,
        ));
    }
}

/// 리스타트 처리
pub fn handle_restart(
    mut commands: Commands,
    animation_config: Res<AnimationConfig>,
    mut game_state: ResMut<GameState>,
    mut events: EventReader<RestartEvent>,
    overlay_query: Query<Entity, With<GameOverOverlay>>,
    tile_query: Query<Entity, With<Tile>>,
) {
    for _ in events.read() {
        // 게임 상태 리셋
        game_state.reset();

        // 오버레이 제거
        for entity in overlay_query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        // 기존 타일 제거
        for entity in tile_query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        // 새 타일 2개 생성
        for _ in 0..2 {
            if let Some((row, col, value)) = game_state.spawn_random_tile() {
                spawn_tile(&mut commands, row, col, value, animation_config.spawn_duration);
            }
        }
    }
}
