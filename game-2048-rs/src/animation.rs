use macroquad::prelude::*;

/// 하나의 타일 애니메이션
#[derive(Clone)]
pub enum TileAnim {
    /// 기존 타일이 한 위치에서 다른 위치로 이동
    Slide {
        value: u16,
        from_r: usize,
        from_c: usize,
        to_r: usize,
        to_c: usize,
    },
    /// 두 타일이 병합되어 새 값이 됨 (병합 위치에서 스케일 효과)
    Merge {
        value: u16,
        r: usize,
        c: usize,
    },
    /// 새 타일 등장 (스케일 0→1)
    Spawn {
        value: u16,
        r: usize,
        c: usize,
    },
}

pub struct AnimationState {
    pub anims: Vec<TileAnim>,
    pub timer: f32,
    pub duration: f32,
    pub spawned_value: Option<(u16, usize, usize)>,
    /// 애니메이션 시작 전 보드 상태 (슬라이드 중 정적 타일 그리기용)
    pub prev_cells: Option<[[u16; 4]; 4]>,
}

impl AnimationState {
    pub fn new() -> Self {
        AnimationState {
            anims: Vec::new(),
            timer: 0.0,
            duration: 0.3, // 300ms
            spawned_value: None,
            prev_cells: None,
        }
    }

    pub fn is_active(&self) -> bool {
        self.timer < self.duration
    }

    pub fn start(&mut self, anims: Vec<TileAnim>, spawned: Option<(u16, usize, usize)>, prev_cells: [[u16; 4]; 4]) {
        self.anims = anims;
        self.spawned_value = spawned;
        self.prev_cells = Some(prev_cells);
        self.timer = 0.0;
    }

    pub fn update(&mut self, dt: f32) {
        if self.is_active() {
            self.timer += dt;
        }
    }
}

/// 애니메이션 진행률에 따라 타일을 그리기
pub fn draw_animated_tile(
    anim: &TileAnim,
    progress: f32,
    board_x: f32,
    board_y: f32,
    gap: f32,
    tile_size: f32,
    tile_color_fn: fn(u16) -> Color,
    text_color_fn: fn(u16) -> Color,
) {
    match anim {
        TileAnim::Slide { value, from_r, from_c, to_r, to_c } => {
            let fx = board_x + gap + *from_c as f32 * (tile_size + gap);
            let fy = board_y + gap + *from_r as f32 * (tile_size + gap);
            let tx = board_x + gap + *to_c as f32 * (tile_size + gap);
            let ty = board_y + gap + *to_r as f32 * (tile_size + gap);

            let x = fx + (tx - fx) * progress;
            let y = fy + (ty - fy) * progress;

            draw_tile_at(x, y, tile_size, *value, tile_color_fn, text_color_fn);
        }
        TileAnim::Merge { value, r, c } => {
            let x = board_x + gap + *c as f32 * (tile_size + gap);
            let y = board_y + gap + *r as f32 * (tile_size + gap);

            // 병합 시 살짝 커졌다가 줄어드는 효과
            let scale = if progress < 0.5 {
                1.0 + 0.15 * (progress / 0.5)
            } else {
                1.0 + 0.15 * (1.0 - (progress - 0.5) / 0.5)
            };

            let offset = (tile_size * (scale - 1.0)) / 2.0;
            draw_tile_at(x - offset, y - offset, tile_size * scale, *value, tile_color_fn, text_color_fn);
        }
        TileAnim::Spawn { value, r, c } => {
            let x = board_x + gap + *c as f32 * (tile_size + gap);
            let y = board_y + gap + *r as f32 * (tile_size + gap);

            // 0에서 1로 스케일 (ease-out, 약간 오버슈트)
            let scale = if progress < 0.6 {
                // 0 → 1.15 (빠르게 커짐)
                (progress / 0.6) * 1.15
            } else {
                // 1.15 → 1.0 (살짝 줄어듦)
                1.15 - 0.15 * ((progress - 0.6) / 0.4)
            };

            let offset = (tile_size * (scale - 1.0)) / 2.0;
            draw_tile_at(x - offset, y - offset, tile_size * scale, *value, tile_color_fn, text_color_fn);
        }
    }
}

fn draw_tile_at(
    x: f32,
    y: f32,
    size: f32,
    value: u16,
    tile_color_fn: fn(u16) -> Color,
    text_color_fn: fn(u16) -> Color,
) {
    draw_rectangle(x, y, size, size, tile_color_fn(value));

    if value != 0 {
        let text = value.to_string();
        let font_size = if value < 100 {
            36.0
        } else if value < 1000 {
            30.0
        } else {
            24.0
        };
        let text_w = measure_text(&text, None, font_size as u16, 1.0).width;
        draw_text(
            &text,
            x + (size - text_w) / 2.0,
            y + size / 2.0 + font_size / 3.0,
            font_size,
            text_color_fn(value),
        );
    }
}
