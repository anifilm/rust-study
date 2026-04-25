pub mod movement;

use bevy::prelude::*;
use bevy::sprite::MeshMaterial2d;

use crate::constants::*;

/// Component for the player
#[derive(Component)]
pub struct Player {
    /// Current grid position (maze coordinates) — the tile the player is on
    pub grid_x: usize,
    pub grid_y: usize,
    /// Target grid position for smooth movement
    pub target_x: usize,
    pub target_y: usize,
    /// Whether the player is currently moving between tiles
    pub is_moving: bool,
    /// Score from collected treasures
    pub score: u32,
    /// Number of treasures collected
    pub treasures_collected: u32,
}

impl Player {
    pub fn new() -> Self {
        Self {
            grid_x: START_POS.0,
            grid_y: START_POS.1,
            target_x: START_POS.0,
            target_y: START_POS.1,
            is_moving: false,
            score: 0,
            treasures_collected: 0,
        }
    }
}

/// Spawn the player entity
pub fn spawn_player(commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<ColorMaterial>>) {
    let (wx, wy) = crate::maze::MazeGrid::grid_to_world(START_POS.0, START_POS.1);

    commands.spawn((
        Mesh2d(meshes.add(Circle::new(PLAYER_RADIUS))),
        MeshMaterial2d(materials.add(COLOR_PLAYER)),
        Transform::from_xyz(wx, wy, 10.0),
        Player::new(),
    ));
}
