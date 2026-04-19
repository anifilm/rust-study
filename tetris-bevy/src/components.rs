use bevy::prelude::*;

#[derive(Component, Clone, Copy)]
pub struct BoardFrame;

#[derive(Component, Clone, Copy)]
pub struct ClearedRowFlash {
    pub row: usize,
}

#[derive(Component, Clone, Copy)]
pub struct LockedBlock;

#[derive(Component, Clone, Copy)]
pub struct ActiveBlock;

#[derive(Component, Clone, Copy)]
pub struct GhostBlock;

#[derive(Component, Clone, Copy)]
pub struct NextBlock;

#[derive(Component, Clone, Copy)]
pub struct PreviewBox;

#[derive(Component, Clone, Copy)]
pub struct ScoreText;

#[derive(Component, Clone, Copy)]
pub struct LinesText;

#[derive(Component, Clone, Copy)]
pub struct LevelText;

#[derive(Component, Clone, Copy)]
pub struct OverlayText;

#[derive(Component, Clone, Copy)]
pub struct OverlayHintText;

#[derive(Component, Clone, Copy)]
pub struct OverlayPanel;

#[derive(Component, Clone, Copy)]
pub struct ClearBadgeText;

#[derive(Component, Clone, Copy)]
pub struct ClearBadgePanel;
