//! 미니맥스 + 알파-베타 기반 체스 AI.
//!
//! Bevy 와 완전히 분리된 순수 로직이며, `rules` 모듈의 API 만 사용한다.

use super::rules::{
    all_legal_moves, apply_move, is_in_check, piece_at, status_for, ChessMove, Color, Kind,
    MoveFlag, Square, Squares, Status,
};

const MATE_SCORE: i32 = 100_000;
const CHECK_BONUS: i32 = 30;

const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 320;
const BISHOP_VALUE: i32 = 330;
const ROOK_VALUE: i32 = 500;
const QUEEN_VALUE: i32 = 900;
const KING_VALUE: i32 = 20_000;

/// 기물별 위치 보너스(백 기준, 흑은 rank 를 뒤집어 적용).
const PAWN_TABLE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 50, 50, 50, 50, 50, 50, 50, 50, 10, 10, 20, 30, 30, 20, 10, 10, 5, 5,
    10, 25, 25, 10, 5, 5, 0, 0, 0, 20, 20, 0, 0, 0, 5, -5, -10, 0, 0, -10, -5, 5, 5, 10, 10, -20,
    -20, 10, 10, 5, 0, 0, 0, 0, 0, 0, 0, 0,
];

const KNIGHT_TABLE: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50, -40, -20, 0, 0, 0, 0, -20, -40, -30, 0, 10, 15, 15,
    10, 0, -30, -30, 5, 15, 20, 20, 15, 5, -30, -30, 0, 15, 20, 20, 15, 0, -30, -30, 5, 10, 15,
    15, 10, 5, -30, -40, -20, 0, 5, 5, 0, -20, -40, -50, -40, -30, -30, -30, -30, -40, -50,
];

const BISHOP_TABLE: [i32; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 10, 10, 5, 0,
    -10, -10, 5, 5, 10, 10, 5, 5, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10, 10, 10, 10, 10, 10,
    10, -10, -10, 5, 0, 0, 0, 0, 5, -10, -20, -10, -10, -10, -10, -10, -10, -20,
];

const ROOK_TABLE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 10, 10, 10, 10, 10, 5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0,
    0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5,
    5, 10, 10, 10, 10, 10, 10, 5,
];

const QUEEN_TABLE: [i32; 64] = [
    -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 5, 5, 5, 0, -10,
    -5, 0, 5, 5, 5, 5, 0, -5, 0, 0, 5, 5, 5, 5, 0, -5, -10, 5, 5, 5, 5, 5, 0, -10, -10, 0, 5, 0,
    0, 0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
];

const KING_MID_TABLE: [i32; 64] = [
    -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40,
    -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -20, -30, -30, -40,
    -40, -30, -30, -20, -10, -20, -20, -20, -20, -20, -20, -10, 20, 20, 0, 0, 0, 0, 20, 20, 20,
    30, 10, 0, 0, 10, 30, 20,
];

/// 주어진 깊이로 최선의 합법 수를 고른다. 합법 수가 없으면 `None`.
pub fn choose_move(
    squares: &Squares,
    en_passant: Option<Square>,
    color: Color,
    depth: u8,
) -> Option<ChessMove> {
    let moves = all_legal_moves(squares, en_passant, color);
    if moves.is_empty() {
        return None;
    }

    let mut best_move = moves[0];
    let mut best_score = i32::MIN;
    let mut alpha = i32::MIN + 1;
    let beta = i32::MAX;

    let mut ordered = moves;
    order_moves(&mut ordered, squares);

    for mv in ordered {
        let (next, ep) = apply_move(squares, mv);
        let score = -negamax(&next, ep, color.opposite(), depth - 1, -beta, -alpha);
        if score > best_score {
            best_score = score;
            best_move = mv;
        }
        alpha = alpha.max(score);
    }

    Some(best_move)
}

fn negamax(
    squares: &Squares,
    en_passant: Option<Square>,
    color: Color,
    depth: u8,
    alpha: i32,
    beta: i32,
) -> i32 {
    let status = status_for(squares, en_passant, color);
    if depth == 0 || matches!(status, Status::Checkmate { .. } | Status::Stalemate) {
        return evaluate(squares, en_passant, color, status);
    }

    let mut moves = all_legal_moves(squares, en_passant, color);
    if moves.is_empty() {
        return evaluate(squares, en_passant, color, status);
    }
    order_moves(&mut moves, squares);

    let mut best = i32::MIN + 1;
    let mut alpha = alpha;

    for mv in moves {
        let (next, ep) = apply_move(squares, mv);
        let score = -negamax(&next, ep, color.opposite(), depth - 1, -beta, -alpha);
        best = best.max(score);
        alpha = alpha.max(score);
        if alpha >= beta {
            break;
        }
    }

    best
}

fn evaluate(squares: &Squares, en_passant: Option<Square>, color: Color, status: Status) -> i32 {
    match status {
        Status::Checkmate { winner } => {
            return if winner == color {
                MATE_SCORE
            } else {
                -MATE_SCORE
            };
        }
        Status::Stalemate => return 0,
        _ => {}
    }

    let mut score = 0;
    for rank in 0..8 {
        for file in 0..8 {
            let sq = Square::new(file, rank);
            if let Some(piece) = squares[sq.index()] {
                let value = material_value(piece.kind) + positional_bonus(piece.kind, piece.color, sq);
                if piece.color == color {
                    score += value;
                } else {
                    score -= value;
                }
            }
        }
    }

    if is_in_check(squares, color) {
        score -= CHECK_BONUS;
    }
    if is_in_check(squares, color.opposite()) {
        score += CHECK_BONUS;
    }

    // 앙파상 타겟이 있으면 미세한 활동성 보너스(동점 타이브레이크용).
    if en_passant.is_some() {
        score += if color == Color::White { 1 } else { -1 };
    }

    score
}

fn material_value(kind: Kind) -> i32 {
    match kind {
        Kind::Pawn => PAWN_VALUE,
        Kind::Knight => KNIGHT_VALUE,
        Kind::Bishop => BISHOP_VALUE,
        Kind::Rook => ROOK_VALUE,
        Kind::Queen => QUEEN_VALUE,
        Kind::King => KING_VALUE,
    }
}

fn positional_bonus(kind: Kind, color: Color, sq: Square) -> i32 {
    let idx = if color == Color::White {
        sq.index()
    } else {
        Square::new(sq.file, 7 - sq.rank).index()
    };

    match kind {
        Kind::Pawn => PAWN_TABLE[idx],
        Kind::Knight => KNIGHT_TABLE[idx],
        Kind::Bishop => BISHOP_TABLE[idx],
        Kind::Rook => ROOK_TABLE[idx],
        Kind::Queen => QUEEN_TABLE[idx],
        Kind::King => KING_MID_TABLE[idx],
    }
}

/// 캡처 수를 앞에 두어 알파-베타 가지치기 효율을 높인다.
fn order_moves(moves: &mut [ChessMove], squares: &Squares) {
    moves.sort_by(|a, b| {
        let score_a = move_order_score(squares, *a);
        let score_b = move_order_score(squares, *b);
        score_b.cmp(&score_a)
    });
}

fn move_order_score(squares: &Squares, mv: ChessMove) -> i32 {
    let mut score = 0;
    if mv.flag == MoveFlag::Castle {
        score += 5;
    }
    if piece_at(squares, mv.to).is_some() || mv.flag == MoveFlag::EnPassant {
        score += 1000;
        if let Some(captured) = piece_at(squares, mv.to) {
            if let Some(mover) = piece_at(squares, mv.from) {
                score += material_value(captured.kind) - material_value(mover.kind);
            }
        }
    }
    if mv.flag == MoveFlag::Promotion {
        score += 900;
    }
    score
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chess::rules::{initial_squares, legal_moves_from, Piece};

    fn piece(color: Color, kind: Kind) -> Piece {
        Piece {
            color,
            kind,
            has_moved: false,
        }
    }

    #[test]
    fn chooses_legal_move_from_start() {
        let squares = initial_squares();
        let mv = choose_move(&squares, None, Color::Black, 2).expect("AI should move");
        let legal = all_legal_moves(&squares, None, Color::Black);
        assert!(legal.contains(&mv));
    }

    #[test]
    fn avoids_leaving_king_in_check() {
        let mut squares: Squares = [None; 64];
        squares[Square::new(4, 0).index()] = Some(piece(Color::White, Kind::King));
        squares[Square::new(4, 7).index()] = Some(piece(Color::Black, Kind::Rook));
        squares[Square::new(0, 0).index()] = Some(piece(Color::White, Kind::Rook));

        let mv = choose_move(&squares, None, Color::White, 3).expect("AI should move");
        let (next, _) = apply_move(&squares, mv);
        assert!(!is_in_check(&next, Color::White));
    }

    #[test]
    fn deterministic_for_same_position() {
        let squares = initial_squares();
        let a = choose_move(&squares, None, Color::Black, 2);
        let b = choose_move(&squares, None, Color::Black, 2);
        assert_eq!(a, b);
    }

    #[test]
    fn prefers_capture_when_obvious() {
        let mut squares: Squares = [None; 64];
        squares[Square::new(4, 0).index()] = Some(piece(Color::White, Kind::King));
        squares[Square::new(0, 0).index()] = Some(piece(Color::White, Kind::Rook));
        squares[Square::new(4, 7).index()] = Some(piece(Color::Black, Kind::King));
        squares[Square::new(4, 6).index()] = Some(piece(Color::Black, Kind::Queen));

        // 흑 퀸이 백 룩을 잡을 수 있는 상황.
        let mv = choose_move(&squares, None, Color::Black, 3).expect("AI should move");
        assert_eq!(mv.from, Square::new(4, 6));
        assert_eq!(mv.to, Square::new(4, 0));
        assert!(legal_moves_from(&squares, None, mv.from).iter().any(|m| *m == mv));
    }
}