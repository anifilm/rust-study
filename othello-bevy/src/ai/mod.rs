use std::collections::VecDeque;

use bevy::prelude::*;

use crate::{
    constants::AI_THINK_DELAY,
    game::{
        assets::GameAssets,
        logic::{get_flips, Board},
        pieces::{spawn_piece, FlipQueue},
    },
    state::{AiDifficulty, CurrentTurn, GameMode, GameState, Player},
};

pub mod minimax;

const FLIP_INTERVAL: f32 = 0.08;

#[derive(Resource)]
pub struct AiTimer(pub Timer);

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            ai_move_system
                .run_if(in_state(GameState::Playing))
                .run_if(not(resource_exists::<FlipQueue>)),
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn ai_move_system(
    mut commands: Commands,
    time: Res<Time>,
    game_mode: Res<GameMode>,
    difficulty: Option<Res<AiDifficulty>>,
    turn: Res<CurrentTurn>,
    mut board: ResMut<Board>,
    assets: Res<GameAssets>,
    ai_timer: Option<ResMut<AiTimer>>,
) {
    if *game_mode != GameMode::PvAI || turn.0 != Player::White {
        return;
    }

    // 타이머 없으면 생성
    if ai_timer.is_none() {
        commands.insert_resource(AiTimer(Timer::from_seconds(
            AI_THINK_DELAY,
            TimerMode::Once,
        )));
        return;
    }

    let mut timer = ai_timer.unwrap();
    timer.0.tick(time.delta());
    if !timer.0.finished() {
        return;
    }
    commands.remove_resource::<AiTimer>();

    let diff = difficulty.map(|d| *d).unwrap_or_default();

    if let Some((row, col)) = minimax::best_move(&board, Player::White, diff) {
        // 뒤집힐 셀 목록 (보드 수정 전에 계산)
        let flips = get_flips(&board, row, col, Player::White);

        // 놓은 돌만 보드에 반영 + 스폰
        board.cells[row][col] = Some(Player::White);
        spawn_piece(&mut commands, &assets, row, col, Player::White);

        // 뒤집기 큐 생성
        commands.insert_resource(FlipQueue {
            flips: VecDeque::from(flips),
            player: Player::White,
            timer: Timer::from_seconds(FLIP_INTERVAL, TimerMode::Repeating),
        });
    }
}
