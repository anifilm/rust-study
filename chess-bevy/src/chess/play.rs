//! InGame 상태의 런타임 로직: 마우스 입력, 보드 다시 그리기, HUD 갱신, 키 입력.

use bevy::prelude::*;

use crate::chess::rules::{
    apply_move, find_king, legal_moves_from, piece_at, status_for, Color as PieceColor, MoveFlag,
    Square, Status,
};
use crate::chess::{
    square_to_world, text_font, world_to_square, Board, BoardDirty, ChessAssets, HighlightVisual,
    OnGameScreen, PieceVisual, Selection, StatusText, TILE, Z_HIGHLIGHT, Z_LETTER, Z_PIECE,
};
use crate::{AppState, GameFont};

/// 메뉴로 돌아가는 HUD 버튼 마커.
#[derive(Component)]
pub struct BackToMenuButton;

/// 좌클릭으로 기물을 선택하고 합법 수 칸을 클릭해 이동시킨다.
pub fn handle_input(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut board: ResMut<Board>,
    mut selection: ResMut<Selection>,
    mut dirty: ResMut<BoardDirty>,
) {
    if !mouse.just_pressed(MouseButton::Left) || board.is_over() {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok((camera, cam_tf)) = cameras.single() else {
        return;
    };
    let Ok(world) = camera.viewport_to_world_2d(cam_tf, cursor) else {
        return;
    };

    let Some(sq) = world_to_square(world) else {
        // 보드 밖 클릭 → 선택 해제.
        if selection.from.is_some() {
            selection.clear();
            dirty.0 = true;
        }
        return;
    };

    // 선택된 기물의 합법 수 도착 칸을 클릭했다면 이동 실행.
    if let Some(mv) = selection.moves.iter().copied().find(|m| m.to == sq) {
        let (next, en_passant) = apply_move(&board.squares, mv);
        board.squares = next;
        board.en_passant = en_passant;
        board.turn = board.turn.opposite();
        board.last_move = Some(mv);
        board.status = status_for(&board.squares, board.en_passant, board.turn);
        selection.clear();
        dirty.0 = true;
        return;
    }

    // 그 외에는 현재 차례 기물을 선택(또는 선택 해제).
    match piece_at(&board.squares, sq) {
        Some(p) if p.color == board.turn => {
            selection.from = Some(sq);
            selection.moves = legal_moves_from(&board.squares, board.en_passant, sq);
            dirty.0 = true;
        }
        _ => {
            if selection.from.is_some() {
                selection.clear();
                dirty.0 = true;
            }
        }
    }
}

/// 보드가 변경되었을 때만 기물/하이라이트를 다시 그린다.
pub fn redraw(
    mut commands: Commands,
    mut dirty: ResMut<BoardDirty>,
    board: Res<Board>,
    selection: Res<Selection>,
    assets: Res<ChessAssets>,
    font: Res<GameFont>,
    pieces: Query<Entity, With<PieceVisual>>,
    highlights: Query<Entity, With<HighlightVisual>>,
) {
    if !dirty.0 {
        return;
    }
    dirty.0 = false;

    for entity in &pieces {
        commands.entity(entity).despawn();
    }
    for entity in &highlights {
        commands.entity(entity).despawn();
    }

    // 마지막 수의 출발/도착 칸.
    if let Some(mv) = board.last_move {
        let tint = Color::srgba(0.95, 0.9, 0.3, 0.30);
        spawn_overlay(&mut commands, mv.from, tint, 0.96);
        spawn_overlay(&mut commands, mv.to, tint, 0.96);
    }

    // 선택 칸 + 합법 수 표시.
    if let Some(from) = selection.from {
        spawn_overlay(&mut commands, from, Color::srgba(0.95, 0.85, 0.2, 0.55), 0.96);
        for mv in &selection.moves {
            let is_capture =
                piece_at(&board.squares, mv.to).is_some() || mv.flag == MoveFlag::EnPassant;
            if is_capture {
                spawn_overlay(&mut commands, mv.to, Color::srgba(0.85, 0.2, 0.2, 0.45), 0.96);
            } else {
                // 빈 칸은 작은 점으로 표시.
                spawn_overlay(&mut commands, mv.to, Color::srgba(0.15, 0.6, 0.25, 0.55), 0.32);
            }
        }
    }

    // 체크/체크메이트인 킹 강조.
    if matches!(board.status, Status::Check | Status::Checkmate { .. }) {
        let king_color = match board.status {
            Status::Checkmate { winner } => winner.opposite(),
            _ => board.turn,
        };
        if let Some(king_sq) = find_king(&board.squares, king_color) {
            spawn_overlay(&mut commands, king_sq, Color::srgba(0.9, 0.1, 0.1, 0.5), 0.96);
        }
    }

    // 기물(원판 + 글자).
    for rank in 0..8 {
        for file in 0..8 {
            let sq = Square::new(file, rank);
            if let Some(piece) = board.squares[sq.index()] {
                let pos = square_to_world(sq);
                let (material, letter_color) = if piece.color == PieceColor::White {
                    (assets.white_disc.clone(), Color::srgb(0.1, 0.1, 0.1))
                } else {
                    (assets.black_disc.clone(), Color::srgb(0.93, 0.93, 0.93))
                };

                commands.spawn((
                    Mesh2d(assets.disc.clone()),
                    MeshMaterial2d(material),
                    Transform::from_translation(pos.extend(Z_PIECE)),
                    PieceVisual,
                    OnGameScreen,
                    children![(
                        Text2d::new(piece.kind.letter()),
                        text_font(&font, TILE * 0.5),
                        TextColor(letter_color),
                        Transform::from_xyz(0.0, 0.0, Z_LETTER - Z_PIECE),
                    )],
                ));
            }
        }
    }
}

fn spawn_overlay(commands: &mut Commands, sq: Square, color: Color, frac: f32) {
    commands.spawn((
        Sprite::from_color(color, Vec2::splat(TILE * frac)),
        Transform::from_translation(square_to_world(sq).extend(Z_HIGHLIGHT)),
        HighlightVisual,
        OnGameScreen,
    ));
}

/// 상단 상태 텍스트를 현재 보드 상태에 맞게 갱신한다.
pub fn update_hud(board: Res<Board>, mut text: Query<&mut Text, With<StatusText>>) {
    if !board.is_changed() {
        return;
    }
    let Ok(mut text) = text.single_mut() else {
        return;
    };

    **text = match board.status {
        Status::Ongoing => format!("{} 차례", board.turn.label_ko()),
        Status::Check => format!("{} 차례 — 체크!", board.turn.label_ko()),
        Status::Checkmate { winner } => {
            format!("체크메이트! {} 승리 — R: 새 게임, Esc: 메뉴", winner.label_ko())
        }
        Status::Stalemate => "스테일메이트(무승부) — R: 새 게임, Esc: 메뉴".to_string(),
    };
}

/// Esc → 메뉴, R → 새 게임.
pub fn handle_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut board: ResMut<Board>,
    mut selection: ResMut<Selection>,
    mut dirty: ResMut<BoardDirty>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::Menu);
    }
    if keys.just_pressed(KeyCode::KeyR) {
        board.reset();
        selection.clear();
        dirty.0 = true;
    }
}

/// HUD 의 "메뉴로" 버튼 클릭 처리.
pub fn handle_back_button(
    interactions: Query<&Interaction, (Changed<Interaction>, With<BackToMenuButton>)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in &interactions {
        if *interaction == Interaction::Pressed {
            next_state.set(AppState::Menu);
        }
    }
}
