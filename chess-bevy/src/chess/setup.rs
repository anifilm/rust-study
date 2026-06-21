//! InGame 진입 시 보드 타일 / 캐시 에셋 / HUD 를 구성한다.

use bevy::prelude::*;

use crate::chess::rules::Square;
use crate::chess::{
    square_to_world, text_font, Board, BoardDirty, ChessAssets, HistoryText, MoveHistory,
    OnGameScreen, Selection, StatusText, DARK_SQUARE, LIGHT_SQUARE, TILE, Z_TILE,
};
use crate::GameFont;

/// 보드 타일과 기물 텍스처를 준비하고, 보드 상태를 초기화한다.
pub fn setup_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut board: ResMut<Board>,
    mut selection: ResMut<Selection>,
    mut history: ResMut<MoveHistory>,
    mut dirty: ResMut<BoardDirty>,
) {
    board.reset();
    selection.clear();
    history.clear();

    commands.insert_resource(ChessAssets::load(&asset_server, &mut images));

    // 8x8 타일. a1(0,0) 이 어두운 칸이 되도록 (file+rank) 짝수를 어둡게.
    for rank in 0..8 {
        for file in 0..8 {
            let sq = Square::new(file, rank);
            let color = if (file + rank) % 2 == 0 {
                DARK_SQUARE
            } else {
                LIGHT_SQUARE
            };
            commands.spawn((
                Sprite::from_color(color, Vec2::splat(TILE)),
                Transform::from_translation(square_to_world(sq).extend(Z_TILE)),
                OnGameScreen,
            ));
        }
    }

    // 보드 좌표 라벨(a~h, 1~8)을 가장자리에 표시.
    dirty.0 = true;
}

/// 상단 상태 텍스트, 좌상단 "메뉴로" 버튼, 오른쪽 이동 기록 패널을 만든다.
pub fn setup_hud(mut commands: Commands, font: Res<GameFont>) {
    // 보드 위(상단 중앙) 상태 텍스트. 보드는 화면 왼쪽 [30, 670] 영역에 그려진다.
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: px(14),
            left: px(30),
            width: px(640),
            justify_content: JustifyContent::Center,
            ..default()
        },
        OnGameScreen,
        children![(
            Text::new(""),
            text_font(&font, 24.0),
            TextColor(Color::WHITE),
            StatusText,
        )],
    ));

    // 오른쪽 이동 기록 패널.
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            right: px(16),
            width: px(330),
            height: px(736),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(12)),
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(px(8)),
            row_gap: px(10),
            ..default()
        },
        BackgroundColor(Color::srgb(0.15, 0.15, 0.19)),
        BorderColor::all(Color::srgb(0.32, 0.32, 0.38)),
        OnGameScreen,
        children![
            (
                Text::new("이동 기록"),
                text_font(&font, 24.0),
                TextColor(Color::srgb(0.95, 0.95, 0.95)),
            ),
            // 기보 목록: 넘치면 위쪽이 잘리고 최신 수가 항상 아래에 보이도록 한다.
            (
                Node {
                    flex_grow: 1.0,
                    width: percent(100),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::FlexEnd,
                    overflow: Overflow::clip(),
                    ..default()
                },
                children![(
                    Text::new("아직 둔 수가 없습니다."),
                    text_font(&font, 18.0),
                    TextColor(Color::srgb(0.8, 0.8, 0.85)),
                    HistoryText,
                )],
            ),
            // 되돌리기 버튼.
            (
                Button,
                crate::chess::play::UndoButton,
                Node {
                    width: percent(100),
                    height: px(48),
                    border: UiRect::all(px(2)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border_radius: BorderRadius::all(px(8)),
                    ..default()
                },
                BorderColor::all(Color::srgb(0.5, 0.5, 0.55)),
                BackgroundColor(Color::srgb(0.22, 0.22, 0.27)),
                children![(
                    Text::new("되돌리기 (U)"),
                    text_font(&font, 20.0),
                    TextColor(Color::srgb(0.92, 0.92, 0.92)),
                )],
            ),
        ],
    ));

    // 좌상단 메뉴 복귀 버튼.
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(16),
            ..default()
        },
        OnGameScreen,
        children![(
            Button,
            crate::chess::play::BackToMenuButton,
            Node {
                padding: UiRect::axes(px(14), px(8)),
                border: UiRect::all(px(2)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(px(6)),
                ..default()
            },
            BorderColor::all(Color::srgb(0.6, 0.6, 0.65)),
            BackgroundColor(Color::srgb(0.18, 0.18, 0.22)),
            children![(
                Text::new("← 메뉴 (Esc)"),
                text_font(&font, 18.0),
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            )]
        )],
    ));
}
