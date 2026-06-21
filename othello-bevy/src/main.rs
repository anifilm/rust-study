use bevy::prelude::*;

mod ai;
mod constants;
mod game;
mod menu;
mod state;

use constants::{WINDOW_HEIGHT, WINDOW_WIDTH};
use state::{GameState, KoreanFont};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Othello".to_string(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        // DefaultPlugins 이후 즉시 동기 삽입 → OnEnter(MainMenu) 실행 전 보장
        .init_resource::<KoreanFont>()
        .add_systems(Startup, setup_camera)
        .add_plugins(menu::MenuPlugin)
        .add_plugins(game::GamePlugin)
        .add_plugins(ai::AiPlugin)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
