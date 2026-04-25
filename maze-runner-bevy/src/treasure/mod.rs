pub mod collection;

use bevy::prelude::*;
use bevy::sprite::MeshMaterial2d;

use crate::constants::*;

/// Component for treasure items
#[derive(Component)]
pub struct Treasure {
    pub grid_x: usize,
    pub grid_y: usize,
    pub collected: bool,
}

/// Event fired when a treasure is collected
#[derive(Event)]
pub struct TreasureCollectedEvent;

/// Spawn treasures at random path cells in the maze
pub fn spawn_treasures(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    maze: &crate::maze::MazeGrid,
) {
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    // Collect all path cells (odd coordinates), excluding start and exit
    let mut path_cells: Vec<(usize, usize)> = Vec::new();
    for x in 0..maze.width {
        for y in 0..maze.height {
            if maze.is_path_cell(x, y) {
                // Skip start and exit positions
                if (x, y) == START_POS || (x, y) == EXIT_POS {
                    continue;
                }
                path_cells.push((x, y));
            }
        }
    }

    // Determine number of treasures (at least 1)
    let count = ((path_cells.len() as f32) * TREASURE_RATIO).max(1.0) as usize;
    let count = count.min(path_cells.len());

    // Randomly select positions
    let mut rng = thread_rng();
    path_cells.shuffle(&mut rng);

    for i in 0..count {
        let (gx, gy) = path_cells[i];
        let (wx, wy) = crate::maze::MazeGrid::grid_to_world(gx, gy);

        commands.spawn((
            Mesh2d(meshes.add(Circle::new(TREASURE_RADIUS))),
            MeshMaterial2d(materials.add(COLOR_TREASURE)),
            Transform::from_xyz(wx, wy, 5.0),
            Treasure {
                grid_x: gx,
                grid_y: gy,
                collected: false,
            },
        ));
    }
}
