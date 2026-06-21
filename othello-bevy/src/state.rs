use bevy::prelude::*;

// ─── 한글 폰트 핸들 ──────────────────────────────────────────────────────────
#[derive(Resource, Clone)]
pub struct KoreanFont(pub Handle<Font>);

impl FromWorld for KoreanFont {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        KoreanFont(asset_server.load("fonts/AppleGothic.ttf"))
    }
}

// ─── 게임 상태 ───────────────────────────────────────────────────────────────
#[derive(States, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    DifficultySelect,
    Playing,
    GameOver,
}

// ─── 게임 모드 ───────────────────────────────────────────────────────────────
#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameMode {
    PvP,
    PvAI,
}

// ─── 플레이어 / 돌 색상 ──────────────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Player {
    Black,
    White,
}

impl Player {
    pub fn opponent(self) -> Self {
        match self {
            Player::Black => Player::White,
            Player::White => Player::Black,
        }
    }
}

// ─── 현재 턴 ─────────────────────────────────────────────────────────────────
#[derive(Resource, Debug)]
pub struct CurrentTurn(pub Player);

// ─── 게임 종료 결과 ──────────────────────────────────────────────────────────
#[derive(Resource, Debug)]
pub struct GameResult {
    pub black_score: u32,
    pub white_score: u32,
}

// ─── 마커: 게임 화면 엔티티 정리용 ──────────────────────────────────────────
#[derive(Component)]
pub struct GameEntity;

// ─── 마커: 메뉴 엔티티 정리용 ───────────────────────────────────────────────
#[derive(Component)]
pub struct MenuEntity;

// ─── AI 난이도 ────────────────────────────────────────────────────────────────
#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AiDifficulty {
    Easy,
    #[default]
    Normal,
    Hard,
}

impl AiDifficulty {
    pub fn depth(self) -> u8 {
        match self {
            AiDifficulty::Easy => 2,
            AiDifficulty::Normal => 4,
            AiDifficulty::Hard => 6,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            AiDifficulty::Easy => "쉬움",
            AiDifficulty::Normal => "보통",
            AiDifficulty::Hard => "어려움",
        }
    }
}
