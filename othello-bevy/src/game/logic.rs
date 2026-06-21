use bevy::prelude::*;

use crate::{
    constants::BOARD_SIZE,
    state::Player,
};

// ─── Board Resource ───────────────────────────────────────────────────────────
#[derive(Resource, Clone)]
pub struct Board {
    pub cells: [[Option<Player>; BOARD_SIZE]; BOARD_SIZE],
}

impl Board {
    /// 초기 4개 돌 배치 (BW / WB)
    pub fn new() -> Self {
        let mut cells = [[None; BOARD_SIZE]; BOARD_SIZE];
        let m = BOARD_SIZE / 2;
        cells[m - 1][m - 1] = Some(Player::White);
        cells[m - 1][m] = Some(Player::Black);
        cells[m][m - 1] = Some(Player::Black);
        cells[m][m] = Some(Player::White);
        Self { cells }
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

// ─── 유효수 Resource ──────────────────────────────────────────────────────────
#[derive(Resource, Default)]
pub struct ValidMoves(pub Vec<(usize, usize)>);

// ─── 패스 이벤트 ─────────────────────────────────────────────────────────────
#[derive(Event)]
pub struct PassEvent(pub Player);

// ─── 8방향 ────────────────────────────────────────────────────────────────────
const DIRECTIONS: [(i32, i32); 8] = [
    (-1, -1), (-1, 0), (-1, 1),
    (0, -1),           (0, 1),
    (1, -1),  (1, 0),  (1, 1),
];

// ─── 유효수 계산 ──────────────────────────────────────────────────────────────
pub fn get_valid_moves(board: &Board, player: Player) -> Vec<(usize, usize)> {
    let mut moves = Vec::new();
    for row in 0..BOARD_SIZE {
        for col in 0..BOARD_SIZE {
            if board.cells[row][col].is_none() && can_flip(board, row, col, player) {
                moves.push((row, col));
            }
        }
    }
    moves
}

/// 해당 위치에 player 돌을 놓으면 하나 이상 뒤집힐 수 있는가
fn can_flip(board: &Board, row: usize, col: usize, player: Player) -> bool {
    let opp = player.opponent();
    for (dr, dc) in DIRECTIONS {
        let mut r = row as i32 + dr;
        let mut c = col as i32 + dc;
        let mut found_opp = false;
        while r >= 0 && r < BOARD_SIZE as i32 && c >= 0 && c < BOARD_SIZE as i32 {
            match board.cells[r as usize][c as usize] {
                Some(p) if p == opp => {
                    found_opp = true;
                }
                Some(_) => {
                    if found_opp {
                        return true;
                    }
                    break;
                }
                None => break,
            }
            r += dr;
            c += dc;
        }
    }
    false
}

/// 돌 놓기 + 모든 방향 뒤집기
pub fn place_stone(board: &mut Board, row: usize, col: usize, player: Player) {
    board.cells[row][col] = Some(player);
    let opp = player.opponent();
    for (dr, dc) in DIRECTIONS {
        let mut to_flip = Vec::new();
        let mut r = row as i32 + dr;
        let mut c = col as i32 + dc;
        while r >= 0 && r < BOARD_SIZE as i32 && c >= 0 && c < BOARD_SIZE as i32 {
            match board.cells[r as usize][c as usize] {
                Some(p) if p == opp => {
                    to_flip.push((r as usize, c as usize));
                }
                Some(_) => {
                    // player 돌 발견 → 이 방향 뒤집기 확정
                    for (fr, fc) in &to_flip {
                        board.cells[*fr][*fc] = Some(player);
                    }
                    break;
                }
                None => break,
            }
            r += dr;
            c += dc;
        }
    }
}

/// 놓을 위치에서 뒤집힐 돌 좌표 목록 반환 (보드 수정 없음)
/// 방향별로 가까운 돌부터 순서대로 반환
pub fn get_flips(board: &Board, row: usize, col: usize, player: Player) -> Vec<(usize, usize)> {
    let opp = player.opponent();
    let mut all_flips = Vec::new();
    for (dr, dc) in DIRECTIONS {
        let mut line_flips = Vec::new();
        let mut r = row as i32 + dr;
        let mut c = col as i32 + dc;
        while r >= 0 && r < BOARD_SIZE as i32 && c >= 0 && c < BOARD_SIZE as i32 {
            match board.cells[r as usize][c as usize] {
                Some(p) if p == opp => {
                    line_flips.push((r as usize, c as usize));
                }
                Some(_) => {
                    // 아군 돌로 끝남 → 이 방향 뒤집기 확정
                    all_flips.extend(line_flips);
                    break;
                }
                None => break,
            }
            r += dr;
            c += dc;
        }
    }
    all_flips
}

/// 점수 집계
pub fn count_stones(board: &Board) -> (u32, u32) {
    let mut black = 0u32;
    let mut white = 0u32;
    for row in &board.cells {
        for cell in row {
            match cell {
                Some(Player::Black) => black += 1,
                Some(Player::White) => white += 1,
                None => {}
            }
        }
    }
    (black, white)
}

/// 게임 종료 여부 (양쪽 모두 유효수 없음)
pub fn is_game_over(board: &Board) -> bool {
    get_valid_moves(board, Player::Black).is_empty()
        && get_valid_moves(board, Player::White).is_empty()
}
