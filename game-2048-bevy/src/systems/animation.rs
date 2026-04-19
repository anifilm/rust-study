use bevy::prelude::*;

use crate::components::{MergeSource, MergeTarget};

/// 애니메이션 리소스
#[derive(Resource)]
pub struct AnimationConfig {
    pub move_duration: f32,
    pub spawn_duration: f32,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            move_duration: 0.15,
            spawn_duration: 0.2,
        }
    }
}

/// 이동 애니메이션 컴포넌트
#[derive(Component)]
pub struct MoveAnimation {
    pub start: Vec2,
    pub end: Vec2,
    pub timer: Timer,
}

/// 등장 애니메이션 컴포넌트
#[derive(Component)]
pub struct SpawnAnimation {
    pub timer: Timer,
}

/// 애니메이션 시스템
pub fn animate_movement(
    _time: Res<Time>,
    mut query: Query<(&mut Transform, &MoveAnimation)>,
) {
    for (mut transform, anim) in query.iter_mut() {
        let t = anim.timer.fraction();
        let t = t * t * (3.0 - 2.0 * t); // smoothstep
        let pos = anim.start.lerp(anim.end, t);
        transform.translation.x = pos.x;
        transform.translation.y = pos.y;
    }
}

pub fn animate_spawn(
    _commands: Commands,
    _time: Res<Time>,
    mut query: Query<(&mut Transform, &SpawnAnimation)>,
) {
    for (mut transform, anim) in query.iter_mut() {
        let t = anim.timer.fraction();
        let scale = t.min(1.0);
        transform.scale = Vec3::splat(scale);
    }
}

pub fn update_animations(
    mut commands: Commands,
    time: Res<Time>,
    mut move_query: Query<(Entity, &mut MoveAnimation, Option<&MergeSource>)>,
    mut spawn_query: Query<(Entity, &mut SpawnAnimation)>,
    mut merge_target_query: Query<(Entity, &mut Transform), With<MergeTarget>>,
    merge_source_query: Query<Entity, With<MergeSource>>,
) {
    // MoveAnimation 완료 처리
    let mut finished_move_entities = Vec::new();
    for (entity, mut anim, merge_source) in move_query.iter_mut() {
        anim.timer.tick(time.delta());
        if anim.timer.finished() {
            finished_move_entities.push((entity, merge_source.is_some()));
        }
    }

    // 완료된 애니메이션 처리
    for (entity, is_merge_source) in finished_move_entities {
        if is_merge_source {
            // 병합 소스는 제거
            commands.entity(entity).despawn_recursive();
        } else {
            commands.entity(entity).remove::<MoveAnimation>();
        }
    }

    // MergeTarget 애니메이션 처리
    // 모든 MergeSource 타일이 제거된 후에만 MergeTarget을 표시
    let merge_source_count = merge_source_query.iter().count();

    // 디버그: 병합 상태 출력
    // println!("update_animations: merge_source_count={}, merge_target_count={}",
    //          merge_source_count, merge_target_query.iter().count());

    for (entity, mut transform) in merge_target_query.iter_mut() {
        // MergeTarget은 점점 커지면서 나타남
        let current_scale = transform.scale.x;

        // 병합 소스 타일이 모두 제거되었는지 확인
        if merge_source_count == 0 && current_scale < 1.0 {
            let new_scale = (current_scale + time.delta_secs() * 10.0).min(1.0);
            transform.scale = Vec3::splat(new_scale);

            if new_scale >= 1.0 {
                // 애니메이션 완료 - MergeTarget 제거
                commands.entity(entity).remove::<MergeTarget>();
            }
        }
    }

    // SpawnAnimation 완료 처리
    for (entity, mut anim) in spawn_query.iter_mut() {
        anim.timer.tick(time.delta());
        if anim.timer.finished() {
            commands.entity(entity).remove::<SpawnAnimation>();
        }
    }
}
