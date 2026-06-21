//! 체스 게임 플레이(보드/기물/입력/HUD) 플러그인.

pub mod rules;
mod ai;
mod play;
mod setup;

use bevy::prelude::*;

use crate::{AppState, GameFont};
use rules::{initial_squares, ChessMove, Color as PieceColor, Kind, Square, Squares, Status};

/// 한 칸의 픽셀 크기.
pub const TILE: f32 = 80.0;

/// 카메라를 오른쪽으로 옮기는 양(보드가 화면 왼쪽으로 이동).
pub const CAMERA_X: f32 = 180.0;

// z 레이어. 반투명 사각형이 같은 z 에서 겹치면 정렬이 불안정해 깜빡이므로,
// 레이어마다 서로 다른 z 값을 충분한 간격으로 둔다.
pub const Z_TILE: f32 = 0.0;
pub const Z_LAST_MOVE: f32 = 1.0;
pub const Z_CHECK: f32 = 1.4;
pub const Z_SELECT: f32 = 1.8;
pub const Z_PIECE: f32 = 5.0;
pub const Z_MOVE_HINT: f32 = 10.0;
/// 슬라이드 중인 기물은 모든 요소 위에 그린다.
pub const Z_PIECE_MOVING: f32 = 12.0;

pub const LIGHT_SQUARE: Color = Color::srgb(0.93, 0.93, 0.82);
pub const DARK_SQUARE: Color = Color::srgb(0.46, 0.59, 0.34);

/// 현재 대국 모드.
#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameMode {
    /// 같은 화면에서 두 사람이 번갈아 두는 모드.
    LocalVersus,
    /// 플레이어(백) vs AI(흑). `search_depth` 는 미니맥스 탐색 깊이.
    AiVersus { search_depth: u8 },
}

impl GameMode {
    pub fn is_ai_versus(self) -> bool {
        matches!(self, GameMode::AiVersus { .. })
    }
}

/// AI 차례 처리 상태(생각 중 표시·짧은 대기).
#[derive(Resource)]
pub struct AiTurnState {
    pub thinking: bool,
    pub delay: Timer,
}

impl Default for AiTurnState {
    fn default() -> Self {
        Self {
            thinking: false,
            delay: Timer::from_seconds(0.35, TimerMode::Once),
        }
    }
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

    /// 현재 상태의 스냅샷(되돌리기용).
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            squares: self.squares,
            turn: self.turn,
            en_passant: self.en_passant,
            status: self.status,
            last_move: self.last_move,
        }
    }

    /// 스냅샷으로 복원한다.
    pub fn restore(&mut self, snap: Snapshot) {
        self.squares = snap.squares;
        self.turn = snap.turn;
        self.en_passant = snap.en_passant;
        self.status = snap.status;
        self.last_move = snap.last_move;
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

/// 되돌리기를 위한 보드 스냅샷(한 수 두기 전 상태).
#[derive(Clone, Copy)]
pub struct Snapshot {
    pub squares: Squares,
    pub turn: PieceColor,
    pub en_passant: Option<Square>,
    pub status: Status,
    pub last_move: Option<ChessMove>,
}

/// 이동 기록(스냅샷 스택 + 기보 문자열). 둘은 항상 같은 길이를 유지한다.
#[derive(Resource, Default)]
pub struct MoveHistory {
    pub snapshots: Vec<Snapshot>,
    pub sans: Vec<String>,
}

impl MoveHistory {
    fn clear(&mut self) {
        self.snapshots.clear();
        self.sans.clear();
    }
}

/// 보드 시각 요소를 다시 그려야 함을 알리는 플래그.
#[derive(Resource, Default)]
pub struct BoardDirty(pub bool);

/// 기물 텍스처와 이동 표시용(원형 점/링) 텍스처 핸들 모음.
/// 이동 표시도 스프라이트로 통일해, Mesh2d 와 스프라이트가 섞이며 생기는
/// 정렬 불안정(깜빡임)을 방지한다.
#[derive(Resource)]
pub struct ChessAssets {
    pieces: [[Handle<Image>; 6]; 2],
    /// 빈 칸 이동 표시용 채워진 원.
    pub move_dot: Handle<Image>,
    /// 캡처 가능 칸 표시용 링.
    pub capture_ring: Handle<Image>,
}

impl ChessAssets {
    /// 기물 이미지와 이동 표시 텍스처를 준비한다.
    pub fn load(asset_server: &AssetServer, images: &mut Assets<Image>) -> Self {
        let load_for = |color: PieceColor| {
            [
                Kind::Pawn,
                Kind::Knight,
                Kind::Bishop,
                Kind::Rook,
                Kind::Queen,
                Kind::King,
            ]
            .map(|kind| asset_server.load(piece_asset_path(color, kind)))
        };
        Self {
            pieces: [load_for(PieceColor::White), load_for(PieceColor::Black)],
            move_dot: images.add(make_disc_texture()),
            capture_ring: images.add(make_ring_texture()),
        }
    }

    /// 특정 색/종류 기물의 텍스처 핸들.
    pub fn image(&self, color: PieceColor, kind: Kind) -> Handle<Image> {
        self.pieces[color.index()][kind.index()].clone()
    }
}

/// 이동 표시 색(반투명 초록).
const HINT_RGB: [f32; 3] = [0.18, 0.55, 0.25];
const HINT_ALPHA: f32 = 0.65;

/// 안티에일리어싱된 채워진 원 텍스처를 만든다.
fn make_disc_texture() -> Image {
    build_texture(96, |dist, outer| {
        // 가장자리 1.5px 페이드.
        ((outer - dist) / 1.5).clamp(0.0, 1.0)
    })
}

/// 안티에일리어싱된 링(원형 테두리) 텍스처를 만든다.
fn make_ring_texture() -> Image {
    let size = 128.0;
    let outer = size * 0.5 - 2.0;
    let inner = size * 0.5 * 0.78;
    build_texture(128, move |dist, edge| {
        let outer_a = ((edge - dist) / 1.5).clamp(0.0, 1.0);
        let inner_a = ((dist - inner) / 1.5).clamp(0.0, 1.0);
        let _ = outer;
        outer_a.min(inner_a)
    })
}

/// 중심 기준 거리(dist)와 바깥 반지름(outer)을 받아 알파(0~1)를 돌려주는
/// 함수로 RGBA8 텍스처를 생성한다.
fn build_texture(size: u32, coverage: impl Fn(f32, f32) -> f32) -> Image {
    use bevy::asset::RenderAssetUsages;
    use bevy::image::{Image, ImageSampler};
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

    let n = size as usize;
    let center = (size as f32 - 1.0) * 0.5;
    let outer = size as f32 * 0.5 - 2.0;
    let mut data = vec![0u8; n * n * 4];

    for y in 0..n {
        for x in 0..n {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let dist = (dx * dx + dy * dy).sqrt();
            let cov = coverage(dist, outer).clamp(0.0, 1.0);
            let a = (cov * HINT_ALPHA * 255.0) as u8;
            let i = (y * n + x) * 4;
            data[i] = (HINT_RGB[0] * 255.0) as u8;
            data[i + 1] = (HINT_RGB[1] * 255.0) as u8;
            data[i + 2] = (HINT_RGB[2] * 255.0) as u8;
            data[i + 3] = a;
        }
    }

    let mut image = Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    image.sampler = ImageSampler::linear();
    image
}

/// 기물 이미지 파일 경로(예: `pieces/wK.png`).
fn piece_asset_path(color: PieceColor, kind: Kind) -> String {
    format!("pieces/{}{}.png", color.prefix(), kind.letter())
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

/// 오른쪽 패널의 이동 기록 텍스트.
#[derive(Component)]
pub struct HistoryText;

pub struct ChessPlugin;

impl Plugin for ChessPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Board::fresh())
            .insert_resource(GameMode::LocalVersus)
            .init_resource::<Selection>()
            .init_resource::<BoardDirty>()
            .init_resource::<MoveHistory>()
            .init_resource::<AiTurnState>()
            .init_resource::<play::MoveAnimations>()
            .add_systems(
                OnEnter(AppState::InGame),
                (setup::setup_game, setup::setup_hud, reset_ai_turn_state).chain(),
            )
            .add_systems(
                Update,
                (
                    play::handle_back_button,
                    play::handle_undo_button,
                    play::handle_input,
                    play::ai_turn,
                    play::handle_keys,
                    play::redraw,
                    play::animate_pieces,
                    play::update_hud,
                    play::update_history,
                )
                    .chain()
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(OnExit(AppState::InGame), teardown);
    }
}

/// AI 대국 진입 시 AI 턴 상태를 초기화한다.
fn reset_ai_turn_state(mut ai_state: ResMut<AiTurnState>) {
    *ai_state = AiTurnState::default();
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
