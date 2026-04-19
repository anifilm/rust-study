use bevy::prelude::*;

/// 타일의 값 (2, 4, 8, ..., 2048)
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileValue(pub u32);

/// 타일의 그리드 위치 (행, 열) - 0-indexed
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridPosition {
    pub row: usize,
    pub col: usize,
}

/// 타일 엔티티 마커
#[derive(Component)]
pub struct Tile;

/// 게임 보드 배경 셀 마커
#[derive(Component)]
pub struct GridCell;

/// 점수 UI 텍스트 마커
#[derive(Component)]
pub struct ScoreText;

/// 게임오버 오버레이 마커
#[derive(Component)]
pub struct GameOverOverlay;

/// 병합 소스 타일 마커 (애니메이션 후 제거)
#[derive(Component)]
pub struct MergeSource;

/// 병합 결과 타일 마커 (애니메이션 후 표시)
#[derive(Component)]
pub struct MergeTarget;

/// 이동 방향
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl GridPosition {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    /// 그리드 좌표를 화면 좌표로 변환
    /// 주의: 그리드에서 row=0은 맨 위 행이지만, 화면에서는 Y축이 위로 증가하므로 반전 필요
    pub fn to_world_pos(&self) -> Vec2 {
        let cell_size = 140.0;
        let gap = 10.0;
        let board_size = cell_size * 4.0 + gap * 5.0;
        let offset = -board_size / 2.0 + gap + cell_size / 2.0;

        Vec2::new(
            offset + self.col as f32 * (cell_size + gap),
            offset + (3 - self.row) as f32 * (cell_size + gap), // row 반전
        )
    }
}

impl TileValue {
    /// 값에 따른 타일 색상 반환
    pub fn color(&self) -> Color {
        match self.0 {
            2 => Color::srgb(0.93, 0.89, 0.85),
            4 => Color::srgb(0.93, 0.88, 0.78),
            8 => Color::srgb(0.95, 0.69, 0.47),
            16 => Color::srgb(0.96, 0.58, 0.39),
            32 => Color::srgb(0.96, 0.49, 0.37),
            64 => Color::srgb(0.96, 0.37, 0.23),
            128 => Color::srgb(0.93, 0.81, 0.45),
            256 => Color::srgb(0.93, 0.80, 0.38),
            512 => Color::srgb(0.93, 0.78, 0.31),
            1024 => Color::srgb(0.93, 0.77, 0.25),
            2048 => Color::srgb(0.93, 0.76, 0.18),
            _ => Color::srgb(0.20, 0.20, 0.20),
        }
    }

    /// 값에 따른 텍스트 색상 반환
    pub fn text_color(&self) -> Color {
        match self.0 {
            2 | 4 => Color::srgb(0.47, 0.43, 0.40),
            _ => Color::srgb(1.0, 1.0, 1.0),
        }
    }
}
