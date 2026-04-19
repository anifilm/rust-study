use bevy::prelude::*;

use crate::components::Direction;
use crate::game::{GameOverEvent, GameState, MoveEvent, MoveResult};

type MoveTile = (usize, usize, usize, usize, u32);
type MergedTile = (usize, usize, usize, usize, usize, usize, u32);

/// 이동 이벤트 처리
pub fn process_movement(
    mut game_state: ResMut<GameState>,
    mut move_events: EventReader<MoveEvent>,
    mut game_over_events: EventWriter<GameOverEvent>,
) {
    for event in move_events.read() {
        if game_state.game_over {
            continue;
        }

        let (moved, result) = match event.0 {
            Direction::Left => move_direction(&mut game_state, Direction::Left),
            Direction::Right => move_direction(&mut game_state, Direction::Right),
            Direction::Up => move_direction(&mut game_state, Direction::Up),
            Direction::Down => move_direction(&mut game_state, Direction::Down),
        };

        if moved {
            let mut result = result;

            // 새 타일 생성
            if let Some((row, col, value)) = game_state.spawn_random_tile() {
                result.spawned.push((row, col, value));
            }

            game_state.last_move = Some(result);
            game_state.needs_sync = true;

            // 게임오버 체크
            if game_state.check_game_over() {
                game_state.game_over = true;
                game_over_events.send(GameOverEvent);
            }
        }
    }
}

/// 방향에 따른 이동 처리
fn move_direction(grid: &mut GameState, dir: Direction) -> (bool, MoveResult) {
    let mut result = MoveResult::new();
    let mut moved = false;

    let is_vertical = matches!(dir, Direction::Up | Direction::Down);
    let reverse = matches!(dir, Direction::Right | Direction::Down);

    // 디버그: 방향 정보 출력
    // println!("move_direction: dir={:?}, is_vertical={}, reverse={}", dir, is_vertical, reverse);

    let count = 4;

    for line_idx in 0..count {
        // 한 줄 추출
        let mut line = [None; 4];
        let mut original_positions = [(0usize, 0usize); 4]; // (row, col)

        for i in 0..4 {
            let (row, col) = if is_vertical {
                (i, line_idx)
            } else {
                (line_idx, i)
            };
            line[i] = grid.grid[row][col];
            original_positions[i] = (row, col);
        }

        // 디버그: 줄 정보 출력
        // println!("line_idx={}, line={:?}, positions={:?}", line_idx, line, original_positions);

        // 왼쪽 방향으로 이동/병합 처리
        let (new_line, moved_tiles, merged_tiles, changed, score_gain) = process_line_left(
            &line,
            &original_positions,
            reverse,
        );

        // 디버그: 결과 출력
        // println!("  new_line={:?}, changed={}, score_gain={}", new_line, changed, score_gain);

        if changed {
            moved = true;
        }
        grid.score += score_gain;

        // 그리드 업데이트
        for i in 0..4 {
            let (row, col) = if is_vertical {
                (i, line_idx)
            } else {
                (line_idx, i)
            };
            grid.grid[row][col] = new_line[i];
        }

        result.moved.extend(moved_tiles);
        result.merged.extend(merged_tiles);
    }

    (moved, result)
}

/// 한 줄을 왼쪽 방향으로 이동/병합 처리
/// reverse=true이면 원래 오른쪽/아래에서 온 것이므로 방향이 반대
/// 반환: (새 줄, 이동 타일, 병합 타일, 변경 여부, 점수 증가량)
fn process_line_left(
    line: &[Option<u32>; 4],
    original_positions: &[(usize, usize); 4],
    reverse: bool,
) -> ([Option<u32>; 4], Vec<MoveTile>, Vec<MergedTile>, bool, u32) {
    let mut new_line = [None; 4];
    let mut moved_tiles = Vec::new();
    let mut merged_tiles = Vec::new();
    let mut score_gain = 0u32;

    let ordered_indices = if reverse {
        [3, 2, 1, 0]
    } else {
        [0, 1, 2, 3]
    };

    let tiles: Vec<(u32, usize, usize)> = ordered_indices
        .into_iter()
        .filter_map(|index| {
            line[index].map(|value| {
                let (row, col) = original_positions[index];
                (value, row, col)
            })
        })
        .collect();

    let mut dest = 0usize;
    let mut tile_index = 0usize;

    while tile_index < tiles.len() {
        let target_index = if reverse { 3 - dest } else { dest };
        let (to_row, to_col) = original_positions[target_index];
        let (value, from_row, from_col) = tiles[tile_index];

        if tile_index + 1 < tiles.len() && value == tiles[tile_index + 1].0 {
            let (_, src2_row, src2_col) = tiles[tile_index + 1];
            let new_value = value * 2;

            new_line[target_index] = Some(new_value);
            merged_tiles.push((
                from_row,
                from_col,
                src2_row,
                src2_col,
                to_row,
                to_col,
                new_value,
            ));
            score_gain += new_value;
            tile_index += 2;
        } else {
            new_line[target_index] = Some(value);

            if from_row != to_row || from_col != to_col {
                moved_tiles.push((from_row, from_col, to_row, to_col, value));
            }
            tile_index += 1;
        }

        dest += 1;
    }

    let changed = line != &new_line;

    (new_line, moved_tiles, merged_tiles, changed, score_gain)
}

#[cfg(test)]
mod tests {
    use super::process_line_left;

    #[test]
    fn merges_two_pairs_when_moving_left() {
        let line = [Some(2), Some(2), Some(2), Some(2)];
        let positions = [(0, 0), (0, 1), (0, 2), (0, 3)];

        let (new_line, moved, merged, changed, score_gain) = process_line_left(&line, &positions, false);

        assert_eq!(new_line, [Some(4), Some(4), None, None]);
        assert!(moved.is_empty());
        assert_eq!(merged.len(), 2);
        assert!(changed);
        assert_eq!(score_gain, 8);
    }

    #[test]
    fn moves_and_merges_to_the_rightmost_slot() {
        let line = [Some(2), None, Some(2), None];
        let positions = [(0, 0), (0, 1), (0, 2), (0, 3)];

        let (new_line, moved, merged, changed, score_gain) = process_line_left(&line, &positions, true);

        assert_eq!(new_line, [None, None, None, Some(4)]);
        assert!(moved.is_empty());
        assert_eq!(merged, vec![(0, 2, 0, 0, 0, 3, 4)]);
        assert!(changed);
        assert_eq!(score_gain, 4);
    }

    #[test]
    fn compresses_tiles_toward_bottom_without_false_merge() {
        let line = [Some(2), None, Some(4), None];
        let positions = [(0, 1), (1, 1), (2, 1), (3, 1)];

        let (new_line, moved, merged, changed, score_gain) = process_line_left(&line, &positions, true);

        assert_eq!(new_line, [None, None, Some(2), Some(4)]);
        assert_eq!(moved, vec![(2, 1, 3, 1, 4), (0, 1, 2, 1, 2)]);
        assert!(merged.is_empty());
        assert!(changed);
        assert_eq!(score_gain, 0);
    }
}
