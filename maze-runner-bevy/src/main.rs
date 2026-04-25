mod constants;
mod game;
mod maze;
mod player;
mod treasure;

use bevy::prelude::*;

use constants::*;
use game::hud;
use game::state::GameState;
use maze::generator;
use maze::renderer;
use player::movement;
use treasure::collection;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Maze Runner".into(),
                resolution: Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                ..default()
            }),
            ..default()
        }))
        // Game state
        .init_state::<GameState>()
        // Events
        .add_event::<treasure::TreasureCollectedEvent>()
        // Startup systems
        .add_systems(Startup, setup_camera)
        // Loading state: generate maze
        .add_systems(OnEnter(GameState::Loading), reset_round_and_generate_maze)
        // Playing state systems
        .add_systems(OnEnter(GameState::Playing), (setup_game_entities, hud::spawn_hud))
        .add_systems(
            Update,
            (
                movement::player_movement,
                collection::collect_treasures,
                collection::handle_treasure_collected,
                hud::update_hud,
                check_exit_reached,
            )
                .run_if(in_state(GameState::Playing)),
        )
        // Cleared state
        .add_systems(OnEnter(GameState::Cleared), hud::show_cleared_message)
            .add_systems(Update, restart_game.run_if(in_state(GameState::Cleared)))
        .add_systems(OnExit(GameState::Cleared), hud::cleanup_cleared_message)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Resource to hold the generated maze
#[derive(Resource)]
struct MazeResource {
    grid: maze::MazeGrid,
}

#[derive(Resource, Clone, Copy)]
struct ClearSummary {
    final_score: u32,
    treasures_collected: u32,
}

/// Reset the current round and generate a new maze.
fn reset_round_and_generate_maze(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    maze_tiles: Query<Entity, With<renderer::MazeTile>>,
    players: Query<Entity, With<player::Player>>,
    treasures: Query<Entity, With<treasure::Treasure>>,
    hud_roots: Query<Entity, With<hud::HudRoot>>,
    cleared_messages: Query<Entity, With<hud::ClearedMessage>>,
) {
    for entity in maze_tiles.iter() {
        commands.entity(entity).despawn();
    }
    for entity in players.iter() {
        commands.entity(entity).despawn();
    }
    for entity in treasures.iter() {
        commands.entity(entity).despawn();
    }
    for entity in hud_roots.iter() {
        commands.entity(entity).despawn();
    }
    for entity in cleared_messages.iter() {
        commands.entity(entity).despawn();
    }

    commands.remove_resource::<ClearSummary>();
    let maze_grid = generator::generate_maze();
    commands.insert_resource(MazeResource { grid: maze_grid });
    next_state.set(GameState::Playing);
}

/// Setup game entities (maze, player, treasures)
fn setup_game_entities(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    maze_resource: Res<MazeResource>,
) {
    // Spawn maze tiles
    renderer::spawn_maze(&mut commands, &mut meshes, &mut materials, &maze_resource.grid);
    // Spawn player
    player::spawn_player(&mut commands, &mut meshes, &mut materials);
    // Spawn treasures
    treasure::spawn_treasures(&mut commands, &mut meshes, &mut materials, &maze_resource.grid);
}

/// Check if player reached the exit
fn check_exit_reached(
    player_query: Query<&player::Player>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Ok(player) = player_query.get_single() else {
        return;
    };

    if player.grid_x == EXIT_POS.0 && player.grid_y == EXIT_POS.1 {
        commands.insert_resource(ClearSummary {
            final_score: player.score,
            treasures_collected: player.treasures_collected,
        });
        next_state.set(GameState::Cleared);
    }
}

/// Restart game when R is pressed in Cleared state
fn restart_game(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        next_state.set(GameState::Loading);
    }
}
