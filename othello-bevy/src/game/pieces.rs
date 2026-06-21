use std::collections::VecDeque;

use bevy::prelude::*;

use crate::{
    constants::*,
    game::{
        assets::GameAssets,
        board::cell_to_world,
        logic::{Board, ValidMoves},
    },
    state::{GameEntity, Player},
};

// ─── 컴포넌트 ─────────────────────────────────────────────────────────────────
#[derive(Component)]
pub struct PieceCell {
    pub row: usize,
    pub col: usize,
}

#[derive(Component)]
pub struct PieceShadow(pub usize, pub usize);

#[derive(Component)]
pub struct PieceHighlight(pub usize, pub usize);

#[derive(Component)]
pub struct ValidHint;

// ─── 뒤집기 애니메이션 ───────────────────────────────────────────────────────
#[derive(Resource)]
pub struct FlipQueue {
    pub flips: VecDeque<(usize, usize)>,
    pub player: Player,
    pub timer: Timer,
}

#[derive(Event)]
pub struct FlipDoneEvent;

/// 뒤집기 애니메이션이 진행 중인가
pub fn is_animating(queue: Option<Res<FlipQueue>>) -> bool {
    queue.is_some()
}

// ─── 초기 돌 스폰 ─────────────────────────────────────────────────────────────
pub fn spawn_pieces(mut commands: Commands, board: Res<Board>, assets: Res<GameAssets>) {
    for row in 0..BOARD_SIZE {
        for col in 0..BOARD_SIZE {
            if let Some(player) = board.cells[row][col] {
                spawn_piece(&mut commands, &assets, row, col, player);
            }
        }
    }
}

pub fn spawn_piece(
    commands: &mut Commands,
    assets: &GameAssets,
    row: usize,
    col: usize,
    player: Player,
) {
    let pos = cell_to_world(row, col);
    let (piece_mat, hl_mat) = match player {
        Player::Black => (assets.black_mat.clone(), assets.black_hl_mat.clone()),
        Player::White => (assets.white_mat.clone(), assets.white_hl_mat.clone()),
    };

    // 그림자
    commands.spawn((
        Mesh2d(assets.shadow_mesh.clone()),
        MeshMaterial2d(assets.shadow_mat.clone()),
        Transform::from_xyz(pos.x + 1.5, pos.y - 1.5, 2.0),
        PieceShadow(row, col),
        GameEntity,
    ));
    // 메인 돌
    commands.spawn((
        Mesh2d(assets.piece_mesh.clone()),
        MeshMaterial2d(piece_mat),
        Transform::from_xyz(pos.x, pos.y, 3.0),
        PieceCell { row, col },
        GameEntity,
    ));
    // 하이라이트
    commands.spawn((
        Mesh2d(assets.highlight_mesh.clone()),
        MeshMaterial2d(hl_mat),
        Transform::from_xyz(pos.x - 6.0, pos.y + 6.0, 4.0),
        PieceHighlight(row, col),
        GameEntity,
    ));
}

/// 특정 셀의 돌 3개 엔티티(그림자·메인·하이라이트) 제거
fn despawn_piece_at(
    commands: &mut Commands,
    row: usize,
    col: usize,
    pieces: &Query<(Entity, &PieceCell)>,
    shadows: &Query<(Entity, &PieceShadow)>,
    highlights: &Query<(Entity, &PieceHighlight)>,
) {
    for (e, pc) in pieces.iter() {
        if pc.row == row && pc.col == col {
            commands.entity(e).despawn();
        }
    }
    for (e, ps) in shadows.iter() {
        if ps.0 == row && ps.1 == col {
            commands.entity(e).despawn();
        }
    }
    for (e, ph) in highlights.iter() {
        if ph.0 == row && ph.1 == col {
            commands.entity(e).despawn();
        }
    }
}

// ─── 순차 뒤집기 애니메이션 시스템 ───────────────────────────────────────────
#[allow(clippy::too_many_arguments)]
pub fn animate_flips(
    mut commands: Commands,
    mut board: ResMut<Board>,
    assets: Res<GameAssets>,
    time: Res<Time>,
    flip_queue: Option<ResMut<FlipQueue>>,
    pieces: Query<(Entity, &PieceCell)>,
    shadows: Query<(Entity, &PieceShadow)>,
    highlights: Query<(Entity, &PieceHighlight)>,
    mut flip_done: EventWriter<FlipDoneEvent>,
) {
    let Some(mut queue) = flip_queue else { return };
    queue.timer.tick(time.delta());
    if !queue.timer.just_finished() {
        return;
    }

    if let Some((r, c)) = queue.flips.pop_front() {
        // Board 갱신
        board.cells[r][c] = Some(queue.player);
        // 기존 돌 제거 + 새 색상으로 재스폰
        despawn_piece_at(&mut commands, r, c, &pieces, &shadows, &highlights);
        spawn_piece(&mut commands, &assets, r, c, queue.player);
    }

    if queue.flips.is_empty() {
        commands.remove_resource::<FlipQueue>();
        flip_done.send(FlipDoneEvent);
    }
}

// ─── 보드 변경 시 돌 전체 동기화 (애니메이션 미진행 시만) ───────────────────
pub fn sync_pieces(
    mut commands: Commands,
    board: Res<Board>,
    assets: Res<GameAssets>,
    pieces: Query<Entity, With<PieceCell>>,
    shadows: Query<Entity, With<PieceShadow>>,
    highlights_q: Query<Entity, With<PieceHighlight>>,
) {
    if !board.is_changed() {
        return;
    }
    for e in pieces.iter().chain(shadows.iter()).chain(highlights_q.iter()) {
        commands.entity(e).despawn();
    }
    for row in 0..BOARD_SIZE {
        for col in 0..BOARD_SIZE {
            if let Some(player) = board.cells[row][col] {
                spawn_piece(&mut commands, &assets, row, col, player);
            }
        }
    }
}

// ─── 유효수 힌트 동기화 ──────────────────────────────────────────────────────
pub fn sync_valid_hints(
    mut commands: Commands,
    valid_moves: Res<ValidMoves>,
    assets: Res<GameAssets>,
    hints: Query<Entity, With<ValidHint>>,
) {
    if !valid_moves.is_changed() {
        return;
    }
    for e in &hints {
        commands.entity(e).despawn();
    }
    for &(row, col) in &valid_moves.0 {
        let pos = cell_to_world(row, col);
        commands.spawn((
            Mesh2d(assets.hint_mesh.clone()),
            MeshMaterial2d(assets.hint_mat.clone()),
            Transform::from_xyz(pos.x, pos.y, 1.5),
            ValidHint,
            GameEntity,
        ));
    }
}
