use crate::constants::{MAZE_HEIGHT, MAZE_WIDTH};
use crate::maze::{MazeGrid, DIRECTIONS};
use rand::seq::SliceRandom;
use rand::thread_rng;

/// Generate a maze using DFS Recursive Backtracker algorithm.
/// The maze is generated on the path cells (odd coordinates).
pub fn generate_maze() -> MazeGrid {
    let mut maze = MazeGrid::new(MAZE_WIDTH, MAZE_HEIGHT);
    let mut rng = thread_rng();

    // Start from (1, 1)
    carve(&mut maze, 1, 1, &mut rng);

    maze
}

fn carve(maze: &mut MazeGrid, x: usize, y: usize, rng: &mut impl rand::Rng) {
    if let Some(cell) = maze.get_cell_mut(x, y) {
        cell.visited = true;
    }

    let mut directions = DIRECTIONS.to_vec();
    directions.shuffle(rng);

    for dir in directions {
        let (dx, dy) = dir.delta();
        let nx = x as isize + dx * 2;
        let ny = y as isize + dy * 2;

        if maze.is_valid_cell(nx, ny) {
            let (nx, ny) = (nx as usize, ny as usize);
            if let Some(next_cell) = maze.get_cell(nx, ny) {
                if !next_cell.visited {
                    // Remove wall between current and next
                    let wall_x = (x as isize + dx) as usize;
                    let wall_y = (y as isize + dy) as usize;

                    if let Some(cell) = maze.get_cell_mut(x, y) {
                        cell.remove_wall(dir);
                    }
                    if let Some(wall_cell) = maze.get_cell_mut(wall_x, wall_y) {
                        wall_cell.remove_wall(dir);
                        wall_cell.remove_wall(dir.opposite());
                    }
                    if let Some(next_cell) = maze.get_cell_mut(nx, ny) {
                        next_cell.remove_wall(dir.opposite());
                    }

                    carve(maze, nx, ny, rng);
                }
            }
        }
    }
}
