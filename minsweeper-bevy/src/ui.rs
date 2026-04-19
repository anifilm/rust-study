use bevy::prelude::*;

use crate::game::{FlagCount, GameConfig, GameState, GameTimer, RestartGame};

// ─── Marker components ───────────────────────────────────────────────────────

#[derive(Component)]
struct MineCounterText;

#[derive(Component)]
struct TimerText;

#[derive(Component)]
struct RestartButton;

#[derive(Component)]
struct HudRoot;

#[derive(Component)]
struct OverlayRoot;

// ─── Plugin ──────────────────────────────────────────────────────────────────

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Setup), cleanup_ui)
            .add_systems(OnEnter(GameState::Playing), spawn_hud)
            .add_systems(OnEnter(GameState::Won), spawn_overlay_won)
            .add_systems(OnEnter(GameState::Lost), spawn_overlay_lost)
            .add_systems(
                Update,
                (update_hud, handle_restart_button, handle_restart_key)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                (handle_restart_button, handle_restart_key)
                    .run_if(in_state(GameState::Won).or(in_state(GameState::Lost))),
            );
    }
}

// ─── HUD ─────────────────────────────────────────────────────────────────────

fn cleanup_ui(
    mut commands: Commands,
    hud_query: Query<Entity, With<HudRoot>>,
    overlay_query: Query<Entity, With<OverlayRoot>>,
) {
    for entity in hud_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    for entity in overlay_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn spawn_hud(mut commands: Commands, config: Res<GameConfig>) {
    // Root: full-width row at the top
    commands
        .spawn((
            HudRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(50.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::horizontal(Val::Px(16.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.18)),
            StateScoped(GameState::Playing),
        ))
        .with_children(|parent| {
            // Mine counter
            parent.spawn((
                MineCounterText,
                Text(format!("Mines: {}", config.mine_count)),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Restart button
            parent
                .spawn((
                    RestartButton,
                    Button,
                    Node {
                        min_width: Val::Px(120.0),
                        padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.30, 0.30, 0.36)),
                ))
                .with_children(|p| {
                    p.spawn((
                        Text("New Game".to_string()),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Timer
            parent.spawn((
                TimerText,
                Text("Time: 0s".to_string()),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn update_hud(
    config: Res<GameConfig>,
    flag_count: Res<FlagCount>,
    timer: Res<GameTimer>,
    mut counter_q: Query<&mut Text, (With<MineCounterText>, Without<TimerText>)>,
    mut timer_q: Query<&mut Text, (With<TimerText>, Without<MineCounterText>)>,
) {
    let remaining = config.mine_count.saturating_sub(flag_count.0);
    if let Ok(mut text) = counter_q.get_single_mut() {
        text.0 = format!("Mines: {}", remaining);
    }
    if let Ok(mut text) = timer_q.get_single_mut() {
        text.0 = format!("Time: {}s", timer.elapsed as u32);
    }
}

// ─── Restart ─────────────────────────────────────────────────────────────────

fn handle_restart_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<RestartButton>)>,
    mut restart_events: EventWriter<RestartGame>,
) {
    for interaction in interaction_q.iter() {
        if *interaction == Interaction::Pressed {
            restart_events.send(RestartGame);
        }
    }
}

fn handle_restart_key(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut restart_events: EventWriter<RestartGame>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        restart_events.send(RestartGame);
    }
}

// ─── Game-over overlays ──────────────────────────────────────────────────────

fn spawn_overlay_won(mut commands: Commands) {
    spawn_overlay(
        &mut commands,
        "You Won!",
        Color::srgba(0.1, 0.5, 0.1, 0.85),
        GameState::Won,
    );
}

fn spawn_overlay_lost(mut commands: Commands) {
    spawn_overlay(
        &mut commands,
        "Game Over",
        Color::srgba(0.5, 0.1, 0.1, 0.85),
        GameState::Lost,
    );
}

fn spawn_overlay(commands: &mut Commands, msg: &str, bg: Color, state: GameState) {
    commands
        .spawn((
            OverlayRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(bg),
            GlobalZIndex(10),
            StateScoped(state),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text(msg.to_string()),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent
                .spawn((
                    RestartButton,
                    Button,
                    Node {
                        min_width: Val::Px(220.0),
                        min_height: Val::Px(64.0),
                        padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.25, 0.25, 0.30)),
                ))
                .with_children(|p| {
                    p.spawn((
                        Text("Play Again".to_string()),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}
