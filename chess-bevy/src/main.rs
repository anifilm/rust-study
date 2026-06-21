//! Bevy 0.19 로 만든 체스 게임.
//!
//! 시작 메뉴에서 "1:1 대전 / AI 대전 / 나가기" 를 선택할 수 있으며,
//! 현재는 1:1 대전(로컬 2인)이 완전히 구현되어 있다.

mod chess;
mod menu;

use bevy::prelude::*;

use chess::ChessPlugin;
use menu::MenuPlugin;

/// 앱 전역 화면 상태.
#[derive(States, Default, Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum AppState {
    #[default]
    Menu,
    InGame,
    /// 아직 구현되지 않은 AI 대전 안내 화면.
    ComingSoon,
}

/// 한글을 렌더링하기 위한 공용 폰트 핸들.
#[derive(Resource)]
pub struct GameFont(pub Handle<Font>);

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "체스 - Bevy".into(),
            resolution: (760u32, 760u32).into(),
            ..default()
        }),
        ..default()
    }))
    .insert_resource(ClearColor(Color::srgb(0.12, 0.12, 0.14)));

    // 폰트는 초기 상태 전환(OnEnter(Menu))에서 곧바로 필요하다.
    // 그 전환은 `Startup` 보다 먼저 실행되므로, 시스템이 아니라 앱 빌드 시점에
    // 미리 핸들을 만들어 리소스로 넣어 둔다.
    let font = app
        .world()
        .resource::<AssetServer>()
        .load("fonts/NanumGothic-Regular.ttf");
    app.insert_resource(GameFont(font));

    app.init_state::<AppState>()
        .add_plugins((MenuPlugin, ChessPlugin))
        .add_systems(Startup, setup)
        .run();
}

/// 2D 카메라를 준비한다.
fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
