//! 시작 메뉴와 "준비 중"(AI 대전) 화면.

use bevy::prelude::*;

use crate::chess::{text_font, GameMode};
use crate::{AppState, GameFont};

const NORMAL: Color = Color::srgb(0.20, 0.20, 0.24);
const HOVERED: Color = Color::srgb(0.30, 0.30, 0.36);
const PRESSED: Color = Color::srgb(0.32, 0.52, 0.34);

/// 메뉴 화면 엔티티 마커.
#[derive(Component)]
struct OnMenuScreen;

/// "준비 중" 화면 엔티티 마커.
#[derive(Component)]
struct OnComingSoonScreen;

/// 메뉴 버튼의 동작.
#[derive(Component, Clone, Copy)]
enum MenuButton {
    OneVsOne,
    VersusAi,
    Quit,
}

/// 준비 중 화면의 "메뉴로" 버튼.
#[derive(Component)]
struct BackToMenu;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Menu), setup_menu)
            .add_systems(OnExit(AppState::Menu), despawn::<OnMenuScreen>)
            .add_systems(
                Update,
                menu_buttons.run_if(in_state(AppState::Menu)),
            )
            .add_systems(OnEnter(AppState::ComingSoon), setup_coming_soon)
            .add_systems(
                OnExit(AppState::ComingSoon),
                despawn::<OnComingSoonScreen>,
            )
            .add_systems(
                Update,
                coming_soon_buttons.run_if(in_state(AppState::ComingSoon)),
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
                    MenuButton::VersusAi => next_state.set(AppState::ComingSoon),
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

fn setup_coming_soon(mut commands: Commands, font: Res<GameFont>) {
    commands.spawn((
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(20),
            ..default()
        },
        OnComingSoonScreen,
        children![
            (
                Text::new("AI 대전"),
                text_font(&font, 52.0),
                TextColor(Color::srgb(0.95, 0.95, 0.95)),
            ),
            (
                Text::new("AI 대전 모드는 준비 중입니다.\n먼저 1:1 대전을 즐겨보세요!"),
                text_font(&font, 22.0),
                TextColor(Color::srgb(0.75, 0.75, 0.8)),
                TextLayout::justify(Justify::Center),
                Node {
                    margin: UiRect::bottom(px(16)),
                    ..default()
                },
            ),
            (
                Button,
                BackToMenu,
                Node {
                    width: px(240),
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
                    Text::new("← 메뉴로"),
                    text_font(&font, 26.0),
                    TextColor(Color::srgb(0.92, 0.92, 0.92)),
                )],
            ),
        ],
    ));
}

fn coming_soon_buttons(
    mut interactions: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<BackToMenu>),
    >,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, mut color) in &mut interactions {
        match interaction {
            Interaction::Pressed => {
                *color = PRESSED.into();
                next_state.set(AppState::Menu);
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
