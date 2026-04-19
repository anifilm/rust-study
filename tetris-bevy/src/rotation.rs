use crate::tetromino::{Tetromino, TetrominoType};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RotationDirection {
    Clockwise,
    CounterClockwise,
}

pub fn next_rotation(current: usize, direction: RotationDirection) -> usize {
    match direction {
        RotationDirection::Clockwise => (current + 1) % 4,
        RotationDirection::CounterClockwise => (current + 3) % 4,
    }
}

pub fn rotation_kicks(piece: TetrominoType, from: usize, to: usize) -> &'static [(i32, i32); 5] {
    match piece {
        TetrominoType::O => &O_KICKS,
        TetrominoType::I => match (from % 4, to % 4) {
            (0, 1) => &I_0_1,
            (1, 0) => &I_1_0,
            (1, 2) => &I_1_2,
            (2, 1) => &I_2_1,
            (2, 3) => &I_2_3,
            (3, 2) => &I_3_2,
            (3, 0) => &I_3_0,
            (0, 3) => &I_0_3,
            _ => &O_KICKS,
        },
        _ => match (from % 4, to % 4) {
            (0, 1) => &JLSTZ_0_1,
            (1, 0) => &JLSTZ_1_0,
            (1, 2) => &JLSTZ_1_2,
            (2, 1) => &JLSTZ_2_1,
            (2, 3) => &JLSTZ_2_3,
            (3, 2) => &JLSTZ_3_2,
            (3, 0) => &JLSTZ_3_0,
            (0, 3) => &JLSTZ_0_3,
            _ => &O_KICKS,
        },
    }
}

pub fn rotated_piece(piece: Tetromino, direction: RotationDirection) -> Tetromino {
    piece.rotated(next_rotation(piece.rotation, direction))
}

const O_KICKS: [(i32, i32); 5] = [(0, 0); 5];

const JLSTZ_0_1: [(i32, i32); 5] = [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)];
const JLSTZ_1_0: [(i32, i32); 5] = [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)];
const JLSTZ_1_2: [(i32, i32); 5] = [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)];
const JLSTZ_2_1: [(i32, i32); 5] = [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)];
const JLSTZ_2_3: [(i32, i32); 5] = [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)];
const JLSTZ_3_2: [(i32, i32); 5] = [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)];
const JLSTZ_3_0: [(i32, i32); 5] = [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)];
const JLSTZ_0_3: [(i32, i32); 5] = [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)];

const I_0_1: [(i32, i32); 5] = [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)];
const I_1_0: [(i32, i32); 5] = [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)];
const I_1_2: [(i32, i32); 5] = [(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)];
const I_2_1: [(i32, i32); 5] = [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)];
const I_2_3: [(i32, i32); 5] = [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)];
const I_3_2: [(i32, i32); 5] = [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)];
const I_3_0: [(i32, i32); 5] = [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)];
const I_0_3: [(i32, i32); 5] = [(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)];
