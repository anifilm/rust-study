use bevy::prelude::*;
use bevy::sprite::MeshMaterial2d;

use crate::constants::*;
use crate::maze::MazeGrid;

/// Component to mark maze wall/floor entities
#[derive(Component)]
pub struct MazeTile;

/// Spawn the maze visualization as colored quads
pub fn spawn_maze(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    maze: &MazeGrid,
) {
    for x in 0..maze.width {
        for y in 0..maze.height {
            let (wx, wy) = MazeGrid::grid_to_world(x, y);

            let is_exit = x == EXIT_POS.0 && y == EXIT_POS.1;

            // A cell is a wall if ALL four walls are still present (0b1111).
            // During generation, DFS removes walls between adjacent path cells,
            // including the wall cells between them — so those become open too.
            let is_wall = if let Some(cell) = maze.get_cell(x, y) {
                cell.walls == 0b1111
            } else {
                true
            };

            let color = if is_exit {
                COLOR_EXIT
            } else if is_wall {
                COLOR_WALL
            } else {
                COLOR_PATH
            };

            commands.spawn((
                Mesh2d(meshes.add(Rectangle::from_size(Vec2::splat(TILE_SIZE)))),
                MeshMaterial2d(materials.add(color)),
                Transform::from_xyz(wx, wy, 0.0),
                MazeTile,
            ));
        }
    }
}
