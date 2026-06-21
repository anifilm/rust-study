//! 체스 게임 플레이(보드/기물/입력/HUD) 플러그인.

pub mod rules;
mod play;
mod setup;

use bevy::prelude::*;

use crate::{AppState, GameFont};
use rules::{initial_squares, ChessMove, Color as PieceColor, Square, Squares, Status};

/// 한 칸의 픽셀 크기.
pub const TILE: f32 = 80.0;

// z 레이어: 보드 < 하이라이트 < 기물 < 글자.
pub const Z_TILE: f32 = 0.0;
pub const Z_HIGHLIGHT: f32 = 1.0;
pub const Z_PIECE: f32 = 2.0;
pub const Z_LETTER: f32 = 3.0;

pub const LIGHT_SQUARE: Color = Color::srgb(0.93, 0.93, 0.82);
pub const DARK_SQUARE: Color = Color::srgb(0.46, 0.59, 0.34);

/// 현재 대국 모드. 지금은 1:1(로컬) 만 실제로 동작한다.
#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameMode {
    /// 같은 화면에서 두 사람이 번갈아 두는 모드.
    LocalVersus,
}

/// 논리적인 보드 상태(렌더링과 무관한 진실의 원천).
#[derive(Resource)]
pub struct Board {
    pub squares: Squares,
    pub turn: PieceColor,
    pub en_passant: Option<Square>,
    pub status: Status,
    pub last_move: Option<ChessMove>,
}

impl Board {
    fn fresh() -> Self {
        Self {
            squares: initial_squares(),
            turn: PieceColor::White,
            en_passant: None,
            status: Status::Ongoing,
            last_move: None,
        }
    }

    /// 새 대국을 위해 초기 배치로 되돌린다.
    pub fn reset(&mut self) {
        *self = Self::fresh();
    }

    /// 게임이 끝났는지(체크메이트/스테일메이트).
    pub fn is_over(&self) -> bool {
        matches!(
            self.status,
            Status::Checkmate { .. } | Status::Stalemate
        )
    }
}

/// 현재 선택된 기물과 그 합법 수 목록.
#[derive(Resource, Default)]
pub struct Selection {
    pub from: Option<Square>,
    pub moves: Vec<ChessMove>,
}

impl Selection {
    fn clear(&mut self) {
        self.from = None;
        self.moves.clear();
    }
}

/// 보드 시각 요소를 다시 그려야 함을 알리는 플래그.
#[derive(Resource, Default)]
pub struct BoardDirty(pub bool);

/// 매 수마다 새로 만들지 않도록 캐싱해 두는 기물용 메시/머티리얼 핸들.
#[derive(Resource)]
pub struct ChessAssets {
    pub disc: Handle<Mesh>,
    pub white_disc: Handle<ColorMaterial>,
    pub black_disc: Handle<ColorMaterial>,
}

/// InGame 상태에서 생성된 모든 엔티티(나갈 때 일괄 제거).
#[derive(Component)]
pub struct OnGameScreen;

/// 기물 시각 엔티티.
#[derive(Component)]
pub struct PieceVisual;

/// 선택/이동/체크 하이라이트 엔티티.
#[derive(Component)]
pub struct HighlightVisual;

/// 상단 상태 텍스트(턴/체크/게임 종료).
#[derive(Component)]
pub struct StatusText;

pub struct ChessPlugin;

impl Plugin for ChessPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Board::fresh())
            .init_resource::<Selection>()
            .init_resource::<BoardDirty>()
            .add_systems(
                OnEnter(AppState::InGame),
                (setup::setup_game, setup::setup_hud).chain(),
            )
            .add_systems(
                Update,
                (
                    play::handle_back_button,
                    play::handle_input,
                    play::redraw,
                    play::update_hud,
                    play::handle_keys,
                )
                    .chain()
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(OnExit(AppState::InGame), teardown);
    }
}

/// InGame 을 떠날 때 모든 게임 엔티티를 정리한다.
fn teardown(mut commands: Commands, entities: Query<Entity, With<OnGameScreen>>) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}

/// 보드 좌표를 월드 좌표(칸 중심)로 변환.
pub fn square_to_world(sq: Square) -> Vec2 {
    Vec2::new(
        (sq.file as f32 - 3.5) * TILE,
        (sq.rank as f32 - 3.5) * TILE,
    )
}

/// 월드 좌표를 보드 좌표로 변환(보드 밖이면 None).
pub fn world_to_square(pos: Vec2) -> Option<Square> {
    let file = (pos.x / TILE + 3.5).round() as i32;
    let rank = (pos.y / TILE + 3.5).round() as i32;
    let sq = Square::new(file, rank);
    sq.in_bounds().then_some(sq)
}

/// 폰트 핸들을 꺼내는 헬퍼(없으면 기본 폰트).
pub fn text_font(font: &GameFont, size: f32) -> TextFont {
    TextFont {
        font: font.0.clone().into(),
        font_size: FontSize::Px(size),
        ..default()
    }
}
