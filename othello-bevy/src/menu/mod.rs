use bevy::prelude::*;

use crate::state::GameState;

pub mod ui;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // ── MainMenu ──────────────────────────────────────────────────
            .add_systems(OnEnter(GameState::MainMenu), ui::spawn_menu)
            .add_systems(
                Update,
                ui::handle_menu_buttons.run_if(in_state(GameState::MainMenu)),
            )
            .add_systems(OnExit(GameState::MainMenu), ui::despawn_menu)
            // ── DifficultySelect ──────────────────────────────────────────
            .add_systems(OnEnter(GameState::DifficultySelect), ui::spawn_difficulty_menu)
            .add_systems(
                Update,
                ui::handle_difficulty_buttons.run_if(in_state(GameState::DifficultySelect)),
            )
            .add_systems(OnExit(GameState::DifficultySelect), ui::despawn_menu);
    }
}
