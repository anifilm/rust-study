use bevy::prelude::*;

use crate::constants::*;
use crate::maze::{Direction, MazeGrid};
use crate::player::Player;
use crate::MazeResource;

/// Tile-based smooth player movement.
/// The player snaps to tile centers and moves one tile at a time
/// with smooth interpolation. Supports instant direction change mid-tile.
pub fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    maze_resource: Res<MazeResource>,
    mut player_query: Query<(&mut Player, &mut Transform)>,
    time: Res<Time>,
) {
    let Ok((mut player, mut transform)) = player_query.get_single_mut() else {
        return;
    };

    let maze = &maze_resource.grid;
    let dt = time.delta_secs();

    // Determine desired direction from currently pressed keys
    let dir = if keyboard.pressed(KeyCode::ArrowUp) || keyboard.pressed(KeyCode::KeyW) {
        Some(Direction::Up)
    } else if keyboard.pressed(KeyCode::ArrowDown) || keyboard.pressed(KeyCode::KeyS) {
        Some(Direction::Down)
    } else if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
        Some(Direction::Left)
    } else if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
        Some(Direction::Right)
    } else {
        None
    };

    let Some(dir) = dir else {
        return;
    };

    // If currently moving towards a target
    if player.is_moving {
        let (target_wx, target_wy) = MazeGrid::grid_to_world(player.target_x, player.target_y);
        let current_pos = Vec2::new(transform.translation.x, transform.translation.y);
        let target_pos = Vec2::new(target_wx, target_wy);
        let dist = current_pos.distance(target_pos);

        let step = PLAYER_MOVE_SPEED * TILE_SIZE * dt;

        if dist <= step {
            // Snap to target tile
            transform.translation.x = target_pos.x;
            transform.translation.y = target_pos.y;
            player.grid_x = player.target_x;
            player.grid_y = player.target_y;
            player.is_moving = false;
        } else {
            // Move towards target
            let direction = (target_pos - current_pos).normalize();
            transform.translation.x += direction.x * step;
            transform.translation.y += direction.y * step;

            // Mid-tile direction change
            let current_move_dir =
                direction_to_target(player.grid_x, player.grid_y, player.target_x, player.target_y);
            if let Some(current_move_dir) = current_move_dir {
                if dir != current_move_dir {
                    try_change_direction(maze, &mut player, dir, current_move_dir);
                }
            }

            return;
        }
    }

    // Player is at a tile center — check if we can move in the desired direction
    // Check the wall from current tile in the desired direction
    if let Some(cell) = maze.get_cell(player.grid_x, player.grid_y) {
        if !cell.has_wall(dir) {
            let (dx, dy) = dir.delta();
            let nx = (player.grid_x as isize + dx) as usize;
            let ny = (player.grid_y as isize + dy) as usize;

            player.target_x = nx;
            player.target_y = ny;
            player.is_moving = true;
        }
    }
}

/// Try to change movement direction mid-tile.
fn try_change_direction(
    maze: &MazeGrid,
    player: &mut Player,
    new_dir: Direction,
    current_dir: Direction,
) {
    // Turning back (opposite direction) — always allowed
    if new_dir == current_dir.opposite() {
        player.target_x = player.grid_x;
        player.target_y = player.grid_y;
        return;
    }

    // Turning to a side direction — check walls from current grid position
    if let Some(cell) = maze.get_cell(player.grid_x, player.grid_y) {
        if !cell.has_wall(new_dir) {
            let (dx, dy) = new_dir.delta();
            let nx = (player.grid_x as isize + dx) as usize;
            let ny = (player.grid_y as isize + dy) as usize;
            player.target_x = nx;
            player.target_y = ny;
        }
    }
}

/// Determine the direction from current position to target position.
fn direction_to_target(cx: usize, cy: usize, tx: usize, ty: usize) -> Option<Direction> {
    let dx = tx as isize - cx as isize;
    let dy = ty as isize - cy as isize;
    match (dx, dy) {
        (0, -1) => Some(Direction::Up),
        (0, 1) => Some(Direction::Down),
        (-1, 0) => Some(Direction::Left),
        (1, 0) => Some(Direction::Right),
        _ => None,
    }
}
