use bevy::prelude::*;

use crate::constants::*;

/// 게임 화면에서 사용하는 Mesh/Material 핸들 (원형 돌, 힌트, 별점 등)
#[derive(Resource)]
#[allow(dead_code)]
pub struct GameAssets {
    // 돌 메쉬
    pub piece_mesh: Handle<Mesh>,
    pub highlight_mesh: Handle<Mesh>,
    pub shadow_mesh: Handle<Mesh>,
    pub hint_mesh: Handle<Mesh>,
    pub star_mesh: Handle<Mesh>,
    // 돌 머티리얼
    pub black_mat: Handle<ColorMaterial>,
    pub black_hl_mat: Handle<ColorMaterial>,
    pub white_mat: Handle<ColorMaterial>,
    pub white_hl_mat: Handle<ColorMaterial>,
    pub shadow_mat: Handle<ColorMaterial>,
    pub hint_mat: Handle<ColorMaterial>,
    pub star_mat: Handle<ColorMaterial>,
}

pub fn init_game_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(GameAssets {
        piece_mesh: meshes.add(Circle::new(PIECE_RADIUS)),
        highlight_mesh: meshes.add(Circle::new(PIECE_HIGHLIGHT_RADIUS)),
        shadow_mesh: meshes.add(Circle::new(PIECE_SHADOW_RADIUS)),
        hint_mesh: meshes.add(Circle::new(VALID_HINT_RADIUS)),
        star_mesh: meshes.add(Circle::new(STAR_POINT_RADIUS)),

        black_mat: materials.add(ColorMaterial::from_color(COLOR_BLACK_PIECE)),
        black_hl_mat: materials.add(ColorMaterial::from_color(COLOR_BLACK_HIGHLIGHT)),
        white_mat: materials.add(ColorMaterial::from_color(COLOR_WHITE_PIECE)),
        white_hl_mat: materials.add(ColorMaterial::from_color(COLOR_WHITE_HIGHLIGHT)),
        shadow_mat: materials.add(ColorMaterial::from_color(COLOR_PIECE_SHADOW)),
        hint_mat: materials.add(ColorMaterial::from_color(COLOR_VALID_HINT)),
        star_mat: materials.add(ColorMaterial::from_color(COLOR_STAR_POINT)),
    });
}
