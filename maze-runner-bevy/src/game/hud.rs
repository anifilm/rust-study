use bevy::prelude::*;
use bevy::text::TextFont;
use bevy::ui::Node;

use crate::constants::*;
use crate::player::Player;
use crate::ClearSummary;

/// Component for the HUD root node
#[derive(Component)]
pub struct HudRoot;

/// Component for the score text
#[derive(Component)]
pub struct ScoreText;

/// Component for the treasures text
#[derive(Component)]
pub struct TreasuresText;

/// Component for the cleared message
#[derive(Component)]
pub struct ClearedMessage;

/// Spawn the HUD
pub fn spawn_hud(mut commands: Commands) {
    // Root node
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            HudRoot,
        ))
        .with_children(|parent| {
            // Score text
            parent.spawn((
                Text::new("Score: 0"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(COLOR_HUD_TEXT),
                ScoreText,
            ));
            // Treasures text
            parent.spawn((
                Text::new("Treasures: 0"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(COLOR_HUD_TEXT),
                TreasuresText,
            ));
        });
}

/// Update HUD text
pub fn update_hud(
    player_query: Query<&Player>,
    mut text_query: Query<(&mut Text, &mut TextColor), Or<(With<ScoreText>, With<TreasuresText>)>>,
) {
    let Ok(player) = player_query.get_single() else {
        return;
    };

    for (mut text, text_color) in text_query.iter_mut() {
        if text_color.0 == COLOR_HUD_TEXT {
            // This is either score or treasures text
            if text.0.starts_with("Score:") {
                text.0 = format!("Score: {}", player.score);
            } else if text.0.starts_with("Treasures:") {
                text.0 = format!("Treasures: {}/?", player.treasures_collected);
            }
        }
    }
}

/// Show game cleared message
pub fn show_cleared_message(mut commands: Commands, clear_summary: Option<Res<ClearSummary>>) {
    let Some(clear_summary) = clear_summary else {
        return;
    };

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
            ClearedMessage,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        padding: UiRect::all(Val::Px(24.0)),
                        row_gap: Val::Px(12.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.92)),
                    BorderRadius::all(Val::Px(16.0)),
                    ClearedMessage,
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("Maze Cleared!"),
                        TextFont {
                            font_size: 48.0,
                            ..default()
                        },
                        TextColor(COLOR_EXIT),
                        ClearedMessage,
                    ));
                    panel.spawn((
                        Text::new(format!("Final Score: {}", clear_summary.final_score)),
                        TextFont {
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(COLOR_HUD_TEXT),
                        ClearedMessage,
                    ));
                    panel.spawn((
                        Text::new(format!("Treasures: {}", clear_summary.treasures_collected)),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(COLOR_TREASURE),
                        ClearedMessage,
                    ));
                    panel.spawn((
                        Text::new("Press R to restart"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(COLOR_HUD_TEXT),
                        ClearedMessage,
                    ));
                });
        });
}

/// Clean up cleared message
pub fn cleanup_cleared_message(mut commands: Commands, query: Query<Entity, With<ClearedMessage>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
