use bevy::prelude::*;

use crate::components::OverlayPanel;
use crate::components::{ClearBadgePanel, ClearBadgeText, ClearedRowFlash};
use crate::game::VisualEffectsState;
use crate::systems::render::{board_pixel_width, BOARD_ORIGIN_X};

pub fn update_visual_effects(
    time: Res<Time>,
    mut effects: ResMut<VisualEffectsState>,
    mut flash_query: Query<(&ClearedRowFlash, &mut Sprite, &mut Transform), Without<Camera2d>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<ClearedRowFlash>)>,
    mut badge_queries: ParamSet<(
        Query<(&mut Text, &mut TextColor, &mut TextFont, &mut Visibility), With<ClearBadgeText>>,
        Query<
            (
                &mut Node,
                &mut BackgroundColor,
                &mut BorderColor,
                &mut Visibility,
            ),
            With<ClearBadgePanel>,
        >,
        Query<(&mut Node, &mut BackgroundColor, &mut BorderColor), With<OverlayPanel>>,
    )>,
) {
    let dt = time.delta_secs();

    if effects.cleared_row_flash_timer > 0.0 {
        effects.cleared_row_flash_timer = (effects.cleared_row_flash_timer - dt).max(0.0);
    }

    if effects.shake_timer > 0.0 {
        effects.shake_timer = (effects.shake_timer - dt).max(0.0);
    }

    if effects.clear_badge_timer > 0.0 {
        effects.clear_badge_timer = (effects.clear_badge_timer - dt).max(0.0);
    }

    if effects.preview_transition_timer > 0.0 {
        effects.preview_transition_timer = (effects.preview_transition_timer - dt).max(0.0);
    }

    if effects.lock_squash_timer > 0.0 {
        effects.lock_squash_timer = (effects.lock_squash_timer - dt).max(0.0);
    }

    if effects.game_over_popup_timer > 0.0 {
        effects.game_over_popup_timer = (effects.game_over_popup_timer - dt).max(0.0);
    }

    if effects.paused_popup_timer > 0.0 {
        effects.paused_popup_timer = (effects.paused_popup_timer - dt).max(0.0);
    }

    let flash_progress = if effects.cleared_row_flash_timer > 0.0 {
        let progress = effects.cleared_row_flash_timer / VisualEffectsState::ROW_FLASH_DURATION;
        progress
    } else {
        0.0
    };

    for (row_flash, mut sprite, mut transform) in flash_query.iter_mut() {
        if flash_progress > 0.0 && effects.cleared_rows.contains(&row_flash.row) {
            let width = (board_pixel_width() - 4.0) * flash_progress.max(0.08);
            let alpha = 0.34 * flash_progress;
            sprite.custom_size = Some(Vec2::new(width, sprite.custom_size.unwrap().y));
            sprite.color = Color::srgba(0.82, 0.91, 1.0, alpha);
            transform.translation.x = BOARD_ORIGIN_X + width / 2.0;
        } else {
            sprite.custom_size = Some(Vec2::new(
                board_pixel_width() - 4.0,
                sprite.custom_size.unwrap().y,
            ));
            sprite.color = Color::srgba(0.82, 0.91, 1.0, 0.0);
            transform.translation.x = BOARD_ORIGIN_X + (board_pixel_width() - 4.0) / 2.0;
        }
    }

    if let Ok(mut transform) = camera_query.get_single_mut() {
        if effects.shake_timer > 0.0 {
            let progress = effects.shake_timer / VisualEffectsState::SHAKE_DURATION;
            let direction = if (effects.shake_timer * 90.0) as i32 % 2 == 0 {
                1.0
            } else {
                -1.0
            };
            transform.translation.x = direction * 6.0 * progress;
            transform.translation.y = -2.0 * progress;
        } else {
            transform.translation.x = 0.0;
            transform.translation.y = 0.0;
        }
    }

    if let Ok((mut node, mut background, mut border)) = badge_queries.p2().get_single_mut() {
        let base_left = 126.0;
        let base_top = 266.0;
        let base_width = 368.0;
        let base_height = 188.0;

        let is_game_over = effects.previous_phase == Some(crate::game::GamePhase::GameOver);
        let is_paused = effects.previous_phase == Some(crate::game::GamePhase::Paused);
        let is_animating = effects.game_over_popup_timer > 0.0 || effects.paused_popup_timer > 0.0;

        if is_animating {
            let (timer, duration) = if effects.game_over_popup_timer > 0.0 {
                (
                    effects.game_over_popup_timer,
                    VisualEffectsState::GAME_OVER_POPUP_DURATION,
                )
            } else {
                (
                    effects.paused_popup_timer,
                    VisualEffectsState::PAUSED_POPUP_DURATION,
                )
            };
            let progress = (1.0 - timer / duration).clamp(0.0, 1.0);
            let eased = 1.0 - (1.0 - progress) * (1.0 - progress);
            let scale = 0.84 + 0.16 * eased;
            let width = base_width * scale;
            let height = base_height * scale;
            let center_x = base_left + base_width / 2.0;
            let center_y = base_top + base_height / 2.0;

            node.left = Val::Px(center_x - width / 2.0);
            node.top = Val::Px(center_y - height / 2.0);
            node.width = Val::Px(width);
            node.height = Val::Px(height);
            background.0 = Color::srgba(0.03, 0.04, 0.08, 0.72 + 0.16 * eased);

            if is_game_over && progress > 0.8 {
                let color_progress = ((progress - 0.8) / 0.2).clamp(0.0, 1.0);
                let r = 0.34 + (0.92 - 0.34) * color_progress;
                let g = 0.45 + (0.36 - 0.45) * color_progress;
                let b = 0.74 + (0.42 - 0.74) * color_progress;
                border.0 = Color::srgba(r, g, b, 0.84 + 0.16 * eased);
            } else {
                border.0 = Color::srgba(0.34, 0.45, 0.74, 0.84 + 0.16 * eased);
            }
        } else if is_game_over {
            border.0 = Color::srgba(0.92, 0.36, 0.42, 1.0);
        } else if is_paused {
            border.0 = Color::srgb(0.31, 0.55, 0.92);
        }
    }

    if let Ok((mut text, mut color, mut font, mut visibility)) = badge_queries.p0().get_single_mut()
    {
        if effects.clear_badge_timer > 0.0 && !effects.clear_badge_text.is_empty() {
            let progress = effects.clear_badge_timer / VisualEffectsState::CLEAR_BADGE_DURATION;
            let fade = if progress > 0.65 {
                (1.0 - progress) / 0.35
            } else {
                progress / 0.65
            }
            .clamp(0.0, 1.0);

            text.0 = effects.clear_badge_text.clone();
            color.0 = Color::srgba(0.88, 0.95, 1.0, fade);
            font.font_size = 22.0 + (1.0 - progress) * 12.0;
            *visibility = Visibility::Visible;
        } else {
            text.0.clear();
            color.0 = Color::srgba(0.88, 0.95, 1.0, 0.0);
            font.font_size = 24.0;
            *visibility = Visibility::Hidden;
        }
    }

    if let Ok((mut node, mut background, mut border, mut visibility)) =
        badge_queries.p1().get_single_mut()
    {
        if effects.clear_badge_timer > 0.0 && !effects.clear_badge_text.is_empty() {
            let progress = effects.clear_badge_timer / VisualEffectsState::CLEAR_BADGE_DURATION;
            let fade = if progress > 0.65 {
                (1.0 - progress) / 0.35
            } else {
                progress / 0.65
            }
            .clamp(0.0, 1.0);

            const BADGE_REST_TOP: f32 = 360.0 - 52.0 / 2.0;
            node.top = Val::Px(BADGE_REST_TOP - (1.0 - progress) * 18.0);
            background.0 = Color::srgba(0.08, 0.14, 0.24, 0.72 * fade);
            border.0 = Color::srgba(0.62, 0.83, 1.0, 0.95 * fade);
            *visibility = Visibility::Visible;
        } else {
            const BADGE_REST_TOP: f32 = 360.0 - 52.0 / 2.0;
            node.top = Val::Px(BADGE_REST_TOP);
            background.0 = Color::srgba(0.08, 0.14, 0.24, 0.0);
            border.0 = Color::srgba(0.62, 0.83, 1.0, 0.0);
            *visibility = Visibility::Hidden;
        }
    }

    if effects.cleared_row_flash_timer == 0.0 {
        effects.cleared_rows.clear();
    }

    if effects.clear_badge_timer == 0.0 {
        effects.clear_badge_text.clear();
    }

    if effects.lock_squash_timer == 0.0 {
        effects.locked_blocks.clear();
    }
}
