use bevy::prelude::*;

use crate::components::Direction;
use crate::game::{GameState, MoveEvent, RestartEvent};
use crate::systems::animation::{MoveAnimation, SpawnAnimation};
use crate::components::{MergeSource, MergeTarget};

/// 키보드 입력 처리
pub fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    game_state: Res<GameState>,
    move_animation_query: Query<(), With<MoveAnimation>>,
    spawn_animation_query: Query<(), With<SpawnAnimation>>,
    merge_source_query: Query<(), With<MergeSource>>,
    merge_target_query: Query<(), With<MergeTarget>>,
    mut move_events: EventWriter<MoveEvent>,
    mut restart_events: EventWriter<RestartEvent>,
) {
    // R키로 리스타트
    if keyboard.just_pressed(KeyCode::KeyR) {
        restart_events.send(RestartEvent);
        return;
    }

    // 게임오버 시 입력 무시
    if game_state.game_over {
        return;
    }

    let animation_in_progress = !move_animation_query.is_empty()
        || !spawn_animation_query.is_empty()
        || !merge_source_query.is_empty()
        || !merge_target_query.is_empty();

    if animation_in_progress {
        return;
    }

    // 방향키 입력
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        move_events.send(MoveEvent(Direction::Up));
    } else if keyboard.just_pressed(KeyCode::ArrowDown) {
        move_events.send(MoveEvent(Direction::Down));
    } else if keyboard.just_pressed(KeyCode::ArrowLeft) {
        move_events.send(MoveEvent(Direction::Left));
    } else if keyboard.just_pressed(KeyCode::ArrowRight) {
        move_events.send(MoveEvent(Direction::Right));
    }
}
