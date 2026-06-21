//! InGame 진입 시 보드 타일 / 캐시 에셋 / HUD 를 구성한다.

use bevy::prelude::*;

use crate::chess::{
    square_to_world, text_font, Board, BoardDirty, ChessAssets, OnGameScreen, Selection,
    StatusText, DARK_SQUARE, LIGHT_SQUARE, TILE, Z_TILE,
};
use crate::chess::rules::Square;
use crate::GameFont;

/// 보드 타일과 기물용 캐시 에셋을 만들고, 보드 상태를 초기화한다.
pub fn setup_game(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut board: ResMut<Board>,
    mut selection: ResMut<Selection>,
    mut dirty: ResMut<BoardDirty>,
) {
    board.reset();
    selection.clear();

    commands.insert_resource(ChessAssets {
        disc: meshes.add(Circle::new(TILE * 0.40)),
        white_disc: materials.add(Color::srgb(0.97, 0.95, 0.90)),
        black_disc: materials.add(Color::srgb(0.16, 0.16, 0.19)),
    });

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

/// 상단 상태 텍스트와 좌상단 "메뉴로" 버튼을 만든다.
pub fn setup_hud(mut commands: Commands, font: Res<GameFont>) {
    // 상단 중앙 상태 텍스트.
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: px(14),
            left: px(0),
            width: percent(100),
            justify_content: JustifyContent::Center,
            ..default()
        },
        OnGameScreen,
        children![(
            Text::new(""),
            text_font(&font, 26.0),
            TextColor(Color::WHITE),
            StatusText,
        )],
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
