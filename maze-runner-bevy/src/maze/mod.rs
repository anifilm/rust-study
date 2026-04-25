pub mod generator;
pub mod renderer;

/// Represents a single cell in the maze grid.
/// `walls` uses bit flags: 1=up, 2=down, 4=left, 8=right
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MazeCell {
    pub walls: u8,
    pub visited: bool,
}

impl MazeCell {
    pub fn new() -> Self {
        Self {
            walls: 0b1111, // all walls present
            visited: false,
        }
    }

    pub fn remove_wall(&mut self, direction: Direction) {
        self.walls &= !(direction as u8);
    }

    pub fn has_wall(&self, direction: Direction) -> bool {
        self.walls & (direction as u8) != 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    Up = 1,
    Down = 2,
    Left = 4,
    Right = 8,
}

pub const DIRECTIONS: [Direction; 4] = [
    Direction::Up,
    Direction::Down,
    Direction::Left,
    Direction::Right,
];

impl Direction {
    pub fn opposite(&self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    pub fn delta(&self) -> (isize, isize) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }
}

/// The maze grid. Cells at odd (x,y) coordinates are actual path cells.
/// Even coordinates are wall rows/columns.
#[derive(Clone, Debug)]
pub struct MazeGrid {
    pub cells: Vec<Vec<MazeCell>>,
    pub width: usize,
    pub height: usize,
}

impl MazeGrid {
    pub fn new(width: usize, height: usize) -> Self {
        let cells = vec![vec![MazeCell::new(); height]; width];
        Self {
            cells,
            width,
            height,
        }
    }

    pub fn get_cell(&self, x: usize, y: usize) -> Option<&MazeCell> {
        self.cells.get(x).and_then(|col| col.get(y))
    }

    pub fn get_cell_mut(&mut self, x: usize, y: usize) -> Option<&mut MazeCell> {
        self.cells.get_mut(x).and_then(|col| col.get_mut(y))
    }

    pub fn is_valid_cell(&self, x: isize, y: isize) -> bool {
        x >= 0 && x < self.width as isize && y >= 0 && y < self.height as isize
    }

    pub fn is_path_cell(&self, x: usize, y: usize) -> bool {
        x % 2 == 1 && y % 2 == 1 && x < self.width && y < self.height
    }

    /// Convert maze grid coordinates to world position (center of tile)
    pub fn grid_to_world(x: usize, y: usize) -> (f32, f32) {
        use crate::constants::{GRID_HEIGHT, GRID_WIDTH, TILE_SIZE};
        let wx = x as f32 * TILE_SIZE - GRID_WIDTH / 2.0 + TILE_SIZE / 2.0;
        let wy = -(y as f32 * TILE_SIZE - GRID_HEIGHT / 2.0 + TILE_SIZE / 2.0);
        (wx, wy)
    }
}
