use bevy::prelude::*;

use crate::state::{CurrentTurn, GameState, Player};

pub mod assets;
pub mod board;
pub mod gameover;
pub mod hud;
pub mod input;
pub mod logic;
pub mod pieces;

use logic::{PassEvent, ValidMoves};
use pieces::FlipDoneEvent;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<PassEvent>()
            .add_event::<FlipDoneEvent>()
            // ── Playing 진입 ──────────────────────────────────────────────
            .add_systems(
                OnEnter(GameState::Playing),
                (
                    assets::init_game_assets,
                    board::init_board_resource,
                    apply_deferred,
                    board::spawn_board,
                    pieces::spawn_pieces,
                    hud::spawn_hud,
                    init_turn_and_moves,
                )
                    .chain(),
            )
            // ── Playing 업데이트 ──────────────────────────────────────────
            .add_systems(
                Update,
                (
                    // 애니메이션 시스템 (항상 실행)
                    pieces::animate_flips,
                    // 애니메이션 완료 후 턴 처리
                    input::on_flip_done,
                    input::on_pass_event,
                    // 입력 (애니메이션 중 차단)
                    input::handle_board_click
                        .run_if(not(pieces::is_animating)),
                    // 동기화 (애니메이션 중 차단)
                    pieces::sync_pieces
                        .run_if(not(pieces::is_animating)),
                    pieces::sync_valid_hints
                        .run_if(not(pieces::is_animating)),
                    hud::update_hud,
                    hud::tick_pass_notice,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            // ── Playing 퇴장 ──────────────────────────────────────────────
            .add_systems(OnExit(GameState::Playing), board::despawn_board)
            // ── GameOver ──────────────────────────────────────────────────
            .add_systems(OnEnter(GameState::GameOver), gameover::spawn_gameover)
            .add_systems(
                Update,
                gameover::handle_gameover_buttons.run_if(in_state(GameState::GameOver)),
            )
            .add_systems(OnExit(GameState::GameOver), gameover::despawn_gameover);
    }
}

fn init_turn_and_moves(mut commands: Commands, board: Res<logic::Board>) {
    commands.insert_resource(CurrentTurn(Player::Black));
    let moves = logic::get_valid_moves(&board, Player::Black);
    commands.insert_resource(ValidMoves(moves));
}
