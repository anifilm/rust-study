use rand::Rng;

use crate::{
    constants::BOARD_SIZE,
    game::logic::{get_valid_moves, place_stone, Board},
    state::{AiDifficulty, Player},
};

// ─── 위치 가중치 행렬 ────────────────────────────────────────────────────────
#[rustfmt::skip]
const POSITION_WEIGHTS: [[i32; BOARD_SIZE]; BOARD_SIZE] = [
    [100, -25, 10,  5,  5, 10, -25, 100],
    [-25, -45,  1,  1,  1,  1, -45, -25],
    [ 10,   1,  3,  2,  2,  3,   1,  10],
    [  5,   1,  2,  1,  1,  2,   1,   5],
    [  5,   1,  2,  1,  1,  2,   1,   5],
    [ 10,   1,  3,  2,  2,  3,   1,  10],
    [-25, -45,  1,  1,  1,  1, -45, -25],
    [100, -25, 10,  5,  5, 10, -25, 100],
];

// ─── 평가 함수 ────────────────────────────────────────────────────────────────
fn evaluate(board: &Board, ai_player: Player) -> i32 {
    let opp = ai_player.opponent();
    let mut score = 0i32;
    for (row, weight_row) in POSITION_WEIGHTS.iter().enumerate() {
        for (col, &w) in weight_row.iter().enumerate() {
            match board.cells[row][col] {
                Some(p) if p == ai_player => score += w,
                Some(_) => score -= w,
                None => {}
            }
        }
    }
    // 이동 가능수 차이 (기동성)
    let ai_moves = get_valid_moves(board, ai_player).len() as i32;
    let opp_moves = get_valid_moves(board, opp).len() as i32;
    score += (ai_moves - opp_moves) * 5;
    score
}

// ─── Minimax + Alpha-Beta ─────────────────────────────────────────────────────
pub fn minimax(
    board: &Board,
    depth: u8,
    mut alpha: i32,
    mut beta: i32,
    maximizing: bool,
    ai_player: Player,
) -> i32 {
    let current_player = if maximizing { ai_player } else { ai_player.opponent() };
    let moves = get_valid_moves(board, current_player);

    if depth == 0 || (moves.is_empty() && get_valid_moves(board, current_player.opponent()).is_empty()) {
        return evaluate(board, ai_player);
    }

    // 패스 처리
    if moves.is_empty() {
        return minimax(board, depth, alpha, beta, !maximizing, ai_player);
    }

    if maximizing {
        let mut best = i32::MIN;
        for (row, col) in moves {
            let mut new_board = board.clone();
            place_stone(&mut new_board, row, col, current_player);
            let val = minimax(&new_board, depth - 1, alpha, beta, false, ai_player);
            best = best.max(val);
            alpha = alpha.max(best);
            if beta <= alpha {
                break; // β 컷
            }
        }
        best
    } else {
        let mut best = i32::MAX;
        for (row, col) in moves {
            let mut new_board = board.clone();
            place_stone(&mut new_board, row, col, current_player);
            let val = minimax(&new_board, depth - 1, alpha, beta, true, ai_player);
            best = best.min(val);
            beta = beta.min(best);
            if beta <= alpha {
                break; // α 컷
            }
        }
        best
    }
}

// ─── 난이도별 최선수 반환 ────────────────────────────────────────────────────
/// Easy: 40% 확률로 무작위 수, 나머지는 depth 2 minimax
/// Normal / Hard: 순수 minimax (depth는 AiDifficulty::depth() 사용)
pub fn best_move(board: &Board, ai_player: Player, difficulty: AiDifficulty) -> Option<(usize, usize)> {
    let moves = get_valid_moves(board, ai_player);
    if moves.is_empty() {
        return None;
    }

    if difficulty == AiDifficulty::Easy {
        let mut rng = rand::thread_rng();
        if rng.gen_bool(0.40) {
            // 40% 확률로 완전 무작위 수
            let idx = rng.gen_range(0..moves.len());
            return Some(moves[idx]);
        }
    }

    let depth = difficulty.depth();
    let mut best_val = i32::MIN;
    let mut best_pos = moves[0];

    for (row, col) in moves {
        let mut new_board = board.clone();
        place_stone(&mut new_board, row, col, ai_player);
        let val = minimax(&new_board, depth - 1, i32::MIN, i32::MAX, false, ai_player);
        if val > best_val {
            best_val = val;
            best_pos = (row, col);
        }
    }
    Some(best_pos)
}
