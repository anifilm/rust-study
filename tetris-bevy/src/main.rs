mod board;
mod components;
mod game;
mod rotation;
mod systems;
mod tetromino;

use bevy::prelude::*;
use game::TetrisPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Tetris Bevy".to_string(),
                resolution: (620.0, 720.0).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(TetrisPlugin)
        .run();
}
