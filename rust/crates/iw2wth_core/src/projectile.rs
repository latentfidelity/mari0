//! Engine-neutral projectile rules ported from Lua gameplay objects.

use crate::{config::LegacyFireballConstants, enemy::LegacyEnemyDirection};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyFireballFrame {
    FlyingOne,
    FlyingTwo,
    FlyingThree,
    FlyingFour,
    ExplosionOne,
    ExplosionTwo,
    ExplosionThree,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyFireballCollisionTarget {
    Tile,
    BulletBill,
    PortalWall,
    Spring,
    Goomba,
    Koopa { beetle: bool },
    HammerBro,
    Plant,
    Cheep,
    Bowser,
    Squid,
    FlyingFish,
    Lakito,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyFireballState {
    pub x: f32,
    pub y: f32,
    pub speed_x: f32,
    pub speed_y: f32,
    pub width: f32,
    pub height: f32,
    pub active: bool,
    pub destroy: bool,
    pub destroy_soon: bool,
    pub rotation: f32,
    pub timer: f32,
    pub frame: LegacyFireballFrame,
}

impl LegacyFireballState {
    #[must_use]
    pub fn spawn(
        x: f32,
        y: f32,
        direction: LegacyEnemyDirection,
        constants: LegacyFireballConstants,
    ) -> Self {
        let (x, speed_x) = match direction {
            LegacyEnemyDirection::Right => (x + 6.0 / 16.0, constants.speed),
            LegacyEnemyDirection::Left => (x, -constants.speed),
        };

        Self {
            x,
            y: y + 4.0 / 16.0,
            speed_x,
            speed_y: 0.0,
            width: 8.0 / 16.0,
            height: 8.0 / 16.0,
            active: true,
            destroy: false,
            destroy_soon: false,
            rotation: 0.0,
            timer: 0.0,
            frame: LegacyFireballFrame::FlyingOne,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyFireballViewport {
    pub x_scroll: f32,
    pub width: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyFireballUpdate {
    pub remove: bool,
    pub released_thrower: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyFireballExplosion {
    pub released_thrower: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyFireballCollisionOutcome {
    pub suppress_default: bool,
    pub released_thrower: bool,
    pub play_block_hit_sound: bool,
    pub shoot_target: Option<LegacyEnemyDirection>,
    pub points: Option<u32>,
}

impl LegacyFireballCollisionOutcome {
    #[must_use]
    const fn empty(suppress_default: bool) -> Self {
        Self {
            suppress_default,
            released_thrower: false,
            play_block_hit_sound: false,
            shoot_target: None,
            points: None,
        }
    }
}

#[must_use]
pub fn update_legacy_fireball(
    state: &mut LegacyFireballState,
    constants: LegacyFireballConstants,
    dt: f32,
    viewport: LegacyFireballViewport,
) -> LegacyFireballUpdate {
    state.rotation = 0.0;
    state.timer += dt;

    if state.destroy_soon {
        advance_legacy_fireball_explosion_animation(state, constants);
    } else {
        advance_legacy_fireball_flying_animation(state, constants);
    }

    let released_thrower = state.x < viewport.x_scroll - 1.0
        || state.x > viewport.x_scroll + viewport.width + 1.0
        || (state.y > constants.offscreen_y && state.active);
    if released_thrower {
        state.destroy = true;
    }

    LegacyFireballUpdate {
        remove: state.destroy,
        released_thrower,
    }
}

pub fn explode_legacy_fireball(state: &mut LegacyFireballState) -> LegacyFireballExplosion {
    if state.active {
        state.destroy_soon = true;
        state.frame = LegacyFireballFrame::ExplosionOne;
        state.active = false;
        LegacyFireballExplosion {
            released_thrower: true,
        }
    } else {
        LegacyFireballExplosion {
            released_thrower: false,
        }
    }
}

#[must_use]
pub fn legacy_fireball_left_collision(
    state: &mut LegacyFireballState,
    constants: LegacyFireballConstants,
    target: LegacyFireballCollisionTarget,
) -> LegacyFireballCollisionOutcome {
    state.x -= 0.5;
    let mut outcome = legacy_fireball_hit_stuff(state, target, true);
    state.speed_x = constants.speed;
    outcome.suppress_default = true;
    outcome
}

#[must_use]
pub fn legacy_fireball_right_collision(
    state: &mut LegacyFireballState,
    constants: LegacyFireballConstants,
    target: LegacyFireballCollisionTarget,
) -> LegacyFireballCollisionOutcome {
    let mut outcome = legacy_fireball_hit_stuff(state, target, true);
    state.speed_x = -constants.speed;
    outcome.suppress_default = true;
    outcome
}

#[must_use]
pub fn legacy_fireball_floor_collision(
    state: &mut LegacyFireballState,
    constants: LegacyFireballConstants,
    target: LegacyFireballCollisionTarget,
) -> LegacyFireballCollisionOutcome {
    let mut outcome = if matches!(
        target,
        LegacyFireballCollisionTarget::Tile | LegacyFireballCollisionTarget::PortalWall
    ) {
        LegacyFireballCollisionOutcome::empty(true)
    } else {
        legacy_fireball_hit_stuff(state, target, true)
    };

    state.speed_y = -constants.jump_force;
    outcome.suppress_default = true;
    outcome
}

#[must_use]
pub fn legacy_fireball_ceil_collision(
    state: &mut LegacyFireballState,
    target: LegacyFireballCollisionTarget,
) -> LegacyFireballCollisionOutcome {
    legacy_fireball_hit_stuff(state, target, false)
}

#[must_use]
pub fn legacy_fireball_passive_collision(
    state: &mut LegacyFireballState,
    target: LegacyFireballCollisionTarget,
) -> LegacyFireballCollisionOutcome {
    let mut outcome = legacy_fireball_hit_stuff(state, target, true);
    outcome.suppress_default = true;
    outcome
}

fn legacy_fireball_hit_stuff(
    state: &mut LegacyFireballState,
    target: LegacyFireballCollisionTarget,
    suppress_default: bool,
) -> LegacyFireballCollisionOutcome {
    let mut outcome = LegacyFireballCollisionOutcome::empty(suppress_default);

    if matches!(
        target,
        LegacyFireballCollisionTarget::Tile
            | LegacyFireballCollisionTarget::BulletBill
            | LegacyFireballCollisionTarget::PortalWall
            | LegacyFireballCollisionTarget::Spring
    ) {
        outcome.released_thrower = explode_legacy_fireball(state).released_thrower;
        outcome.play_block_hit_sound = true;
        return outcome;
    }

    if let Some(points) = legacy_fireball_target_points(target) {
        outcome.released_thrower = explode_legacy_fireball(state).released_thrower;
        if !matches!(
            target,
            LegacyFireballCollisionTarget::Koopa { beetle: true }
        ) {
            outcome.shoot_target = Some(LegacyEnemyDirection::Right);
            outcome.points = points;
        }
    }

    outcome
}

fn advance_legacy_fireball_flying_animation(
    state: &mut LegacyFireballState,
    constants: LegacyFireballConstants,
) {
    while state.timer > constants.animation_delay {
        state.frame = match state.frame {
            LegacyFireballFrame::FlyingOne => LegacyFireballFrame::FlyingTwo,
            LegacyFireballFrame::FlyingTwo => LegacyFireballFrame::FlyingThree,
            LegacyFireballFrame::FlyingThree => LegacyFireballFrame::FlyingFour,
            LegacyFireballFrame::FlyingFour => LegacyFireballFrame::FlyingOne,
            LegacyFireballFrame::ExplosionOne
            | LegacyFireballFrame::ExplosionTwo
            | LegacyFireballFrame::ExplosionThree => LegacyFireballFrame::FlyingOne,
        };
        state.timer -= constants.animation_delay;
    }
}

fn advance_legacy_fireball_explosion_animation(
    state: &mut LegacyFireballState,
    constants: LegacyFireballConstants,
) {
    while state.timer > constants.animation_delay {
        state.frame = match state.frame {
            LegacyFireballFrame::ExplosionOne => LegacyFireballFrame::ExplosionTwo,
            LegacyFireballFrame::ExplosionTwo => LegacyFireballFrame::ExplosionThree,
            LegacyFireballFrame::ExplosionThree => {
                state.destroy = true;
                LegacyFireballFrame::ExplosionThree
            }
            LegacyFireballFrame::FlyingOne
            | LegacyFireballFrame::FlyingTwo
            | LegacyFireballFrame::FlyingThree
            | LegacyFireballFrame::FlyingFour => LegacyFireballFrame::ExplosionOne,
        };
        state.timer -= constants.animation_delay;
    }
}

const fn legacy_fireball_target_points(
    target: LegacyFireballCollisionTarget,
) -> Option<Option<u32>> {
    match target {
        LegacyFireballCollisionTarget::Goomba => Some(Some(100)),
        LegacyFireballCollisionTarget::Koopa { beetle: false } => Some(Some(200)),
        LegacyFireballCollisionTarget::Koopa { beetle: true } => Some(None),
        LegacyFireballCollisionTarget::HammerBro => Some(Some(1000)),
        LegacyFireballCollisionTarget::Plant
        | LegacyFireballCollisionTarget::Cheep
        | LegacyFireballCollisionTarget::Squid
        | LegacyFireballCollisionTarget::FlyingFish
        | LegacyFireballCollisionTarget::Lakito => Some(Some(200)),
        LegacyFireballCollisionTarget::Bowser => Some(None),
        LegacyFireballCollisionTarget::Tile
        | LegacyFireballCollisionTarget::BulletBill
        | LegacyFireballCollisionTarget::PortalWall
        | LegacyFireballCollisionTarget::Spring
        | LegacyFireballCollisionTarget::Other => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LegacyFireballCollisionTarget, LegacyFireballFrame, LegacyFireballState,
        LegacyFireballUpdate, LegacyFireballViewport, explode_legacy_fireball,
        legacy_fireball_ceil_collision, legacy_fireball_floor_collision,
        legacy_fireball_left_collision, legacy_fireball_passive_collision,
        legacy_fireball_right_collision, update_legacy_fireball,
    };
    use crate::{config::LegacyFireballConstants, enemy::LegacyEnemyDirection};

    #[test]
    fn fireball_spawn_matches_legacy_offsets_and_direction_speed() {
        let constants = LegacyFireballConstants::default();

        let right = LegacyFireballState::spawn(3.0, 4.0, LegacyEnemyDirection::Right, constants);
        assert_eq!(right.x, 3.0 + 6.0 / 16.0);
        assert_eq!(right.y, 4.0 + 4.0 / 16.0);
        assert_eq!(right.speed_x, constants.speed);
        assert_eq!(right.speed_y, 0.0);
        assert_eq!(right.width, 8.0 / 16.0);
        assert_eq!(right.height, 8.0 / 16.0);
        assert!(right.active);
        assert!(!right.destroy);
        assert!(!right.destroy_soon);
        assert_eq!(right.frame, LegacyFireballFrame::FlyingOne);

        let left = LegacyFireballState::spawn(3.0, 4.0, LegacyEnemyDirection::Left, constants);
        assert_eq!(left.x, 3.0);
        assert_eq!(left.speed_x, -constants.speed);
    }

    #[test]
    fn fireball_flying_animation_uses_strict_star_timer_and_wraps_four_frames() {
        let constants = LegacyFireballConstants::default();
        let viewport = LegacyFireballViewport {
            x_scroll: 0.0,
            width: 20.0,
        };
        let mut state =
            LegacyFireballState::spawn(3.0, 4.0, LegacyEnemyDirection::Right, constants);

        assert_eq!(
            update_legacy_fireball(&mut state, constants, constants.animation_delay, viewport),
            LegacyFireballUpdate {
                remove: false,
                released_thrower: false
            }
        );
        assert_eq!(state.frame, LegacyFireballFrame::FlyingOne);
        assert_close(state.timer, constants.animation_delay);

        let _ = update_legacy_fireball(&mut state, constants, 0.01, viewport);
        assert_eq!(state.frame, LegacyFireballFrame::FlyingTwo);
        assert_close(state.timer, 0.01);

        let _ = update_legacy_fireball(
            &mut state,
            constants,
            constants.animation_delay * 3.0 + 0.01,
            viewport,
        );
        assert_eq!(state.frame, LegacyFireballFrame::FlyingOne);
        assert_close(state.timer, 0.02);
    }

    #[test]
    fn fireball_explosion_releases_once_and_removes_after_third_explosion_tick() {
        let constants = LegacyFireballConstants::default();
        let viewport = LegacyFireballViewport {
            x_scroll: 0.0,
            width: 20.0,
        };
        let mut state =
            LegacyFireballState::spawn(3.0, 4.0, LegacyEnemyDirection::Right, constants);

        assert!(explode_legacy_fireball(&mut state).released_thrower);
        assert!(!explode_legacy_fireball(&mut state).released_thrower);
        assert!(!state.active);
        assert!(state.destroy_soon);
        assert_eq!(state.frame, LegacyFireballFrame::ExplosionOne);

        let _ = update_legacy_fireball(&mut state, constants, constants.animation_delay, viewport);
        assert_eq!(state.frame, LegacyFireballFrame::ExplosionOne);
        assert!(!state.destroy);

        let _ = update_legacy_fireball(&mut state, constants, 0.01, viewport);
        assert_eq!(state.frame, LegacyFireballFrame::ExplosionTwo);

        let _ = update_legacy_fireball(
            &mut state,
            constants,
            constants.animation_delay + 0.01,
            viewport,
        );
        assert_eq!(state.frame, LegacyFireballFrame::ExplosionThree);
        assert!(!state.destroy);

        assert_eq!(
            update_legacy_fireball(
                &mut state,
                constants,
                constants.animation_delay + 0.01,
                viewport
            ),
            LegacyFireballUpdate {
                remove: true,
                released_thrower: false
            }
        );
        assert_eq!(state.frame, LegacyFireballFrame::ExplosionThree);
    }

    #[test]
    fn fireball_update_preserves_legacy_offscreen_release_gates() {
        let constants = LegacyFireballConstants::default();
        let viewport = LegacyFireballViewport {
            x_scroll: 2.0,
            width: 10.0,
        };
        let mut active_x =
            LegacyFireballState::spawn(0.5, 4.0, LegacyEnemyDirection::Right, constants);

        assert_eq!(
            update_legacy_fireball(&mut active_x, constants, 0.0, viewport),
            LegacyFireballUpdate {
                remove: true,
                released_thrower: true
            }
        );

        let mut inactive_x =
            LegacyFireballState::spawn(0.5, 4.0, LegacyEnemyDirection::Right, constants);
        inactive_x.active = false;
        assert_eq!(
            update_legacy_fireball(&mut inactive_x, constants, 0.0, viewport),
            LegacyFireballUpdate {
                remove: true,
                released_thrower: true
            }
        );

        let mut inactive_y =
            LegacyFireballState::spawn(5.0, 16.0, LegacyEnemyDirection::Right, constants);
        inactive_y.active = false;
        assert_eq!(
            update_legacy_fireball(&mut inactive_y, constants, 0.0, viewport),
            LegacyFireballUpdate {
                remove: false,
                released_thrower: false
            }
        );
    }

    #[test]
    fn fireball_side_collisions_bounce_and_emit_block_or_enemy_effects() {
        let constants = LegacyFireballConstants::default();
        let mut tile = LegacyFireballState::spawn(3.0, 4.0, LegacyEnemyDirection::Right, constants);
        let tile_x = tile.x;

        let tile_outcome = legacy_fireball_left_collision(
            &mut tile,
            constants,
            LegacyFireballCollisionTarget::Tile,
        );
        assert!(tile_outcome.suppress_default);
        assert!(tile_outcome.released_thrower);
        assert!(tile_outcome.play_block_hit_sound);
        assert_eq!(tile_outcome.shoot_target, None);
        assert_eq!(tile_outcome.points, None);
        assert_eq!(tile.x, tile_x - 0.5);
        assert_eq!(tile.speed_x, constants.speed);

        let mut enemy =
            LegacyFireballState::spawn(3.0, 4.0, LegacyEnemyDirection::Right, constants);
        let enemy_outcome = legacy_fireball_right_collision(
            &mut enemy,
            constants,
            LegacyFireballCollisionTarget::Goomba,
        );
        assert!(enemy_outcome.suppress_default);
        assert!(enemy_outcome.released_thrower);
        assert!(!enemy_outcome.play_block_hit_sound);
        assert_eq!(
            enemy_outcome.shoot_target,
            Some(LegacyEnemyDirection::Right)
        );
        assert_eq!(enemy_outcome.points, Some(100));
        assert_eq!(enemy.speed_x, -constants.speed);
    }

    #[test]
    fn fireball_floor_collision_bounces_without_exploding_on_tile_or_portalwall() {
        let constants = LegacyFireballConstants::default();
        let mut tile = LegacyFireballState::spawn(3.0, 4.0, LegacyEnemyDirection::Right, constants);

        let tile_outcome = legacy_fireball_floor_collision(
            &mut tile,
            constants,
            LegacyFireballCollisionTarget::Tile,
        );
        assert!(tile_outcome.suppress_default);
        assert!(!tile_outcome.released_thrower);
        assert!(!tile.destroy_soon);
        assert_eq!(tile.speed_y, -constants.jump_force);

        let mut spring =
            LegacyFireballState::spawn(3.0, 4.0, LegacyEnemyDirection::Right, constants);
        let spring_outcome = legacy_fireball_floor_collision(
            &mut spring,
            constants,
            LegacyFireballCollisionTarget::Spring,
        );
        assert!(spring_outcome.suppress_default);
        assert!(spring_outcome.released_thrower);
        assert!(spring_outcome.play_block_hit_sound);
        assert!(spring.destroy_soon);
        assert_eq!(spring.speed_y, -constants.jump_force);
    }

    #[test]
    fn fireball_ceil_and_passive_collisions_preserve_default_contracts() {
        let constants = LegacyFireballConstants::default();
        let mut ceil = LegacyFireballState::spawn(3.0, 4.0, LegacyEnemyDirection::Right, constants);

        let ceil_outcome =
            legacy_fireball_ceil_collision(&mut ceil, LegacyFireballCollisionTarget::BulletBill);
        assert!(!ceil_outcome.suppress_default);
        assert!(ceil_outcome.released_thrower);
        assert!(ceil_outcome.play_block_hit_sound);

        let mut passive =
            LegacyFireballState::spawn(3.0, 4.0, LegacyEnemyDirection::Right, constants);
        let passive_outcome =
            legacy_fireball_passive_collision(&mut passive, LegacyFireballCollisionTarget::Plant);
        assert!(passive_outcome.suppress_default);
        assert!(passive_outcome.released_thrower);
        assert_eq!(
            passive_outcome.shoot_target,
            Some(LegacyEnemyDirection::Right)
        );
        assert_eq!(passive_outcome.points, Some(200));
    }

    #[test]
    fn fireball_enemy_hits_preserve_bowser_and_beetle_point_quirks() {
        let constants = LegacyFireballConstants::default();
        let mut bowser =
            LegacyFireballState::spawn(3.0, 4.0, LegacyEnemyDirection::Right, constants);

        let bowser_outcome = legacy_fireball_right_collision(
            &mut bowser,
            constants,
            LegacyFireballCollisionTarget::Bowser,
        );
        assert!(bowser_outcome.released_thrower);
        assert_eq!(
            bowser_outcome.shoot_target,
            Some(LegacyEnemyDirection::Right)
        );
        assert_eq!(bowser_outcome.points, None);

        let mut beetle =
            LegacyFireballState::spawn(3.0, 4.0, LegacyEnemyDirection::Right, constants);
        let beetle_outcome = legacy_fireball_right_collision(
            &mut beetle,
            constants,
            LegacyFireballCollisionTarget::Koopa { beetle: true },
        );
        assert!(beetle_outcome.released_thrower);
        assert_eq!(beetle_outcome.shoot_target, None);
        assert_eq!(beetle_outcome.points, None);
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.0001,
            "expected {actual} to be close to {expected}"
        );
    }
}
