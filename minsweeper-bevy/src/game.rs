use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Setup,
    Playing,
    Won,
    Lost,
}

#[derive(Resource)]
pub struct GameConfig {
    pub cols: u32,
    pub rows: u32,
    pub mine_count: u32,
    pub cell_size: f32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            cols: 9,
            rows: 9,
            mine_count: 10,
            cell_size: 44.0,
        }
    }
}

#[derive(Resource, Default)]
pub struct GameTimer {
    pub elapsed: f32,
    pub running: bool,
}

#[derive(Resource, Default)]
pub struct FlagCount(pub u32);

#[derive(Resource, Clone)]
pub struct GameFonts {
    pub text: Handle<Font>,
    pub icons: Handle<Font>,
}

impl FromWorld for GameFonts {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            text: asset_server.load("fonts/FiraSans-Bold.ttf"),
            icons: asset_server.load("fonts/fa-solid-900.ttf"),
        }
    }
}

#[derive(Event)]
pub struct RevealCell {
    pub col: u32,
    pub row: u32,
}

#[derive(Event)]
pub struct FlagCell {
    pub col: u32,
    pub row: u32,
}

#[derive(Event)]
pub struct RestartGame;

#[derive(Event)]
pub struct GameOver {
    pub won: bool,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .init_resource::<GameConfig>()
            .init_resource::<GameTimer>()
            .init_resource::<FlagCount>()
            .init_resource::<GameFonts>()
            .add_event::<RevealCell>()
            .add_event::<FlagCell>()
            .add_event::<RestartGame>()
            .add_event::<GameOver>()
            .add_systems(Startup, spawn_camera)
            .add_systems(OnEnter(GameState::Playing), start_timer)
            .add_systems(
                Update,
                (
                    process_restart,
                    tick_timer.run_if(in_state(GameState::Playing)),
                ),
            );
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}

fn start_timer(mut timer: ResMut<GameTimer>) {
    timer.running = true;
}

pub fn tick_timer(mut timer: ResMut<GameTimer>, time: Res<Time>) {
    if timer.running {
        timer.elapsed += time.delta_secs();
    }
}

pub fn process_restart(
    mut events: EventReader<RestartGame>,
    mut next_state: ResMut<NextState<GameState>>,
    mut timer: ResMut<GameTimer>,
    mut flag_count: ResMut<FlagCount>,
) {
    for _ in events.read() {
        *timer = GameTimer::default();
        flag_count.0 = 0;
        next_state.set(GameState::Setup);
    }
}
