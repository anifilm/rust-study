use crate::rotation::{rotated_piece, rotation_kicks, RotationDirection};
use crate::tetromino::{Tetromino, TetrominoType};

pub const BOARD_WIDTH: usize = 10;
pub const BOARD_HEIGHT: usize = 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Empty,
    Filled(TetrominoType),
}

#[derive(Clone)]
pub struct Board {
    cells: [[Cell; BOARD_WIDTH]; BOARD_HEIGHT],
}

impl Board {
    pub fn new() -> Self {
        Self {
            cells: [[Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT],
        }
    }

    pub fn cell(&self, x: usize, y: usize) -> Cell {
        self.cells[y][x]
    }

    pub fn collides(&self, piece: &Tetromino) -> bool {
        piece.blocks().iter().any(|block| {
            if block.x < 0 || block.x >= BOARD_WIDTH as i32 || block.y >= BOARD_HEIGHT as i32 {
                return true;
            }

            if block.y < 0 {
                return false;
            }

            self.cells[block.y as usize][block.x as usize] != Cell::Empty
        })
    }

    pub fn try_move(&self, piece: &Tetromino, dx: i32, dy: i32) -> Option<Tetromino> {
        let moved = piece.with_offset(dx, dy);
        (!self.collides(&moved)).then_some(moved)
    }

    pub fn try_rotate(&self, piece: &Tetromino, direction: RotationDirection) -> Option<Tetromino> {
        let rotated = rotated_piece(*piece, direction);
        for (kick_x, kick_y) in rotation_kicks(piece.kind, piece.rotation, rotated.rotation) {
            let candidate = rotated.with_offset(*kick_x, -*kick_y);
            if !self.collides(&candidate) {
                return Some(candidate);
            }
        }
        None
    }

    pub fn hard_drop_position(&self, piece: &Tetromino) -> Tetromino {
        let mut dropped = *piece;
        while let Some(next) = self.try_move(&dropped, 0, 1) {
            dropped = next;
        }
        dropped
    }

    pub fn lock_piece(&mut self, piece: &Tetromino) {
        for block in piece.blocks() {
            if block.y < 0 {
                continue;
            }

            self.cells[block.y as usize][block.x as usize] = Cell::Filled(piece.kind);
        }
    }

    #[allow(dead_code)]
    pub fn clear_lines(&mut self) -> usize {
        let cleared = self.full_rows().len();
        self.clear_full_rows();
        cleared
    }

    pub fn full_rows(&self) -> Vec<usize> {
        self.cells
            .iter()
            .enumerate()
            .filter_map(|(index, row)| row.iter().all(|cell| *cell != Cell::Empty).then_some(index))
            .collect()
    }

    pub fn clear_full_rows(&mut self) {
        let mut retained: Vec<[Cell; BOARD_WIDTH]> = self
            .cells
            .iter()
            .copied()
            .filter(|row| row.iter().any(|cell| *cell == Cell::Empty))
            .collect();

        while retained.len() < BOARD_HEIGHT {
            retained.insert(0, [Cell::Empty; BOARD_WIDTH]);
        }

        self.cells = retained.try_into().expect("board rows must remain fixed");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tetromino::TetrominoType;

    #[test]
    fn clears_full_line() {
        let mut board = Board::new();
        for x in 0..BOARD_WIDTH {
            board.cells[BOARD_HEIGHT - 1][x] = Cell::Filled(TetrominoType::I);
        }

        assert_eq!(board.clear_lines(), 1);
        assert!(board.cells[0].iter().all(|cell| *cell == Cell::Empty));
    }
}
