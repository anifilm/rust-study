use bevy::prelude::*;

use crate::{
    constants::*,
    state::{AiDifficulty, GameMode, GameResult, GameState, KoreanFont, MenuEntity},
};

#[derive(Component)]
pub struct GameOverEntity;

#[derive(Component)]
pub enum GameOverButton {
    Restart,
    ChangeDifficulty,
    MainMenu,
}

// ─── 게임 오버 화면 스폰 ─────────────────────────────────────────────────────
pub fn spawn_gameover(
    mut commands: Commands,
    font: Res<KoreanFont>,
    result: Res<GameResult>,
    game_mode: Res<GameMode>,
    difficulty: Option<Res<AiDifficulty>>,
) {
    let f = font.0.clone();
    let (b, w) = (result.black_score, result.white_score);
    let winner_text = if b > w {
        format!("Black Wins!\n● {b}  ○ {w}")
    } else if w > b {
        format!("White Wins!\n● {b}  ○ {w}")
    } else {
        format!("Draw!\n● {b}  ○ {w}")
    };

    let is_ai = *game_mode == GameMode::PvAI;

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.78)),
            GameOverEntity,
            MenuEntity,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(winner_text),
                TextFont { font: f.clone(), font_size: 48.0, ..default() },
                TextColor(Color::srgb(1.0, 0.9, 0.3)),
                TextLayout::new_with_justify(JustifyText::Center),
            ));

            if is_ai {
                let diff = difficulty.map(|d| *d).unwrap_or_default();
                let (label, color) = match diff {
                    AiDifficulty::Easy => ("난이도: 쉬움", Color::srgb(0.30, 0.85, 0.30)),
                    AiDifficulty::Normal => ("난이도: 보통", Color::srgb(0.85, 0.80, 0.10)),
                    AiDifficulty::Hard => ("난이도: 어려움", Color::srgb(0.90, 0.25, 0.25)),
                };
                parent.spawn((
                    Text::new(label),
                    TextFont { font: f.clone(), font_size: 20.0, ..default() },
                    TextColor(color),
                ));
            }

            parent.spawn(Node { height: Val::Px(8.0), ..default() });
            spawn_button(parent, "다시 시작", GameOverButton::Restart, f.clone());
            if is_ai {
                spawn_button(parent, "난이도 변경", GameOverButton::ChangeDifficulty, f.clone());
            }
            spawn_button(parent, "메뉴로", GameOverButton::MainMenu, f.clone());
        });
}

fn spawn_button(parent: &mut ChildBuilder, label: &str, btn: GameOverButton, font: Handle<Font>) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(220.0),
                height: Val::Px(56.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(COLOR_BTN_NORMAL),
            BorderColor(Color::srgb(0.35, 0.35, 0.35)),
            BorderRadius::all(Val::Px(8.0)),
            btn,
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(label),
                TextFont { font, font_size: 24.0, ..default() },
                TextColor(COLOR_TEXT),
            ));
        });
}

pub fn handle_gameover_buttons(
    mut interaction_q: Query<
        (&Interaction, &mut BackgroundColor, &GameOverButton),
        Changed<Interaction>,
    >,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut bg, btn) in &mut interaction_q {
        match interaction {
            Interaction::Hovered => *bg = BackgroundColor(COLOR_BTN_HOVER),
            Interaction::Pressed => {
                *bg = BackgroundColor(COLOR_BTN_PRESS);
                match btn {
                    GameOverButton::Restart => {
                        commands.insert_resource(crate::game::logic::Board::new());
                        commands.insert_resource(crate::game::logic::ValidMoves::default());
                        commands.insert_resource(crate::state::CurrentTurn(
                            crate::state::Player::Black,
                        ));
                        next_state.set(GameState::Playing);
                    }
                    GameOverButton::ChangeDifficulty => {
                        next_state.set(GameState::DifficultySelect);
                    }
                    GameOverButton::MainMenu => {
                        next_state.set(GameState::MainMenu);
                    }
                }
            }
            Interaction::None => *bg = BackgroundColor(COLOR_BTN_NORMAL),
        }
    }
}

pub fn despawn_gameover(mut commands: Commands, query: Query<Entity, With<GameOverEntity>>) {
    for e in &query {
        commands.entity(e).despawn_recursive();
    }
}
