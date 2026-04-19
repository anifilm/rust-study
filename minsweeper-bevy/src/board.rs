use bevy::prelude::*;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::{HashSet, VecDeque};

use crate::game::{FlagCell, FlagCount, GameConfig, GameFonts, GameOver, GameState, RevealCell};

// ─── Components ──────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct Mine;

#[derive(Component)]
pub struct AdjacentMines(pub u8);

#[derive(Component)]
pub struct Covered;

#[derive(Component)]
pub struct Revealed;

#[derive(Component)]
pub struct Flagged;

#[derive(Component)]
pub struct Questioned;

#[derive(Component)]
pub struct CellLabel;

#[derive(Component)]
pub struct BoardRoot;

// ─── Resource ────────────────────────────────────────────────────────────────

#[derive(Resource)]
pub struct Board {
    pub cols: u32,
    pub rows: u32,
    pub cells: Vec<Vec<Entity>>,
    pub mine_positions: HashSet<(u32, u32)>,
    pub adjacent_counts: Vec<Vec<u8>>,
    pub origin: Vec2,
    pub cell_size: f32,
    pub mines_initialized: bool,
}

impl Board {
    /// Convert a world-space position to (col, row). Returns None if outside.
    pub fn world_to_coords(&self, world_pos: Vec2) -> Option<(u32, u32)> {
        let local = world_pos - self.origin;
        let col = (local.x / self.cell_size).floor();
        let row = (local.y / self.cell_size).floor();
        if col < 0.0 || row < 0.0 || col >= self.cols as f32 || row >= self.rows as f32 {
            return None;
        }
        Some((col as u32, row as u32))
    }
}

// ─── Plugin ──────────────────────────────────────────────────────────────────

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Setup),
            (cleanup_board, spawn_board).chain(),
        )
        .add_systems(
            Update,
            (
                handle_reveal,
                handle_flag,
                check_win,
                sync_cell_visuals,
                handle_game_over_board,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
    }
}

// ─── Spawn ───────────────────────────────────────────────────────────────────

fn cleanup_board(
    mut commands: Commands,
    board_root_query: Query<Entity, With<BoardRoot>>,
    board: Option<Res<Board>>,
) {
    for entity in board_root_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    if board.is_some() {
        commands.remove_resource::<Board>();
    }
}

fn spawn_board(
    mut commands: Commands,
    config: Res<GameConfig>,
    fonts: Res<GameFonts>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let cols = config.cols;
    let rows = config.rows;
    let cell_size = config.cell_size;
    let gap = 2.0_f32;
    let step = cell_size + gap;

    // Centre the board; leave top 60px for HUD
    let board_w = step * cols as f32 - gap;
    let board_h = step * rows as f32 - gap;
    let origin = Vec2::new(-board_w / 2.0, -board_h / 2.0 - 30.0);

    let mut cells: Vec<Vec<Entity>> = vec![vec![Entity::PLACEHOLDER; rows as usize]; cols as usize];

    let root = commands
        .spawn((
            BoardRoot,
            Transform::default(),
            Visibility::default(),
            StateScoped(GameState::Playing),
        ))
        .id();

    for r in 0..rows {
        for c in 0..cols {
            let world_x = origin.x + c as f32 * step + cell_size / 2.0;
            let world_y = origin.y + r as f32 * step + cell_size / 2.0;

            let cell = commands
                .spawn((
                    AdjacentMines(0),
                    Covered,
                    StateScoped(GameState::Playing),
                    Sprite {
                        color: COLOR_COVERED,
                        custom_size: Some(Vec2::splat(cell_size)),
                        ..default()
                    },
                    Transform::from_xyz(world_x, world_y, 0.0),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        CellLabel,
                        Text2d::new(""),
                        TextFont {
                            font: fonts.text.clone(),
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Transform::from_xyz(0.0, 0.0, 1.0),
                    ));
                })
                .id();

            commands.entity(root).add_child(cell);
            cells[c as usize][r as usize] = cell;
        }
    }

    commands.insert_resource(Board {
        cols,
        rows,
        cells,
        mine_positions: HashSet::new(),
        adjacent_counts: vec![vec![0; rows as usize]; cols as usize],
        origin,
        cell_size: step,
        mines_initialized: false,
    });

    next_state.set(GameState::Playing);
}

// ─── Reveal + Flood-fill ─────────────────────────────────────────────────────

fn handle_reveal(
    mut commands: Commands,
    mut events: EventReader<RevealCell>,
    board: Option<ResMut<Board>>,
    config: Res<GameConfig>,
    mut game_over: EventWriter<GameOver>,
    covered_query: Query<(), (With<Covered>, Without<Flagged>)>,
) {
    let mut board = match board {
        Some(b) => b,
        None => return,
    };

    for ev in events.read() {
        if !board.mines_initialized {
            initialize_mines(&mut commands, &mut board, &config, (ev.col, ev.row));
        }

        let entity = board.cells[ev.col as usize][ev.row as usize];

        // Only act on covered (and not flagged) cells
        if covered_query.get(entity).is_err() {
            continue;
        }

        if board.mine_positions.contains(&(ev.col, ev.row)) {
            // Hit a mine — reveal it and end game
            commands.entity(entity).remove::<Covered>().insert(Revealed);
            game_over.send(GameOver { won: false });
            continue;
        }

        // BFS flood-fill for zero-adjacent cells
        let mut queue: VecDeque<(u32, u32)> = VecDeque::new();
        let mut visited: HashSet<(u32, u32)> = HashSet::new();
        queue.push_back((ev.col, ev.row));
        visited.insert((ev.col, ev.row));

        while let Some((c, r)) = queue.pop_front() {
            let ent = board.cells[c as usize][r as usize];
            if covered_query.get(ent).is_err() {
                continue;
            }
            commands.entity(ent).remove::<Covered>().insert(Revealed);

            // Spread to neighbours only if this cell has 0 adjacent mines
            if board.adjacent_counts[c as usize][r as usize] != 0 {
                continue;
            }

            for dc in -1i32..=1 {
                for dr in -1i32..=1 {
                    if dc == 0 && dr == 0 {
                        continue;
                    }
                    let nc = c as i32 + dc;
                    let nr = r as i32 + dr;
                    if nc >= 0 && nr >= 0 && nc < board.cols as i32 && nr < board.rows as i32 {
                        let neighbour = (nc as u32, nr as u32);
                        if !visited.contains(&neighbour)
                            && !board.mine_positions.contains(&neighbour)
                        {
                            visited.insert(neighbour);
                            queue.push_back(neighbour);
                        }
                    }
                }
            }
        }
    }
}

fn initialize_mines(
    commands: &mut Commands,
    board: &mut Board,
    config: &GameConfig,
    safe_cell: (u32, u32),
) {
    let mut positions: Vec<(u32, u32)> = (0..board.rows)
        .flat_map(|r| (0..board.cols).map(move |c| (c, r)))
        .filter(|&pos| pos != safe_cell)
        .collect();
    positions.shuffle(&mut thread_rng());

    let mine_set: HashSet<(u32, u32)> = positions
        .into_iter()
        .take(config.mine_count as usize)
        .collect();

    for c in 0..board.cols {
        for r in 0..board.rows {
            let entity = board.cells[c as usize][r as usize];

            if mine_set.contains(&(c, r)) {
                commands.entity(entity).insert(Mine);
            }

            let mut count = 0u8;
            for dc in -1i32..=1 {
                for dr in -1i32..=1 {
                    if dc == 0 && dr == 0 {
                        continue;
                    }
                    let nc = c as i32 + dc;
                    let nr = r as i32 + dr;
                    if nc >= 0
                        && nr >= 0
                        && nc < board.cols as i32
                        && nr < board.rows as i32
                        && mine_set.contains(&(nc as u32, nr as u32))
                    {
                        count += 1;
                    }
                }
            }

            board.adjacent_counts[c as usize][r as usize] = count;
            commands.entity(entity).insert(AdjacentMines(count));
        }
    }

    board.mine_positions = mine_set;
    board.mines_initialized = true;
}

// ─── Flag toggle ─────────────────────────────────────────────────────────────

fn handle_flag(
    mut commands: Commands,
    mut events: EventReader<FlagCell>,
    board: Option<Res<Board>>,
    covered_query: Query<
        (),
        (
            With<Covered>,
            Without<Flagged>,
            Without<Questioned>,
            Without<Revealed>,
        ),
    >,
    flagged_query: Query<(), With<Flagged>>,
    questioned_query: Query<(), With<Questioned>>,
    mut flag_count: ResMut<FlagCount>,
) {
    let board = match board {
        Some(b) => b,
        None => return,
    };

    for ev in events.read() {
        let entity = board.cells[ev.col as usize][ev.row as usize];

        if covered_query.get(entity).is_ok() {
            commands.entity(entity).insert(Flagged);
            flag_count.0 = flag_count.0.saturating_add(1);
        } else if flagged_query.get(entity).is_ok() {
            commands
                .entity(entity)
                .remove::<Flagged>()
                .insert(Questioned);
            flag_count.0 = flag_count.0.saturating_sub(1);
        } else if questioned_query.get(entity).is_ok() {
            commands
                .entity(entity)
                .remove::<Questioned>()
                .remove::<Covered>()
                .insert(Covered);
        }
    }
}

// ─── Win check ───────────────────────────────────────────────────────────────

fn check_win(
    board: Option<Res<Board>>,
    config: Res<GameConfig>,
    revealed_safe: Query<(), (With<Revealed>, Without<Mine>)>,
    mut game_over: EventWriter<GameOver>,
) {
    let board = match board {
        Some(b) => b,
        None => return,
    };
    let safe_total = board.cols * board.rows - config.mine_count;
    // 현재 보드의 셀만 카운트 (이전 게임의 엔티티 포함 방지)
    let revealed_count = board
        .cells
        .iter()
        .flatten()
        .filter(|&&e| revealed_safe.get(e).is_ok())
        .count() as u32;
    if revealed_count == safe_total {
        game_over.send(GameOver { won: true });
    }
}

// ─── Reveal all mines on game over ───────────────────────────────────────────

fn handle_game_over_board(
    mut commands: Commands,
    mut events: EventReader<GameOver>,
    board: Option<Res<Board>>,
    covered_mines: Query<Entity, (With<Mine>, With<Covered>)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut timer: ResMut<crate::game::GameTimer>,
) {
    let board = match board {
        Some(b) => b,
        None => return,
    };
    let _ = board; // used to ensure board exists

    for ev in events.read() {
        // Reveal all remaining mines
        if !ev.won {
            for ent in covered_mines.iter() {
                commands
                    .entity(ent)
                    .remove::<Covered>()
                    .remove::<Flagged>()
                    .remove::<Questioned>()
                    .insert(Revealed);
            }
        }
        timer.running = false;
        if ev.won {
            next_state.set(GameState::Won);
        } else {
            next_state.set(GameState::Lost);
        }
    }
}

// ─── Visual sync ─────────────────────────────────────────────────────────────

pub fn sync_cell_visuals(
    cell_query: Query<
        (
            Entity,
            &AdjacentMines,
            Option<&Mine>,
            Option<&Covered>,
            Option<&Revealed>,
            Option<&Flagged>,
            Option<&Questioned>,
            &Children,
        ),
        Or<(
            Changed<Covered>,
            Changed<Revealed>,
            Changed<Flagged>,
            Changed<Questioned>,
        )>,
    >,
    mut sprite_query: Query<&mut Sprite>,
    fonts: Res<GameFonts>,
    mut label_query: Query<(&mut Text2d, &mut TextColor, &mut TextFont), With<CellLabel>>,
) {
    for (entity, adj, mine, _covered, revealed, flagged, questioned, children) in cell_query.iter()
    {
        let _ = entity;

        // Determine cell colour
        let color = if flagged.is_some() {
            COLOR_FLAGGED
        } else if questioned.is_some() {
            COLOR_COVERED
        } else if revealed.is_some() {
            if mine.is_some() {
                COLOR_MINE
            } else {
                COLOR_REVEALED
            }
        } else {
            // covered
            COLOR_COVERED
        };

        if let Ok(mut sprite) = sprite_query.get_mut(entity) {
            sprite.color = color;
        }

        // Update child label text
        for &child in children.iter() {
            if let Ok((mut text, mut text_color, mut text_font)) = label_query.get_mut(child) {
                if revealed.is_some() && mine.is_none() && adj.0 > 0 {
                    text_font.font = fonts.text.clone();
                    text.0 = adj.0.to_string();
                    let idx = (adj.0 - 1) as usize;
                    text_color.0 = NUMBER_COLORS[idx.min(7)];
                } else if revealed.is_some() && mine.is_some() {
                    text_font.font = fonts.icons.clone();
                    text.0 = MINE_ICON.to_string();
                    text_color.0 = Color::WHITE;
                } else if flagged.is_some() {
                    text_font.font = fonts.icons.clone();
                    text.0 = FLAG_ICON.to_string();
                    text_color.0 = Color::WHITE;
                } else if questioned.is_some() {
                    text_font.font = fonts.icons.clone();
                    text.0 = QUESTION_ICON.to_string();
                    text_color.0 = Color::srgb(0.95, 0.95, 0.95);
                } else {
                    text_font.font = fonts.text.clone();
                    text.0 = String::new();
                }
            }
        }
    }
}

// ─── Colours ─────────────────────────────────────────────────────────────────

pub const COLOR_COVERED: Color = Color::srgb(0.45, 0.49, 0.56);
pub const COLOR_REVEALED: Color = Color::srgb(0.85, 0.82, 0.76);
pub const COLOR_FLAGGED: Color = Color::srgb(0.82, 0.18, 0.18);
pub const COLOR_MINE: Color = Color::srgb(0.18, 0.18, 0.22);

pub const FLAG_ICON: &str = "\u{f024}";
pub const QUESTION_ICON: &str = "\u{f128}";
pub const MINE_ICON: &str = "\u{f1e2}";

pub const NUMBER_COLORS: [Color; 8] = [
    Color::srgb(0.10, 0.20, 0.90), // 1 - blue
    Color::srgb(0.05, 0.50, 0.05), // 2 - green
    Color::srgb(0.80, 0.10, 0.10), // 3 - red
    Color::srgb(0.10, 0.05, 0.55), // 4 - dark blue
    Color::srgb(0.55, 0.05, 0.05), // 5 - dark red
    Color::srgb(0.05, 0.50, 0.50), // 6 - teal
    Color::srgb(0.10, 0.10, 0.10), // 7 - black
    Color::srgb(0.50, 0.50, 0.50), // 8 - grey
];
