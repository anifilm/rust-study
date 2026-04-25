use bevy::prelude::*;

use crate::constants::TREASURE_SCORE;
use crate::player::Player;
use crate::treasure::{Treasure, TreasureCollectedEvent};

/// Check if the player has reached any treasure and collect it.
/// Fires a TreasureCollectedEvent when a treasure is collected.
pub fn collect_treasures(
    mut commands: Commands,
    mut event_writer: EventWriter<TreasureCollectedEvent>,
    player_query: Query<&Player>,
    treasure_query: Query<(Entity, &Treasure)>,
) {
    let Ok(player) = player_query.get_single() else {
        return;
    };

    for (entity, treasure) in treasure_query.iter() {
        if treasure.collected {
            continue;
        }

        if player.grid_x == treasure.grid_x && player.grid_y == treasure.grid_y {
            // Mark as collected and despawn
            commands.entity(entity).despawn();
            event_writer.send(TreasureCollectedEvent);
        }
    }
}

/// Handle treasure collection events to update player score
pub fn handle_treasure_collected(
    mut event_reader: EventReader<TreasureCollectedEvent>,
    mut player_query: Query<&mut Player>,
) {
    let Ok(mut player) = player_query.get_single_mut() else {
        return;
    };

    for _ in event_reader.read() {
        player.score += TREASURE_SCORE;
        player.treasures_collected += 1;
    }
}
