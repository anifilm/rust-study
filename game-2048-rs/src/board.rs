use rand::seq::SliceRandom;
use rand::Rng;

pub const SIZE: usize = 4;

/// 슬라이드 결과: 이동/병합 애니메이션 정보
#[derive(Clone)]
pub struct SlideInfo {
    /// (value, from_r, from_c, to_r, to_c)
    pub slides: Vec<(u16, usize, usize, usize, usize)>,
    /// (merged_value, r, c)
    pub merges: Vec<(u16, usize, usize)>,
}

#[derive(Clone)]
pub struct Board {
    pub cells: [[u16; SIZE]; SIZE],
    pub score: u32,
}

impl Board {
    pub fn new() -> Self {
        let mut board = Board {
            cells: [[0; SIZE]; SIZE],
            score: 0,
        };
        board.add_random_tile();
        board.add_random_tile();
        board
    }

    /// 빈 칸 중 하나에 2 또는 4를 무작위로 배치
    pub fn add_random_tile(&mut self) {
        let empty: Vec<(usize, usize)> = (0..SIZE)
            .flat_map(|r| (0..SIZE).map(move |c| (r, c)))
            .filter(|&(r, c)| self.cells[r][c] == 0)
            .collect();

        if let Some(&(r, c)) = empty.choose(&mut rand::thread_rng()) {
            self.cells[r][c] = if rand::thread_rng().gen_bool(0.9) { 2 } else { 4 };
        }
    }

    /// 한 행을 왼쪽으로 슬라이드 & 병합, 이동 정보 반환
    fn slide_row(row: &mut [u16; SIZE], row_idx: usize, slides: &mut Vec<(u16, usize, usize, usize, usize)>, merges: &mut Vec<(u16, usize, usize)>, score: &mut u32) -> bool {
        let original = *row;

        // 1) 0 제거 (왼쪽으로 압축) — 각 타일의 원래 열 추적
        let mut compact = [0u16; SIZE];
        let mut src_col = [0usize; SIZE];
        let mut idx = 0;
        for c in 0..SIZE {
            if row[c] != 0 {
                compact[idx] = row[c];
                src_col[idx] = c;
                idx += 1;
            }
        }

        // 2) 인접한 같은 값 병합
        let mut did_merge = [false; SIZE];
        for i in 0..SIZE - 1 {
            if compact[i] != 0 && compact[i] == compact[i + 1] {
                compact[i] *= 2;
                *score += compact[i] as u32;
                compact[i + 1] = 0;
                did_merge[i] = true;
            }
        }

        // 3) 다시 0 제거 — 병합 후 최종 결과
        let mut result = [0u16; SIZE];
        let mut result_src = [0usize; SIZE];
        let mut idx = 0;
        for i in 0..SIZE {
            if compact[i] != 0 {
                result[idx] = compact[i];
                result_src[idx] = i;
                idx += 1;
            }
        }

        // compact 인덱스 → result 인덱스 매핑
        let mut compact_to_result = [0usize; SIZE];
        {
            let mut ri = 0;
            for ci in 0..SIZE {
                if compact[ci] != 0 {
                    compact_to_result[ci] = ri;
                    ri += 1;
                }
            }
        }

        // 각 compact 타일의 이동 기록
        for ci in 0..SIZE {
            if compact[ci] == 0 {
                continue;
            }
            let orig_col = src_col[ci];
            let dest_col = compact_to_result[ci];
            if orig_col != dest_col {
                // 병합된 타일은 원래 값(병합 전)으로 이동 애니메이션
                let display_val = if did_merge[ci] { compact[ci] / 2 } else { compact[ci] };
                slides.push((display_val, row_idx, orig_col, row_idx, dest_col));
            }
        }

        // 병합에 참여한 두 번째 타일도 같은 도착 위치로 이동
        for ci in 0..SIZE {
            if !did_merge[ci] {
                continue;
            }
            let second_orig_col = src_col[ci + 1];
            let dest_col = compact_to_result[ci];
            slides.push((compact[ci] / 2, row_idx, second_orig_col, row_idx, dest_col));
        }

        // 병합 정보 기록
        for ci in 0..SIZE {
            if did_merge[ci] {
                let dest_col = compact_to_result[ci];
                merges.push((compact[ci], row_idx, dest_col));
            }
        }

        *row = result;
        *row != original
    }

    /// 보드 전체를 왼쪽으로 슬라이드
    pub fn slide_left(&mut self) -> (bool, SlideInfo) {
        let mut slides = Vec::new();
        let mut merges = Vec::new();
        let mut moved = false;
        for r in 0..SIZE {
            if Self::slide_row(&mut self.cells[r], r, &mut slides, &mut merges, &mut self.score) {
                moved = true;
            }
        }
        (moved, SlideInfo { slides, merges })
    }

    /// 시계 방향 90도 회전
    fn rotate(&mut self) {
        let mut rotated = [[0u16; SIZE]; SIZE];
        for r in 0..SIZE {
            for c in 0..SIZE {
                rotated[c][SIZE - 1 - r] = self.cells[r][c];
            }
        }
        self.cells = rotated;
    }

    /// 오른쪽 슬라이드
    pub fn slide_right(&mut self) -> (bool, SlideInfo) {
        self.rotate();
        self.rotate();
        let (moved, info) = self.slide_left();
        self.rotate();
        self.rotate();
        // 좌표 변환: 180도 회전 복원
        let slides = info.slides.into_iter().map(|(v, r, c, tr, tc)| {
            (v, SIZE - 1 - r, SIZE - 1 - c, SIZE - 1 - tr, SIZE - 1 - tc)
        }).collect();
        let merges = info.merges.into_iter().map(|(v, r, c)| {
            (v, SIZE - 1 - r, SIZE - 1 - c)
        }).collect();
        (moved, SlideInfo { slides, merges })
    }

    /// 위쪽 슬라이드
    pub fn slide_up(&mut self) -> (bool, SlideInfo) {
        self.rotate();
        self.rotate();
        self.rotate();
        let (moved, info) = self.slide_left();
        self.rotate();
        // 좌표 변환: 반시계 90도 회전 복원 (r,c) -> (c, SIZE-1-r)
        let slides = info.slides.into_iter().map(|(v, r, c, tr, tc)| {
            (v, c, SIZE - 1 - r, tc, SIZE - 1 - tr)
        }).collect();
        let merges = info.merges.into_iter().map(|(v, r, c)| {
            (v, c, SIZE - 1 - r)
        }).collect();
        (moved, SlideInfo { slides, merges })
    }

    /// 아래쪽 슬라이드
    pub fn slide_down(&mut self) -> (bool, SlideInfo) {
        self.rotate();
        let (moved, info) = self.slide_left();
        self.rotate();
        self.rotate();
        self.rotate();
        // 좌표 변환: 시계 90도 회전 복원 (r,c) -> (SIZE-1-c, r)
        let slides = info.slides.into_iter().map(|(v, r, c, tr, tc)| {
            (v, SIZE - 1 - c, r, SIZE - 1 - tc, tr)
        }).collect();
        let merges = info.merges.into_iter().map(|(v, r, c)| {
            (v, SIZE - 1 - c, r)
        }).collect();
        (moved, SlideInfo { slides, merges })
    }

    /// 더 이상 움직일 수 있는지 확인
    pub fn can_move(&self) -> bool {
        // 빈 칸이 있으면 이동 가능
        for r in 0..SIZE {
            for c in 0..SIZE {
                if self.cells[r][c] == 0 {
                    return true;
                }
            }
        }
        // 인접한 같은 값이 있으면 이동 가능
        for r in 0..SIZE {
            for c in 0..SIZE {
                let val = self.cells[r][c];
                if c + 1 < SIZE && val == self.cells[r][c + 1] {
                    return true;
                }
                if r + 1 < SIZE && val == self.cells[r + 1][c] {
                    return true;
                }
            }
        }
        false
    }

    /// 2048 타일이 있는지 확인
    pub fn has_won(&self) -> bool {
        for r in 0..SIZE {
            for c in 0..SIZE {
                if self.cells[r][c] == 2048 {
                    return true;
                }
            }
        }
        false
    }
}
