use bevy::prelude::*;

use crate::{
    constants::*,
    state::{AiDifficulty, GameMode, GameState, KoreanFont, MenuEntity},
};

// ─── 버튼 마커 ───────────────────────────────────────────────────────────────
#[derive(Component)]
pub enum MenuButton {
    PvP,
    PvAI,
}

#[derive(Component)]
pub enum DifficultyButton {
    Easy,
    Normal,
    Hard,
    Back,
}

#[allow(dead_code)]
#[derive(Component)]
pub struct SelectedDifficultyMarker(pub AiDifficulty);

// ─── 메인 메뉴 스폰 ──────────────────────────────────────────────────────────
pub fn spawn_menu(mut commands: Commands, font: Res<KoreanFont>) {
    let f = font.0.clone();
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(24.0),
                ..default()
            },
            BackgroundColor(COLOR_BACKGROUND),
            MenuEntity,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("OTHELLO"),
                TextFont { font: f.clone(), font_size: 72.0, ..default() },
                TextColor(COLOR_TITLE),
            ));
            parent.spawn((
                Text::new("Reversi"),
                TextFont { font: f.clone(), font_size: 24.0, ..default() },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
            parent.spawn(Node { height: Val::Px(40.0), ..default() });
            spawn_menu_button(parent, "1 : 1  개인전", MenuButton::PvP, f.clone());
            spawn_menu_button(parent, "AI  대전", MenuButton::PvAI, f.clone());
        });
}

fn spawn_menu_button(
    parent: &mut ChildBuilder,
    label: &str,
    btn: MenuButton,
    font: Handle<Font>,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(280.0),
                height: Val::Px(64.0),
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
                TextFont { font, font_size: 26.0, ..default() },
                TextColor(COLOR_TEXT),
            ));
        });
}

// ─── 메인 메뉴 버튼 인터랙션 ─────────────────────────────────────────────────
pub fn handle_menu_buttons(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &MenuButton),
        Changed<Interaction>,
    >,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut bg, btn) in &mut interaction_query {
        match interaction {
            Interaction::Hovered => *bg = BackgroundColor(COLOR_BTN_HOVER),
            Interaction::Pressed => {
                *bg = BackgroundColor(COLOR_BTN_PRESS);
                match btn {
                    MenuButton::PvP => {
                        commands.insert_resource(GameMode::PvP);
                        next_state.set(GameState::Playing);
                    }
                    MenuButton::PvAI => {
                        commands.insert_resource(GameMode::PvAI);
                        next_state.set(GameState::DifficultySelect);
                    }
                }
            }
            Interaction::None => *bg = BackgroundColor(COLOR_BTN_NORMAL),
        }
    }
}

// ─── 난이도 선택 화면 스폰 ───────────────────────────────────────────────────
pub fn spawn_difficulty_menu(
    mut commands: Commands,
    font: Res<KoreanFont>,
    difficulty: Option<Res<AiDifficulty>>,
) {
    let f = font.0.clone();
    let current = difficulty.map(|d| *d).unwrap_or_default();

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
            BackgroundColor(COLOR_BACKGROUND),
            MenuEntity,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("OTHELLO"),
                TextFont { font: f.clone(), font_size: 60.0, ..default() },
                TextColor(COLOR_TITLE),
            ));
            parent.spawn((
                Text::new("AI 난이도 선택"),
                TextFont { font: f.clone(), font_size: 28.0, ..default() },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));
            parent.spawn(Node { height: Val::Px(20.0), ..default() });

            spawn_difficulty_button(parent, AiDifficulty::Easy, current, f.clone());
            spawn_difficulty_button(parent, AiDifficulty::Normal, current, f.clone());
            spawn_difficulty_button(parent, AiDifficulty::Hard, current, f.clone());

            parent.spawn(Node { height: Val::Px(10.0), ..default() });

            // 뒤로 버튼
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(160.0),
                        height: Val::Px(44.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.12, 0.12)),
                    BorderColor(Color::srgb(0.3, 0.3, 0.3)),
                    BorderRadius::all(Val::Px(6.0)),
                    DifficultyButton::Back,
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new("← 뒤로"),
                        TextFont { font: f.clone(), font_size: 20.0, ..default() },
                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                    ));
                });
        });
}

fn spawn_difficulty_button(
    parent: &mut ChildBuilder,
    difficulty: AiDifficulty,
    current: AiDifficulty,
    font: Handle<Font>,
) {
    let is_selected = difficulty == current;
    let (bg_color, border_color, desc) = match difficulty {
        AiDifficulty::Easy => (
            Color::srgb(0.10, 0.22, 0.10),
            Color::srgb(0.20, 0.70, 0.20),
            "무작위 수 혼합  |  depth 2",
        ),
        AiDifficulty::Normal => (
            Color::srgb(0.16, 0.16, 0.10),
            Color::srgb(0.70, 0.65, 0.10),
            "균형 잡힌 플레이  |  depth 4",
        ),
        AiDifficulty::Hard => (
            Color::srgb(0.22, 0.10, 0.10),
            Color::srgb(0.80, 0.20, 0.20),
            "최강 위치 전략  |  depth 6",
        ),
    };

    let border = if is_selected { border_color } else { Color::srgb(0.30, 0.30, 0.30) };
    let bg = if is_selected { bg_color } else { COLOR_BTN_NORMAL };

    let btn_variant = match difficulty {
        AiDifficulty::Easy => DifficultyButton::Easy,
        AiDifficulty::Normal => DifficultyButton::Normal,
        AiDifficulty::Hard => DifficultyButton::Hard,
    };

    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(320.0),
                height: Val::Px(72.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(2.0)),
                row_gap: Val::Px(4.0),
                ..default()
            },
            BackgroundColor(bg),
            BorderColor(border),
            BorderRadius::all(Val::Px(10.0)),
            btn_variant,
            SelectedDifficultyMarker(difficulty),
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(difficulty.label()),
                TextFont { font: font.clone(), font_size: 26.0, ..default() },
                TextColor(if is_selected { border_color } else { COLOR_TEXT }),
            ));
            b.spawn((
                Text::new(desc),
                TextFont { font: font.clone(), font_size: 13.0, ..default() },
                TextColor(Color::srgb(0.55, 0.55, 0.55)),
            ));
        });
}

// ─── 난이도 버튼 인터랙션 ────────────────────────────────────────────────────
pub fn handle_difficulty_buttons(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor, &DifficultyButton),
        Changed<Interaction>,
    >,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut bg, mut border, btn) in &mut interaction_query {
        match interaction {
            Interaction::Hovered => {
                if !matches!(btn, DifficultyButton::Back) {
                    *bg = BackgroundColor(COLOR_BTN_HOVER);
                } else {
                    *bg = BackgroundColor(Color::srgb(0.20, 0.20, 0.20));
                }
            }
            Interaction::Pressed => match btn {
                DifficultyButton::Easy => {
                    commands.insert_resource(AiDifficulty::Easy);
                    next_state.set(GameState::Playing);
                }
                DifficultyButton::Normal => {
                    commands.insert_resource(AiDifficulty::Normal);
                    next_state.set(GameState::Playing);
                }
                DifficultyButton::Hard => {
                    commands.insert_resource(AiDifficulty::Hard);
                    next_state.set(GameState::Playing);
                }
                DifficultyButton::Back => {
                    next_state.set(GameState::MainMenu);
                }
            },
            Interaction::None => {
                *bg = BackgroundColor(COLOR_BTN_NORMAL);
                *border = BorderColor(Color::srgb(0.30, 0.30, 0.30));
            }
        }
    }
}

// ─── 메뉴 정리 ───────────────────────────────────────────────────────────────
pub fn despawn_menu(mut commands: Commands, query: Query<Entity, With<MenuEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
