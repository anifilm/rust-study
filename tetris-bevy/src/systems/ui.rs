use bevy::prelude::*;

use crate::components::{
    ClearBadgePanel, ClearBadgeText, LevelText, LinesText, NextBlock, OverlayHintText,
    OverlayPanel, OverlayText, PreviewBox, ScoreText,
};
use crate::game::{GamePhase, TetrisSession, VisualEffectsState};
use crate::systems::render::{
    HUD_HEIGHT_PX, HUD_TOP_PX, NEXT_CARD_TOP_PX, PREVIEW_BOX_HEIGHT_PX, PREVIEW_BOX_LEFT_PX,
    PREVIEW_BOX_TOP_PX, PREVIEW_BOX_WIDTH_PX, SIDE_PANEL_LEFT_PX, SIDE_PANEL_WIDTH,
};
use crate::tetromino::Tetromino;

pub fn setup_ui(mut commands: Commands) {
    let panel_padding = 14.0;
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(SIDE_PANEL_LEFT_PX - panel_padding),
                top: Val::Px(HUD_TOP_PX - panel_padding),
                width: Val::Px(SIDE_PANEL_WIDTH + panel_padding * 2.0),
                height: Val::Px(HUD_HEIGHT_PX + panel_padding * 2.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.18, 0.31)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BorderColor(Color::srgb(0.36, 0.56, 0.95)),
            ));
            parent
                .spawn((Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(panel_padding),
                    top: Val::Px(panel_padding),
                    width: Val::Px(SIDE_PANEL_WIDTH),
                    height: Val::Px(HUD_HEIGHT_PX),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(16.0),
                    ..default()
                },))
                .with_children(|parent| {
                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(170.0),
                                flex_direction: FlexDirection::Column,
                                padding: UiRect::all(Val::Px(18.0)),
                                border: UiRect::all(Val::Px(2.0)),
                                overflow: Overflow::clip(),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.05, 0.08, 0.15, 0.96)),
                            BorderColor(Color::srgb(0.30, 0.51, 0.92)),
                        ))
                        .with_children(|parent| {
                            spawn_card_title(parent, "NEXT", Color::srgb(0.84, 0.91, 1.0));
                            parent.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(PREVIEW_BOX_LEFT_PX - SIDE_PANEL_LEFT_PX),
                                    top: Val::Px(PREVIEW_BOX_TOP_PX - NEXT_CARD_TOP_PX),
                                    width: Val::Px(PREVIEW_BOX_WIDTH_PX),
                                    height: Val::Px(PREVIEW_BOX_HEIGHT_PX),
                                    border: UiRect::all(Val::Px(1.0)),
                                    overflow: Overflow::clip(),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.08, 0.11, 0.20, 0.02)),
                                BorderColor(Color::srgb(0.22, 0.33, 0.58)),
                                PreviewBox,
                            ));
                        });

                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                flex_grow: 1.0,
                                flex_direction: FlexDirection::Column,
                                padding: UiRect::all(Val::Px(18.0)),
                                border: UiRect::all(Val::Px(2.0)),
                                row_gap: Val::Px(10.0),
                                overflow: Overflow::clip(),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.06, 0.09, 0.16, 0.96)),
                            BorderColor(Color::srgb(0.20, 0.31, 0.52)),
                        ))
                        .with_children(|parent| {
                            spawn_card_title(parent, "STATS", Color::srgb(0.84, 0.91, 1.0));
                            spawn_primary_stat(
                                parent,
                                "SCORE",
                                "0",
                                ScoreText,
                                32.0,
                                Color::srgb(0.83, 0.94, 1.0),
                            );
                            parent
                                .spawn((Node {
                                    width: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(10.0),
                                    ..default()
                                },))
                                .with_children(|parent| {
                                    spawn_secondary_stat(
                                        parent,
                                        "LINES",
                                        "0",
                                        LinesText,
                                        Color::srgb(0.74, 0.95, 0.79),
                                    );
                                    spawn_secondary_stat(
                                        parent,
                                        "LEVEL",
                                        "1",
                                        LevelText,
                                        Color::srgb(1.0, 0.89, 0.66),
                                    );
                                });
                        });
                });
        });

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(126.0),
                top: Val::Px(266.0),
                width: Val::Px(368.0),
                height: Val::Px(188.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(10.0),
                padding: UiRect::all(Val::Px(18.0)),
                border: UiRect::all(Val::Px(2.0)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.04, 0.08, 0.88)),
            BorderColor(Color::srgb(0.34, 0.45, 0.74)),
            Visibility::Hidden,
            OverlayPanel,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont::from_font_size(44.0),
                TextColor::WHITE,
                Visibility::Hidden,
                OverlayText,
            ));

            parent.spawn((
                Text::new(""),
                TextFont::from_font_size(20.0),
                TextColor(Color::srgb(0.76, 0.81, 0.90)),
                Node {
                    margin: UiRect::top(Val::Px(6.0)),
                    ..default()
                },
                Visibility::Hidden,
                OverlayHintText,
            ));
        });

    const BADGE_WIDTH: f32 = 188.0;
    const BADGE_HEIGHT: f32 = 52.0;
    const BOARD_CENTER_PX_X: f32 = 200.0;
    const BOARD_CENTER_PX_Y: f32 = 360.0;
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(BOARD_CENTER_PX_X - BADGE_WIDTH / 2.0),
                top: Val::Px(BOARD_CENTER_PX_Y - BADGE_HEIGHT / 2.0),
                width: Val::Px(BADGE_WIDTH),
                height: Val::Px(BADGE_HEIGHT),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.14, 0.24, 0.0)),
            BorderColor(Color::srgba(0.62, 0.83, 1.0, 0.0)),
            Visibility::Hidden,
            ClearBadgePanel,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont::from_font_size(24.0),
                TextColor(Color::srgba(0.88, 0.95, 1.0, 0.0)),
                Visibility::Hidden,
                ClearBadgeText,
            ));
        });
}

pub fn update_ui(
    session: Res<TetrisSession>,
    mut text_queries: ParamSet<(
        Query<&mut Text, With<ScoreText>>,
        Query<&mut Text, With<LinesText>>,
        Query<&mut Text, With<LevelText>>,
        Query<&mut Text, With<OverlayText>>,
        Query<&mut Text, With<OverlayHintText>>,
    )>,
    mut visibility_queries: ParamSet<(
        Query<(&mut Visibility, &mut BorderColor), With<OverlayPanel>>,
        Query<&mut Visibility, With<OverlayText>>,
        Query<&mut Visibility, With<OverlayHintText>>,
    )>,
) {
    let game = &session.game;

    if let Ok(mut text) = text_queries.p0().get_single_mut() {
        text.0 = game.score.to_string();
    }

    if let Ok(mut text) = text_queries.p1().get_single_mut() {
        text.0 = game.lines.to_string();
    }

    if let Ok(mut text) = text_queries.p2().get_single_mut() {
        text.0 = game.level.to_string();
    }

    if let Ok(mut text) = text_queries.p3().get_single_mut() {
        text.0 = match game.phase {
            GamePhase::Playing => String::new(),
            GamePhase::Paused => "PAUSED".to_string(),
            GamePhase::GameOver => "GAME OVER".to_string(),
        };
    }

    if let Ok(mut text) = text_queries.p4().get_single_mut() {
        text.0 = match game.phase {
            GamePhase::Playing => String::new(),
            GamePhase::Paused => "Press P to jump back in".to_string(),
            GamePhase::GameOver => "Press R to start a new run".to_string(),
        };
    }

    let is_overlay_visible = game.phase != GamePhase::Playing;
    let border_color = match game.phase {
        GamePhase::Playing => Color::srgb(0.34, 0.45, 0.74),
        GamePhase::Paused => Color::srgb(0.31, 0.55, 0.92),
        GamePhase::GameOver => Color::srgb(0.34, 0.45, 0.74),
    };

    if let Ok((mut visibility, mut border)) = visibility_queries.p0().get_single_mut() {
        *visibility = if is_overlay_visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if game.phase != GamePhase::GameOver && game.phase != GamePhase::Paused {
            border.0 = border_color;
        }
    }

    if let Ok(mut visibility) = visibility_queries.p1().get_single_mut() {
        *visibility = if is_overlay_visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    if let Ok(mut visibility) = visibility_queries.p2().get_single_mut() {
        *visibility = if is_overlay_visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn sync_next_preview(
    mut commands: Commands,
    session: Res<TetrisSession>,
    mut effects: ResMut<VisualEffectsState>,
    preview_box: Query<Entity, With<PreviewBox>>,
    preview_blocks: Query<Entity, With<NextBlock>>,
) {
    let Ok(preview_box) = preview_box.get_single() else {
        return;
    };

    for entity in preview_blocks.iter() {
        commands.entity(entity).despawn();
    }

    if effects.displayed_next != Some(session.game.next) {
        effects.trigger_preview_transition(session.game.next);
    }

    let preview = Tetromino::new(session.game.next);
    let blocks = preview.blocks();
    let min_x = blocks.iter().map(|block| block.x).min().unwrap_or(0) as f32;
    let max_x = blocks.iter().map(|block| block.x).max().unwrap_or(0) as f32;
    let min_y = blocks.iter().map(|block| block.y).min().unwrap_or(0) as f32;
    let max_y = blocks.iter().map(|block| block.y).max().unwrap_or(0) as f32;
    let center_x = (min_x + max_x) * 0.5;
    let center_y = (min_y + max_y) * 0.5;

    const PREVIEW_BLOCK_SIZE: f32 = 24.0;
    const PREVIEW_SPACING: f32 = 24.0;
    let preview_progress = if effects.preview_transition_timer > 0.0 {
        1.0 - (effects.preview_transition_timer / VisualEffectsState::PREVIEW_TRANSITION_DURATION)
    } else {
        1.0
    }
    .clamp(0.0, 1.0);
    let slide_offset = (1.0 - preview_progress) * 14.0;
    let alpha = preview_progress.max(0.25);

    commands.entity(preview_box).with_children(|parent| {
        for block in blocks {
            let left = PREVIEW_BOX_WIDTH_PX / 2.0 + (block.x as f32 - center_x) * PREVIEW_SPACING
                - PREVIEW_BLOCK_SIZE / 2.0
                + slide_offset;
            let top = PREVIEW_BOX_HEIGHT_PX / 2.0 + (block.y as f32 - center_y) * PREVIEW_SPACING
                - PREVIEW_BLOCK_SIZE / 2.0;
            let mut fill = crate::systems::render::color_for(session.game.next);
            fill.set_alpha(alpha);
            let mut border = crate::systems::render::shell_color_for(session.game.next);
            border.set_alpha(alpha);

            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(left),
                    top: Val::Px(top),
                    width: Val::Px(PREVIEW_BLOCK_SIZE),
                    height: Val::Px(PREVIEW_BLOCK_SIZE),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(fill),
                BorderColor(border),
                NextBlock,
            ));
        }
    });
}

fn spawn_card_title(parent: &mut ChildBuilder, label: &str, color: Color) {
    parent.spawn((
        Text::new(label),
        TextFont::from_font_size(16.0),
        TextColor(color),
    ));
}

fn spawn_primary_stat<V: Component>(
    parent: &mut ChildBuilder,
    label: &str,
    value: &str,
    value_marker: V,
    value_font_size: f32,
    value_color: Color,
) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::axes(Val::Px(14.0), Val::Px(14.0)),
                border: UiRect::all(Val::Px(1.0)),
                row_gap: Val::Px(6.0),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.11, 0.19, 0.94)),
            BorderColor(Color::srgb(0.18, 0.25, 0.42)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(label),
                TextFont::from_font_size(14.0),
                TextColor(Color::srgb(0.63, 0.70, 0.82)),
            ));

            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.06)),
            ));

            parent.spawn((
                Text::new(value),
                TextFont::from_font_size(value_font_size),
                TextColor(value_color),
                Node {
                    width: Val::Percent(100.0),
                    ..default()
                },
                value_marker,
            ));
        });
}

fn spawn_secondary_stat<V: Component>(
    parent: &mut ChildBuilder,
    label: &str,
    value: &str,
    value_marker: V,
    value_color: Color,
) {
    parent
        .spawn((
            Node {
                flex_grow: 1.0,
                flex_basis: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::axes(Val::Px(10.0), Val::Px(10.0)),
                border: UiRect::all(Val::Px(1.0)),
                row_gap: Val::Px(4.0),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.11, 0.19, 0.94)),
            BorderColor(Color::srgb(0.18, 0.25, 0.42)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(label),
                TextFont::from_font_size(13.0),
                TextColor(Color::srgb(0.63, 0.70, 0.82)),
            ));

            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.06)),
            ));

            parent.spawn((
                Text::new(value),
                TextFont::from_font_size(20.0),
                TextColor(value_color),
                Node {
                    width: Val::Percent(100.0),
                    ..default()
                },
                value_marker,
            ));
        });
}
