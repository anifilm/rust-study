//! 시작 메뉴와 AI 대전 난이도 선택 화면.

use bevy::prelude::*;

use crate::chess::{text_font, GameMode};
use crate::{AppState, GameFont};

const NORMAL: Color = Color::srgb(0.20, 0.20, 0.24);
const HOVERED: Color = Color::srgb(0.30, 0.30, 0.36);
const PRESSED: Color = Color::srgb(0.32, 0.52, 0.34);

/// 메뉴 화면 엔티티 마커.
#[derive(Component)]
struct OnMenuScreen;

/// AI 난이도 선택 화면 엔티티 마커.
#[derive(Component)]
struct OnAiSetupScreen;

/// 메뉴 버튼의 동작.
#[derive(Component, Clone, Copy)]
enum MenuButton {
    OneVsOne,
    VersusAi,
    Quit,
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Menu), setup_menu)
            .add_systems(OnExit(AppState::Menu), despawn::<OnMenuScreen>)
            .add_systems(
                Update,
                menu_buttons.run_if(in_state(AppState::Menu)),
            )
            .add_systems(OnEnter(AppState::AiSetup), setup_ai_setup)
            .add_systems(OnExit(AppState::AiSetup), despawn::<OnAiSetupScreen>)
            .add_systems(
                Update,
                ai_setup_buttons.run_if(in_state(AppState::AiSetup)),
            );
    }
}

fn setup_menu(mut commands: Commands, font: Res<GameFont>) {
    commands.spawn((
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(18),
            ..default()
        },
        OnMenuScreen,
        children![
            (
                Text::new("체스 (Chess)"),
                text_font(&font, 64.0),
                TextColor(Color::srgb(0.95, 0.95, 0.95)),
                Node {
                    margin: UiRect::bottom(px(8)),
                    ..default()
                },
            ),
            (
                Text::new("Bevy 로 만든 체스 — 모드를 선택하세요"),
                text_font(&font, 20.0),
                TextColor(Color::srgb(0.7, 0.7, 0.75)),
                Node {
                    margin: UiRect::bottom(px(24)),
                    ..default()
                },
            ),
            menu_button(&font, MenuButton::OneVsOne, "1:1 대전"),
            menu_button(&font, MenuButton::VersusAi, "AI 대전"),
            menu_button(&font, MenuButton::Quit, "나가기"),
        ],
    ));
}

/// 메뉴 버튼 하나를 만든다.
fn menu_button(font: &GameFont, action: MenuButton, label: &str) -> impl Bundle {
    (
        Button,
        action,
        Node {
            width: px(280),
            height: px(64),
            border: UiRect::all(px(2)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(px(10)),
            ..default()
        },
        BorderColor::all(Color::srgb(0.45, 0.45, 0.5)),
        BackgroundColor(NORMAL),
        children![(
            Text::new(label),
            text_font(font, 28.0),
            TextColor(Color::srgb(0.92, 0.92, 0.92)),
        )],
    )
}

fn menu_buttons(
    mut interactions: Query<
        (&Interaction, &MenuButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut next_state: ResMut<NextState<AppState>>,
    mut exit: MessageWriter<AppExit>,
    mut commands: Commands,
) {
    for (interaction, button, mut color) in &mut interactions {
        match interaction {
            Interaction::Pressed => {
                *color = PRESSED.into();
                match button {
                    MenuButton::OneVsOne => {
                        commands.insert_resource(GameMode::LocalVersus);
                        next_state.set(AppState::InGame);
                    }
                    MenuButton::VersusAi => next_state.set(AppState::AiSetup),
                    MenuButton::Quit => {
                        exit.write(AppExit::Success);
                    }
                }
            }
            Interaction::Hovered => *color = HOVERED.into(),
            Interaction::None => *color = NORMAL.into(),
        }
    }
}

/// AI 난이도 버튼.
#[derive(Component, Clone, Copy)]
enum AiDifficultyButton {
    Easy,
    Medium,
    Hard,
    Back,
}

fn setup_ai_setup(mut commands: Commands, font: Res<GameFont>) {
    commands.spawn((
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(16),
            ..default()
        },
        OnAiSetupScreen,
        children![
            (
                Text::new("AI 대전"),
                text_font(&font, 52.0),
                TextColor(Color::srgb(0.95, 0.95, 0.95)),
            ),
            (
                Text::new("난이도를 선택하세요 (플레이어: 백 / AI: 흑)"),
                text_font(&font, 20.0),
                TextColor(Color::srgb(0.7, 0.7, 0.75)),
                Node {
                    margin: UiRect::bottom(px(12)),
                    ..default()
                },
            ),
            ai_difficulty_button(&font, AiDifficultyButton::Easy, "쉬움"),
            ai_difficulty_button(&font, AiDifficultyButton::Medium, "보통"),
            ai_difficulty_button(&font, AiDifficultyButton::Hard, "어려움"),
            (
                Button,
                AiDifficultyButton::Back,
                Node {
                    width: px(280),
                    height: px(58),
                    margin: UiRect::top(px(8)),
                    border: UiRect::all(px(2)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border_radius: BorderRadius::all(px(10)),
                    ..default()
                },
                BorderColor::all(Color::srgb(0.45, 0.45, 0.5)),
                BackgroundColor(NORMAL),
                children![(
                    Text::new("← 메뉴로"),
                    text_font(&font, 24.0),
                    TextColor(Color::srgb(0.92, 0.92, 0.92)),
                )],
            ),
        ],
    ));
}

fn ai_difficulty_button(font: &GameFont, action: AiDifficultyButton, label: &str) -> impl Bundle {
    (
        Button,
        action,
        Node {
            width: px(280),
            height: px(58),
            border: UiRect::all(px(2)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(px(10)),
            ..default()
        },
        BorderColor::all(Color::srgb(0.45, 0.45, 0.5)),
        BackgroundColor(NORMAL),
        children![(
            Text::new(label),
            text_font(font, 26.0),
            TextColor(Color::srgb(0.92, 0.92, 0.92)),
        )],
    )
}

fn ai_setup_buttons(
    mut interactions: Query<
        (&Interaction, &AiDifficultyButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut next_state: ResMut<NextState<AppState>>,
    mut commands: Commands,
) {
    for (interaction, button, mut color) in &mut interactions {
        match interaction {
            Interaction::Pressed => {
                *color = PRESSED.into();
                match button {
                    AiDifficultyButton::Easy => {
                        commands.insert_resource(GameMode::AiVersus { search_depth: 2 });
                        next_state.set(AppState::InGame);
                    }
                    AiDifficultyButton::Medium => {
                        commands.insert_resource(GameMode::AiVersus { search_depth: 3 });
                        next_state.set(AppState::InGame);
                    }
                    AiDifficultyButton::Hard => {
                        commands.insert_resource(GameMode::AiVersus { search_depth: 4 });
                        next_state.set(AppState::InGame);
                    }
                    AiDifficultyButton::Back => next_state.set(AppState::Menu),
                }
            }
            Interaction::Hovered => *color = HOVERED.into(),
            Interaction::None => *color = NORMAL.into(),
        }
    }
}

/// 마커 컴포넌트를 가진 모든 엔티티를 제거하는 범용 정리 시스템.
fn despawn<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
