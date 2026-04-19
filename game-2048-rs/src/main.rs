mod animation;
mod board;

use animation::{AnimationState, TileAnim, draw_animated_tile};
use board::{Board, SlideInfo, SIZE};
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "2048".to_owned(),
        window_width: 500,
        window_height: 660,
        ..Default::default()
    }
}

fn tile_color(value: u16) -> Color {
    match value {
        0 => Color::from_hex(0xcdc1b4),
        2 => Color::from_hex(0xeee4da),
        4 => Color::from_hex(0xede0c8),
        8 => Color::from_hex(0xf2b179),
        16 => Color::from_hex(0xf59563),
        32 => Color::from_hex(0xf67c5f),
        64 => Color::from_hex(0xf65e3b),
        128 => Color::from_hex(0xedcf72),
        256 => Color::from_hex(0xedcc61),
        512 => Color::from_hex(0xedc850),
        1024 => Color::from_hex(0xedc53f),
        2048 => Color::from_hex(0xedc22e),
        _ => Color::from_hex(0x3c3a32),
    }
}

fn text_color(value: u16) -> Color {
    if value <= 4 {
        Color::from_hex(0x776e65)
    } else {
        WHITE
    }
}

/// SlideInfo를 TileAnim 목록으로 변환
fn build_anims(info: &SlideInfo) -> Vec<TileAnim> {
    let mut anims = Vec::new();

    // 슬라이드 애니메이션
    for &(value, from_r, from_c, to_r, to_c) in &info.slides {
        anims.push(TileAnim::Slide {
            value,
            from_r,
            from_c,
            to_r,
            to_c,
        });
    }

    // 병합 애니메이션
    for &(value, r, c) in &info.merges {
        anims.push(TileAnim::Merge { value, r, c });
    }

    anims
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut board = Board::new();
    let mut game_over = false;
    let mut won = false;
    let mut keep_playing = false;
    let mut best_score: u32 = 0;
    let mut anim_state = AnimationState::new();

    loop {
        let dt = get_frame_time();

        // 배경색 (오리지널 2048 느낌)
        clear_background(Color::from_hex(0xfaf8ef));

        // 보드 레이아웃
        let board_size = 400.0;
        let gap = 10.0;
        let tile_size = (board_size - gap * (SIZE as f32 + 1.0)) / SIZE as f32;
        let board_x = (screen_width() - board_size) / 2.0;
        let board_y = 120.0;

        // ── 헤더 영역 ──

        // 제목
        draw_text("2048", board_x, 55.0, 52.0, Color::from_hex(0x776e65));

        // 점수 박스
        let score_box_w = 90.0;
        let score_box_h = 50.0;
        let score_box_x = board_x + board_size - score_box_w * 2.0 - 10.0;
        let score_box_y = 15.0;

        // SCORE 박스
        draw_rectangle(score_box_x, score_box_y, score_box_w, score_box_h, Color::from_hex(0xbbada0));
        draw_text("SCORE", score_box_x + 8.0, score_box_y + 14.0, 12.0, Color::from_hex(0xeee4da));
        let score_text = board.score.to_string();
        let score_w = measure_text(&score_text, None, 20u16, 1.0).width;
        draw_text(&score_text, score_box_x + (score_box_w - score_w) / 2.0, score_box_y + 40.0, 20.0, WHITE);

        // BEST 박스
        let best_box_x = score_box_x + score_box_w + 10.0;
        draw_rectangle(best_box_x, score_box_y, score_box_w, score_box_h, Color::from_hex(0xbbada0));
        draw_text("BEST", best_box_x + 8.0, score_box_y + 14.0, 12.0, Color::from_hex(0xeee4da));
        let best_text = best_score.to_string();
        let best_w = measure_text(&best_text, None, 20u16, 1.0).width;
        draw_text(&best_text, best_box_x + (score_box_w - best_w) / 2.0, score_box_y + 40.0, 20.0, WHITE);

        // 부제목
        draw_text("Join the tiles, get to 2048!", board_x, 95.0, 16.0, Color::from_hex(0x776e65));

        // ── 보드 ──

        // 보드 배경
        draw_rectangle(board_x, board_y, board_size, board_size, Color::from_hex(0xbbada0));

        // 빈 타일 슬롯 그리기
        for r in 0..SIZE {
            for c in 0..SIZE {
                let x = board_x + gap + c as f32 * (tile_size + gap);
                let y = board_y + gap + r as f32 * (tile_size + gap);
                draw_rectangle(x, y, tile_size, tile_size, tile_color(0));
            }
        }

        if anim_state.is_active() {
            // 애니메이션 재생 중
            anim_state.update(dt);
            let t = (anim_state.timer / anim_state.duration).min(1.0); // raw 0~1

            // 타이밍 분할
            let slide_end = 0.5;   // 0~50%: 슬라이드
            let merge_end = 0.8;   // 50~80%: 병합
            let spawn_start = 0.7; // 70~100%: 스폰

            // 현재 프레임에서 애니메이션이 점유하는 위치만 수집
            let mut active_positions = std::collections::HashSet::new();
            if t < slide_end {
                // 슬라이드 중: 출발지와 도착지 모두 차단 (슬라이드 애니메이션이 그림)
                for anim in &anim_state.anims {
                    if let TileAnim::Slide { from_r, from_c, to_r, to_c, .. } = anim {
                        active_positions.insert((*from_r, *from_c));
                        active_positions.insert((*to_r, *to_c));
                    }
                }
            }
            if t >= slide_end && t < merge_end {
                // 병합 중: 병합 위치만 차단
                for anim in &anim_state.anims {
                    if let TileAnim::Merge { r, c, .. } = anim {
                        active_positions.insert((*r, *c));
                    }
                }
            }
            // 스폰 위치는 애니메이션 전체 기간 동안 항상 차단 (정적 그리기 방지)
            if let Some((_, r, c)) = anim_state.spawned_value {
                active_positions.insert((r, c));
            }

            // 정적 타일 그리기
            // 슬라이드 중: 이전 보드 상태 기준 (병합 전 값)
            // 병합/스폰 중: 최종 보드 상태 기준
            let ref_cells = if t < slide_end {
                anim_state.prev_cells.as_ref().unwrap_or(&board.cells)
            } else {
                &board.cells
            };

            for r in 0..SIZE {
                for c in 0..SIZE {
                    if ref_cells[r][c] != 0 && !active_positions.contains(&(r, c)) {
                        let x = board_x + gap + c as f32 * (tile_size + gap);
                        let y = board_y + gap + r as f32 * (tile_size + gap);

                        draw_rectangle(x, y, tile_size, tile_size, tile_color(ref_cells[r][c]));

                        let text = ref_cells[r][c].to_string();
                        let font_size = if ref_cells[r][c] < 100 {
                            36.0
                        } else if ref_cells[r][c] < 1000 {
                            30.0
                        } else {
                            24.0
                        };
                        let text_w = measure_text(&text, None, font_size as u16, 1.0).width;
                        draw_text(
                            &text,
                            x + (tile_size - text_w) / 2.0,
                            y + tile_size / 2.0 + font_size / 3.0,
                            font_size,
                            text_color(ref_cells[r][c]),
                        );
                    }
                }
            }

            // 1) 슬라이드 애니메이션 (0 ~ slide_end)
            if t < slide_end {
                let slide_progress = t / slide_end;
                let eased = 1.0 - (1.0 - slide_progress) * (1.0 - slide_progress);
                for anim in &anim_state.anims {
                    if let TileAnim::Slide { .. } = anim {
                        draw_animated_tile(anim, eased, board_x, board_y, gap, tile_size, tile_color, text_color);
                    }
                }
            }

            // 2) 병합 애니메이션 (slide_end ~ merge_end)
            if t >= slide_end && t < merge_end {
                let merge_progress = (t - slide_end) / (merge_end - slide_end);
                for anim in &anim_state.anims {
                    if let TileAnim::Merge { .. } = anim {
                        draw_animated_tile(anim, merge_progress, board_x, board_y, gap, tile_size, tile_color, text_color);
                    }
                }
            }

            // 3) 새 타일 등장 애니메이션 (spawn_start ~ 100%)
            if let Some((value, r, c)) = anim_state.spawned_value {
                if t >= spawn_start {
                    let spawn_progress = ((t - spawn_start) / (1.0 - spawn_start)).min(1.0);
                    draw_animated_tile(
                        &TileAnim::Spawn { value, r, c },
                        spawn_progress,
                        board_x, board_y, gap, tile_size, tile_color, text_color,
                    );
                }
            }
        } else {
            // 정적 타일 그리기
            for r in 0..SIZE {
                for c in 0..SIZE {
                    let value = board.cells[r][c];
                    if value != 0 {
                        let x = board_x + gap + c as f32 * (tile_size + gap);
                        let y = board_y + gap + r as f32 * (tile_size + gap);

                        draw_rectangle(x, y, tile_size, tile_size, tile_color(value));

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
                            x + (tile_size - text_w) / 2.0,
                            y + tile_size / 2.0 + font_size / 3.0,
                            font_size,
                            text_color(value),
                        );
                    }
                }
            }

            // 입력 처리 (애니메이션이 끝난 후에만)
            if !game_over && !(won && !keep_playing) {
                let result: Option<(bool, SlideInfo)> = if is_key_pressed(KeyCode::Left) {
                    Some(board.slide_left())
                } else if is_key_pressed(KeyCode::Right) {
                    Some(board.slide_right())
                } else if is_key_pressed(KeyCode::Up) {
                    Some(board.slide_up())
                } else if is_key_pressed(KeyCode::Down) {
                    Some(board.slide_down())
                } else {
                    None
                };

                if let Some((moved, info)) = result {
                    if moved {
                        // 애니메이션 시작 전 보드 상태 저장
                        let prev_cells = board.cells;

                        let anims = build_anims(&info);

                        // 새 타일 추가
                        board.add_random_tile();
                        let spawned = find_new_tile(&board, &info);

                        anim_state.start(anims, spawned, prev_cells);

                        // 베스트 스코어 갱신
                        if board.score > best_score {
                            best_score = board.score;
                        }

                        if board.has_won() && !keep_playing {
                            won = true;
                        } else if !board.can_move() {
                            game_over = true;
                        }
                    }
                }
            }
        }

        // ── 오버레이 메시지 ──

        // 게임오버 표시
        if game_over && !anim_state.is_active() {
            draw_rectangle(board_x, board_y, board_size, board_size, Color { r: 0.93, g: 0.89, b: 0.85, a: 0.7 });
            let text = "Game Over!";
            let font_size = 40.0;
            let text_w = measure_text(text, None, font_size as u16, 1.0).width;
            draw_text(
                text,
                board_x + (board_size - text_w) / 2.0,
                board_y + board_size / 2.0 - 10.0,
                font_size,
                Color::from_hex(0x776e65),
            );
            let hint = "Press R to restart";
            let hint_w = measure_text(hint, None, 18u16, 1.0).width;
            draw_text(
                hint,
                board_x + (board_size - hint_w) / 2.0,
                board_y + board_size / 2.0 + 30.0,
                18.0,
                Color::from_hex(0x776e65),
            );
        }

        // 승리 표시
        if won && !keep_playing && !anim_state.is_active() {
            draw_rectangle(board_x, board_y, board_size, board_size, Color { r: 0.93, g: 0.89, b: 0.85, a: 0.7 });
            let text = "You Win!";
            let font_size = 40.0;
            let text_w = measure_text(text, None, font_size as u16, 1.0).width;
            draw_text(
                text,
                board_x + (board_size - text_w) / 2.0,
                board_y + board_size / 2.0 - 20.0,
                font_size,
                Color::from_hex(0x776e65),
            );
            let hint = "Press C to continue, R to restart";
            let hint_w = measure_text(hint, None, 16u16, 1.0).width;
            draw_text(
                hint,
                board_x + (board_size - hint_w) / 2.0,
                board_y + board_size / 2.0 + 25.0,
                16.0,
                Color::from_hex(0x776e65),
            );
        }

        // ── 하단 안내 ──
        let footer = "Arrow keys to move  |  R to restart";
        let footer_w = measure_text(footer, None, 14u16, 1.0).width;
        draw_text(
            footer,
            (screen_width() - footer_w) / 2.0,
            board_y + board_size + 25.0,
            14.0,
            Color::from_hex(0x776e65),
        );

        // ── R 키: 재시작 ──
        if is_key_pressed(KeyCode::R) {
            board = Board::new();
            game_over = false;
            won = false;
            keep_playing = false;
            anim_state = AnimationState::new();
        }

        // ── C 키: 승리 후 계속 플레이 ──
        if is_key_pressed(KeyCode::C) && won && !keep_playing {
            keep_playing = true;
        }

        next_frame().await
    }
}

/// 슬라이드 후 새로 생성된 타일 위치 찾기
fn find_new_tile(board: &Board, info: &SlideInfo) -> Option<(u16, usize, usize)> {
    // 병합/도착 위치가 아닌 빈 칸이 아닌 셀 중, 이전에 비어있던 곳
    let mut occupied = std::collections::HashSet::new();
    for &(_, _, _, tr, tc) in &info.slides {
        occupied.insert((tr, tc));
    }
    for &(_, r, c) in &info.merges {
        occupied.insert((r, c));
    }

    for r in 0..SIZE {
        for c in 0..SIZE {
            if board.cells[r][c] != 0 && !occupied.contains(&(r, c)) {
                return Some((board.cells[r][c], r, c));
            }
        }
    }
    None
}
