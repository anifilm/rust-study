//! InGame 상태의 런타임 로직: 마우스 입력, 보드 다시 그리기, HUD 갱신, 키 입력.

use bevy::prelude::*;

use crate::chess::ai::choose_move;
use crate::chess::rules::{
    apply_move, find_king, legal_moves_from, move_to_san, piece_at, status_for, ChessMove,
    Color as PieceColor, MoveFlag, Square, Status,
};
use crate::chess::{
    square_to_world, world_to_square, AiTurnState, Board, BoardDirty, ChessAssets, GameMode,
    HighlightVisual, HistoryText, MoveHistory, OnGameScreen, PieceVisual, Selection, StatusText,
    TILE, Z_CHECK, Z_LAST_MOVE, Z_MOVE_HINT, Z_PIECE, Z_PIECE_MOVING, Z_SELECT,
};
use crate::AppState;

/// 이동 애니메이션 지속 시간(초).
const ANIM_DURATION: f32 = 0.18;

/// 메뉴로 돌아가는 HUD 버튼 마커.
#[derive(Component)]
pub struct BackToMenuButton;

/// 되돌리기 버튼 마커.
#[derive(Component)]
pub struct UndoButton;

/// 다음 redraw 때 특정 도착 칸의 기물을 어디서부터 슬라이드시킬지 기록.
#[derive(Clone, Copy)]
pub struct AnimSpec {
    dest: Square,
    from_world: Vec2,
}

/// 이번 수에서 애니메이션할 기물 목록(redraw 가 소비).
#[derive(Resource, Default)]
pub struct MoveAnimations(Vec<AnimSpec>);

/// 슬라이드 중인 기물에 붙는 컴포넌트.
#[derive(Component)]
pub struct MoveAnim {
    start: Vec2,
    end: Vec2,
    elapsed: f32,
}

/// 한 수(또는 그 역방향)에 대한 애니메이션 스펙을 쌓는다.
/// 캐슬링이면 룩 이동도 함께 포함한다.
fn push_move_anims(anims: &mut MoveAnimations, mv: ChessMove, reverse: bool) {
    let (src, dst) = if reverse {
        (mv.to, mv.from)
    } else {
        (mv.from, mv.to)
    };
    anims.0.push(AnimSpec {
        dest: dst,
        from_world: square_to_world(src),
    });

    if mv.flag == MoveFlag::Castle {
        let rank = mv.from.rank;
        let (rook_from, rook_to) = if mv.to.file == 6 {
            (Square::new(7, rank), Square::new(5, rank))
        } else {
            (Square::new(0, rank), Square::new(3, rank))
        };
        let (rsrc, rdst) = if reverse {
            (rook_to, rook_from)
        } else {
            (rook_from, rook_to)
        };
        anims.0.push(AnimSpec {
            dest: rdst,
            from_world: square_to_world(rsrc),
        });
    }
}

/// 한 수를 보드에 적용하고 기록·애니메이션·dirty 플래그를 갱신한다.
pub fn execute_move(
    board: &mut Board,
    history: &mut MoveHistory,
    anims: &mut MoveAnimations,
    dirty: &mut BoardDirty,
    mv: ChessMove,
) {
    let snapshot = board.snapshot();
    let san = move_to_san(&board.squares, board.en_passant, mv);

    let (next, en_passant) = apply_move(&board.squares, mv);
    board.squares = next;
    board.en_passant = en_passant;
    board.turn = board.turn.opposite();
    board.last_move = Some(mv);
    board.status = status_for(&board.squares, board.en_passant, board.turn);

    let suffix = match board.status {
        Status::Checkmate { .. } => "#",
        Status::Check => "+",
        _ => "",
    };
    history.snapshots.push(snapshot);
    history.sans.push(format!("{san}{suffix}"));

    push_move_anims(anims, mv, false);
    dirty.0 = true;
}

/// 좌클릭으로 기물을 선택하고 합법 수 칸을 클릭해 이동시킨다.
pub fn handle_input(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    game_mode: Res<GameMode>,
    ai_state: Res<AiTurnState>,
    mut board: ResMut<Board>,
    mut selection: ResMut<Selection>,
    mut history: ResMut<MoveHistory>,
    mut anims: ResMut<MoveAnimations>,
    mut dirty: ResMut<BoardDirty>,
) {
    if game_mode.is_ai_versus() {
        if board.turn != PieceColor::White || ai_state.thinking {
            return;
        }
    }

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
        execute_move(
            &mut board,
            &mut history,
            &mut anims,
            &mut dirty,
            mv,
        );
        selection.clear();
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
    mut anims: ResMut<MoveAnimations>,
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
        spawn_overlay(&mut commands, mv.from, tint, 0.96, Z_LAST_MOVE);
        spawn_overlay(&mut commands, mv.to, tint, 0.96, Z_LAST_MOVE);
    }

    // 체크/체크메이트인 킹 강조.
    if matches!(board.status, Status::Check | Status::Checkmate { .. }) {
        let king_color = match board.status {
            Status::Checkmate { winner } => winner.opposite(),
            _ => board.turn,
        };
        if let Some(king_sq) = find_king(&board.squares, king_color) {
            spawn_overlay(&mut commands, king_sq, Color::srgba(0.9, 0.1, 0.1, 0.5), 0.96, Z_CHECK);
        }
    }

    // 선택 칸(노란 사각형) + 합법 수(원형 표시).
    if let Some(from) = selection.from {
        spawn_overlay(
            &mut commands,
            from,
            Color::srgba(0.95, 0.85, 0.2, 0.55),
            0.96,
            Z_SELECT,
        );
        for mv in &selection.moves {
            let is_capture =
                piece_at(&board.squares, mv.to).is_some() || mv.flag == MoveFlag::EnPassant;
            // 빈 칸은 가운데 점, 잡을 수 있는 칸은 링(원형 테두리)으로 표시.
            let (image, frac) = if is_capture {
                (assets.capture_ring.clone(), 0.98)
            } else {
                (assets.move_dot.clone(), 0.32)
            };
            spawn_hint(&mut commands, image, frac, mv.to);
        }
    }

    // 기물(이미지 스프라이트). 이번 수에 움직인 기물은 출발 칸에서 슬라이드시킨다.
    for rank in 0..8 {
        for file in 0..8 {
            let sq = Square::new(file, rank);
            if let Some(piece) = board.squares[sq.index()] {
                let target = square_to_world(sq);
                let anim_start = anims.0.iter().find(|a| a.dest == sq).map(|a| a.from_world);
                // 움직이는 기물은 다른 기물 위로 지나가도록 살짝 높은 z 로 시작.
                let (initial, z) = match anim_start {
                    Some(from) => (from, Z_PIECE_MOVING),
                    None => (target, Z_PIECE),
                };

                let mut entity = commands.spawn((
                    Sprite {
                        image: assets.image(piece.color, piece.kind),
                        custom_size: Some(Vec2::splat(TILE * 0.86)),
                        ..default()
                    },
                    Transform::from_translation(initial.extend(z)),
                    PieceVisual,
                    OnGameScreen,
                ));

                if let Some(from) = anim_start {
                    entity.insert(MoveAnim {
                        start: from,
                        end: target,
                        elapsed: 0.0,
                    });
                }
            }
        }
    }

    anims.0.clear();
}

/// 슬라이드 중인 기물을 매 프레임 보간(ease-out)하여 이동시킨다.
pub fn animate_pieces(
    time: Res<Time>,
    mut commands: Commands,
    mut pieces: Query<(Entity, &mut Transform, &mut MoveAnim)>,
) {
    for (entity, mut transform, mut anim) in &mut pieces {
        anim.elapsed += time.delta_secs();
        let t = (anim.elapsed / ANIM_DURATION).clamp(0.0, 1.0);
        let eased = 1.0 - (1.0 - t).powi(3);
        let pos = anim.start.lerp(anim.end, eased);

        if t >= 1.0 {
            transform.translation = anim.end.extend(Z_PIECE);
            commands.entity(entity).remove::<MoveAnim>();
        } else {
            transform.translation = pos.extend(Z_PIECE_MOVING);
        }
    }
}

fn spawn_overlay(commands: &mut Commands, sq: Square, color: Color, frac: f32, z: f32) {
    commands.spawn((
        Sprite::from_color(color, Vec2::splat(TILE * frac)),
        Transform::from_translation(square_to_world(sq).extend(z)),
        HighlightVisual,
        OnGameScreen,
    ));
}

/// 이동 가능 칸을 원형 텍스처 스프라이트(점/링)로 표시한다.
fn spawn_hint(commands: &mut Commands, image: Handle<Image>, frac: f32, sq: Square) {
    commands.spawn((
        Sprite {
            image,
            custom_size: Some(Vec2::splat(TILE * frac)),
            ..default()
        },
        Transform::from_translation(square_to_world(sq).extend(Z_MOVE_HINT)),
        HighlightVisual,
        OnGameScreen,
    ));
}

/// AI 차례에 수를 계산하고 적용한다.
pub fn ai_turn(
    time: Res<Time>,
    game_mode: Res<GameMode>,
    mut ai_state: ResMut<AiTurnState>,
    mut board: ResMut<Board>,
    mut selection: ResMut<Selection>,
    mut history: ResMut<MoveHistory>,
    mut anims: ResMut<MoveAnimations>,
    mut dirty: ResMut<BoardDirty>,
) {
    let GameMode::AiVersus { search_depth } = *game_mode else {
        return;
    };

    if board.is_over() || board.turn != PieceColor::Black {
        ai_state.thinking = false;
        return;
    }

    if !ai_state.thinking {
        ai_state.thinking = true;
        ai_state.delay.reset();
        return;
    }

    if !ai_state.delay.tick(time.delta()).is_finished() {
        return;
    }

    let Some(mv) = choose_move(
        &board.squares,
        board.en_passant,
        PieceColor::Black,
        search_depth,
    ) else {
        ai_state.thinking = false;
        return;
    };

    execute_move(
        &mut board,
        &mut history,
        &mut anims,
        &mut dirty,
        mv,
    );
    selection.clear();
    ai_state.thinking = false;
}

/// 상단 상태 텍스트를 현재 보드 상태에 맞게 갱신한다.
pub fn update_hud(
    board: Res<Board>,
    game_mode: Res<GameMode>,
    ai_state: Res<AiTurnState>,
    mut text: Query<&mut Text, With<StatusText>>,
) {
    if !board.is_changed() && !game_mode.is_changed() && !ai_state.is_changed() {
        return;
    }
    let Ok(mut text) = text.single_mut() else {
        return;
    };

    if game_mode.is_ai_versus() && ai_state.thinking {
        **text = "AI 생각 중...".to_string();
        return;
    }

    let mode_suffix = if game_mode.is_ai_versus() {
        " — AI 대전"
    } else {
        ""
    };

    **text = match board.status {
        Status::Ongoing => format!("{} 차례{}", board.turn.label_ko(), mode_suffix),
        Status::Check => format!("{} 차례 — 체크!{}", board.turn.label_ko(), mode_suffix),
        Status::Checkmate { winner } => {
            format!("체크메이트! {} 승리 — R: 새 게임, Esc: 메뉴", winner.label_ko())
        }
        Status::Stalemate => "스테일메이트(무승부) — R: 새 게임, Esc: 메뉴".to_string(),
    };
}

/// Esc → 메뉴, R → 새 게임, U → 되돌리기.
pub fn handle_keys(
    keys: Res<ButtonInput<KeyCode>>,
    game_mode: Res<GameMode>,
    mut ai_state: ResMut<AiTurnState>,
    mut next_state: ResMut<NextState<AppState>>,
    mut board: ResMut<Board>,
    mut selection: ResMut<Selection>,
    mut history: ResMut<MoveHistory>,
    mut anims: ResMut<MoveAnimations>,
    mut dirty: ResMut<BoardDirty>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::Menu);
    }
    if keys.just_pressed(KeyCode::KeyR) {
        board.reset();
        selection.clear();
        history.clear();
        ai_state.thinking = false;
        dirty.0 = true;
    }
    if keys.just_pressed(KeyCode::KeyU) {
        undo_moves(
            game_mode.as_ref(),
            &mut board,
            &mut selection,
            &mut history,
            &mut anims,
            &mut ai_state,
            &mut dirty,
        );
    }
}

/// 한 수 되돌린다. 직전 스냅샷과 기보를 함께 제거하고, 역방향 슬라이드를 재생한다.
fn undo_once(
    board: &mut Board,
    selection: &mut Selection,
    history: &mut MoveHistory,
    anims: &mut MoveAnimations,
    dirty: &mut BoardDirty,
) -> bool {
    let Some(snapshot) = history.snapshots.pop() else {
        return false;
    };
    history.sans.pop();
    let undone = board.last_move;
    board.restore(snapshot);
    if let Some(mv) = undone {
        push_move_anims(anims, mv, true);
    }
    selection.clear();
    dirty.0 = true;
    true
}

/// AI 모드에서는 플레이어+AI 한 턴(최대 2수)을 되돌린다.
fn undo_moves(
    game_mode: &GameMode,
    board: &mut Board,
    selection: &mut Selection,
    history: &mut MoveHistory,
    anims: &mut MoveAnimations,
    ai_state: &mut AiTurnState,
    dirty: &mut BoardDirty,
) {
    ai_state.thinking = false;

    if !undo_once(board, selection, history, anims, dirty) {
        return;
    }

    if game_mode.is_ai_versus()
        && board.turn == PieceColor::White
        && !history.snapshots.is_empty()
    {
        undo_once(board, selection, history, anims, dirty);
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

/// "되돌리기" 버튼 클릭 처리.
pub fn handle_undo_button(
    interactions: Query<&Interaction, (Changed<Interaction>, With<UndoButton>)>,
    game_mode: Res<GameMode>,
    mut ai_state: ResMut<AiTurnState>,
    mut board: ResMut<Board>,
    mut selection: ResMut<Selection>,
    mut history: ResMut<MoveHistory>,
    mut anims: ResMut<MoveAnimations>,
    mut dirty: ResMut<BoardDirty>,
) {
    for interaction in &interactions {
        if *interaction == Interaction::Pressed {
            undo_moves(
                game_mode.as_ref(),
                &mut board,
                &mut selection,
                &mut history,
                &mut anims,
                &mut ai_state,
                &mut dirty,
            );
        }
    }
}

/// 오른쪽 패널의 이동 기록 텍스트를 갱신한다.
pub fn update_history(history: Res<MoveHistory>, mut text: Query<&mut Text, With<HistoryText>>) {
    if !history.is_changed() {
        return;
    }
    let Ok(mut text) = text.single_mut() else {
        return;
    };

    if history.sans.is_empty() {
        **text = "아직 둔 수가 없습니다.".to_string();
        return;
    }

    let mut lines = String::new();
    for (i, pair) in history.sans.chunks(2).enumerate() {
        match pair {
            [white, black] => lines.push_str(&format!("{}. {}  {}\n", i + 1, white, black)),
            [white] => lines.push_str(&format!("{}. {}\n", i + 1, white)),
            _ => {}
        }
    }
    **text = lines.trim_end().to_string();
}
