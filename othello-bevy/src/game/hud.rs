use bevy::prelude::*;

use crate::{
    constants::*,
    game::logic::{count_stones, Board},
    state::{AiDifficulty, CurrentTurn, GameEntity, GameMode, KoreanFont, Player},
};

#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct ScoreBlackText;

#[derive(Component)]
pub struct ScoreWhiteText;

#[derive(Component)]
pub struct TurnText;

#[derive(Component)]
pub struct PassNotice;

#[derive(Component)]
pub struct PassTimer(pub Timer);

// ─── HUD 스폰 (UI Node 패널) ─────────────────────────────────────────────────
pub fn spawn_hud(
    mut commands: Commands,
    font: Res<KoreanFont>,
    game_mode: Res<GameMode>,
    difficulty: Option<Res<AiDifficulty>>,
) {
    let f = font.0.clone();

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(16.0),
                top: Val::Px(50.0),
                width: Val::Px(HUD_PANEL_WIDTH),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(COLOR_HUD_BG),
            BorderColor(COLOR_HUD_BORDER),
            BorderRadius::all(Val::Px(12.0)),
            HudRoot,
            GameEntity,
        ))
        .with_children(|panel| {
            // ── 타이틀 ───────
            panel.spawn((
                Text::new("OTHELLO"),
                TextFont { font: f.clone(), font_size: 22.0, ..default() },
                TextColor(COLOR_TITLE),
            ));

            // ── 구분선 ───────
            panel.spawn((
                Node { width: Val::Percent(100.0), height: Val::Px(1.0), ..default() },
                BackgroundColor(COLOR_HUD_BORDER),
            ));

            // ── 흑 점수 ──────
            panel
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    ..default()
                })
                .with_children(|row| {
                    // 돌 아이콘 (CSS 원형)
                    row.spawn((
                        Node {
                            width: Val::Px(22.0),
                            height: Val::Px(22.0),
                            ..default()
                        },
                        BackgroundColor(COLOR_BLACK_PIECE),
                        BorderRadius::all(Val::Px(11.0)),
                    ));
                    row.spawn((
                        Text::new("Black   2"),
                        TextFont { font: f.clone(), font_size: 20.0, ..default() },
                        TextColor(Color::srgb(0.85, 0.85, 0.85)),
                        ScoreBlackText,
                    ));
                });

            // ── 백 점수 ──────
            panel
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Node {
                            width: Val::Px(22.0),
                            height: Val::Px(22.0),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(COLOR_WHITE_PIECE),
                        BorderColor(Color::srgb(0.6, 0.6, 0.6)),
                        BorderRadius::all(Val::Px(11.0)),
                    ));
                    row.spawn((
                        Text::new("White   2"),
                        TextFont { font: f.clone(), font_size: 20.0, ..default() },
                        TextColor(Color::srgb(0.85, 0.85, 0.85)),
                        ScoreWhiteText,
                    ));
                });

            // ── 구분선 ───────
            panel.spawn((
                Node { width: Val::Percent(100.0), height: Val::Px(1.0), ..default() },
                BackgroundColor(COLOR_HUD_BORDER),
            ));

            // ── 턴 표시 ──────
            panel.spawn((
                Text::new("● Black"),
                TextFont { font: f.clone(), font_size: 18.0, ..default() },
                TextColor(Color::srgb(0.7, 0.9, 0.7)),
                TurnText,
            ));

            // ── AI 난이도 뱃지 ────
            if *game_mode == GameMode::PvAI {
                let diff = difficulty.map(|d| *d).unwrap_or_default();
                let (label, badge_color) = match diff {
                    AiDifficulty::Easy => ("쉬움", Color::srgb(0.20, 0.55, 0.20)),
                    AiDifficulty::Normal => ("보통", Color::srgb(0.55, 0.50, 0.10)),
                    AiDifficulty::Hard => ("어려움", Color::srgb(0.65, 0.15, 0.15)),
                };
                panel
                    .spawn((
                        Node {
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(badge_color),
                        BorderRadius::all(Val::Px(6.0)),
                    ))
                    .with_children(|badge| {
                        badge.spawn((
                            Text::new(format!("AI {label}")),
                            TextFont { font: f.clone(), font_size: 14.0, ..default() },
                            TextColor(COLOR_TEXT),
                        ));
                    });
            }
        });
}

// ─── HUD 업데이트 ────────────────────────────────────────────────────────────
#[allow(clippy::type_complexity)]
pub fn update_hud(
    board: Res<Board>,
    turn: Res<CurrentTurn>,
    mut black_q: Query<&mut Text, (With<ScoreBlackText>, Without<ScoreWhiteText>, Without<TurnText>)>,
    mut white_q: Query<&mut Text, (With<ScoreWhiteText>, Without<ScoreBlackText>, Without<TurnText>)>,
    mut turn_q: Query<(&mut Text, &mut TextColor), (With<TurnText>, Without<ScoreBlackText>, Without<ScoreWhiteText>)>,
) {
    if !board.is_changed() && !turn.is_changed() {
        return;
    }
    let (black, white) = count_stones(&board);

    if let Ok(mut text) = black_q.get_single_mut() {
        text.0 = format!("Black   {black}");
    }
    if let Ok(mut text) = white_q.get_single_mut() {
        text.0 = format!("White   {white}");
    }

    if let Ok((mut text, mut color)) = turn_q.get_single_mut() {
        match turn.0 {
            Player::Black => {
                text.0 = "● Black".to_string();
                *color = TextColor(Color::srgb(0.7, 0.9, 0.7));
            }
            Player::White => {
                text.0 = "○ White".to_string();
                *color = TextColor(Color::srgb(0.9, 0.9, 0.7));
            }
        }
    }
}

// ─── 패스 알림 (월드 텍스트) ─────────────────────────────────────────────────
pub fn show_pass_notice(commands: &mut Commands, player: Player, font: Handle<Font>) {
    let msg = match player {
        Player::Black => "Black  PASS",
        Player::White => "White  PASS",
    };
    commands.spawn((
        Text2d::new(msg),
        TextFont { font, font_size: 36.0, ..default() },
        TextColor(Color::srgba(1.0, 0.8, 0.2, 1.0)),
        Transform::from_xyz(0.0, 0.0, 10.0),
        PassNotice,
        PassTimer(Timer::from_seconds(1.8, TimerMode::Once)),
        GameEntity,
    ));
}

pub fn tick_pass_notice(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut PassTimer)>,
) {
    for (entity, mut timer) in &mut query {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            commands.entity(entity).despawn();
        }
    }
}
