#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TetrominoType {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockCell {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tetromino {
    pub kind: TetrominoType,
    pub rotation: usize,
    pub x: i32,
    pub y: i32,
}

impl Tetromino {
    pub fn new(kind: TetrominoType) -> Self {
        Self {
            kind,
            rotation: 0,
            x: 3,
            y: 0,
        }
    }

    pub fn with_offset(mut self, dx: i32, dy: i32) -> Self {
        self.x += dx;
        self.y += dy;
        self
    }

    pub fn rotated(mut self, rotation: usize) -> Self {
        self.rotation = rotation % 4;
        self
    }

    pub fn blocks(&self) -> [BlockCell; 4] {
        let shape = shape(self.kind, self.rotation);
        shape.map(|(x, y)| BlockCell {
            x: self.x + x,
            y: self.y + y,
        })
    }
}

pub fn all_bag_pieces() -> [TetrominoType; 7] {
    [
        TetrominoType::I,
        TetrominoType::O,
        TetrominoType::T,
        TetrominoType::S,
        TetrominoType::Z,
        TetrominoType::J,
        TetrominoType::L,
    ]
}

pub fn shape(kind: TetrominoType, rotation: usize) -> [(i32, i32); 4] {
    match kind {
        TetrominoType::I => I_SHAPES[rotation % 4],
        TetrominoType::O => O_SHAPES[rotation % 4],
        TetrominoType::T => T_SHAPES[rotation % 4],
        TetrominoType::S => S_SHAPES[rotation % 4],
        TetrominoType::Z => Z_SHAPES[rotation % 4],
        TetrominoType::J => J_SHAPES[rotation % 4],
        TetrominoType::L => L_SHAPES[rotation % 4],
    }
}

const I_SHAPES: [[(i32, i32); 4]; 4] = [
    [(0, 1), (1, 1), (2, 1), (3, 1)],
    [(2, 0), (2, 1), (2, 2), (2, 3)],
    [(0, 2), (1, 2), (2, 2), (3, 2)],
    [(1, 0), (1, 1), (1, 2), (1, 3)],
];

const O_SHAPES: [[(i32, i32); 4]; 4] = [
    [(1, 0), (2, 0), (1, 1), (2, 1)],
    [(1, 0), (2, 0), (1, 1), (2, 1)],
    [(1, 0), (2, 0), (1, 1), (2, 1)],
    [(1, 0), (2, 0), (1, 1), (2, 1)],
];

const T_SHAPES: [[(i32, i32); 4]; 4] = [
    [(1, 0), (0, 1), (1, 1), (2, 1)],
    [(1, 0), (1, 1), (2, 1), (1, 2)],
    [(0, 1), (1, 1), (2, 1), (1, 2)],
    [(1, 0), (0, 1), (1, 1), (1, 2)],
];

const S_SHAPES: [[(i32, i32); 4]; 4] = [
    [(1, 0), (2, 0), (0, 1), (1, 1)],
    [(1, 0), (1, 1), (2, 1), (2, 2)],
    [(1, 1), (2, 1), (0, 2), (1, 2)],
    [(0, 0), (0, 1), (1, 1), (1, 2)],
];

const Z_SHAPES: [[(i32, i32); 4]; 4] = [
    [(0, 0), (1, 0), (1, 1), (2, 1)],
    [(2, 0), (1, 1), (2, 1), (1, 2)],
    [(0, 1), (1, 1), (1, 2), (2, 2)],
    [(1, 0), (0, 1), (1, 1), (0, 2)],
];

const J_SHAPES: [[(i32, i32); 4]; 4] = [
    [(0, 0), (0, 1), (1, 1), (2, 1)],
    [(1, 0), (2, 0), (1, 1), (1, 2)],
    [(0, 1), (1, 1), (2, 1), (2, 2)],
    [(1, 0), (1, 1), (0, 2), (1, 2)],
];

const L_SHAPES: [[(i32, i32); 4]; 4] = [
    [(2, 0), (0, 1), (1, 1), (2, 1)],
    [(1, 0), (1, 1), (1, 2), (2, 2)],
    [(0, 1), (1, 1), (2, 1), (0, 2)],
    [(0, 0), (1, 0), (1, 1), (1, 2)],
];
