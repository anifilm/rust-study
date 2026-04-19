slint::include_modules!();

use std::cell::RefCell;
use std::rc::Rc;
use slint::{Model, ModelRc, VecModel};

const ROWS: usize = 9;
const COLS: usize = 9;
const MINE_COUNT: usize = 10;

#[derive(Clone, Default)]
struct Cell {
    is_mine: bool,
    is_revealed: bool,
    is_flagged: bool,
    adjacent_mines: u8,
}

#[derive(Clone, Copy, PartialEq)]
enum GameStatus {
    Playing,
    Won,
    Lost,
}

struct Board {
    cells: Vec<Cell>,
    mine_count: usize,
    flags_placed: usize,
    revealed_count: usize,
    status: GameStatus,
    initialized: bool,
}

impl Board {
    fn new() -> Self {
        Board {
            cells: vec![Cell::default(); ROWS * COLS],
            mine_count: MINE_COUNT,
            flags_placed: 0,
            revealed_count: 0,
            status: GameStatus::Playing,
            initialized: false,
        }
    }

    fn place_mines(&mut self, safe_index: usize) {
        let candidates: Vec<usize> = (0..ROWS * COLS)
            .filter(|&i| i != safe_index)
            .collect();
        let chosen = rand::seq::index::sample(
            &mut rand::thread_rng(),
            candidates.len(),
            self.mine_count,
        );
        for i in chosen.iter() {
            self.cells[candidates[i]].is_mine = true;
        }
        self.compute_adjacency();
    }

    fn compute_adjacency(&mut self) {
        for row in 0..ROWS {
            for col in 0..COLS {
                let idx = row * COLS + col;
                if self.cells[idx].is_mine {
                    continue;
                }
                let count = self
                    .neighbors(row, col)
                    .iter()
                    .filter(|&&ni| self.cells[ni].is_mine)
                    .count();
                self.cells[idx].adjacent_mines = count as u8;
            }
        }
    }

    fn neighbors(&self, row: usize, col: usize) -> Vec<usize> {
        let mut result = Vec::with_capacity(8);
        for dr in -1i32..=1 {
            for dc in -1i32..=1 {
                if dr == 0 && dc == 0 {
                    continue;
                }
                let nr = row as i32 + dr;
                let nc = col as i32 + dc;
                if nr >= 0 && nr < ROWS as i32 && nc >= 0 && nc < COLS as i32 {
                    result.push(nr as usize * COLS + nc as usize);
                }
            }
        }
        result
    }

    fn reveal(&mut self, index: usize) {
        if self.status != GameStatus::Playing {
            return;
        }
        if self.cells[index].is_revealed || self.cells[index].is_flagged {
            return;
        }

        if !self.initialized {
            self.place_mines(index);
            self.initialized = true;
        }

        if self.cells[index].is_mine {
            self.cells[index].is_revealed = true;
            self.status = GameStatus::Lost;
            for c in &mut self.cells {
                if c.is_mine {
                    c.is_revealed = true;
                }
            }
            return;
        }

        let mut stack = vec![index];
        while let Some(idx) = stack.pop() {
            if self.cells[idx].is_revealed || self.cells[idx].is_flagged {
                continue;
            }
            self.cells[idx].is_revealed = true;
            self.revealed_count += 1;
            if self.cells[idx].adjacent_mines == 0 {
                let row = idx / COLS;
                let col = idx % COLS;
                for ni in self.neighbors(row, col) {
                    if !self.cells[ni].is_revealed && !self.cells[ni].is_flagged {
                        stack.push(ni);
                    }
                }
            }
        }
        self.check_win();
    }

    fn toggle_flag(&mut self, index: usize) {
        if self.status != GameStatus::Playing {
            return;
        }
        let cell = &mut self.cells[index];
        if cell.is_revealed {
            return;
        }
        if cell.is_flagged {
            cell.is_flagged = false;
            self.flags_placed -= 1;
        } else {
            cell.is_flagged = true;
            self.flags_placed += 1;
        }
    }

    fn check_win(&mut self) {
        let non_mine_count = ROWS * COLS - self.mine_count;
        if self.revealed_count == non_mine_count {
            self.status = GameStatus::Won;
        }
    }
}

fn board_to_slint(board: &Board) -> Vec<CellData> {
    board
        .cells
        .iter()
        .map(|c| CellData {
            is_mine: c.is_mine,
            is_revealed: c.is_revealed,
            is_flagged: c.is_flagged,
            adjacent_mines: c.adjacent_mines as i32,
        })
        .collect()
}

fn sync_model(board: &Board, model: &VecModel<CellData>) {
    for (i, cell) in board.cells.iter().enumerate() {
        model.set_row_data(
            i,
            CellData {
                is_mine: cell.is_mine,
                is_revealed: cell.is_revealed,
                is_flagged: cell.is_flagged,
                adjacent_mines: cell.adjacent_mines as i32,
            },
        );
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    let board = Rc::new(RefCell::new(Board::new()));
    let model: Rc<VecModel<CellData>> =
        Rc::new(VecModel::from(board_to_slint(&board.borrow())));

    ui.set_cells(ModelRc::from(model.clone()));
    ui.set_mines_remaining(MINE_COUNT as i32);
    ui.set_status_text("playing".into());

    // Left-click: reveal cell
    {
        let board = board.clone();
        let model = model.clone();
        let ui_weak = ui.as_weak();
        ui.on_cell_left_clicked(move |index| {
            let mut b = board.borrow_mut();
            b.reveal(index as usize);
            sync_model(&b, &model);
            let ui = ui_weak.unwrap();
            ui.set_mines_remaining((b.mine_count as i32) - (b.flags_placed as i32));
            ui.set_status_text(match b.status {
                GameStatus::Won => "won".into(),
                GameStatus::Lost => "lost".into(),
                GameStatus::Playing => "playing".into(),
            });
        });
    }

    // Right-click: toggle flag
    {
        let board = board.clone();
        let model = model.clone();
        let ui_weak = ui.as_weak();
        ui.on_cell_right_clicked(move |index| {
            let mut b = board.borrow_mut();
            b.toggle_flag(index as usize);
            let cell = &b.cells[index as usize];
            model.set_row_data(
                index as usize,
                CellData {
                    is_mine: cell.is_mine,
                    is_revealed: cell.is_revealed,
                    is_flagged: cell.is_flagged,
                    adjacent_mines: cell.adjacent_mines as i32,
                },
            );
            let ui = ui_weak.unwrap();
            ui.set_mines_remaining((b.mine_count as i32) - (b.flags_placed as i32));
        });
    }

    // Reset: new game
    {
        let board = board.clone();
        let model = model.clone();
        let ui_weak = ui.as_weak();
        ui.on_reset_game(move || {
            let mut b = board.borrow_mut();
            *b = Board::new();
            sync_model(&b, &model);
            let ui = ui_weak.unwrap();
            ui.set_mines_remaining(b.mine_count as i32);
            ui.set_status_text("playing".into());
        });
    }

    ui.run()
}
