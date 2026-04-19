use bevy::prelude::*;
use rand::Rng;

use crate::components::*;
use crate::systems::{animation, input, movement, render, ui};



/// 이동 결과 추적
#[derive(Debug, Clone)]
pub struct MoveResult {
    /// (이전 행, 이전 열, 새 행, 새 열, 값) - 이동만 한 타일
    pub moved: Vec<(usize, usize, usize, usize, u32)>,
    /// (소스1 행, 소스1 열, 소스2 행, 소스2 열, 새 행, 새 열, 새 값) - 병합된 타일
    pub merged: Vec<(usize, usize, usize, usize, usize, usize, u32)>,
    /// (행, 열, 값) - 새로 생성된 타일
    pub spawned: Vec<(usize, usize, u32)>,
}

impl MoveResult {
    pub fn new() -> Self {
        Self {
            moved: Vec::new(),
            merged: Vec::new(),
            spawned: Vec::new(),
        }
    }
}

/// 게임 상태 리소스
#[derive(Resource, Debug)]
pub struct GameState {
    /// 4x4 그리드 (None = 빈 칸, Some(value) = 타일 값)
    pub grid: [[Option<u32>; 4]; 4],
    pub score: u32,
    pub game_over: bool,
    pub needs_sync: bool,
    pub last_move: Option<MoveResult>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            grid: [[None; 4]; 4],
            score: 0,
            game_over: false,
            needs_sync: false,
            last_move: None,
        }
    }
}

impl GameState {
    /// 빈 칸 목록 반환
    pub fn empty_cells(&self) -> Vec<(usize, usize)> {
        let mut cells = Vec::new();
        for row in 0..4 {
            for col in 0..4 {
                if self.grid[row][col].is_none() {
                    cells.push((row, col));
                }
            }
        }
        cells
    }

    /// 새 타일 생성 (90% 확률로 2, 10%로 4)
    pub fn spawn_random_tile(&mut self) -> Option<(usize, usize, u32)> {
        let empty = self.empty_cells();
        if empty.is_empty() {
            return None;
        }
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0..empty.len());
        let (row, col) = empty[idx];
        let value = if rng.gen_bool(0.9) { 2 } else { 4 };
        self.grid[row][col] = Some(value);
        Some((row, col, value))
    }

    /// 게임오버 판정 (이동 불가능한 상태)
    pub fn check_game_over(&self) -> bool {
        // 빈 칸이 있으면 게임오버 아님
        if !self.empty_cells().is_empty() {
            return false;
        }
        // 인접한 셀과 합칠 수 있는지 확인
        for row in 0..4 {
            for col in 0..4 {
                let val = self.grid[row][col].unwrap();
                // 오른쪽 확인
                if col < 3 && self.grid[row][col + 1] == Some(val) {
                    return false;
                }
                // 아래 확인
                if row < 3 && self.grid[row + 1][col] == Some(val) {
                    return false;
                }
            }
        }
        true
    }

    /// 게임 리셋
    pub fn reset(&mut self) {
        self.grid = [[None; 4]; 4];
        self.score = 0;
        self.game_over = false;
        self.needs_sync = false;
        self.last_move = None;
    }
}

/// 게임 이벤트
#[derive(Event)]
pub struct MoveEvent(pub Direction);

#[derive(Event)]
pub struct GameOverEvent;

#[derive(Event)]
pub struct RestartEvent;

/// 게임 플러그인
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameState>()
            .add_event::<MoveEvent>()
            .add_event::<GameOverEvent>()
            .add_event::<RestartEvent>()
            .add_systems(Startup, (render::setup_camera, render::setup_grid, ui::setup_score_ui, render::spawn_initial_tiles))
            .init_resource::<animation::AnimationConfig>()
            .add_systems(
                Update,
                (
                    input::handle_input,
                    movement::process_movement,
                    render::sync_tiles,
                    animation::animate_movement,
                    animation::animate_spawn,
                    animation::update_animations,
                    ui::update_score,
                    ui::handle_game_over,
                    ui::handle_restart,
                ),
            );
    }
}
