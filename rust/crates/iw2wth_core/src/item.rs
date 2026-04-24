//! Engine-neutral item and power-up rules ported from Lua objects.

use crate::config::LegacyPowerUpConstants;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyMushroomState {
    pub x: f32,
    pub y: f32,
    pub speed_x: f32,
    pub speed_y: f32,
    pub width: f32,
    pub height: f32,
    pub is_static: bool,
    pub active: bool,
    pub drawable: bool,
    pub destroy: bool,
    pub rotation: f32,
    pub up_timer: f32,
    pub falling: bool,
}

impl LegacyMushroomState {
    #[must_use]
    pub fn spawn(block_x: f32, block_y: f32) -> Self {
        Self {
            x: block_x - 6.0 / 16.0,
            y: block_y - 11.0 / 16.0,
            speed_x: 0.0,
            speed_y: 0.0,
            width: 12.0 / 16.0,
            height: 12.0 / 16.0,
            is_static: true,
            active: true,
            drawable: false,
            destroy: false,
            rotation: 0.0,
            up_timer: 0.0,
            falling: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyOneUpState {
    pub x: f32,
    pub y: f32,
    pub speed_x: f32,
    pub speed_y: f32,
    pub width: f32,
    pub height: f32,
    pub is_static: bool,
    pub active: bool,
    pub drawable: bool,
    pub destroy: bool,
    pub rotation: f32,
    pub up_timer: f32,
    pub falling: bool,
}

impl LegacyOneUpState {
    #[must_use]
    pub fn spawn(block_x: f32, block_y: f32) -> Self {
        Self {
            x: block_x - 6.0 / 16.0,
            y: block_y - 11.0 / 16.0,
            speed_x: 0.0,
            speed_y: 0.0,
            width: 12.0 / 16.0,
            height: 12.0 / 16.0,
            is_static: true,
            active: false,
            drawable: false,
            destroy: false,
            rotation: 0.0,
            up_timer: 0.0,
            falling: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyStarState {
    pub x: f32,
    pub y: f32,
    pub speed_x: f32,
    pub speed_y: f32,
    pub width: f32,
    pub height: f32,
    pub is_static: bool,
    pub active: bool,
    pub drawable: bool,
    pub destroy: bool,
    pub gravity: f32,
    pub rotation: f32,
    pub up_timer: f32,
    pub animation_timer: f32,
    pub frame: u8,
    pub falling: bool,
}

impl LegacyStarState {
    #[must_use]
    pub fn spawn(block_x: f32, block_y: f32, constants: LegacyPowerUpConstants) -> Self {
        Self {
            x: block_x - 6.0 / 16.0,
            y: block_y - 11.0 / 16.0,
            speed_x: 0.0,
            speed_y: 0.0,
            width: 12.0 / 16.0,
            height: 12.0 / 16.0,
            is_static: true,
            active: true,
            drawable: false,
            destroy: false,
            gravity: constants.star_gravity,
            rotation: 0.0,
            up_timer: 0.0,
            animation_timer: 0.0,
            frame: 1,
            falling: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyFlowerState {
    pub start_y: f32,
    pub x: f32,
    pub y: f32,
    pub speed_x: f32,
    pub speed_y: f32,
    pub width: f32,
    pub height: f32,
    pub is_static: bool,
    pub active: bool,
    pub drawable: bool,
    pub destroy: bool,
    pub gravity: f32,
    pub rotation: f32,
    pub up_timer: f32,
    pub animation_timer: f32,
    pub frame: u8,
    pub falling: bool,
}

impl LegacyFlowerState {
    #[must_use]
    pub fn spawn(block_x: f32, block_y: f32, constants: LegacyPowerUpConstants) -> Self {
        Self {
            start_y: block_y,
            x: block_x - 6.0 / 16.0,
            y: block_y - 11.0 / 16.0,
            speed_x: 0.0,
            speed_y: 0.0,
            width: 12.0 / 16.0,
            height: 12.0 / 16.0,
            is_static: true,
            active: true,
            drawable: false,
            destroy: false,
            gravity: constants.flower_gravity,
            rotation: 0.0,
            up_timer: 0.0,
            animation_timer: 0.0,
            frame: 1,
            falling: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyPowerUpUpdate {
    pub remove: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyOneUpViewport {
    pub x_scroll: f32,
    pub width: f32,
}

#[must_use]
pub fn update_legacy_mushroom(
    state: &mut LegacyMushroomState,
    constants: LegacyPowerUpConstants,
    dt: f32,
) -> LegacyPowerUpUpdate {
    align_legacy_item_rotation(&mut state.rotation, constants.rotation_alignment_speed, dt);

    if state.up_timer < constants.emergence_time {
        state.up_timer += dt;
        state.y -= dt * (1.0 / constants.emergence_time);
        state.speed_x = constants.mushroom_speed;
    } else if state.is_static {
        state.is_static = false;
        state.active = true;
        state.drawable = true;
    }

    LegacyPowerUpUpdate {
        remove: state.destroy,
    }
}

#[must_use]
pub fn update_legacy_one_up(
    state: &mut LegacyOneUpState,
    constants: LegacyPowerUpConstants,
    viewport: LegacyOneUpViewport,
    dt: f32,
) -> LegacyPowerUpUpdate {
    align_legacy_item_rotation(&mut state.rotation, constants.rotation_alignment_speed, dt);

    if state.up_timer < constants.emergence_time {
        state.up_timer += dt;
        state.y -= dt * (1.0 / constants.emergence_time);
        state.speed_x = constants.mushroom_speed;

        return LegacyPowerUpUpdate { remove: false };
    }

    if state.is_static {
        state.is_static = false;
        state.active = true;
        state.drawable = true;
    }

    let offscreen_left =
        state.x < viewport.x_scroll - viewport.width + constants.one_up_offscreen_left_margin;
    let offscreen_bottom = state.y > constants.one_up_offscreen_y;

    LegacyPowerUpUpdate {
        remove: offscreen_left || offscreen_bottom || state.destroy,
    }
}

#[must_use]
pub fn update_legacy_star(
    state: &mut LegacyStarState,
    constants: LegacyPowerUpConstants,
    dt: f32,
) -> LegacyPowerUpUpdate {
    align_legacy_item_rotation(&mut state.rotation, constants.rotation_alignment_speed, dt);

    if state.up_timer < constants.emergence_time {
        state.up_timer += dt;
        state.y -= dt * (1.0 / constants.emergence_time);
        state.speed_x = constants.mushroom_speed;
    } else if state.is_static {
        state.is_static = false;
        state.active = true;
        state.drawable = true;
        state.speed_y = -constants.star_jump_force / 2.0;
    }

    state.animation_timer += dt;
    while state.animation_timer > constants.star_animation_delay {
        state.frame += 1;
        if state.frame == 5 {
            state.frame = 1;
        }
        state.animation_timer -= constants.star_animation_delay;
    }

    LegacyPowerUpUpdate {
        remove: state.destroy,
    }
}

#[must_use]
pub fn update_legacy_flower(
    state: &mut LegacyFlowerState,
    constants: LegacyPowerUpConstants,
    dt: f32,
) -> LegacyPowerUpUpdate {
    if state.up_timer < constants.emergence_time {
        state.up_timer += dt;
        state.y -= dt * (1.0 / constants.emergence_time);
    }

    if state.up_timer > constants.emergence_time {
        state.y = state.start_y - constants.flower_emerged_y_offset;
        state.active = true;
        state.drawable = true;
    }

    state.animation_timer += dt;
    while state.animation_timer > constants.star_animation_delay {
        state.frame += 1;
        if state.frame == 5 {
            state.frame = 1;
        }
        state.animation_timer -= constants.star_animation_delay;
    }

    LegacyPowerUpUpdate {
        remove: state.destroy,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyPowerUpCollisionActor {
    Player,
    Other,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyMushroomCollection {
    None,
    GrowPlayer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyMushroomCollision {
    pub suppress_default: bool,
    pub collection: LegacyMushroomCollection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyOneUpReward {
    pub grant_life_to_all_players: bool,
    pub respawn_players: bool,
    pub show_one_up_score: bool,
    pub play_sound: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyOneUpCollection {
    None,
    Collected(LegacyOneUpReward),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyOneUpCollision {
    pub suppress_default: bool,
    pub collection: LegacyOneUpCollection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyStarCollection {
    None,
    GrantStarPower,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyStarCollision {
    pub suppress_default: bool,
    pub collection: LegacyStarCollection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyFlowerCollection {
    None,
    GrowPlayer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyFlowerCollision {
    pub suppress_default: bool,
    pub collection: LegacyFlowerCollection,
}

#[must_use]
pub fn legacy_mushroom_left_collision(
    state: &mut LegacyMushroomState,
    actor: LegacyPowerUpCollisionActor,
    constants: LegacyPowerUpConstants,
) -> LegacyMushroomCollision {
    state.speed_x = constants.mushroom_speed;

    LegacyMushroomCollision {
        suppress_default: true,
        collection: collect_legacy_mushroom(state, actor),
    }
}

#[must_use]
pub fn legacy_mushroom_right_collision(
    state: &mut LegacyMushroomState,
    actor: LegacyPowerUpCollisionActor,
    constants: LegacyPowerUpConstants,
) -> LegacyMushroomCollision {
    state.speed_x = -constants.mushroom_speed;

    LegacyMushroomCollision {
        suppress_default: true,
        collection: collect_legacy_mushroom(state, actor),
    }
}

#[must_use]
pub fn legacy_mushroom_floor_collision(
    state: &mut LegacyMushroomState,
    actor: LegacyPowerUpCollisionActor,
) -> LegacyMushroomCollision {
    LegacyMushroomCollision {
        suppress_default: false,
        collection: collect_legacy_mushroom(state, actor),
    }
}

#[must_use]
pub fn legacy_mushroom_ceiling_collision(
    state: &mut LegacyMushroomState,
    actor: LegacyPowerUpCollisionActor,
) -> LegacyMushroomCollision {
    LegacyMushroomCollision {
        suppress_default: false,
        collection: collect_legacy_mushroom(state, actor),
    }
}

#[must_use]
pub fn legacy_one_up_left_collision(
    state: &mut LegacyOneUpState,
    actor: LegacyPowerUpCollisionActor,
    constants: LegacyPowerUpConstants,
    life_count_enabled: bool,
) -> LegacyOneUpCollision {
    state.speed_x = constants.mushroom_speed;

    LegacyOneUpCollision {
        suppress_default: true,
        collection: collect_legacy_one_up(state, actor, life_count_enabled),
    }
}

#[must_use]
pub fn legacy_one_up_right_collision(
    state: &mut LegacyOneUpState,
    actor: LegacyPowerUpCollisionActor,
    constants: LegacyPowerUpConstants,
    life_count_enabled: bool,
) -> LegacyOneUpCollision {
    state.speed_x = -constants.mushroom_speed;

    LegacyOneUpCollision {
        suppress_default: true,
        collection: collect_legacy_one_up(state, actor, life_count_enabled),
    }
}

#[must_use]
pub fn legacy_one_up_floor_collision(
    state: &mut LegacyOneUpState,
    actor: LegacyPowerUpCollisionActor,
    life_count_enabled: bool,
) -> LegacyOneUpCollision {
    LegacyOneUpCollision {
        suppress_default: false,
        collection: collect_legacy_one_up(state, actor, life_count_enabled),
    }
}

#[must_use]
pub fn legacy_one_up_ceiling_collision(
    state: &mut LegacyOneUpState,
    actor: LegacyPowerUpCollisionActor,
    life_count_enabled: bool,
) -> LegacyOneUpCollision {
    LegacyOneUpCollision {
        suppress_default: false,
        collection: collect_legacy_one_up(state, actor, life_count_enabled),
    }
}

#[must_use]
pub fn legacy_star_left_collision(
    state: &mut LegacyStarState,
    actor: LegacyPowerUpCollisionActor,
    constants: LegacyPowerUpConstants,
) -> LegacyStarCollision {
    state.speed_x = constants.mushroom_speed;

    LegacyStarCollision {
        suppress_default: true,
        collection: collect_legacy_star(state, actor),
    }
}

#[must_use]
pub fn legacy_star_right_collision(
    state: &mut LegacyStarState,
    actor: LegacyPowerUpCollisionActor,
    constants: LegacyPowerUpConstants,
) -> LegacyStarCollision {
    state.speed_x = -constants.mushroom_speed;

    LegacyStarCollision {
        suppress_default: true,
        collection: collect_legacy_star(state, actor),
    }
}

#[must_use]
pub fn legacy_star_floor_collision(
    state: &mut LegacyStarState,
    actor: LegacyPowerUpCollisionActor,
    constants: LegacyPowerUpConstants,
) -> LegacyStarCollision {
    let collection = if state.active {
        collect_legacy_star(state, actor)
    } else {
        LegacyStarCollection::None
    };
    state.speed_y = -constants.star_jump_force;

    LegacyStarCollision {
        suppress_default: true,
        collection,
    }
}

#[must_use]
pub fn legacy_star_ceiling_collision(
    state: &mut LegacyStarState,
    actor: LegacyPowerUpCollisionActor,
) -> LegacyStarCollision {
    let collection = if state.active {
        collect_legacy_star(state, actor)
    } else {
        LegacyStarCollection::None
    };

    LegacyStarCollision {
        suppress_default: false,
        collection,
    }
}

#[must_use]
pub fn legacy_flower_left_collision(
    state: &mut LegacyFlowerState,
    actor: LegacyPowerUpCollisionActor,
) -> LegacyFlowerCollision {
    LegacyFlowerCollision {
        suppress_default: true,
        collection: collect_legacy_flower(state, actor),
    }
}

#[must_use]
pub fn legacy_flower_right_collision(
    state: &mut LegacyFlowerState,
    actor: LegacyPowerUpCollisionActor,
) -> LegacyFlowerCollision {
    LegacyFlowerCollision {
        suppress_default: true,
        collection: collect_legacy_flower(state, actor),
    }
}

#[must_use]
pub fn legacy_flower_floor_collision(
    state: &mut LegacyFlowerState,
    actor: LegacyPowerUpCollisionActor,
) -> LegacyFlowerCollision {
    let collection = if state.active {
        collect_legacy_flower(state, actor)
    } else {
        LegacyFlowerCollection::None
    };

    LegacyFlowerCollision {
        suppress_default: false,
        collection,
    }
}

#[must_use]
pub fn legacy_flower_ceiling_collision(
    state: &mut LegacyFlowerState,
    actor: LegacyPowerUpCollisionActor,
) -> LegacyFlowerCollision {
    let collection = if state.active {
        collect_legacy_flower(state, actor)
    } else {
        LegacyFlowerCollection::None
    };

    LegacyFlowerCollision {
        suppress_default: false,
        collection,
    }
}

pub fn apply_legacy_mushroom_jump(
    state: &mut LegacyMushroomState,
    constants: LegacyPowerUpConstants,
    source_x: f32,
) {
    state.falling = true;
    state.speed_y = -constants.mushroom_jump_force;

    let center_x = state.x + state.width / 2.0;
    let source_center_x = source_x - 0.5;

    if center_x < source_center_x {
        state.speed_x = -constants.mushroom_speed;
    } else if center_x > source_center_x {
        state.speed_x = constants.mushroom_speed;
    }
}

pub fn apply_legacy_one_up_jump(
    state: &mut LegacyOneUpState,
    constants: LegacyPowerUpConstants,
    source_x: f32,
) {
    state.falling = true;
    state.speed_y = -constants.mushroom_jump_force;

    let center_x = state.x + state.width / 2.0;
    let source_center_x = source_x - 0.5;

    if center_x < source_center_x {
        state.speed_x = -constants.mushroom_speed;
    } else if center_x > source_center_x {
        state.speed_x = constants.mushroom_speed;
    }
}

pub fn apply_legacy_star_jump(
    state: &mut LegacyStarState,
    constants: LegacyPowerUpConstants,
    source_x: f32,
) {
    state.falling = true;
    state.speed_y = -constants.mushroom_jump_force;

    let center_x = state.x + state.width / 2.0;
    let source_center_x = source_x - 0.5;

    if center_x < source_center_x {
        state.speed_x = -constants.mushroom_speed;
    } else if center_x > source_center_x {
        state.speed_x = constants.mushroom_speed;
    }
}

pub fn apply_legacy_flower_jump(_state: &mut LegacyFlowerState, _source_x: f32) {}

fn collect_legacy_mushroom(
    state: &mut LegacyMushroomState,
    actor: LegacyPowerUpCollisionActor,
) -> LegacyMushroomCollection {
    match actor {
        LegacyPowerUpCollisionActor::Player => {
            state.active = false;
            state.destroy = true;
            state.drawable = false;
            LegacyMushroomCollection::GrowPlayer
        }
        LegacyPowerUpCollisionActor::Other => LegacyMushroomCollection::None,
    }
}

fn collect_legacy_one_up(
    state: &mut LegacyOneUpState,
    actor: LegacyPowerUpCollisionActor,
    life_count_enabled: bool,
) -> LegacyOneUpCollection {
    match actor {
        LegacyPowerUpCollisionActor::Player => {
            state.destroy = true;
            state.active = false;
            LegacyOneUpCollection::Collected(LegacyOneUpReward {
                grant_life_to_all_players: life_count_enabled,
                respawn_players: life_count_enabled,
                show_one_up_score: true,
                play_sound: true,
            })
        }
        LegacyPowerUpCollisionActor::Other => LegacyOneUpCollection::None,
    }
}

fn collect_legacy_star(
    state: &mut LegacyStarState,
    actor: LegacyPowerUpCollisionActor,
) -> LegacyStarCollection {
    match actor {
        LegacyPowerUpCollisionActor::Player => {
            state.destroy = true;
            LegacyStarCollection::GrantStarPower
        }
        LegacyPowerUpCollisionActor::Other => LegacyStarCollection::None,
    }
}

fn collect_legacy_flower(
    state: &mut LegacyFlowerState,
    actor: LegacyPowerUpCollisionActor,
) -> LegacyFlowerCollection {
    match actor {
        LegacyPowerUpCollisionActor::Player => {
            state.active = false;
            state.destroy = true;
            state.drawable = false;
            LegacyFlowerCollection::GrowPlayer
        }
        LegacyPowerUpCollisionActor::Other => LegacyFlowerCollection::None,
    }
}

fn align_legacy_item_rotation(rotation: &mut f32, alignment_speed: f32, dt: f32) {
    *rotation %= core::f32::consts::TAU;

    let delta = alignment_speed * dt;
    if *rotation > 0.0 {
        *rotation -= delta;
        if *rotation < 0.0 {
            *rotation = 0.0;
        }
    } else if *rotation < 0.0 {
        *rotation += delta;
        if *rotation > 0.0 {
            *rotation = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LegacyFlowerCollection, LegacyFlowerCollision, LegacyFlowerState, LegacyMushroomCollection,
        LegacyMushroomCollision, LegacyMushroomState, LegacyOneUpCollection, LegacyOneUpCollision,
        LegacyOneUpReward, LegacyOneUpState, LegacyOneUpViewport, LegacyPowerUpCollisionActor,
        LegacyPowerUpUpdate, LegacyStarCollection, LegacyStarCollision, LegacyStarState,
        apply_legacy_flower_jump, apply_legacy_mushroom_jump, apply_legacy_one_up_jump,
        apply_legacy_star_jump, legacy_flower_ceiling_collision, legacy_flower_floor_collision,
        legacy_flower_left_collision, legacy_flower_right_collision,
        legacy_mushroom_ceiling_collision, legacy_mushroom_floor_collision,
        legacy_mushroom_left_collision, legacy_mushroom_right_collision,
        legacy_one_up_ceiling_collision, legacy_one_up_floor_collision,
        legacy_one_up_left_collision, legacy_one_up_right_collision, legacy_star_ceiling_collision,
        legacy_star_floor_collision, legacy_star_left_collision, legacy_star_right_collision,
        update_legacy_flower, update_legacy_mushroom, update_legacy_one_up, update_legacy_star,
    };
    use crate::config::LegacyPowerUpConstants;

    #[test]
    fn mushroom_spawn_matches_legacy_body_and_flags() {
        let state = LegacyMushroomState::spawn(10.0, 5.0);

        assert_eq!(state.x, 10.0 - 6.0 / 16.0);
        assert_eq!(state.y, 5.0 - 11.0 / 16.0);
        assert_eq!(state.speed_x, 0.0);
        assert_eq!(state.speed_y, 0.0);
        assert_eq!(state.width, 12.0 / 16.0);
        assert_eq!(state.height, 12.0 / 16.0);
        assert!(state.is_static);
        assert!(state.active);
        assert!(!state.drawable);
        assert!(!state.destroy);
        assert_eq!(state.rotation, 0.0);
        assert_eq!(state.up_timer, 0.0);
        assert!(!state.falling);
    }

    #[test]
    fn mushroom_update_emerges_until_previous_timer_reaches_threshold() {
        let constants = LegacyPowerUpConstants::default();
        let mut state = LegacyMushroomState::spawn(10.0, 5.0);

        assert_eq!(
            update_legacy_mushroom(&mut state, constants, constants.emergence_time),
            LegacyPowerUpUpdate { remove: false }
        );
        assert_eq!(state.up_timer, constants.emergence_time);
        assert_eq!(
            state.y,
            5.0 - 11.0 / 16.0 - constants.emergence_time * (1.0 / constants.emergence_time)
        );
        assert_eq!(state.speed_x, constants.mushroom_speed);
        assert!(state.is_static);
        assert!(!state.drawable);

        assert_eq!(
            update_legacy_mushroom(&mut state, constants, 0.0),
            LegacyPowerUpUpdate { remove: false }
        );
        assert!(!state.is_static);
        assert!(state.active);
        assert!(state.drawable);
    }

    #[test]
    fn mushroom_update_aligns_portal_rotation_and_reports_destroy_removal() {
        let constants = LegacyPowerUpConstants::default();
        let mut state = LegacyMushroomState::spawn(10.0, 5.0);
        state.rotation = core::f32::consts::TAU + 1.0;
        state.destroy = true;

        assert_eq!(
            update_legacy_mushroom(&mut state, constants, 0.05),
            LegacyPowerUpUpdate { remove: true }
        );
        assert_eq!(
            state.rotation,
            1.0 - constants.rotation_alignment_speed * 0.05
        );

        state.rotation = -1.0;
        let _ = update_legacy_mushroom(&mut state, constants, 0.05);
        assert_eq!(
            state.rotation,
            -1.0 + constants.rotation_alignment_speed * 0.05
        );
    }

    #[test]
    fn mushroom_side_collisions_bounce_and_collect_player_while_suppressing_default() {
        let constants = LegacyPowerUpConstants::default();
        let mut left = LegacyMushroomState::spawn(10.0, 5.0);

        assert_eq!(
            legacy_mushroom_left_collision(
                &mut left,
                LegacyPowerUpCollisionActor::Other,
                constants
            ),
            LegacyMushroomCollision {
                suppress_default: true,
                collection: LegacyMushroomCollection::None
            }
        );
        assert_eq!(left.speed_x, constants.mushroom_speed);
        assert!(!left.destroy);

        let mut right = LegacyMushroomState::spawn(10.0, 5.0);
        assert_eq!(
            legacy_mushroom_right_collision(
                &mut right,
                LegacyPowerUpCollisionActor::Player,
                constants
            ),
            LegacyMushroomCollision {
                suppress_default: true,
                collection: LegacyMushroomCollection::GrowPlayer
            }
        );
        assert_eq!(right.speed_x, -constants.mushroom_speed);
        assert!(!right.active);
        assert!(right.destroy);
        assert!(!right.drawable);
    }

    #[test]
    fn mushroom_floor_and_ceiling_collect_player_without_suppressing_default() {
        let mut floor = LegacyMushroomState::spawn(10.0, 5.0);

        assert_eq!(
            legacy_mushroom_floor_collision(&mut floor, LegacyPowerUpCollisionActor::Player),
            LegacyMushroomCollision {
                suppress_default: false,
                collection: LegacyMushroomCollection::GrowPlayer
            }
        );
        assert!(floor.destroy);

        let mut ceiling = LegacyMushroomState::spawn(10.0, 5.0);
        assert_eq!(
            legacy_mushroom_ceiling_collision(&mut ceiling, LegacyPowerUpCollisionActor::Other),
            LegacyMushroomCollision {
                suppress_default: false,
                collection: LegacyMushroomCollection::None
            }
        );
        assert!(!ceiling.destroy);
    }

    #[test]
    fn mushroom_jump_sets_falling_impulse_and_direction_from_source_x() {
        let constants = LegacyPowerUpConstants::default();
        let mut state = LegacyMushroomState::spawn(10.0, 5.0);

        apply_legacy_mushroom_jump(&mut state, constants, 12.0);
        assert!(state.falling);
        assert_eq!(state.speed_y, -constants.mushroom_jump_force);
        assert_eq!(state.speed_x, -constants.mushroom_speed);

        apply_legacy_mushroom_jump(&mut state, constants, 9.0);
        assert_eq!(state.speed_x, constants.mushroom_speed);

        let mut equal = LegacyMushroomState::spawn(10.0, 5.0);
        equal.speed_x = 1.25;
        let equal_source_x = equal.x + equal.width / 2.0 + 0.5;
        apply_legacy_mushroom_jump(&mut equal, constants, equal_source_x);
        assert_eq!(equal.speed_x, 1.25);
    }

    #[test]
    fn one_up_spawn_matches_legacy_body_and_inactive_start() {
        let state = LegacyOneUpState::spawn(10.0, 5.0);

        assert_eq!(state.x, 10.0 - 6.0 / 16.0);
        assert_eq!(state.y, 5.0 - 11.0 / 16.0);
        assert_eq!(state.speed_x, 0.0);
        assert_eq!(state.speed_y, 0.0);
        assert_eq!(state.width, 12.0 / 16.0);
        assert_eq!(state.height, 12.0 / 16.0);
        assert!(state.is_static);
        assert!(!state.active);
        assert!(!state.drawable);
        assert!(!state.destroy);
        assert_eq!(state.rotation, 0.0);
        assert_eq!(state.up_timer, 0.0);
        assert!(!state.falling);
    }

    #[test]
    fn one_up_update_emerges_then_applies_offscreen_and_destroy_removal() {
        let constants = LegacyPowerUpConstants::default();
        let viewport = LegacyOneUpViewport {
            x_scroll: 20.0,
            width: 10.0,
        };
        let mut state = LegacyOneUpState::spawn(10.0, 5.0);
        state.destroy = true;

        assert_eq!(
            update_legacy_one_up(&mut state, constants, viewport, constants.emergence_time),
            LegacyPowerUpUpdate { remove: false }
        );
        assert_eq!(state.up_timer, constants.emergence_time);
        assert!(state.is_static);
        assert!(!state.active);
        assert!(!state.drawable);

        assert_eq!(
            update_legacy_one_up(&mut state, constants, viewport, 0.0),
            LegacyPowerUpUpdate { remove: true }
        );
        assert!(!state.is_static);
        assert!(state.active);
        assert!(state.drawable);

        let mut offscreen_left = LegacyOneUpState::spawn(10.0, 5.0);
        offscreen_left.up_timer = constants.emergence_time;
        offscreen_left.x =
            viewport.x_scroll - viewport.width + constants.one_up_offscreen_left_margin - 0.01;
        assert_eq!(
            update_legacy_one_up(&mut offscreen_left, constants, viewport, 0.0),
            LegacyPowerUpUpdate { remove: true }
        );

        let mut offscreen_bottom = LegacyOneUpState::spawn(10.0, 5.0);
        offscreen_bottom.up_timer = constants.emergence_time;
        offscreen_bottom.y = constants.one_up_offscreen_y + 0.01;
        assert_eq!(
            update_legacy_one_up(&mut offscreen_bottom, constants, viewport, 0.0),
            LegacyPowerUpUpdate { remove: true }
        );
    }

    #[test]
    fn one_up_update_aligns_portal_rotation_like_mushroom() {
        let constants = LegacyPowerUpConstants::default();
        let viewport = LegacyOneUpViewport {
            x_scroll: 0.0,
            width: 10.0,
        };
        let mut state = LegacyOneUpState::spawn(10.0, 5.0);
        state.rotation = core::f32::consts::TAU + 1.0;

        let _ = update_legacy_one_up(&mut state, constants, viewport, 0.05);

        assert_eq!(
            state.rotation,
            1.0 - constants.rotation_alignment_speed * 0.05
        );
    }

    #[test]
    fn one_up_side_collisions_bounce_and_emit_life_reward_events() {
        let constants = LegacyPowerUpConstants::default();
        let mut left = LegacyOneUpState::spawn(10.0, 5.0);

        assert_eq!(
            legacy_one_up_left_collision(
                &mut left,
                LegacyPowerUpCollisionActor::Other,
                constants,
                true
            ),
            LegacyOneUpCollision {
                suppress_default: true,
                collection: LegacyOneUpCollection::None
            }
        );
        assert_eq!(left.speed_x, constants.mushroom_speed);
        assert!(!left.destroy);

        let mut right = LegacyOneUpState::spawn(10.0, 5.0);
        assert_eq!(
            legacy_one_up_right_collision(
                &mut right,
                LegacyPowerUpCollisionActor::Player,
                constants,
                true
            ),
            LegacyOneUpCollision {
                suppress_default: true,
                collection: LegacyOneUpCollection::Collected(LegacyOneUpReward {
                    grant_life_to_all_players: true,
                    respawn_players: true,
                    show_one_up_score: true,
                    play_sound: true,
                })
            }
        );
        assert_eq!(right.speed_x, -constants.mushroom_speed);
        assert!(!right.active);
        assert!(right.destroy);
    }

    #[test]
    fn one_up_floor_and_ceiling_collect_player_without_suppressing_default() {
        let mut floor = LegacyOneUpState::spawn(10.0, 5.0);

        assert_eq!(
            legacy_one_up_floor_collision(&mut floor, LegacyPowerUpCollisionActor::Player, false),
            LegacyOneUpCollision {
                suppress_default: false,
                collection: LegacyOneUpCollection::Collected(LegacyOneUpReward {
                    grant_life_to_all_players: false,
                    respawn_players: false,
                    show_one_up_score: true,
                    play_sound: true,
                })
            }
        );
        assert!(floor.destroy);
        assert!(!floor.active);

        let mut ceiling = LegacyOneUpState::spawn(10.0, 5.0);
        assert_eq!(
            legacy_one_up_ceiling_collision(&mut ceiling, LegacyPowerUpCollisionActor::Other, true),
            LegacyOneUpCollision {
                suppress_default: false,
                collection: LegacyOneUpCollection::None
            }
        );
        assert!(!ceiling.destroy);
    }

    #[test]
    fn one_up_jump_sets_falling_impulse_and_direction_from_source_x() {
        let constants = LegacyPowerUpConstants::default();
        let mut state = LegacyOneUpState::spawn(10.0, 5.0);

        apply_legacy_one_up_jump(&mut state, constants, 12.0);
        assert!(state.falling);
        assert_eq!(state.speed_y, -constants.mushroom_jump_force);
        assert_eq!(state.speed_x, -constants.mushroom_speed);

        apply_legacy_one_up_jump(&mut state, constants, 9.0);
        assert_eq!(state.speed_x, constants.mushroom_speed);

        let mut equal = LegacyOneUpState::spawn(10.0, 5.0);
        equal.speed_x = 1.25;
        let equal_source_x = equal.x + equal.width / 2.0 + 0.5;
        apply_legacy_one_up_jump(&mut equal, constants, equal_source_x);
        assert_eq!(equal.speed_x, 1.25);
    }

    #[test]
    fn star_spawn_matches_legacy_body_animation_and_gravity() {
        let constants = LegacyPowerUpConstants::default();
        let state = LegacyStarState::spawn(10.0, 5.0, constants);

        assert_eq!(state.x, 10.0 - 6.0 / 16.0);
        assert_eq!(state.y, 5.0 - 11.0 / 16.0);
        assert_eq!(state.speed_x, 0.0);
        assert_eq!(state.speed_y, 0.0);
        assert_eq!(state.width, 12.0 / 16.0);
        assert_eq!(state.height, 12.0 / 16.0);
        assert!(state.is_static);
        assert!(state.active);
        assert!(!state.drawable);
        assert!(!state.destroy);
        assert_eq!(state.gravity, constants.star_gravity);
        assert_eq!(state.rotation, 0.0);
        assert_eq!(state.up_timer, 0.0);
        assert_eq!(state.animation_timer, 0.0);
        assert_eq!(state.frame, 1);
        assert!(!state.falling);
    }

    #[test]
    fn star_update_emerges_then_launches_with_half_star_jump_force() {
        let constants = LegacyPowerUpConstants::default();
        let mut state = LegacyStarState::spawn(10.0, 5.0, constants);

        assert_eq!(
            update_legacy_star(&mut state, constants, constants.emergence_time),
            LegacyPowerUpUpdate { remove: false }
        );
        assert_eq!(state.up_timer, constants.emergence_time);
        assert_eq!(
            state.y,
            5.0 - 11.0 / 16.0 - constants.emergence_time * (1.0 / constants.emergence_time)
        );
        assert_eq!(state.speed_x, constants.mushroom_speed);
        assert_eq!(state.speed_y, 0.0);
        assert!(state.is_static);
        assert!(!state.drawable);

        assert_eq!(
            update_legacy_star(&mut state, constants, 0.0),
            LegacyPowerUpUpdate { remove: false }
        );
        assert!(!state.is_static);
        assert!(state.active);
        assert!(state.drawable);
        assert_eq!(state.speed_y, -constants.star_jump_force / 2.0);
    }

    #[test]
    fn star_update_animates_with_strict_delay_and_wraps_four_frames() {
        let constants = LegacyPowerUpConstants::default();
        let mut state = LegacyStarState::spawn(10.0, 5.0, constants);
        state.up_timer = constants.emergence_time;
        state.is_static = false;
        state.animation_timer = constants.star_animation_delay;

        let _ = update_legacy_star(&mut state, constants, 0.0);
        assert_eq!(state.frame, 1);
        assert_eq!(state.animation_timer, constants.star_animation_delay);

        let _ = update_legacy_star(&mut state, constants, 0.001);
        assert_eq!(state.frame, 2);
        assert!((state.animation_timer - 0.001).abs() < f32::EPSILON);

        state.frame = 4;
        state.animation_timer = 0.0;
        let _ = update_legacy_star(
            &mut state,
            constants,
            constants.star_animation_delay * 2.0 + 0.001,
        );
        assert_eq!(state.frame, 2);
        assert!((state.animation_timer - 0.001).abs() < f32::EPSILON);
    }

    #[test]
    fn star_update_aligns_rotation_and_reports_destroy_removal() {
        let constants = LegacyPowerUpConstants::default();
        let mut state = LegacyStarState::spawn(10.0, 5.0, constants);
        state.rotation = core::f32::consts::TAU + 1.0;
        state.destroy = true;

        assert_eq!(
            update_legacy_star(&mut state, constants, 0.05),
            LegacyPowerUpUpdate { remove: true }
        );
        assert_eq!(
            state.rotation,
            1.0 - constants.rotation_alignment_speed * 0.05
        );
    }

    #[test]
    fn star_side_collisions_bounce_and_collect_player_while_suppressing_default() {
        let constants = LegacyPowerUpConstants::default();
        let mut left = LegacyStarState::spawn(10.0, 5.0, constants);

        assert_eq!(
            legacy_star_left_collision(&mut left, LegacyPowerUpCollisionActor::Other, constants),
            LegacyStarCollision {
                suppress_default: true,
                collection: LegacyStarCollection::None
            }
        );
        assert_eq!(left.speed_x, constants.mushroom_speed);
        assert!(!left.destroy);

        let mut right = LegacyStarState::spawn(10.0, 5.0, constants);
        assert_eq!(
            legacy_star_right_collision(&mut right, LegacyPowerUpCollisionActor::Player, constants),
            LegacyStarCollision {
                suppress_default: true,
                collection: LegacyStarCollection::GrantStarPower
            }
        );
        assert_eq!(right.speed_x, -constants.mushroom_speed);
        assert!(right.destroy);
    }

    #[test]
    fn star_floor_collision_bounces_and_only_collects_when_active() {
        let constants = LegacyPowerUpConstants::default();
        let mut inactive = LegacyStarState::spawn(10.0, 5.0, constants);
        inactive.active = false;

        assert_eq!(
            legacy_star_floor_collision(
                &mut inactive,
                LegacyPowerUpCollisionActor::Player,
                constants
            ),
            LegacyStarCollision {
                suppress_default: true,
                collection: LegacyStarCollection::None
            }
        );
        assert_eq!(inactive.speed_y, -constants.star_jump_force);
        assert!(!inactive.destroy);

        let mut active = LegacyStarState::spawn(10.0, 5.0, constants);
        assert_eq!(
            legacy_star_floor_collision(
                &mut active,
                LegacyPowerUpCollisionActor::Player,
                constants
            ),
            LegacyStarCollision {
                suppress_default: true,
                collection: LegacyStarCollection::GrantStarPower
            }
        );
        assert_eq!(active.speed_y, -constants.star_jump_force);
        assert!(active.destroy);
    }

    #[test]
    fn star_ceiling_collision_collects_only_when_active_without_suppressing_default() {
        let constants = LegacyPowerUpConstants::default();
        let mut active = LegacyStarState::spawn(10.0, 5.0, constants);

        assert_eq!(
            legacy_star_ceiling_collision(&mut active, LegacyPowerUpCollisionActor::Player),
            LegacyStarCollision {
                suppress_default: false,
                collection: LegacyStarCollection::GrantStarPower
            }
        );
        assert!(active.destroy);

        let mut inactive = LegacyStarState::spawn(10.0, 5.0, constants);
        inactive.active = false;
        assert_eq!(
            legacy_star_ceiling_collision(&mut inactive, LegacyPowerUpCollisionActor::Player),
            LegacyStarCollision {
                suppress_default: false,
                collection: LegacyStarCollection::None
            }
        );
        assert!(!inactive.destroy);
    }

    #[test]
    fn star_jump_uses_mushroom_jump_force_and_direction_from_source_x() {
        let constants = LegacyPowerUpConstants::default();
        let mut state = LegacyStarState::spawn(10.0, 5.0, constants);

        apply_legacy_star_jump(&mut state, constants, 12.0);
        assert!(state.falling);
        assert_eq!(state.speed_y, -constants.mushroom_jump_force);
        assert_eq!(state.speed_x, -constants.mushroom_speed);

        apply_legacy_star_jump(&mut state, constants, 9.0);
        assert_eq!(state.speed_x, constants.mushroom_speed);

        let mut equal = LegacyStarState::spawn(10.0, 5.0, constants);
        equal.speed_x = 1.25;
        let equal_source_x = equal.x + equal.width / 2.0 + 0.5;
        apply_legacy_star_jump(&mut equal, constants, equal_source_x);
        assert_eq!(equal.speed_x, 1.25);
    }

    #[test]
    fn flower_spawn_matches_legacy_body_animation_and_gravity() {
        let constants = LegacyPowerUpConstants::default();
        let state = LegacyFlowerState::spawn(10.0, 5.0, constants);

        assert_eq!(state.start_y, 5.0);
        assert_eq!(state.x, 10.0 - 6.0 / 16.0);
        assert_eq!(state.y, 5.0 - 11.0 / 16.0);
        assert_eq!(state.speed_x, 0.0);
        assert_eq!(state.speed_y, 0.0);
        assert_eq!(state.width, 12.0 / 16.0);
        assert_eq!(state.height, 12.0 / 16.0);
        assert!(state.is_static);
        assert!(state.active);
        assert!(!state.drawable);
        assert!(!state.destroy);
        assert_eq!(state.gravity, constants.flower_gravity);
        assert_eq!(state.rotation, 0.0);
        assert_eq!(state.up_timer, 0.0);
        assert_eq!(state.animation_timer, 0.0);
        assert_eq!(state.frame, 1);
        assert!(!state.falling);
    }

    #[test]
    fn flower_update_emerges_with_strict_greater_than_snap() {
        let constants = LegacyPowerUpConstants::default();
        let mut exact = LegacyFlowerState::spawn(10.0, 5.0, constants);

        assert_eq!(
            update_legacy_flower(&mut exact, constants, constants.emergence_time),
            LegacyPowerUpUpdate { remove: false }
        );
        assert_eq!(exact.up_timer, constants.emergence_time);
        assert_eq!(
            exact.y,
            5.0 - 11.0 / 16.0 - constants.emergence_time * (1.0 / constants.emergence_time)
        );
        assert!(exact.is_static);
        assert!(exact.active);
        assert!(!exact.drawable);

        let _ = update_legacy_flower(&mut exact, constants, 0.1);
        assert_eq!(exact.up_timer, constants.emergence_time);
        assert!(!exact.drawable);

        let mut crossed = LegacyFlowerState::spawn(10.0, 5.0, constants);
        crossed.up_timer = constants.emergence_time - 0.01;

        let _ = update_legacy_flower(&mut crossed, constants, 0.02);
        assert!(crossed.up_timer > constants.emergence_time);
        assert_eq!(
            crossed.y,
            crossed.start_y - constants.flower_emerged_y_offset
        );
        assert!(crossed.active);
        assert!(crossed.drawable);
        assert_eq!(crossed.speed_x, 0.0);
        assert_eq!(crossed.speed_y, 0.0);
        assert!(crossed.is_static);
    }

    #[test]
    fn flower_update_animates_with_strict_delay_and_wraps_four_frames() {
        let constants = LegacyPowerUpConstants::default();
        let mut state = LegacyFlowerState::spawn(10.0, 5.0, constants);
        state.up_timer = constants.emergence_time;
        state.animation_timer = constants.star_animation_delay;

        let _ = update_legacy_flower(&mut state, constants, 0.0);
        assert_eq!(state.frame, 1);
        assert_eq!(state.animation_timer, constants.star_animation_delay);

        let _ = update_legacy_flower(&mut state, constants, 0.001);
        assert_eq!(state.frame, 2);
        assert!((state.animation_timer - 0.001).abs() < f32::EPSILON);

        state.frame = 4;
        state.animation_timer = 0.0;
        let _ = update_legacy_flower(
            &mut state,
            constants,
            constants.star_animation_delay * 2.0 + 0.001,
        );
        assert_eq!(state.frame, 2);
        assert!((state.animation_timer - 0.001).abs() < f32::EPSILON);
    }

    #[test]
    fn flower_update_reports_destroy_removal_without_rotation_alignment() {
        let constants = LegacyPowerUpConstants::default();
        let mut state = LegacyFlowerState::spawn(10.0, 5.0, constants);
        state.rotation = core::f32::consts::TAU + 1.0;
        state.destroy = true;

        assert_eq!(
            update_legacy_flower(&mut state, constants, 0.05),
            LegacyPowerUpUpdate { remove: true }
        );
        assert_eq!(state.rotation, core::f32::consts::TAU + 1.0);
    }

    #[test]
    fn flower_side_collisions_collect_player_and_suppress_default() {
        let constants = LegacyPowerUpConstants::default();
        let mut left = LegacyFlowerState::spawn(10.0, 5.0, constants);

        assert_eq!(
            legacy_flower_left_collision(&mut left, LegacyPowerUpCollisionActor::Other),
            LegacyFlowerCollision {
                suppress_default: true,
                collection: LegacyFlowerCollection::None
            }
        );
        assert!(!left.destroy);

        let mut right = LegacyFlowerState::spawn(10.0, 5.0, constants);
        assert_eq!(
            legacy_flower_right_collision(&mut right, LegacyPowerUpCollisionActor::Player),
            LegacyFlowerCollision {
                suppress_default: true,
                collection: LegacyFlowerCollection::GrowPlayer
            }
        );
        assert!(!right.active);
        assert!(right.destroy);
        assert!(!right.drawable);
    }

    #[test]
    fn flower_floor_and_ceiling_collect_only_when_active_without_suppressing_default() {
        let constants = LegacyPowerUpConstants::default();
        let mut inactive = LegacyFlowerState::spawn(10.0, 5.0, constants);
        inactive.active = false;

        assert_eq!(
            legacy_flower_floor_collision(&mut inactive, LegacyPowerUpCollisionActor::Player),
            LegacyFlowerCollision {
                suppress_default: false,
                collection: LegacyFlowerCollection::None
            }
        );
        assert!(!inactive.destroy);

        let mut ceiling = LegacyFlowerState::spawn(10.0, 5.0, constants);
        assert_eq!(
            legacy_flower_ceiling_collision(&mut ceiling, LegacyPowerUpCollisionActor::Player),
            LegacyFlowerCollision {
                suppress_default: false,
                collection: LegacyFlowerCollection::GrowPlayer
            }
        );
        assert!(!ceiling.active);
        assert!(ceiling.destroy);
        assert!(!ceiling.drawable);
    }

    #[test]
    fn flower_jump_is_no_op() {
        let constants = LegacyPowerUpConstants::default();
        let mut state = LegacyFlowerState::spawn(10.0, 5.0, constants);
        state.speed_x = 1.25;
        state.speed_y = -2.5;
        state.falling = true;
        let before = state;

        apply_legacy_flower_jump(&mut state, 12.0);

        assert_eq!(state, before);
    }
}
