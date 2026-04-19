use bevy::{
    prelude::*,
    window::PrimaryWindow,
};

use crate::{
    board::Board,
    game::{FlagCell, GameState, RevealCell},
};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            handle_mouse_input.run_if(in_state(GameState::Playing)),
        );
    }
}

fn handle_mouse_input(
    mouse: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    board: Option<Res<Board>>,
    mut reveal_events: EventWriter<RevealCell>,
    mut flag_events: EventWriter<FlagCell>,
) {
    let board = match board {
        Some(b) => b,
        None => return,
    };

    let left = mouse.just_pressed(MouseButton::Left);
    let right = mouse.just_pressed(MouseButton::Right);
    if !left && !right {
        return;
    }

    let Ok(window) = window_query.get_single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else {
        return;
    };
    let world_pos = ray.origin.truncate();

    let Some((col, row)) = board.world_to_coords(world_pos) else {
        return;
    };

    if left {
        reveal_events.send(RevealCell { col, row });
    } else {
        flag_events.send(FlagCell { col, row });
    }
}
