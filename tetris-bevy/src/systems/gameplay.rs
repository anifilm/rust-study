use bevy::prelude::*;

use crate::game::{GamePhase, SoftDropState, TetrisSession, VisualEffectsState};

pub fn update_game(
    time: Res<Time>,
    soft_drop_state: Res<SoftDropState>,
    mut effects: ResMut<VisualEffectsState>,
    mut session: ResMut<TetrisSession>,
) {
    let current_phase_before = session.game.phase;
    session.game.update(time.delta_secs(), soft_drop_state.0);
    if !session.game.last_locked_blocks.is_empty() {
        effects.trigger_lock_squash(&session.game.last_locked_blocks);
        session.game.last_locked_blocks.clear();
    }
    if !session.game.last_cleared_rows.is_empty() {
        effects.trigger_clear_badge(session.game.last_cleared_rows.len());
        effects.trigger_cleared_rows_flash(&session.game.last_cleared_rows);
        session.game.last_cleared_rows.clear();
    }
    if current_phase_before != GamePhase::GameOver && session.game.phase == GamePhase::GameOver {
        effects.trigger_game_over_popup();
    }
    effects.previous_phase = Some(session.game.phase);
}
