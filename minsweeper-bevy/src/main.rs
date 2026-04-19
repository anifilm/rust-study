mod board;
mod game;
mod input;
mod ui;

use bevy::prelude::*;
use board::BoardPlugin;
use game::GamePlugin;
use input::InputPlugin;
use ui::UIPlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Minesweeper".into(),
                    resolution: (500.0, 560.0).into(),
                    ..default()
                }),
                ..default()
            }),
        )
        .add_plugins((GamePlugin, BoardPlugin, InputPlugin, UIPlugin))
        .run();
}
