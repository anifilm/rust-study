use std::collections::VecDeque;

use bevy::prelude::*;

use crate::{
    game::{
        assets::GameAssets,
        board::world_to_cell,
        hud::show_pass_notice,
        logic::{
            count_stones, get_flips, get_valid_moves, is_game_over, Board, PassEvent, ValidMoves,
        },
        pieces::{spawn_piece, FlipDoneEvent, FlipQueue},
    },
    state::{CurrentTurn, GameMode, GameResult, GameState, KoreanFont, Player},
};

const FLIP_INTERVAL: f32 = 0.08;

// ─── 마우스 클릭으로 돌 놓기 ─────────────────────────────────────────────────
#[allow(clippy::too_many_arguments)]
pub fn handle_board_click(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut board: ResMut<Board>,
    valid_moves: Res<ValidMoves>,
    turn: Res<CurrentTurn>,
    game_mode: Res<GameMode>,
    assets: Res<GameAssets>,
    mut commands: Commands,
) {
    // AI 턴이면 무시
    if *game_mode == GameMode::PvAI && turn.0 == Player::White {
        return;
    }

    if !mouse_buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.get_single() else { return };
    let Some(cursor) = window.cursor_position() else { return };
    let Ok((camera, cam_transform)) = camera_q.get_single() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor) else { return };
    let Some((row, col)) = world_to_cell(world_pos) else { return };

    if !valid_moves.0.contains(&(row, col)) {
        return;
    }

    // 뒤집힐 셀 목록 (보드 수정 전에 계산)
    let flips = get_flips(&board, row, col, turn.0);

    // 놓은 돌만 보드에 반영 + 스폰
    board.cells[row][col] = Some(turn.0);
    spawn_piece(&mut commands, &assets, row, col, turn.0);

    // 뒤집기 큐 생성 (순차 애니메이션)
    commands.insert_resource(FlipQueue {
        flips: VecDeque::from(flips),
        player: turn.0,
        timer: Timer::from_seconds(FLIP_INTERVAL, TimerMode::Repeating),
    });
}

// ─── 뒤집기 완료 후 턴 전환 + 게임오버 체크 ─────────────────────────────────
#[allow(clippy::too_many_arguments)]
pub fn on_flip_done(
    mut events: EventReader<FlipDoneEvent>,
    board: Res<Board>,
    mut valid_moves: ResMut<ValidMoves>,
    mut turn: ResMut<CurrentTurn>,
    mut pass_events: EventWriter<PassEvent>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    for _ in events.read() {
        // 게임 종료 체크
        if is_game_over(&board) {
            let (black, white) = count_stones(&board);
            commands.insert_resource(GameResult {
                black_score: black,
                white_score: white,
            });
            next_state.set(GameState::GameOver);
            return;
        }

        // 턴 전환
        advance_turn(&board, &mut valid_moves, &mut turn, &mut pass_events);
    }
}

/// 턴 교체 + 패스 처리
pub fn advance_turn(
    board: &Board,
    valid_moves: &mut ResMut<ValidMoves>,
    turn: &mut ResMut<CurrentTurn>,
    pass_events: &mut EventWriter<PassEvent>,
) {
    let next_player = turn.0.opponent();
    let moves = get_valid_moves(board, next_player);

    if moves.is_empty() {
        // 상대 패스 → 현재 플레이어 유효수 재계산
        pass_events.send(PassEvent(next_player));
        valid_moves.0 = get_valid_moves(board, turn.0);
    } else {
        turn.0 = next_player;
        valid_moves.0 = moves;
    }
}

// ─── 패스 이벤트 → 알림 표시 ─────────────────────────────────────────────────
pub fn on_pass_event(
    mut commands: Commands,
    mut events: EventReader<PassEvent>,
    font: Res<KoreanFont>,
) {
    for PassEvent(player) in events.read() {
        show_pass_notice(&mut commands, *player, font.0.clone());
    }
}
