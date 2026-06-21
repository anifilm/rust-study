//! 체스 규칙 엔진.
//!
//! 이 모듈은 Bevy 와 완전히 분리된 순수 로직이다. 보드 상태는 단순한
//! 64칸 배열(`Squares`)로 표현하며, 합법 수 생성/체크 판정/게임 종료 판정 등을
//! 모두 여기서 처리한다. 덕분에 렌더링 코드(`play.rs`)는 규칙을 신경 쓰지 않고
//! "선택한 칸의 합법 수 목록"만 받아 사용할 수 있다.

/// 기물 색.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Color {
    White,
    Black,
}

impl Color {
    /// 상대 색.
    pub fn opposite(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    pub fn label_ko(self) -> &'static str {
        match self {
            Color::White => "백(White)",
            Color::Black => "흑(Black)",
        }
    }
}

/// 기물 종류.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Kind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl Kind {
    /// 기물 위에 그릴 글자.
    pub fn letter(self) -> &'static str {
        match self {
            Kind::Pawn => "P",
            Kind::Knight => "N",
            Kind::Bishop => "B",
            Kind::Rook => "R",
            Kind::Queen => "Q",
            Kind::King => "K",
        }
    }
}

/// 한 칸에 놓인 기물.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Piece {
    pub color: Color,
    pub kind: Kind,
    /// 캐슬링/폰 더블 푸시 판정을 위해 한 번이라도 움직였는지 기록한다.
    pub has_moved: bool,
}

impl Piece {
    fn new(color: Color, kind: Kind) -> Self {
        Self {
            color,
            kind,
            has_moved: false,
        }
    }
}

/// 보드 좌표. `file` 은 열(0=a ~ 7=h), `rank` 는 행(0=1행 ~ 7=8행).
/// 백은 rank 0,1 에서 시작하여 rank 가 커지는 방향으로 전진한다.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Square {
    pub file: i32,
    pub rank: i32,
}

impl Square {
    pub fn new(file: i32, rank: i32) -> Self {
        Self { file, rank }
    }

    pub fn in_bounds(self) -> bool {
        (0..8).contains(&self.file) && (0..8).contains(&self.rank)
    }

    pub fn index(self) -> usize {
        (self.rank * 8 + self.file) as usize
    }

    fn offset(self, df: i32, dr: i32) -> Square {
        Square::new(self.file + df, self.rank + dr)
    }
}

/// 64칸 보드. 인덱스는 `Square::index()`.
pub type Squares = [Option<Piece>; 64];

/// 수의 특수 종류.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MoveFlag {
    Normal,
    /// 폰 2칸 전진(앙파상 타겟 발생).
    DoublePawn,
    /// 앙파상 캡처.
    EnPassant,
    /// 캐슬링.
    Castle,
    /// 승급(이 구현에서는 자동으로 퀸 승급).
    Promotion,
}

/// 하나의 수.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ChessMove {
    pub from: Square,
    pub to: Square,
    pub flag: MoveFlag,
}

/// 게임 진행 상태.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    Ongoing,
    Check,
    Checkmate { winner: Color },
    Stalemate,
}

/// 표준 체스 시작 배치를 생성한다.
pub fn initial_squares() -> Squares {
    let mut sq: Squares = [None; 64];

    let back_rank = [
        Kind::Rook,
        Kind::Knight,
        Kind::Bishop,
        Kind::Queen,
        Kind::King,
        Kind::Bishop,
        Kind::Knight,
        Kind::Rook,
    ];

    for (file, kind) in back_rank.iter().enumerate() {
        let file = file as i32;
        sq[Square::new(file, 0).index()] = Some(Piece::new(Color::White, *kind));
        sq[Square::new(file, 1).index()] = Some(Piece::new(Color::White, Kind::Pawn));
        sq[Square::new(file, 7).index()] = Some(Piece::new(Color::Black, *kind));
        sq[Square::new(file, 6).index()] = Some(Piece::new(Color::Black, Kind::Pawn));
    }

    sq
}

pub fn piece_at(squares: &Squares, sq: Square) -> Option<Piece> {
    if sq.in_bounds() {
        squares[sq.index()]
    } else {
        None
    }
}

/// 특정 칸이 `by` 색의 기물에게 공격받고 있는지 판정한다.
/// 체크 판정과 캐슬링 경로 안전성 검사에 쓰인다.
pub fn is_square_attacked(squares: &Squares, target: Square, by: Color) -> bool {
    // 폰 공격: by 색 폰은 자기 전진 방향의 대각선을 공격한다.
    // by 색 폰이 target 을 공격하려면, target 의 (전진 반대 방향) 대각선에 폰이 있어야 한다.
    let pawn_dir = match by {
        Color::White => 1,
        Color::Black => -1,
    };
    for df in [-1, 1] {
        let from = target.offset(df, -pawn_dir);
        if let Some(p) = piece_at(squares, from) {
            if p.color == by && p.kind == Kind::Pawn {
                return true;
            }
        }
    }

    // 나이트.
    const KNIGHT: [(i32, i32); 8] = [
        (1, 2),
        (2, 1),
        (2, -1),
        (1, -2),
        (-1, -2),
        (-2, -1),
        (-2, 1),
        (-1, 2),
    ];
    for (df, dr) in KNIGHT {
        if let Some(p) = piece_at(squares, target.offset(df, dr)) {
            if p.color == by && p.kind == Kind::Knight {
                return true;
            }
        }
    }

    // 킹(인접 8칸).
    for df in -1..=1 {
        for dr in -1..=1 {
            if df == 0 && dr == 0 {
                continue;
            }
            if let Some(p) = piece_at(squares, target.offset(df, dr)) {
                if p.color == by && p.kind == Kind::King {
                    return true;
                }
            }
        }
    }

    // 슬라이딩(비숍/룩/퀸).
    const DIAG: [(i32, i32); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
    const ORTHO: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

    for (df, dr) in DIAG {
        if ray_hits(squares, target, df, dr, by, &[Kind::Bishop, Kind::Queen]) {
            return true;
        }
    }
    for (df, dr) in ORTHO {
        if ray_hits(squares, target, df, dr, by, &[Kind::Rook, Kind::Queen]) {
            return true;
        }
    }

    false
}

/// `from` 에서 (df,dr) 방향으로 광선을 쏴, 처음 만나는 기물이 `by` 색이면서
/// `kinds` 중 하나면 true.
fn ray_hits(
    squares: &Squares,
    from: Square,
    df: i32,
    dr: i32,
    by: Color,
    kinds: &[Kind],
) -> bool {
    let mut cur = from.offset(df, dr);
    while cur.in_bounds() {
        if let Some(p) = squares[cur.index()] {
            return p.color == by && kinds.contains(&p.kind);
        }
        cur = cur.offset(df, dr);
    }
    false
}

/// 특정 색 킹의 위치.
pub fn find_king(squares: &Squares, color: Color) -> Option<Square> {
    for rank in 0..8 {
        for file in 0..8 {
            let s = Square::new(file, rank);
            if let Some(p) = squares[s.index()] {
                if p.color == color && p.kind == Kind::King {
                    return Some(s);
                }
            }
        }
    }
    None
}

pub fn is_in_check(squares: &Squares, color: Color) -> bool {
    match find_king(squares, color) {
        Some(king) => is_square_attacked(squares, king, color.opposite()),
        None => false,
    }
}

/// 수를 적용한 새 보드와 새 앙파상 타겟을 반환한다(원본은 변경하지 않음).
pub fn apply_move(squares: &Squares, mv: ChessMove) -> (Squares, Option<Square>) {
    let mut next = *squares;
    let mut new_en_passant = None;

    let mut piece = match next[mv.from.index()] {
        Some(p) => p,
        None => return (next, None),
    };
    piece.has_moved = true;

    next[mv.from.index()] = None;

    match mv.flag {
        MoveFlag::EnPassant => {
            // 앙파상: 도착 칸 뒤(자기 전진 반대 방향)에 있는 상대 폰을 제거.
            let dir = if piece.color == Color::White { 1 } else { -1 };
            let captured = Square::new(mv.to.file, mv.to.rank - dir);
            next[captured.index()] = None;
            next[mv.to.index()] = Some(piece);
        }
        MoveFlag::Castle => {
            next[mv.to.index()] = Some(piece);
            // 룩 이동: 킹사이드(to.file=6) / 퀸사이드(to.file=2).
            let (rook_from_file, rook_to_file) = if mv.to.file == 6 {
                (7, 5)
            } else {
                (0, 3)
            };
            let rook_from = Square::new(rook_from_file, mv.from.rank);
            let rook_to = Square::new(rook_to_file, mv.from.rank);
            if let Some(mut rook) = next[rook_from.index()] {
                rook.has_moved = true;
                next[rook_from.index()] = None;
                next[rook_to.index()] = Some(rook);
            }
        }
        MoveFlag::Promotion => {
            piece.kind = Kind::Queen;
            next[mv.to.index()] = Some(piece);
        }
        MoveFlag::DoublePawn => {
            let dir = if piece.color == Color::White { 1 } else { -1 };
            new_en_passant = Some(Square::new(mv.from.file, mv.from.rank + dir));
            next[mv.to.index()] = Some(piece);
        }
        MoveFlag::Normal => {
            next[mv.to.index()] = Some(piece);
        }
    }

    (next, new_en_passant)
}

/// `from` 칸 기물의 합법 수(자기 킹이 체크되지 않는 수)를 생성한다.
pub fn legal_moves_from(
    squares: &Squares,
    en_passant: Option<Square>,
    from: Square,
) -> Vec<ChessMove> {
    let piece = match piece_at(squares, from) {
        Some(p) => p,
        None => return Vec::new(),
    };

    pseudo_moves_from(squares, en_passant, from)
        .into_iter()
        .filter(|mv| {
            let (next, _) = apply_move(squares, *mv);
            !is_in_check(&next, piece.color)
        })
        .collect()
}

/// 해당 색의 모든 합법 수.
pub fn all_legal_moves(
    squares: &Squares,
    en_passant: Option<Square>,
    color: Color,
) -> Vec<ChessMove> {
    let mut moves = Vec::new();
    for rank in 0..8 {
        for file in 0..8 {
            let s = Square::new(file, rank);
            if let Some(p) = squares[s.index()] {
                if p.color == color {
                    moves.extend(legal_moves_from(squares, en_passant, s));
                }
            }
        }
    }
    moves
}

/// 한 수를 둔 뒤 `to_move` 색 입장에서의 게임 상태.
pub fn status_for(squares: &Squares, en_passant: Option<Square>, to_move: Color) -> Status {
    let has_moves = !all_legal_moves(squares, en_passant, to_move).is_empty();
    let in_check = is_in_check(squares, to_move);

    if has_moves {
        if in_check {
            Status::Check
        } else {
            Status::Ongoing
        }
    } else if in_check {
        Status::Checkmate {
            winner: to_move.opposite(),
        }
    } else {
        Status::Stalemate
    }
}

/// 체크 여부를 고려하지 않은 의사 합법 수(pseudo-legal).
fn pseudo_moves_from(
    squares: &Squares,
    en_passant: Option<Square>,
    from: Square,
) -> Vec<ChessMove> {
    let piece = match piece_at(squares, from) {
        Some(p) => p,
        None => return Vec::new(),
    };

    let mut moves = Vec::new();

    match piece.kind {
        Kind::Pawn => pawn_moves(squares, en_passant, from, piece, &mut moves),
        Kind::Knight => {
            const KNIGHT: [(i32, i32); 8] = [
                (1, 2),
                (2, 1),
                (2, -1),
                (1, -2),
                (-1, -2),
                (-2, -1),
                (-2, 1),
                (-1, 2),
            ];
            for (df, dr) in KNIGHT {
                push_step(squares, from, df, dr, piece.color, &mut moves);
            }
        }
        Kind::King => {
            for df in -1..=1 {
                for dr in -1..=1 {
                    if df == 0 && dr == 0 {
                        continue;
                    }
                    push_step(squares, from, df, dr, piece.color, &mut moves);
                }
            }
            castling_moves(squares, from, piece, &mut moves);
        }
        Kind::Bishop => slide(squares, from, piece.color, &DIAG, &mut moves),
        Kind::Rook => slide(squares, from, piece.color, &ORTHO, &mut moves),
        Kind::Queen => {
            slide(squares, from, piece.color, &DIAG, &mut moves);
            slide(squares, from, piece.color, &ORTHO, &mut moves);
        }
    }

    moves
}

const DIAG: [(i32, i32); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
const ORTHO: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

/// 한 칸 이동(나이트/킹): 빈 칸이거나 적 기물이면 이동 가능.
fn push_step(
    squares: &Squares,
    from: Square,
    df: i32,
    dr: i32,
    color: Color,
    moves: &mut Vec<ChessMove>,
) {
    let to = from.offset(df, dr);
    if !to.in_bounds() {
        return;
    }
    match squares[to.index()] {
        Some(p) if p.color == color => {}
        _ => moves.push(ChessMove {
            from,
            to,
            flag: MoveFlag::Normal,
        }),
    }
}

/// 슬라이딩 이동(비숍/룩/퀸).
fn slide(
    squares: &Squares,
    from: Square,
    color: Color,
    dirs: &[(i32, i32)],
    moves: &mut Vec<ChessMove>,
) {
    for &(df, dr) in dirs {
        let mut to = from.offset(df, dr);
        while to.in_bounds() {
            match squares[to.index()] {
                None => moves.push(ChessMove {
                    from,
                    to,
                    flag: MoveFlag::Normal,
                }),
                Some(p) => {
                    if p.color != color {
                        moves.push(ChessMove {
                            from,
                            to,
                            flag: MoveFlag::Normal,
                        });
                    }
                    break;
                }
            }
            to = to.offset(df, dr);
        }
    }
}

fn pawn_moves(
    squares: &Squares,
    en_passant: Option<Square>,
    from: Square,
    piece: Piece,
    moves: &mut Vec<ChessMove>,
) {
    let dir = if piece.color == Color::White { 1 } else { -1 };
    let start_rank = if piece.color == Color::White { 1 } else { 6 };
    let promo_rank = if piece.color == Color::White { 7 } else { 0 };

    // 한 칸 전진.
    let one = from.offset(0, dir);
    if one.in_bounds() && squares[one.index()].is_none() {
        push_pawn_forward(from, one, promo_rank, MoveFlag::Normal, moves);

        // 두 칸 전진.
        let two = from.offset(0, dir * 2);
        if from.rank == start_rank && two.in_bounds() && squares[two.index()].is_none() {
            moves.push(ChessMove {
                from,
                to: two,
                flag: MoveFlag::DoublePawn,
            });
        }
    }

    // 대각선 캡처 + 앙파상.
    for df in [-1, 1] {
        let to = from.offset(df, dir);
        if !to.in_bounds() {
            continue;
        }
        match squares[to.index()] {
            Some(p) if p.color != piece.color => {
                push_pawn_forward(from, to, promo_rank, MoveFlag::Normal, moves);
            }
            None => {
                if Some(to) == en_passant {
                    moves.push(ChessMove {
                        from,
                        to,
                        flag: MoveFlag::EnPassant,
                    });
                }
            }
            _ => {}
        }
    }
}

/// 폰이 도착하는 칸이 승급 행이면 Promotion, 아니면 주어진 flag 로 추가.
fn push_pawn_forward(
    from: Square,
    to: Square,
    promo_rank: i32,
    flag: MoveFlag,
    moves: &mut Vec<ChessMove>,
) {
    if to.rank == promo_rank {
        moves.push(ChessMove {
            from,
            to,
            flag: MoveFlag::Promotion,
        });
    } else {
        moves.push(ChessMove { from, to, flag });
    }
}

fn castling_moves(squares: &Squares, from: Square, king: Piece, moves: &mut Vec<ChessMove>) {
    if king.has_moved {
        return;
    }
    // 현재 체크 상태면 캐슬링 불가.
    if is_square_attacked(squares, from, king.color.opposite()) {
        return;
    }

    let rank = from.rank;
    let enemy = king.color.opposite();

    // 킹사이드: f,g 칸이 비어 있고, 킹이 e->f->g 로 지나는 칸이 공격받지 않으며, h 룩이 안 움직였어야 함.
    if rook_unmoved(squares, Square::new(7, rank), king.color)
        && squares[Square::new(5, rank).index()].is_none()
        && squares[Square::new(6, rank).index()].is_none()
        && !is_square_attacked(squares, Square::new(5, rank), enemy)
        && !is_square_attacked(squares, Square::new(6, rank), enemy)
    {
        moves.push(ChessMove {
            from,
            to: Square::new(6, rank),
            flag: MoveFlag::Castle,
        });
    }

    // 퀸사이드: b,c,d 칸이 비어 있고, 킹이 e->d->c 로 지나는 칸이 안전하며, a 룩이 안 움직였어야 함.
    if rook_unmoved(squares, Square::new(0, rank), king.color)
        && squares[Square::new(1, rank).index()].is_none()
        && squares[Square::new(2, rank).index()].is_none()
        && squares[Square::new(3, rank).index()].is_none()
        && !is_square_attacked(squares, Square::new(3, rank), enemy)
        && !is_square_attacked(squares, Square::new(2, rank), enemy)
    {
        moves.push(ChessMove {
            from,
            to: Square::new(2, rank),
            flag: MoveFlag::Castle,
        });
    }
}

fn rook_unmoved(squares: &Squares, at: Square, color: Color) -> bool {
    matches!(
        squares[at.index()],
        Some(Piece {
            kind: Kind::Rook,
            color: c,
            has_moved: false,
        }) if c == color
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 합법 수를 깊이만큼 펼쳐 말단 노드 수를 센다(perft).
    /// 시작 위치에서 처음 3수까지는 승급/캐슬링/앙파상이 발생하지 않으므로
    /// 표준 perft 값과 정확히 일치해야 한다.
    fn perft(squares: &Squares, en_passant: Option<Square>, color: Color, depth: u32) -> u64 {
        if depth == 0 {
            return 1;
        }
        let mut nodes = 0;
        for mv in all_legal_moves(squares, en_passant, color) {
            let (next, ep) = apply_move(squares, mv);
            nodes += perft(&next, ep, color.opposite(), depth - 1);
        }
        nodes
    }

    #[test]
    fn perft_matches_known_values() {
        let squares = initial_squares();
        assert_eq!(perft(&squares, None, Color::White, 1), 20);
        assert_eq!(perft(&squares, None, Color::White, 2), 400);
        assert_eq!(perft(&squares, None, Color::White, 3), 8902);
    }

    #[test]
    fn cannot_leave_king_in_check() {
        // 백 킹 e1, 흑 룩 e8 만 있는 상황: 킹은 e파일을 벗어나는 수만 합법.
        let mut squares: Squares = [None; 64];
        squares[Square::new(4, 0).index()] = Some(Piece::new(Color::White, Kind::King));
        squares[Square::new(4, 7).index()] = Some(Piece::new(Color::Black, Kind::Rook));

        let moves = legal_moves_from(&squares, None, Square::new(4, 0));
        // e파일(file==4)로 가는 수는 여전히 룩의 사정권이라 불법.
        assert!(moves.iter().all(|m| m.to.file != 4));
        assert!(!moves.is_empty());
    }
}
