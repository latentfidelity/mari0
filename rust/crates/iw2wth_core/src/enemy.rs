//! Deterministic legacy enemy state machines.

use crate::config::{
    LegacyBulletBillConstants, LegacyCheepCheepConstants, LegacyFlyingFishConstants,
    LegacyGoombaConstants, LegacyHammerBroConstants, LegacyHammerConstants, LegacyKoopaConstants,
    LegacyLakitoConstants, LegacyPlantConstants, LegacySquidConstants,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyEnemyDirection {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyEnemyUpdate {
    Keep,
    Remove,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyEnemyCollisionResponse {
    pub suppress_default: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyCheepCheepColor {
    Red,
    White,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyCheepCheepFrame {
    One,
    Two,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyCheepCheepVerticalDirection {
    Up,
    Down,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyCheepCheepLifecycle {
    Swimming,
    Shot,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyCheepCheepState {
    pub color: LegacyCheepCheepColor,
    pub x: f32,
    pub y: f32,
    pub start_y: f32,
    pub speed_x: f32,
    pub speed_y: f32,
    pub width: f32,
    pub height: f32,
    pub active: bool,
    pub gravity: f32,
    pub rotation: f32,
    pub animation_timer: f32,
    pub frame: Option<LegacyCheepCheepFrame>,
    pub vertical_moving: bool,
    pub vertical_direction: LegacyCheepCheepVerticalDirection,
    pub lifecycle: LegacyCheepCheepLifecycle,
}

impl LegacyCheepCheepState {
    #[must_use]
    pub fn spawn(
        x: f32,
        y: f32,
        color: LegacyCheepCheepColor,
        vertical_moving: bool,
        constants: LegacyCheepCheepConstants,
    ) -> Self {
        Self {
            color,
            x,
            y,
            start_y: y,
            speed_x: match color {
                LegacyCheepCheepColor::Red => -constants.red_speed,
                LegacyCheepCheepColor::White => -constants.white_speed,
            },
            speed_y: 0.0,
            width: 12.0 / 16.0,
            height: 12.0 / 16.0,
            active: true,
            gravity: 0.0,
            rotation: 0.0,
            animation_timer: 0.0,
            frame: None,
            vertical_moving,
            vertical_direction: LegacyCheepCheepVerticalDirection::Up,
            lifecycle: LegacyCheepCheepLifecycle::Swimming,
        }
    }
}

#[must_use]
pub fn update_legacy_cheep_cheep(
    state: &mut LegacyCheepCheepState,
    constants: LegacyCheepCheepConstants,
    dt: f32,
) -> LegacyEnemyUpdate {
    align_legacy_enemy_rotation(&mut state.rotation, constants.rotation_alignment_speed, dt);

    if state.lifecycle == LegacyCheepCheepLifecycle::Shot {
        state.speed_y += constants.shot_gravity * dt;
        state.x += state.speed_x * dt;
        state.y += state.speed_y * dt;
        return LegacyEnemyUpdate::Keep;
    }

    advance_legacy_cheep_cheep_animation(state, constants, dt);

    if state.vertical_moving {
        match state.vertical_direction {
            LegacyCheepCheepVerticalDirection::Up => {
                state.speed_y = -constants.vertical_speed;
                if state.y < state.start_y - constants.vertical_range {
                    state.vertical_direction = LegacyCheepCheepVerticalDirection::Down;
                }
            }
            LegacyCheepCheepVerticalDirection::Down => {
                state.speed_y = constants.vertical_speed;
                if state.y > state.start_y + constants.vertical_range {
                    state.vertical_direction = LegacyCheepCheepVerticalDirection::Up;
                }
            }
        }
    }

    LegacyEnemyUpdate::Keep
}

pub fn shoot_legacy_cheep_cheep(
    state: &mut LegacyCheepCheepState,
    constants: LegacyCheepCheepConstants,
    direction: Option<LegacyEnemyDirection>,
) {
    let direction = direction.unwrap_or(LegacyEnemyDirection::Right);

    state.lifecycle = LegacyCheepCheepLifecycle::Shot;
    state.active = false;
    state.gravity = constants.shot_gravity;
    state.speed_y = -constants.shot_jump_force;
    state.speed_x = match direction {
        LegacyEnemyDirection::Left => -constants.shot_speed_x,
        LegacyEnemyDirection::Right => constants.shot_speed_x,
    };
}

#[must_use]
pub const fn legacy_cheep_cheep_collision() -> LegacyEnemyCollisionResponse {
    LegacyEnemyCollisionResponse {
        suppress_default: true,
    }
}

fn advance_legacy_cheep_cheep_animation(
    state: &mut LegacyCheepCheepState,
    constants: LegacyCheepCheepConstants,
    dt: f32,
) {
    state.animation_timer += dt;

    while state.animation_timer > constants.animation_speed {
        state.animation_timer -= constants.animation_speed;
        state.frame = match state.frame {
            Some(LegacyCheepCheepFrame::One) => Some(LegacyCheepCheepFrame::Two),
            _ => Some(LegacyCheepCheepFrame::One),
        };
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacySquidFrame {
    Open,
    Closed,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LegacySquidMotion {
    Idle,
    Upward { start_x: f32 },
    Downward { start_y: f32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacySquidLifecycle {
    Swimming,
    Shot,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacySquidPlayerTarget {
    pub x: f32,
    pub y: f32,
    pub height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacySquidState {
    pub x: f32,
    pub y: f32,
    pub speed_x: f32,
    pub speed_y: f32,
    pub width: f32,
    pub height: f32,
    pub active: bool,
    pub gravity: f32,
    pub rotation: f32,
    pub direction: LegacyEnemyDirection,
    pub frame: LegacySquidFrame,
    pub motion: LegacySquidMotion,
    pub lifecycle: LegacySquidLifecycle,
}

impl LegacySquidState {
    #[must_use]
    pub fn spawn(tile_x: f32, tile_y: f32) -> Self {
        Self {
            x: tile_x - 1.0 + 2.0 / 16.0,
            y: tile_y - 1.0 + 4.0 / 16.0,
            speed_x: 0.0,
            speed_y: 0.0,
            width: 12.0 / 16.0,
            height: 12.0 / 16.0,
            active: true,
            gravity: 0.0,
            rotation: 0.0,
            direction: LegacyEnemyDirection::Left,
            frame: LegacySquidFrame::Open,
            motion: LegacySquidMotion::Idle,
            lifecycle: LegacySquidLifecycle::Swimming,
        }
    }
}

#[must_use]
pub fn update_legacy_squid(
    state: &mut LegacySquidState,
    constants: LegacySquidConstants,
    dt: f32,
    players: &[LegacySquidPlayerTarget],
) -> LegacyEnemyUpdate {
    align_legacy_enemy_rotation(&mut state.rotation, constants.rotation_alignment_speed, dt);

    if state.lifecycle == LegacySquidLifecycle::Shot {
        state.speed_y += constants.shot_gravity * dt;
        state.x += state.speed_x * dt;
        state.y += state.speed_y * dt;
        return LegacyEnemyUpdate::Keep;
    }

    match state.motion {
        LegacySquidMotion::Idle => {
            state.speed_y = constants.fall_speed;
            if let Some(player) = nearest_legacy_squid_player(state.x, players) {
                let next_bottom = state.y + state.speed_y * dt + state.height + 1.0 / 16.0;
                let trigger_y = player.y - (24.0 / 16.0 - player.height);
                if next_bottom >= trigger_y {
                    state.motion = LegacySquidMotion::Upward { start_x: state.x };
                    state.speed_x = 0.0;
                    state.speed_y = 0.0;
                    turn_legacy_squid_toward_player(state, player.x);
                }
            }
        }
        LegacySquidMotion::Upward { start_x } => {
            match state.direction {
                LegacyEnemyDirection::Right => {
                    state.speed_x = (state.speed_x + constants.acceleration * dt)
                        .min(constants.horizontal_speed);
                }
                LegacyEnemyDirection::Left => {
                    state.speed_x = (state.speed_x - constants.acceleration * dt)
                        .max(-constants.horizontal_speed);
                }
            }

            state.speed_y = (state.speed_y - constants.acceleration * dt).max(-constants.up_speed);

            if (state.x - start_x).abs() >= 2.0 {
                state.motion = LegacySquidMotion::Downward { start_y: state.y };
                state.frame = LegacySquidFrame::Closed;
                state.speed_x = 0.0;
            }
        }
        LegacySquidMotion::Downward { start_y } => {
            state.speed_y = constants.fall_speed;
            if state.y > start_y + constants.down_distance {
                state.motion = LegacySquidMotion::Idle;
                state.frame = LegacySquidFrame::Open;
            }
        }
    }

    state.x += state.speed_x * dt;
    state.y += state.speed_y * dt;

    LegacyEnemyUpdate::Keep
}

pub fn shoot_legacy_squid(
    state: &mut LegacySquidState,
    constants: LegacySquidConstants,
    direction: Option<LegacyEnemyDirection>,
) {
    let direction = direction.unwrap_or(LegacyEnemyDirection::Right);

    state.lifecycle = LegacySquidLifecycle::Shot;
    state.active = false;
    state.gravity = constants.shot_gravity;
    state.speed_y = -constants.shot_jump_force;
    state.direction = direction;
    state.speed_x = match direction {
        LegacyEnemyDirection::Left => -constants.shot_speed_x,
        LegacyEnemyDirection::Right => constants.shot_speed_x,
    };
}

#[must_use]
pub const fn legacy_squid_collision() -> LegacyEnemyCollisionResponse {
    LegacyEnemyCollisionResponse {
        suppress_default: true,
    }
}

fn nearest_legacy_squid_player(
    squid_x: f32,
    players: &[LegacySquidPlayerTarget],
) -> Option<&LegacySquidPlayerTarget> {
    let mut nearest = players.first()?;

    for player in &players[1..] {
        if (squid_x - player.x).abs() < (squid_x - nearest.x).abs() {
            nearest = player;
        }
    }

    Some(nearest)
}

fn turn_legacy_squid_toward_player(state: &mut LegacySquidState, player_x: f32) {
    match state.direction {
        LegacyEnemyDirection::Right if state.x > player_x => {
            state.direction = LegacyEnemyDirection::Left;
        }
        LegacyEnemyDirection::Left if state.x < player_x => {
            state.direction = LegacyEnemyDirection::Right;
        }
        LegacyEnemyDirection::Left | LegacyEnemyDirection::Right => {}
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyLakitoLifecycle {
    Active,
    Shot,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyLakitoPlayerTarget {
    pub x: f32,
    pub speed_x: f32,
    pub dead: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyLakitoState {
    pub start_y: f32,
    pub x: f32,
    pub y: f32,
    pub speed_x: f32,
    pub speed_y: f32,
    pub width: f32,
    pub height: f32,
    pub active: bool,
    pub gravity: f32,
    pub passive: bool,
    pub direction: LegacyEnemyDirection,
    pub timer: f32,
    pub lifecycle: LegacyLakitoLifecycle,
}

impl LegacyLakitoState {
    #[must_use]
    pub fn spawn(tile_x: f32, tile_y: f32) -> Self {
        Self {
            start_y: tile_y,
            x: tile_x - 1.0 + 2.0 / 16.0,
            y: tile_y - 12.0 / 16.0,
            speed_x: 0.0,
            speed_y: 0.0,
            width: 12.0 / 16.0,
            height: 12.0 / 16.0,
            active: true,
            gravity: 0.0,
            passive: false,
            direction: LegacyEnemyDirection::Left,
            timer: 0.0,
            lifecycle: LegacyLakitoLifecycle::Active,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LegacyLakitoUpdate {
    Idle,
    ThrowSpikey { x: f32, y: f32 },
    Respawned,
}

#[must_use]
pub fn update_legacy_lakito(
    state: &mut LegacyLakitoState,
    constants: LegacyLakitoConstants,
    dt: f32,
    current_spikey_count: usize,
    players: &[LegacyLakitoPlayerTarget],
    lakito_end: bool,
    respawn_x: f32,
) -> LegacyLakitoUpdate {
    let update = if state.lifecycle == LegacyLakitoLifecycle::Shot {
        update_shot_legacy_lakito(state, constants, dt, respawn_x)
    } else if state.passive {
        state.speed_x = -constants.passive_speed;
        LegacyLakitoUpdate::Idle
    } else {
        update_active_legacy_lakito(state, constants, dt, current_spikey_count, players)
    };

    if lakito_end {
        state.passive = true;
    }

    update
}

pub fn shoot_legacy_lakito(state: &mut LegacyLakitoState, constants: LegacyLakitoConstants) {
    state.lifecycle = LegacyLakitoLifecycle::Shot;
    state.speed_y = -constants.shot_jump_force;
    state.direction = LegacyEnemyDirection::Right;
    state.active = false;
    state.gravity = constants.shot_gravity;
    state.speed_x = 0.0;
    state.timer = 0.0;
}

pub fn stomp_legacy_lakito(state: &mut LegacyLakitoState, constants: LegacyLakitoConstants) {
    shoot_legacy_lakito(state, constants);
    state.speed_y = 0.0;
}

pub fn legacy_lakito_spikeyfall_collision(
    state: &mut LegacyLakitoState,
    constants: LegacyLakitoConstants,
) -> LegacyEnemyCollisionResponse {
    shoot_legacy_lakito(state, constants);
    legacy_lakito_collision()
}

#[must_use]
pub const fn legacy_lakito_collision() -> LegacyEnemyCollisionResponse {
    LegacyEnemyCollisionResponse {
        suppress_default: true,
    }
}

fn update_shot_legacy_lakito(
    state: &mut LegacyLakitoState,
    constants: LegacyLakitoConstants,
    dt: f32,
    respawn_x: f32,
) -> LegacyLakitoUpdate {
    state.speed_y += constants.shot_gravity * dt;
    state.x += state.speed_x * dt;
    state.y += state.speed_y * dt;

    if !state.passive {
        state.timer += dt;
        if state.timer > constants.respawn_time {
            state.y = state.start_y - 12.0 / 16.0;
            state.x = respawn_x;
            state.timer = 0.0;
            state.lifecycle = LegacyLakitoLifecycle::Active;
            state.active = true;
            state.gravity = 0.0;
            state.speed_y = 0.0;
            state.speed_x = 0.0;
            return LegacyLakitoUpdate::Respawned;
        }
    }

    LegacyLakitoUpdate::Idle
}

fn update_active_legacy_lakito(
    state: &mut LegacyLakitoState,
    constants: LegacyLakitoConstants,
    dt: f32,
    current_spikey_count: usize,
    players: &[LegacyLakitoPlayerTarget],
) -> LegacyLakitoUpdate {
    state.timer += dt;

    let update = if current_spikey_count < constants.max_spikey_count
        && state.timer > constants.throw_time
    {
        state.timer = 0.0;
        LegacyLakitoUpdate::ThrowSpikey {
            x: state.x + 6.0 / 16.0,
            y: state.y,
        }
    } else {
        LegacyLakitoUpdate::Idle
    };

    if let Some(nearest_player_x) =
        nearest_legacy_lakito_player_probe_x(players, constants.distance_time, state.x)
    {
        let distance = (state.x - nearest_player_x).abs();
        match state.direction {
            LegacyEnemyDirection::Left if state.x < nearest_player_x - constants.space => {
                state.direction = LegacyEnemyDirection::Right;
            }
            LegacyEnemyDirection::Right if state.x > nearest_player_x + constants.space => {
                state.direction = LegacyEnemyDirection::Left;
            }
            LegacyEnemyDirection::Left | LegacyEnemyDirection::Right => {}
        }

        state.speed_x = match state.direction {
            LegacyEnemyDirection::Right => lua_round((distance - 3.0) * 2.0).max(2.0),
            LegacyEnemyDirection::Left => -2.0,
        };
    }

    update
}

fn nearest_legacy_lakito_player_probe_x(
    players: &[LegacyLakitoPlayerTarget],
    distance_time: f32,
    lakito_x: f32,
) -> Option<f32> {
    let first = players.first()?;
    let mut nearest_probe_x = first.x + first.speed_x * distance_time;

    for player in &players[1..] {
        let probe_x = player.x + player.speed_x * distance_time;
        if (lakito_x - probe_x).abs() < nearest_probe_x && !player.dead {
            nearest_probe_x = probe_x;
        }
    }

    Some(nearest_probe_x)
}

fn lua_round(value: f32) -> f32 {
    (value + 0.5).floor()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyHammerFrame {
    One,
    Two,
    Three,
    Four,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyHammerState {
    pub x: f32,
    pub y: f32,
    pub start_y: f32,
    pub speed_x: f32,
    pub speed_y: f32,
    pub width: f32,
    pub height: f32,
    pub active: bool,
    pub gravity: f32,
    pub animation_direction: LegacyEnemyDirection,
    pub animation_timer: f32,
    pub frame: LegacyHammerFrame,
    pub kill_stuff: bool,
    pub hammer_bro_collision_mask_active: bool,
}

impl LegacyHammerState {
    #[must_use]
    pub fn spawn(
        x: f32,
        y: f32,
        direction: LegacyEnemyDirection,
        constants: LegacyHammerConstants,
    ) -> Self {
        let y = y - 1.0;

        Self {
            x,
            y,
            start_y: y,
            speed_x: match direction {
                LegacyEnemyDirection::Left => -constants.speed,
                LegacyEnemyDirection::Right => constants.speed,
            },
            speed_y: -constants.start_y_speed,
            width: 12.0 / 16.0,
            height: 12.0 / 16.0,
            active: true,
            gravity: constants.gravity,
            animation_direction: direction,
            animation_timer: 0.0,
            frame: LegacyHammerFrame::One,
            kill_stuff: false,
            hammer_bro_collision_mask_active: false,
        }
    }
}

#[must_use]
pub fn update_legacy_hammer(
    state: &mut LegacyHammerState,
    constants: LegacyHammerConstants,
    dt: f32,
) -> LegacyEnemyUpdate {
    state.animation_timer += dt;

    while state.animation_timer > constants.animation_speed {
        state.frame = next_legacy_hammer_frame(state.frame);
        state.animation_timer -= constants.animation_speed;
    }

    if state.hammer_bro_collision_mask_active && state.y > state.start_y + 1.0 {
        state.hammer_bro_collision_mask_active = false;
    }

    LegacyEnemyUpdate::Keep
}

pub fn portal_legacy_hammer(state: &mut LegacyHammerState) {
    state.kill_stuff = true;
}

#[must_use]
pub const fn legacy_hammer_collision() -> LegacyEnemyCollisionResponse {
    LegacyEnemyCollisionResponse {
        suppress_default: true,
    }
}

const fn next_legacy_hammer_frame(frame: LegacyHammerFrame) -> LegacyHammerFrame {
    match frame {
        LegacyHammerFrame::One => LegacyHammerFrame::Two,
        LegacyHammerFrame::Two => LegacyHammerFrame::Three,
        LegacyHammerFrame::Three => LegacyHammerFrame::Four,
        LegacyHammerFrame::Four => LegacyHammerFrame::One,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyHammerBroFrame {
    One,
    Two,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyHammerBroJumpState {
    None,
    Up,
    Down,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyHammerBroJumpDecision {
    Up,
    Down,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyHammerBroLifecycle {
    Active,
    Shot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyHammerBroCollisionActor {
    Other,
    MovingKoopaShell,
    BulletBill,
    KillHammer,
    PlayerOrBox,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LegacyHammerBroUpdate {
    Idle,
    ThrowHammer { direction: LegacyEnemyDirection },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyHammerBroState {
    pub start_x: f32,
    pub start_y: f32,
    pub x: f32,
    pub y: f32,
    pub speed_x: f32,
    pub speed_y: f32,
    pub width: f32,
    pub height: f32,
    pub active: bool,
    pub gravity: f32,
    pub rotation: f32,
    pub direction: LegacyEnemyDirection,
    pub animation_direction: LegacyEnemyDirection,
    pub animation_timer: f32,
    pub frame: LegacyHammerBroFrame,
    pub preparing_throw: bool,
    pub falling: bool,
    pub throw_timer: f32,
    pub jump_timer: f32,
    pub jumping: LegacyHammerBroJumpState,
    pub jumping_y: f32,
    pub floor_collision_mask_active: bool,
    pub lifecycle: LegacyHammerBroLifecycle,
}

impl LegacyHammerBroState {
    #[must_use]
    pub fn spawn(
        tile_x: f32,
        tile_y: f32,
        initial_throw_timer: f32,
        constants: LegacyHammerBroConstants,
    ) -> Self {
        Self {
            start_x: tile_x,
            start_y: tile_y,
            x: tile_x - 6.0 / 16.0,
            y: tile_y - 12.0 / 16.0,
            speed_x: -constants.speed,
            speed_y: 0.0,
            width: 12.0 / 16.0,
            height: 12.0 / 16.0,
            active: true,
            gravity: constants.gravity,
            rotation: 0.0,
            direction: LegacyEnemyDirection::Left,
            animation_direction: LegacyEnemyDirection::Left,
            animation_timer: 0.0,
            frame: LegacyHammerBroFrame::One,
            preparing_throw: false,
            falling: false,
            throw_timer: initial_throw_timer,
            jump_timer: 0.0,
            jumping: LegacyHammerBroJumpState::None,
            jumping_y: 0.0,
            floor_collision_mask_active: false,
            lifecycle: LegacyHammerBroLifecycle::Active,
        }
    }
}

#[must_use]
pub fn update_legacy_hammer_bro_active(
    state: &mut LegacyHammerBroState,
    constants: LegacyHammerBroConstants,
    dt: f32,
    player_xs: &[f32],
    random_jump_direction: LegacyHammerBroJumpDecision,
    next_throw_timer: impl FnOnce() -> f32,
) -> LegacyHammerBroUpdate {
    align_legacy_enemy_rotation(&mut state.rotation, constants.rotation_alignment_speed, dt);

    if state.lifecycle != LegacyHammerBroLifecycle::Active {
        return LegacyHammerBroUpdate::Idle;
    }

    apply_legacy_hammer_bro_patrol_bounds(state, constants);
    let update = update_legacy_hammer_bro_throw_timer(state, dt, next_throw_timer);
    update_legacy_hammer_bro_jump_timer(state, constants, dt, random_jump_direction);
    clear_legacy_hammer_bro_finished_jump(state);
    turn_legacy_hammer_bro_toward_nearest_player(state, player_xs);
    advance_legacy_hammer_bro_animation(state, constants, dt);
    ease_legacy_hammer_bro_speed(state, constants, dt);

    update
}

#[must_use]
pub fn update_legacy_hammer_bro_shot(
    state: &mut LegacyHammerBroState,
    constants: LegacyHammerBroConstants,
    dt: f32,
) -> LegacyEnemyUpdate {
    align_legacy_enemy_rotation(&mut state.rotation, constants.rotation_alignment_speed, dt);

    if state.lifecycle != LegacyHammerBroLifecycle::Shot {
        return LegacyEnemyUpdate::Keep;
    }

    state.speed_y += constants.shot_gravity * dt;
    state.x += state.speed_x * dt;
    state.y += state.speed_y * dt;

    if state.y > constants.shot_removal_y {
        LegacyEnemyUpdate::Remove
    } else {
        LegacyEnemyUpdate::Keep
    }
}

pub fn shoot_legacy_hammer_bro(
    state: &mut LegacyHammerBroState,
    constants: LegacyHammerBroConstants,
) {
    state.lifecycle = LegacyHammerBroLifecycle::Shot;
    state.speed_y = -constants.shot_jump_force;
    state.direction = LegacyEnemyDirection::Right;
    state.active = false;
    state.gravity = constants.shot_gravity;
    state.speed_x = 0.0;
}

pub fn stomp_legacy_hammer_bro(
    state: &mut LegacyHammerBroState,
    constants: LegacyHammerBroConstants,
) {
    shoot_legacy_hammer_bro(state, constants);
    state.speed_y = 0.0;
}

pub fn portal_legacy_hammer_bro(state: &mut LegacyHammerBroState) {
    state.jumping = LegacyHammerBroJumpState::None;
    state.floor_collision_mask_active = false;
}

pub fn emancipate_legacy_hammer_bro(
    state: &mut LegacyHammerBroState,
    constants: LegacyHammerBroConstants,
) {
    shoot_legacy_hammer_bro(state, constants);
}

pub fn legacy_hammer_bro_left_collision(
    state: &mut LegacyHammerBroState,
    constants: LegacyHammerBroConstants,
    actor: LegacyHammerBroCollisionActor,
) -> LegacyEnemyCollisionResponse {
    state.speed_x = constants.side_collision_speed;
    if legacy_hammer_bro_actor_shoots(actor) {
        shoot_legacy_hammer_bro(state, constants);
    }

    LegacyEnemyCollisionResponse {
        suppress_default: true,
    }
}

pub fn legacy_hammer_bro_right_collision(
    state: &mut LegacyHammerBroState,
    constants: LegacyHammerBroConstants,
    actor: LegacyHammerBroCollisionActor,
) -> LegacyEnemyCollisionResponse {
    state.speed_x = -constants.side_collision_speed;
    if legacy_hammer_bro_actor_shoots(actor) {
        shoot_legacy_hammer_bro(state, constants);
    }

    LegacyEnemyCollisionResponse {
        suppress_default: true,
    }
}

pub fn legacy_hammer_bro_ceil_collision(
    state: &mut LegacyHammerBroState,
    constants: LegacyHammerBroConstants,
    actor: LegacyHammerBroCollisionActor,
) -> LegacyEnemyCollisionResponse {
    if actor == LegacyHammerBroCollisionActor::PlayerOrBox {
        stomp_legacy_hammer_bro(state, constants);
    } else if legacy_hammer_bro_actor_shoots(actor) {
        shoot_legacy_hammer_bro(state, constants);
    }

    LegacyEnemyCollisionResponse {
        suppress_default: false,
    }
}

pub fn legacy_hammer_bro_floor_collision(
    state: &mut LegacyHammerBroState,
    constants: LegacyHammerBroConstants,
    actor: LegacyHammerBroCollisionActor,
) -> LegacyEnemyCollisionResponse {
    if matches!(
        actor,
        LegacyHammerBroCollisionActor::BulletBill | LegacyHammerBroCollisionActor::KillHammer
    ) {
        shoot_legacy_hammer_bro(state, constants);
    }

    LegacyEnemyCollisionResponse {
        suppress_default: false,
    }
}

const fn legacy_hammer_bro_actor_shoots(actor: LegacyHammerBroCollisionActor) -> bool {
    matches!(
        actor,
        LegacyHammerBroCollisionActor::MovingKoopaShell
            | LegacyHammerBroCollisionActor::BulletBill
            | LegacyHammerBroCollisionActor::KillHammer
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyFlyingFishFrame {
    One,
    Two,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyFlyingFishLifecycle {
    Flying,
    Shot,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyFlyingFishState {
    pub x: f32,
    pub y: f32,
    pub speed_x: f32,
    pub speed_y: f32,
    pub width: f32,
    pub height: f32,
    pub active: bool,
    pub gravity: f32,
    pub rotation: f32,
    pub animation_direction: LegacyEnemyDirection,
    pub animation_timer: f32,
    pub frame: Option<LegacyFlyingFishFrame>,
    pub direction: LegacyEnemyDirection,
    pub lifecycle: LegacyFlyingFishLifecycle,
}

impl LegacyFlyingFishState {
    #[must_use]
    pub fn spawn(
        x: f32,
        player_speed_x: f32,
        random_speed_delta: f32,
        constants: LegacyFlyingFishConstants,
    ) -> Self {
        let mut speed_x = player_speed_x + random_speed_delta;
        if speed_x == 0.0 {
            speed_x = 1.0;
        }

        Self {
            x,
            y: 15.0,
            speed_x,
            speed_y: -constants.jump_force,
            width: 12.0 / 16.0,
            height: 12.0 / 16.0,
            active: true,
            gravity: constants.gravity,
            rotation: 0.0,
            animation_direction: if speed_x > 0.0 {
                LegacyEnemyDirection::Left
            } else {
                LegacyEnemyDirection::Right
            },
            animation_timer: 0.0,
            frame: None,
            direction: LegacyEnemyDirection::Right,
            lifecycle: LegacyFlyingFishLifecycle::Flying,
        }
    }
}

#[must_use]
pub fn update_legacy_flying_fish(
    state: &mut LegacyFlyingFishState,
    constants: LegacyFlyingFishConstants,
    dt: f32,
) -> LegacyEnemyUpdate {
    align_legacy_enemy_rotation_without_mod(
        &mut state.rotation,
        constants.rotation_alignment_speed,
        dt,
    );

    if state.lifecycle == LegacyFlyingFishLifecycle::Shot {
        state.speed_y += constants.shot_gravity * dt;
        state.x += state.speed_x * dt;
        state.y += state.speed_y * dt;
        return LegacyEnemyUpdate::Keep;
    }

    state.animation_timer += dt;
    while state.animation_timer > constants.animation_speed {
        state.animation_timer -= constants.animation_speed;
        state.frame = match state.frame {
            Some(LegacyFlyingFishFrame::One) => Some(LegacyFlyingFishFrame::Two),
            _ => Some(LegacyFlyingFishFrame::One),
        };
    }

    LegacyEnemyUpdate::Keep
}

pub fn shoot_legacy_flying_fish(
    state: &mut LegacyFlyingFishState,
    constants: LegacyFlyingFishConstants,
    direction: Option<LegacyEnemyDirection>,
) {
    let direction = direction.unwrap_or(LegacyEnemyDirection::Right);

    state.lifecycle = LegacyFlyingFishLifecycle::Shot;
    state.active = false;
    state.gravity = constants.shot_gravity;
    state.speed_y = -constants.shot_jump_force;
    state.direction = direction;
    state.speed_x = match direction {
        LegacyEnemyDirection::Left => -constants.shot_speed_x,
        LegacyEnemyDirection::Right => constants.shot_speed_x,
    };
}

pub fn stomp_legacy_flying_fish(
    state: &mut LegacyFlyingFishState,
    constants: LegacyFlyingFishConstants,
) {
    shoot_legacy_flying_fish(state, constants, None);
}

#[must_use]
pub const fn legacy_flying_fish_collision() -> LegacyEnemyCollisionResponse {
    LegacyEnemyCollisionResponse {
        suppress_default: true,
    }
}

fn align_legacy_enemy_rotation_without_mod(rotation: &mut f32, alignment_speed: f32, dt: f32) {
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

fn apply_legacy_hammer_bro_patrol_bounds(
    state: &mut LegacyHammerBroState,
    constants: LegacyHammerBroConstants,
) {
    if state.speed_x < 0.0 {
        if state.x < state.start_x - 1.0 {
            state.speed_x = constants.speed;
        }
    } else if state.x > state.start_x {
        state.speed_x = -constants.speed;
    }
}

fn update_legacy_hammer_bro_throw_timer(
    state: &mut LegacyHammerBroState,
    dt: f32,
    next_throw_timer: impl FnOnce() -> f32,
) -> LegacyHammerBroUpdate {
    state.throw_timer -= dt;
    if state.throw_timer <= 0.0 {
        let direction = state.direction;
        state.throw_timer = next_throw_timer();
        LegacyHammerBroUpdate::ThrowHammer { direction }
    } else {
        LegacyHammerBroUpdate::Idle
    }
}

fn update_legacy_hammer_bro_jump_timer(
    state: &mut LegacyHammerBroState,
    constants: LegacyHammerBroConstants,
    dt: f32,
    random_jump_direction: LegacyHammerBroJumpDecision,
) {
    state.jump_timer += dt;
    if state.jump_timer <= constants.jump_time {
        return;
    }

    state.jump_timer -= constants.jump_time;
    let direction = if state.y > 12.0 {
        LegacyHammerBroJumpDecision::Up
    } else if state.y < 6.0 {
        LegacyHammerBroJumpDecision::Down
    } else {
        random_jump_direction
    };

    match direction {
        LegacyHammerBroJumpDecision::Up => {
            state.speed_y = -constants.jump_force;
            state.floor_collision_mask_active = true;
            state.jumping = LegacyHammerBroJumpState::Up;
        }
        LegacyHammerBroJumpDecision::Down => {
            state.speed_y = -constants.jump_force_down;
            state.floor_collision_mask_active = true;
            state.jumping = LegacyHammerBroJumpState::Down;
            state.jumping_y = state.y;
        }
    }
}

fn clear_legacy_hammer_bro_finished_jump(state: &mut LegacyHammerBroState) {
    match state.jumping {
        LegacyHammerBroJumpState::Up if state.speed_y > 0.0 => {
            state.jumping = LegacyHammerBroJumpState::None;
            state.floor_collision_mask_active = false;
        }
        LegacyHammerBroJumpState::Down if state.y > state.jumping_y + 2.0 => {
            state.jumping = LegacyHammerBroJumpState::None;
            state.floor_collision_mask_active = false;
        }
        LegacyHammerBroJumpState::None
        | LegacyHammerBroJumpState::Up
        | LegacyHammerBroJumpState::Down => {}
    }
}

fn turn_legacy_hammer_bro_toward_nearest_player(
    state: &mut LegacyHammerBroState,
    player_xs: &[f32],
) {
    if let Some(player_x) = nearest_legacy_hammer_bro_player_x(state.x, player_xs) {
        match state.direction {
            LegacyEnemyDirection::Left if player_x > state.x => {
                state.direction = LegacyEnemyDirection::Right;
                state.animation_direction = LegacyEnemyDirection::Right;
            }
            LegacyEnemyDirection::Right if player_x < state.x => {
                state.direction = LegacyEnemyDirection::Left;
                state.animation_direction = LegacyEnemyDirection::Left;
            }
            LegacyEnemyDirection::Left | LegacyEnemyDirection::Right => {}
        }
    }
}

fn nearest_legacy_hammer_bro_player_x(hammer_bro_x: f32, player_xs: &[f32]) -> Option<f32> {
    let mut nearest = *player_xs.first()?;

    for player_x in &player_xs[1..] {
        if (hammer_bro_x - *player_x).abs() < (hammer_bro_x - nearest).abs() {
            nearest = *player_x;
        }
    }

    Some(nearest)
}

fn advance_legacy_hammer_bro_animation(
    state: &mut LegacyHammerBroState,
    constants: LegacyHammerBroConstants,
    dt: f32,
) {
    state.animation_timer += dt;

    while state.animation_timer > constants.animation_speed {
        state.animation_timer -= constants.animation_speed;
        state.frame = match state.frame {
            LegacyHammerBroFrame::One => LegacyHammerBroFrame::Two,
            LegacyHammerBroFrame::Two => LegacyHammerBroFrame::One,
        };
        state.preparing_throw = state.throw_timer < constants.prepare_time;
    }
}

fn ease_legacy_hammer_bro_speed(
    state: &mut LegacyHammerBroState,
    constants: LegacyHammerBroConstants,
    dt: f32,
) {
    let delta = constants.friction * dt;

    if state.speed_x > constants.speed {
        state.speed_x -= delta;
        if state.speed_x < constants.speed {
            state.speed_x = constants.speed;
        }
    } else if state.speed_x < -constants.speed {
        state.speed_x += delta;
        if state.speed_x > constants.speed {
            state.speed_x = -constants.speed;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LegacyGoombaLifecycle {
    Walking,
    Stomped { death_timer: f32 },
    Shot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyGoombaVariant {
    Goomba,
    Spikey,
    SpikeyFalling,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyGoombaFrame {
    Goomba,
    SpikeyOne,
    SpikeyTwo,
    SpikeyFallingOne,
    SpikeyFallingTwo,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyGoombaState {
    pub variant: LegacyGoombaVariant,
    pub x: f32,
    pub y: f32,
    pub start_y: f32,
    pub speed_x: f32,
    pub speed_y: f32,
    pub width: f32,
    pub height: f32,
    pub active: bool,
    pub rotation: f32,
    pub animation_direction: LegacyEnemyDirection,
    pub animation_timer: f32,
    pub frame: LegacyGoombaFrame,
    pub gravity: Option<f32>,
    pub lakito_collision_mask_active: bool,
    pub lifecycle: LegacyGoombaLifecycle,
}

impl LegacyGoombaState {
    #[must_use]
    pub fn spawn(tile_x: f32, tile_y: f32, constants: LegacyGoombaConstants) -> Self {
        Self::spawn_variant(tile_x, tile_y, LegacyGoombaVariant::Goomba, constants)
    }

    #[must_use]
    pub fn spawn_spikey(tile_x: f32, tile_y: f32, constants: LegacyGoombaConstants) -> Self {
        Self::spawn_variant(tile_x, tile_y, LegacyGoombaVariant::Spikey, constants)
    }

    #[must_use]
    pub fn spawn_spikey_falling(
        tile_x: f32,
        tile_y: f32,
        constants: LegacyGoombaConstants,
    ) -> Self {
        Self::spawn_variant(
            tile_x,
            tile_y,
            LegacyGoombaVariant::SpikeyFalling,
            constants,
        )
    }

    #[must_use]
    pub fn spawn_variant(
        tile_x: f32,
        tile_y: f32,
        variant: LegacyGoombaVariant,
        constants: LegacyGoombaConstants,
    ) -> Self {
        let y = tile_y - 11.0 / 16.0;

        Self {
            variant,
            x: tile_x - 6.0 / 16.0,
            y,
            start_y: y,
            speed_x: match variant {
                LegacyGoombaVariant::SpikeyFalling => 0.0,
                LegacyGoombaVariant::Goomba | LegacyGoombaVariant::Spikey => -constants.speed,
            },
            speed_y: if variant == LegacyGoombaVariant::SpikeyFalling {
                -10.0
            } else {
                0.0
            },
            width: 12.0 / 16.0,
            height: 12.0 / 16.0,
            active: true,
            rotation: 0.0,
            animation_direction: LegacyEnemyDirection::Left,
            animation_timer: 0.0,
            frame: match variant {
                LegacyGoombaVariant::Goomba => LegacyGoombaFrame::Goomba,
                LegacyGoombaVariant::Spikey => LegacyGoombaFrame::SpikeyOne,
                LegacyGoombaVariant::SpikeyFalling => LegacyGoombaFrame::SpikeyFallingOne,
            },
            gravity: if variant == LegacyGoombaVariant::SpikeyFalling {
                Some(30.0)
            } else {
                None
            },
            lakito_collision_mask_active: variant == LegacyGoombaVariant::SpikeyFalling,
            lifecycle: LegacyGoombaLifecycle::Walking,
        }
    }
}

#[must_use]
pub fn update_legacy_goomba(
    state: &mut LegacyGoombaState,
    constants: LegacyGoombaConstants,
    dt: f32,
) -> LegacyEnemyUpdate {
    if state.variant == LegacyGoombaVariant::SpikeyFalling {
        state.rotation = 0.0;
    } else {
        align_legacy_enemy_rotation(&mut state.rotation, constants.rotation_alignment_speed, dt);
    }

    match &mut state.lifecycle {
        LegacyGoombaLifecycle::Stomped { death_timer } => {
            *death_timer += dt;
            if *death_timer > constants.death_time {
                LegacyEnemyUpdate::Remove
            } else {
                LegacyEnemyUpdate::Keep
            }
        }
        LegacyGoombaLifecycle::Shot => {
            state.speed_y += constants.shot_gravity * dt;
            state.x += state.speed_x * dt;
            state.y += state.speed_y * dt;
            LegacyEnemyUpdate::Keep
        }
        LegacyGoombaLifecycle::Walking => {
            advance_legacy_goomba_animation(state, constants, dt);
            ease_legacy_goomba_speed(state, constants, dt);
            LegacyEnemyUpdate::Keep
        }
    }
}

pub fn stomp_legacy_goomba(state: &mut LegacyGoombaState) {
    state.lifecycle = LegacyGoombaLifecycle::Stomped { death_timer: 0.0 };
    state.active = false;
}

pub fn shoot_legacy_goomba(
    state: &mut LegacyGoombaState,
    constants: LegacyGoombaConstants,
    direction: LegacyEnemyDirection,
) {
    state.lifecycle = LegacyGoombaLifecycle::Shot;
    state.speed_y = -constants.shot_jump_force;
    state.speed_x = match direction {
        LegacyEnemyDirection::Left => -constants.shot_speed_x,
        LegacyEnemyDirection::Right => constants.shot_speed_x,
    };
    state.active = false;
}

pub fn legacy_goomba_left_collision(
    state: &mut LegacyGoombaState,
    constants: LegacyGoombaConstants,
) -> LegacyEnemyCollisionResponse {
    if state.lifecycle == LegacyGoombaLifecycle::Walking {
        state.speed_x = constants.speed;
        if state.variant == LegacyGoombaVariant::Spikey {
            state.animation_direction = LegacyEnemyDirection::Left;
        }
    }

    LegacyEnemyCollisionResponse {
        suppress_default: true,
    }
}

pub fn legacy_goomba_right_collision(
    state: &mut LegacyGoombaState,
    constants: LegacyGoombaConstants,
) -> LegacyEnemyCollisionResponse {
    if state.lifecycle == LegacyGoombaLifecycle::Walking {
        state.speed_x = -constants.speed;
        if state.variant == LegacyGoombaVariant::Spikey {
            state.animation_direction = LegacyEnemyDirection::Right;
        }
    }

    LegacyEnemyCollisionResponse {
        suppress_default: true,
    }
}

#[must_use]
pub fn legacy_spikey_falling_suppresses_lakito_collision(state: &LegacyGoombaState) -> bool {
    state.variant == LegacyGoombaVariant::SpikeyFalling
}

pub fn legacy_spikey_falling_floor_collision(
    state: &mut LegacyGoombaState,
    constants: LegacyGoombaConstants,
    player_xs: &[f32],
) -> LegacyEnemyCollisionResponse {
    if state.variant == LegacyGoombaVariant::SpikeyFalling {
        state.variant = LegacyGoombaVariant::Spikey;
        state.frame = LegacyGoombaFrame::SpikeyOne;
        state.gravity = None;
        state.lakito_collision_mask_active = false;

        if let Some(nearest_player_x) = nearest_legacy_spikey_floor_player_x(state.x, player_xs) {
            if state.x >= nearest_player_x {
                state.speed_x = -constants.speed;
            } else {
                state.speed_x = constants.speed;
                state.animation_direction = LegacyEnemyDirection::Left;
            }
        }
    }

    LegacyEnemyCollisionResponse {
        suppress_default: false,
    }
}

fn align_legacy_enemy_rotation(rotation: &mut f32, alignment_speed: f32, dt: f32) {
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

fn advance_legacy_goomba_animation(
    state: &mut LegacyGoombaState,
    constants: LegacyGoombaConstants,
    dt: f32,
) {
    state.animation_timer += dt;

    while state.animation_timer > constants.animation_speed {
        state.animation_timer -= constants.animation_speed;
        match state.variant {
            LegacyGoombaVariant::Goomba => {
                state.animation_direction = match state.animation_direction {
                    LegacyEnemyDirection::Left => LegacyEnemyDirection::Right,
                    LegacyEnemyDirection::Right => LegacyEnemyDirection::Left,
                };
            }
            LegacyGoombaVariant::Spikey => {
                state.frame = match state.frame {
                    LegacyGoombaFrame::SpikeyOne => LegacyGoombaFrame::SpikeyTwo,
                    _ => LegacyGoombaFrame::SpikeyOne,
                };
            }
            LegacyGoombaVariant::SpikeyFalling => {
                state.frame = match state.frame {
                    LegacyGoombaFrame::SpikeyFallingOne => LegacyGoombaFrame::SpikeyFallingTwo,
                    _ => LegacyGoombaFrame::SpikeyFallingOne,
                };
                if state.lakito_collision_mask_active && state.y > state.start_y + 2.0 {
                    state.lakito_collision_mask_active = false;
                }
            }
        }
    }
}

fn ease_legacy_goomba_speed(
    state: &mut LegacyGoombaState,
    constants: LegacyGoombaConstants,
    dt: f32,
) {
    let delta = constants.friction * dt * 2.0;

    if state.variant == LegacyGoombaVariant::SpikeyFalling {
        return;
    }

    if state.speed_x > 0.0 {
        if state.speed_x > constants.speed {
            state.speed_x -= delta;
            if state.speed_x < constants.speed {
                state.speed_x = constants.speed;
            }
        } else if state.speed_x < constants.speed {
            state.speed_x += delta;
            if state.speed_x > constants.speed {
                state.speed_x = constants.speed;
            }
        }
    } else if state.speed_x < -constants.speed {
        state.speed_x += delta;
        if state.speed_x > -constants.speed {
            state.speed_x = -constants.speed;
        }
    } else if state.speed_x > -constants.speed {
        state.speed_x -= delta;
        if state.speed_x < -constants.speed {
            state.speed_x = -constants.speed;
        }
    }
}

fn nearest_legacy_spikey_floor_player_x(spikey_x: f32, player_xs: &[f32]) -> Option<f32> {
    let mut nearest_player_x = *player_xs.first()?;

    for player_x in &player_xs[1..] {
        if (spikey_x - *player_x).abs() < nearest_player_x {
            nearest_player_x = *player_x;
        }
    }

    Some(nearest_player_x)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyKoopaVariant {
    Green,
    Red,
    Flying,
    RedFlying,
    Beetle,
}

impl LegacyKoopaVariant {
    #[must_use]
    const fn starts_flying(self) -> bool {
        matches!(self, Self::Flying | Self::RedFlying)
    }

    #[must_use]
    const fn turns_at_edges(self) -> bool {
        matches!(self, Self::Red | Self::RedFlying)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyKoopaFrame {
    WalkingOne,
    WalkingTwo,
    FlyingOne,
    FlyingTwo,
    Shell,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyKoopaLifecycle {
    Normal,
    Shot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyKoopaStompOutcome {
    ClearedFlying,
    EnteredShell,
    StartedShell,
    StoppedShell,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyKoopaEdgeTurn {
    Ineligible,
    NoTurn { probe: LegacyEnemyTileProbe },
    Turned { probe: LegacyEnemyTileProbe },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyEnemyTileProbe {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyKoopaSideCollisionTarget {
    Solid,
    Other,
}

impl LegacyKoopaSideCollisionTarget {
    #[must_use]
    const fn is_solid(self) -> bool {
        matches!(self, Self::Solid)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyKoopaSideCollisionResponse {
    pub suppress_default: bool,
    pub hit_solid_target: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyKoopaState {
    pub variant: LegacyKoopaVariant,
    pub x: f32,
    pub y: f32,
    pub speed_x: f32,
    pub speed_y: f32,
    pub width: f32,
    pub height: f32,
    pub active: bool,
    pub rotation: f32,
    pub animation_direction: LegacyEnemyDirection,
    pub animation_timer: f32,
    pub frame: LegacyKoopaFrame,
    pub small: bool,
    pub flying: bool,
    pub falling: bool,
    pub gravity: Option<f32>,
    pub combo: usize,
    pub lifecycle: LegacyKoopaLifecycle,
}

impl LegacyKoopaState {
    #[must_use]
    pub fn spawn(
        tile_x: f32,
        tile_y: f32,
        variant: LegacyKoopaVariant,
        constants: LegacyKoopaConstants,
    ) -> Self {
        let flying = variant.starts_flying();

        Self {
            variant,
            x: tile_x - 6.0 / 16.0,
            y: tile_y - 11.0 / 16.0,
            speed_x: if variant == LegacyKoopaVariant::RedFlying {
                0.0
            } else {
                -constants.speed
            },
            speed_y: 0.0,
            width: 12.0 / 16.0,
            height: 12.0 / 16.0,
            active: true,
            rotation: 0.0,
            animation_direction: LegacyEnemyDirection::Right,
            animation_timer: 0.0,
            frame: if flying {
                LegacyKoopaFrame::FlyingOne
            } else {
                LegacyKoopaFrame::WalkingOne
            },
            small: false,
            flying,
            falling: false,
            gravity: match variant {
                LegacyKoopaVariant::Flying => Some(constants.flying_gravity),
                LegacyKoopaVariant::RedFlying => Some(0.0),
                LegacyKoopaVariant::Green
                | LegacyKoopaVariant::Red
                | LegacyKoopaVariant::Beetle => None,
            },
            combo: 1,
            lifecycle: LegacyKoopaLifecycle::Normal,
        }
    }
}

#[must_use]
pub fn update_legacy_koopa(
    state: &mut LegacyKoopaState,
    constants: LegacyKoopaConstants,
    dt: f32,
) -> LegacyEnemyUpdate {
    align_legacy_enemy_rotation(&mut state.rotation, constants.rotation_alignment_speed, dt);

    if state.lifecycle == LegacyKoopaLifecycle::Shot {
        state.speed_y += constants.shot_gravity * dt;
        state.x += state.speed_x * dt;
        state.y += state.speed_y * dt;
        return LegacyEnemyUpdate::Keep;
    }

    state.animation_direction = if state.speed_x > 0.0 {
        LegacyEnemyDirection::Left
    } else {
        LegacyEnemyDirection::Right
    };

    if !state.small {
        advance_legacy_koopa_animation(state, constants, dt);
    }

    ease_legacy_koopa_speed(state, constants, dt);

    LegacyEnemyUpdate::Keep
}

pub fn shoot_legacy_koopa(
    state: &mut LegacyKoopaState,
    constants: LegacyKoopaConstants,
    direction: LegacyEnemyDirection,
) {
    state.lifecycle = LegacyKoopaLifecycle::Shot;
    state.small = true;
    state.flying = false;
    state.frame = LegacyKoopaFrame::Shell;
    state.speed_y = -constants.shot_jump_force;
    state.speed_x = match direction {
        LegacyEnemyDirection::Left => -constants.shot_speed_x,
        LegacyEnemyDirection::Right => constants.shot_speed_x,
    };
    state.active = false;
    state.gravity = Some(constants.shot_gravity);
}

#[must_use]
pub fn legacy_koopa_resists_fireball(state: &LegacyKoopaState) -> bool {
    state.variant == LegacyKoopaVariant::Beetle
}

pub fn stomp_legacy_koopa(
    state: &mut LegacyKoopaState,
    constants: LegacyKoopaConstants,
    player_x: f32,
    frame_dt: f32,
) -> LegacyKoopaStompOutcome {
    if state.flying {
        state.flying = false;
        state.frame = LegacyKoopaFrame::WalkingOne;
        if state.speed_x == 0.0 {
            state.speed_x = -constants.speed;
        }
        state.gravity = Some(constants.normal_gravity);
        return LegacyKoopaStompOutcome::ClearedFlying;
    }

    if !state.small {
        state.frame = LegacyKoopaFrame::Shell;
        state.small = true;
        state.speed_x = 0.0;
        return LegacyKoopaStompOutcome::EnteredShell;
    }

    if state.speed_x == 0.0 {
        if state.x > player_x {
            state.speed_x = constants.shell_speed;
            state.x = player_x + 12.0 / 16.0 + constants.shell_speed * frame_dt;
        } else {
            state.speed_x = -constants.shell_speed;
            state.x = player_x - state.width - constants.shell_speed * frame_dt;
        }
        return LegacyKoopaStompOutcome::StartedShell;
    }

    state.speed_x = 0.0;
    state.combo = 1;
    LegacyKoopaStompOutcome::StoppedShell
}

pub fn legacy_koopa_left_collision(
    state: &mut LegacyKoopaState,
    target: LegacyKoopaSideCollisionTarget,
) -> LegacyKoopaSideCollisionResponse {
    legacy_koopa_side_collision(state, target, LegacyEnemyDirection::Left)
}

pub fn legacy_koopa_right_collision(
    state: &mut LegacyKoopaState,
    target: LegacyKoopaSideCollisionTarget,
) -> LegacyKoopaSideCollisionResponse {
    legacy_koopa_side_collision(state, target, LegacyEnemyDirection::Right)
}

pub fn legacy_koopa_floor_collision(
    state: &mut LegacyKoopaState,
    constants: LegacyKoopaConstants,
) -> LegacyEnemyCollisionResponse {
    state.falling = false;

    if state.flying {
        state.speed_y = -constants.jump_force;
    }

    LegacyEnemyCollisionResponse {
        suppress_default: false,
    }
}

pub fn legacy_koopa_start_fall(state: &mut LegacyKoopaState) {
    state.falling = true;
}

pub fn apply_legacy_red_koopa_edge_turn(
    state: &mut LegacyKoopaState,
    mut tile_solid_at: impl FnMut(i32, i32) -> Option<bool>,
) -> LegacyKoopaEdgeTurn {
    if !state.variant.turns_at_edges() || state.falling || state.flying || state.small {
        return LegacyKoopaEdgeTurn::Ineligible;
    }

    let probe = LegacyEnemyTileProbe {
        x: (state.x + state.width / 2.0 + 1.0).floor() as i32,
        y: (state.y + state.height + 1.5).floor() as i32,
    };

    if tile_solid_at(probe.x, probe.y) != Some(false) {
        return LegacyKoopaEdgeTurn::NoTurn { probe };
    }

    let has_solid_neighbor = tile_solid_at(probe.x + 1, probe.y) == Some(true)
        || tile_solid_at(probe.x - 1, probe.y) == Some(true);
    if !has_solid_neighbor {
        return LegacyKoopaEdgeTurn::NoTurn { probe };
    }

    if state.speed_x < 0.0 {
        state.animation_direction = LegacyEnemyDirection::Left;
        state.x = probe.x as f32 - state.width / 2.0;
    } else {
        state.animation_direction = LegacyEnemyDirection::Right;
        state.x = probe.x as f32 - 1.0 - state.width / 2.0;
    }
    state.speed_x = -state.speed_x;

    LegacyKoopaEdgeTurn::Turned { probe }
}

fn legacy_koopa_side_collision(
    state: &mut LegacyKoopaState,
    target: LegacyKoopaSideCollisionTarget,
    side: LegacyEnemyDirection,
) -> LegacyKoopaSideCollisionResponse {
    if state.small {
        if target.is_solid() {
            state.speed_x = -state.speed_x;
        }

        LegacyKoopaSideCollisionResponse {
            suppress_default: true,
            hit_solid_target: target.is_solid(),
        }
    } else {
        state.animation_direction = side;
        state.speed_x = -state.speed_x;

        LegacyKoopaSideCollisionResponse {
            suppress_default: false,
            hit_solid_target: false,
        }
    }
}

fn advance_legacy_koopa_animation(
    state: &mut LegacyKoopaState,
    constants: LegacyKoopaConstants,
    dt: f32,
) {
    state.animation_timer += dt;

    while state.animation_timer > constants.animation_speed {
        state.animation_timer -= constants.animation_speed;
        state.frame = match (state.flying, state.frame) {
            (true, LegacyKoopaFrame::FlyingOne) => LegacyKoopaFrame::FlyingTwo,
            (true, _) => LegacyKoopaFrame::FlyingOne,
            (false, LegacyKoopaFrame::WalkingOne) => LegacyKoopaFrame::WalkingTwo,
            (false, _) => LegacyKoopaFrame::WalkingOne,
        };
    }
}

fn ease_legacy_koopa_speed(state: &mut LegacyKoopaState, constants: LegacyKoopaConstants, dt: f32) {
    let target_speed = if state.small {
        constants.shell_speed
    } else {
        constants.speed
    };
    let delta = constants.friction * dt * 2.0;

    if state.speed_x > 0.0 {
        if state.speed_x > target_speed {
            state.speed_x -= delta;
            if state.speed_x < target_speed {
                state.speed_x = target_speed;
            }
        } else if state.speed_x < target_speed {
            state.speed_x += delta;
            if state.speed_x > target_speed {
                state.speed_x = target_speed;
            }
        }
    } else if state.speed_x < 0.0 {
        if state.speed_x < -target_speed {
            state.speed_x += delta;
            if state.speed_x > -target_speed {
                state.speed_x = -target_speed;
            }
        } else if state.speed_x > -target_speed {
            state.speed_x -= delta;
            if state.speed_x < -target_speed {
                state.speed_x = -target_speed;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyPlantFrame {
    One,
    Two,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyPlantState {
    pub x: f32,
    pub y: f32,
    pub start_y: f32,
    pub width: f32,
    pub height: f32,
    pub active: bool,
    pub destroy: bool,
    pub animation_frame: LegacyPlantFrame,
    pub animation_timer: f32,
    pub cycle_timer: f32,
}

impl LegacyPlantState {
    #[must_use]
    pub fn spawn(tile_x: f32, tile_y: f32, constants: LegacyPlantConstants) -> Self {
        let y = tile_y + 9.0 / 16.0;

        Self {
            x: tile_x - 8.0 / 16.0,
            y,
            start_y: y,
            width: 1.0,
            height: 14.0 / 16.0,
            active: true,
            destroy: false,
            animation_frame: LegacyPlantFrame::One,
            animation_timer: 0.0,
            cycle_timer: constants.out_time + 1.5,
        }
    }
}

#[must_use]
pub fn update_legacy_plant(
    state: &mut LegacyPlantState,
    constants: LegacyPlantConstants,
    dt: f32,
    player_center_in_range: impl FnOnce(f32, f32) -> bool,
) -> LegacyEnemyUpdate {
    advance_legacy_plant_animation(state, constants, dt);

    state.cycle_timer += dt;
    if state.cycle_timer < constants.out_time {
        state.y -= constants.move_speed * dt;
        if state.y < state.start_y - constants.move_distance {
            state.y = state.start_y - constants.move_distance;
        }
    } else if state.cycle_timer < constants.out_time + constants.in_time {
        state.y += constants.move_speed * dt;
        if state.y > state.start_y {
            state.y = state.start_y;
        }
    } else {
        let center_x = state.x + state.width / 2.0;
        if !player_center_in_range(center_x - 3.0, center_x + 3.0) {
            state.cycle_timer = 0.0;
        }
    }

    if state.destroy {
        LegacyEnemyUpdate::Remove
    } else {
        LegacyEnemyUpdate::Keep
    }
}

pub fn shoot_legacy_plant(state: &mut LegacyPlantState) {
    state.destroy = true;
    state.active = false;
}

fn advance_legacy_plant_animation(
    state: &mut LegacyPlantState,
    constants: LegacyPlantConstants,
    dt: f32,
) {
    state.animation_timer += dt;

    while state.animation_timer > constants.animation_delay {
        state.animation_timer -= constants.animation_delay;
        state.animation_frame = match state.animation_frame {
            LegacyPlantFrame::One => LegacyPlantFrame::Two,
            LegacyPlantFrame::Two => LegacyPlantFrame::One,
        };
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyBulletBillLifecycle {
    Flying,
    Shot,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyBulletBillState {
    pub start_x: f32,
    pub x: f32,
    pub y: f32,
    pub speed_x: f32,
    pub speed_y: f32,
    pub width: f32,
    pub height: f32,
    pub active: bool,
    pub gravity: f32,
    pub rotation: f32,
    pub timer: f32,
    pub animation_direction: LegacyEnemyDirection,
    pub custom_scissor_active: bool,
    pub kill_stuff: bool,
    pub lifecycle: LegacyBulletBillLifecycle,
}

impl LegacyBulletBillState {
    #[must_use]
    pub fn spawn(
        tile_x: f32,
        tile_y: f32,
        direction: LegacyEnemyDirection,
        constants: LegacyBulletBillConstants,
    ) -> Self {
        let start_x = tile_x - 14.0 / 16.0;

        Self {
            start_x,
            x: start_x,
            y: tile_y - 14.0 / 16.0,
            speed_x: match direction {
                LegacyEnemyDirection::Left => -constants.speed,
                LegacyEnemyDirection::Right => constants.speed,
            },
            speed_y: 0.0,
            width: 12.0 / 16.0,
            height: 12.0 / 16.0,
            active: true,
            gravity: 0.0,
            rotation: 0.0,
            timer: 0.0,
            animation_direction: direction,
            custom_scissor_active: true,
            kill_stuff: false,
            lifecycle: LegacyBulletBillLifecycle::Flying,
        }
    }
}

#[must_use]
pub fn update_legacy_bullet_bill(
    state: &mut LegacyBulletBillState,
    constants: LegacyBulletBillConstants,
    dt: f32,
) -> LegacyEnemyUpdate {
    if state.x < state.start_x - 1.0 || state.x > state.start_x + 1.0 {
        state.custom_scissor_active = false;
    }

    if state.lifecycle == LegacyBulletBillLifecycle::Shot {
        state.speed_y += constants.shot_gravity * dt;
        state.x += state.speed_x * dt;
        state.y += state.speed_y * dt;

        if state.y > constants.removal_y {
            LegacyEnemyUpdate::Remove
        } else {
            LegacyEnemyUpdate::Keep
        }
    } else {
        state.timer += dt;
        if state.timer >= constants.lifetime {
            return LegacyEnemyUpdate::Remove;
        }

        snap_legacy_bullet_bill_rotation(state);
        update_legacy_bullet_bill_animation_direction(state);

        LegacyEnemyUpdate::Keep
    }
}

pub fn stomp_legacy_bullet_bill(
    state: &mut LegacyBulletBillState,
    constants: LegacyBulletBillConstants,
    direction: Option<LegacyEnemyDirection>,
) {
    let direction = direction.unwrap_or(LegacyEnemyDirection::Right);

    state.lifecycle = LegacyBulletBillLifecycle::Shot;
    state.speed_y = 0.0;
    state.active = false;
    state.gravity = constants.shot_gravity;
    state.speed_x = match direction {
        LegacyEnemyDirection::Left => -constants.shot_speed_x,
        LegacyEnemyDirection::Right => constants.shot_speed_x,
    };
}

pub fn shoot_legacy_bullet_bill(
    state: &mut LegacyBulletBillState,
    constants: LegacyBulletBillConstants,
    direction: Option<LegacyEnemyDirection>,
) {
    stomp_legacy_bullet_bill(state, constants, direction);
}

pub fn portal_legacy_bullet_bill(state: &mut LegacyBulletBillState) {
    state.kill_stuff = true;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyBulletBillLauncherState {
    pub x: f32,
    pub y: f32,
    pub timer: f32,
    pub time: f32,
    pub autodelete: bool,
}

impl LegacyBulletBillLauncherState {
    #[must_use]
    pub fn spawn(x: f32, y: f32, initial_time: f32) -> Self {
        Self {
            x,
            y,
            timer: initial_time - 0.5,
            time: initial_time,
            autodelete: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyBulletBillLauncherViewport {
    pub left: f32,
    pub width: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyBulletBillLauncherUpdate {
    Idle,
    Fired { direction: LegacyEnemyDirection },
}

#[must_use]
pub fn update_legacy_bullet_bill_launcher(
    state: &mut LegacyBulletBillLauncherState,
    constants: LegacyBulletBillConstants,
    dt: f32,
    viewport: LegacyBulletBillLauncherViewport,
    current_bullet_bill_count: usize,
    player_xs: &[f32],
    next_time_after_fire: impl FnOnce() -> f32,
) -> LegacyBulletBillLauncherUpdate {
    state.timer += dt;

    if state.timer > state.time
        && state.x > viewport.left
        && state.x < viewport.left + viewport.width + 2.0
    {
        if let Some(direction) =
            fire_legacy_bullet_bill_launcher(state, constants, current_bullet_bill_count, player_xs)
        {
            state.timer = 0.0;
            state.time = next_time_after_fire();
            return LegacyBulletBillLauncherUpdate::Fired { direction };
        }
    }

    LegacyBulletBillLauncherUpdate::Idle
}

#[must_use]
pub fn fire_legacy_bullet_bill_launcher(
    state: &LegacyBulletBillLauncherState,
    constants: LegacyBulletBillConstants,
    current_bullet_bill_count: usize,
    player_xs: &[f32],
) -> Option<LegacyEnemyDirection> {
    if current_bullet_bill_count >= constants.max_count || player_xs.is_empty() {
        return None;
    }

    let nearest_player_probe_x = nearest_legacy_bullet_bill_player_probe_x(state.x, player_xs);
    if nearest_player_probe_x > state.x + constants.range {
        Some(LegacyEnemyDirection::Right)
    } else if nearest_player_probe_x < state.x - constants.range {
        Some(LegacyEnemyDirection::Left)
    } else {
        None
    }
}

fn snap_legacy_bullet_bill_rotation(state: &mut LegacyBulletBillState) {
    if state.rotation != 0.0 {
        if (state.rotation.abs() - core::f32::consts::FRAC_PI_2).abs() < 0.1 {
            state.rotation = -core::f32::consts::FRAC_PI_2;
        } else {
            state.rotation = 0.0;
        }
    }
}

fn update_legacy_bullet_bill_animation_direction(state: &mut LegacyBulletBillState) {
    if state.speed_x < 0.0 {
        state.animation_direction = LegacyEnemyDirection::Left;
    } else if state.speed_x > 0.0 || state.speed_y < 0.0 {
        state.animation_direction = LegacyEnemyDirection::Right;
    } else if state.speed_y > 0.0 {
        state.animation_direction = LegacyEnemyDirection::Left;
    }
}

fn nearest_legacy_bullet_bill_player_probe_x(launcher_x: f32, player_xs: &[f32]) -> f32 {
    let first = player_xs[0] + 14.0 / 16.0;
    player_xs[1..].iter().fold(first, |nearest, player_x| {
        let probe_x = *player_x + 14.0 / 16.0;
        if (probe_x - launcher_x).abs() < (nearest - launcher_x).abs() {
            probe_x
        } else {
            nearest
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{
        LegacyBulletBillLauncherState, LegacyBulletBillLauncherUpdate,
        LegacyBulletBillLauncherViewport, LegacyBulletBillLifecycle, LegacyBulletBillState,
        LegacyCheepCheepColor, LegacyCheepCheepFrame, LegacyCheepCheepLifecycle,
        LegacyCheepCheepState, LegacyCheepCheepVerticalDirection, LegacyEnemyDirection,
        LegacyEnemyTileProbe, LegacyEnemyUpdate, LegacyFlyingFishFrame, LegacyFlyingFishLifecycle,
        LegacyFlyingFishState, LegacyGoombaFrame, LegacyGoombaLifecycle, LegacyGoombaState,
        LegacyGoombaVariant, LegacyHammerBroCollisionActor, LegacyHammerBroFrame,
        LegacyHammerBroJumpDecision, LegacyHammerBroJumpState, LegacyHammerBroLifecycle,
        LegacyHammerBroState, LegacyHammerBroUpdate, LegacyHammerFrame, LegacyHammerState,
        LegacyKoopaEdgeTurn, LegacyKoopaFrame, LegacyKoopaLifecycle,
        LegacyKoopaSideCollisionTarget, LegacyKoopaState, LegacyKoopaStompOutcome,
        LegacyKoopaVariant, LegacyLakitoLifecycle, LegacyLakitoPlayerTarget, LegacyLakitoState,
        LegacyLakitoUpdate, LegacyPlantFrame, LegacyPlantState, LegacySquidFrame,
        LegacySquidLifecycle, LegacySquidMotion, LegacySquidPlayerTarget, LegacySquidState,
        apply_legacy_red_koopa_edge_turn, emancipate_legacy_hammer_bro,
        fire_legacy_bullet_bill_launcher, legacy_cheep_cheep_collision,
        legacy_flying_fish_collision, legacy_goomba_left_collision, legacy_goomba_right_collision,
        legacy_hammer_bro_ceil_collision, legacy_hammer_bro_floor_collision,
        legacy_hammer_bro_left_collision, legacy_hammer_bro_right_collision,
        legacy_hammer_collision, legacy_koopa_floor_collision, legacy_koopa_left_collision,
        legacy_koopa_resists_fireball, legacy_koopa_right_collision, legacy_koopa_start_fall,
        legacy_lakito_collision, legacy_lakito_spikeyfall_collision,
        legacy_spikey_falling_floor_collision, legacy_spikey_falling_suppresses_lakito_collision,
        legacy_squid_collision, portal_legacy_bullet_bill, portal_legacy_hammer,
        portal_legacy_hammer_bro, shoot_legacy_bullet_bill, shoot_legacy_cheep_cheep,
        shoot_legacy_flying_fish, shoot_legacy_goomba, shoot_legacy_hammer_bro, shoot_legacy_koopa,
        shoot_legacy_lakito, shoot_legacy_plant, shoot_legacy_squid, stomp_legacy_bullet_bill,
        stomp_legacy_flying_fish, stomp_legacy_goomba, stomp_legacy_hammer_bro, stomp_legacy_koopa,
        stomp_legacy_lakito, update_legacy_bullet_bill, update_legacy_bullet_bill_launcher,
        update_legacy_cheep_cheep, update_legacy_flying_fish, update_legacy_goomba,
        update_legacy_hammer, update_legacy_hammer_bro_active, update_legacy_hammer_bro_shot,
        update_legacy_koopa, update_legacy_lakito, update_legacy_plant, update_legacy_squid,
    };
    use crate::config::{
        LegacyBulletBillConstants, LegacyCheepCheepConstants, LegacyFlyingFishConstants,
        LegacyGoombaConstants, LegacyHammerBroConstants, LegacyHammerConstants,
        LegacyKoopaConstants, LegacyLakitoConstants, LegacyPlantConstants, LegacySquidConstants,
    };

    const DT: f32 = 0.1;

    #[test]
    fn cheep_cheep_spawn_matches_legacy_color_speed_and_vertical_random_injection() {
        let constants = LegacyCheepCheepConstants::default();
        let red =
            LegacyCheepCheepState::spawn(4.5, 8.0, LegacyCheepCheepColor::Red, true, constants);
        let white =
            LegacyCheepCheepState::spawn(4.5, 8.0, LegacyCheepCheepColor::White, false, constants);

        assert_eq!(red.x, 4.5);
        assert_eq!(red.y, 8.0);
        assert_eq!(red.start_y, 8.0);
        assert_eq!(red.width, 12.0 / 16.0);
        assert_eq!(red.height, 12.0 / 16.0);
        assert_eq!(red.speed_x, -constants.red_speed);
        assert_eq!(red.speed_y, 0.0);
        assert!(red.vertical_moving);
        assert_eq!(
            red.vertical_direction,
            LegacyCheepCheepVerticalDirection::Up
        );
        assert_eq!(red.frame, None);
        assert_eq!(red.lifecycle, LegacyCheepCheepLifecycle::Swimming);
        assert_eq!(red.gravity, 0.0);
        assert!(red.active);

        assert_eq!(white.speed_x, -constants.white_speed);
        assert!(!white.vertical_moving);
    }

    #[test]
    fn cheep_cheep_animation_preserves_uninitialized_frame_quirk() {
        let constants = LegacyCheepCheepConstants::default();
        let mut state =
            LegacyCheepCheepState::spawn(1.0, 2.0, LegacyCheepCheepColor::Red, false, constants);

        assert_eq!(
            update_legacy_cheep_cheep(&mut state, constants, constants.animation_speed),
            LegacyEnemyUpdate::Keep
        );
        assert_eq!(state.frame, None);
        assert_eq!(state.animation_timer, constants.animation_speed);

        let _ = update_legacy_cheep_cheep(&mut state, constants, 0.01);
        assert_eq!(state.frame, Some(LegacyCheepCheepFrame::One));
        assert_close(state.animation_timer, 0.01);

        let _ = update_legacy_cheep_cheep(&mut state, constants, constants.animation_speed + 0.01);
        assert_eq!(state.frame, Some(LegacyCheepCheepFrame::Two));
        assert_close(state.animation_timer, 0.02);
    }

    #[test]
    fn cheep_cheep_vertical_motion_sets_speed_and_turns_after_strict_range_edges() {
        let constants = LegacyCheepCheepConstants::default();
        let mut state =
            LegacyCheepCheepState::spawn(1.0, 5.0, LegacyCheepCheepColor::Red, true, constants);

        let _ = update_legacy_cheep_cheep(&mut state, constants, DT);
        assert_eq!(state.speed_y, -constants.vertical_speed);
        assert_eq!(
            state.vertical_direction,
            LegacyCheepCheepVerticalDirection::Up
        );

        state.y = state.start_y - constants.vertical_range;
        let _ = update_legacy_cheep_cheep(&mut state, constants, DT);
        assert_eq!(
            state.vertical_direction,
            LegacyCheepCheepVerticalDirection::Up
        );

        state.y = state.start_y - constants.vertical_range - 0.01;
        let _ = update_legacy_cheep_cheep(&mut state, constants, DT);
        assert_eq!(
            state.vertical_direction,
            LegacyCheepCheepVerticalDirection::Down
        );
        assert_eq!(state.speed_y, -constants.vertical_speed);

        state.y = state.start_y + constants.vertical_range + 0.01;
        let _ = update_legacy_cheep_cheep(&mut state, constants, DT);
        assert_eq!(
            state.vertical_direction,
            LegacyCheepCheepVerticalDirection::Up
        );
        assert_eq!(state.speed_y, constants.vertical_speed);
    }

    #[test]
    fn cheep_cheep_shot_uses_legacy_impulse_gravity_and_direct_position_step() {
        let constants = LegacyCheepCheepConstants::default();
        let mut state =
            LegacyCheepCheepState::spawn(3.0, 4.0, LegacyCheepCheepColor::White, true, constants);

        shoot_legacy_cheep_cheep(&mut state, constants, Some(LegacyEnemyDirection::Left));
        assert!(!state.active);
        assert_eq!(state.lifecycle, LegacyCheepCheepLifecycle::Shot);
        assert_eq!(state.gravity, constants.shot_gravity);
        assert_eq!(state.speed_x, -constants.shot_speed_x);
        assert_eq!(state.speed_y, -constants.shot_jump_force);

        let start_x = state.x;
        let start_y = state.y;
        assert_eq!(
            update_legacy_cheep_cheep(&mut state, constants, DT),
            LegacyEnemyUpdate::Keep
        );
        assert_eq!(state.speed_y, -2.0);
        assert_eq!(state.x, start_x - 0.4);
        assert_eq!(state.y, start_y - 0.2);
    }

    #[test]
    fn cheep_cheep_rotation_aligns_and_collisions_suppress_default_response() {
        let constants = LegacyCheepCheepConstants::default();
        let mut state =
            LegacyCheepCheepState::spawn(1.0, 2.0, LegacyCheepCheepColor::Red, false, constants);

        state.rotation = 1.0;
        let _ = update_legacy_cheep_cheep(&mut state, constants, 0.05);
        assert_eq!(state.rotation, 0.25);

        assert_eq!(
            legacy_cheep_cheep_collision(),
            super::LegacyEnemyCollisionResponse {
                suppress_default: true
            }
        );
    }

    #[test]
    fn squid_spawn_matches_legacy_offsets_and_initial_state() {
        let state = LegacySquidState::spawn(10.0, 5.0);

        assert_eq!(state.x, 10.0 - 1.0 + 2.0 / 16.0);
        assert_eq!(state.y, 5.0 - 1.0 + 4.0 / 16.0);
        assert_eq!(state.width, 12.0 / 16.0);
        assert_eq!(state.height, 12.0 / 16.0);
        assert_eq!(state.speed_x, 0.0);
        assert_eq!(state.speed_y, 0.0);
        assert!(state.active);
        assert_eq!(state.gravity, 0.0);
        assert_eq!(state.direction, LegacyEnemyDirection::Left);
        assert_eq!(state.frame, LegacySquidFrame::Open);
        assert_eq!(state.motion, LegacySquidMotion::Idle);
        assert_eq!(state.lifecycle, LegacySquidLifecycle::Swimming);
    }

    #[test]
    fn squid_idle_falls_and_switches_upward_when_it_reaches_nearest_player_band() {
        let constants = LegacySquidConstants::default();
        let mut state = LegacySquidState::spawn(5.0, 6.0);
        let start_x = state.x;
        let players = [
            LegacySquidPlayerTarget {
                x: state.x + 10.0,
                y: 100.0,
                height: 12.0 / 16.0,
            },
            LegacySquidPlayerTarget {
                x: state.x + 1.0,
                y: state.y + state.height + 24.0 / 16.0 - 12.0 / 16.0,
                height: 12.0 / 16.0,
            },
        ];

        assert_eq!(
            update_legacy_squid(&mut state, constants, DT, &players),
            LegacyEnemyUpdate::Keep
        );
        assert_eq!(state.motion, LegacySquidMotion::Upward { start_x });
        assert_eq!(state.direction, LegacyEnemyDirection::Right);
        assert_eq!(state.speed_x, 0.0);
        assert_eq!(state.speed_y, 0.0);
        assert_eq!(state.x, start_x);
        assert_eq!(state.y, 6.0 - 1.0 + 4.0 / 16.0);
    }

    #[test]
    fn squid_idle_trigger_uses_future_bottom_and_strict_nearest_player_ties() {
        let constants = LegacySquidConstants::default();
        let mut state = LegacySquidState::spawn(5.0, 6.0);
        let threshold_player_y =
            state.y + constants.fall_speed * DT + state.height + 1.0 / 16.0 + 24.0 / 16.0
                - 12.0 / 16.0;
        let players = [
            LegacySquidPlayerTarget {
                x: state.x - 1.0,
                y: threshold_player_y + 0.01,
                height: 12.0 / 16.0,
            },
            LegacySquidPlayerTarget {
                x: state.x + 1.0,
                y: threshold_player_y,
                height: 12.0 / 16.0,
            },
        ];

        let _ = update_legacy_squid(&mut state, constants, DT, &players);
        assert_eq!(state.motion, LegacySquidMotion::Idle);
        assert_eq!(state.direction, LegacyEnemyDirection::Left);
        assert_eq!(state.speed_y, constants.fall_speed);
        assert_close(state.y, 6.0 - 1.0 + 4.0 / 16.0 + constants.fall_speed * DT);
    }

    #[test]
    fn squid_upward_motion_accelerates_clamps_and_enters_downward_after_two_tiles() {
        let constants = LegacySquidConstants::default();
        let mut state = LegacySquidState::spawn(5.0, 6.0);
        state.direction = LegacyEnemyDirection::Right;
        state.speed_x = 2.5;
        state.speed_y = -2.5;
        state.motion = LegacySquidMotion::Upward {
            start_x: state.x - 2.0,
        };
        let start_y = state.y;

        let _ = update_legacy_squid(&mut state, constants, DT, &[]);

        assert_eq!(state.speed_x, 0.0);
        assert_eq!(state.speed_y, -constants.up_speed);
        assert_eq!(state.motion, LegacySquidMotion::Downward { start_y });
        assert_eq!(state.frame, LegacySquidFrame::Closed);
        assert_eq!(state.x, 5.0 - 1.0 + 2.0 / 16.0);
        assert_close(state.y, start_y - constants.up_speed * DT);
    }

    #[test]
    fn squid_downward_motion_falls_and_reopens_after_strict_distance() {
        let constants = LegacySquidConstants::default();
        let mut state = LegacySquidState::spawn(5.0, 6.0);
        let down_start = state.y - constants.down_distance;
        state.motion = LegacySquidMotion::Downward {
            start_y: down_start,
        };
        state.frame = LegacySquidFrame::Closed;

        let _ = update_legacy_squid(&mut state, constants, DT, &[]);
        assert_eq!(
            state.motion,
            LegacySquidMotion::Downward {
                start_y: down_start
            }
        );
        assert_eq!(state.frame, LegacySquidFrame::Closed);
        assert_eq!(state.speed_y, constants.fall_speed);

        state.y = down_start + constants.down_distance + 0.01;
        let _ = update_legacy_squid(&mut state, constants, DT, &[]);
        assert_eq!(state.motion, LegacySquidMotion::Idle);
        assert_eq!(state.frame, LegacySquidFrame::Open);
    }

    #[test]
    fn squid_shot_uses_legacy_impulse_gravity_and_direct_position_step() {
        let constants = LegacySquidConstants::default();
        let mut state = LegacySquidState::spawn(3.0, 4.0);

        shoot_legacy_squid(&mut state, constants, Some(LegacyEnemyDirection::Left));
        assert!(!state.active);
        assert_eq!(state.lifecycle, LegacySquidLifecycle::Shot);
        assert_eq!(state.gravity, constants.shot_gravity);
        assert_eq!(state.direction, LegacyEnemyDirection::Left);
        assert_eq!(state.speed_x, -constants.shot_speed_x);
        assert_eq!(state.speed_y, -constants.shot_jump_force);

        let start_x = state.x;
        let start_y = state.y;
        assert_eq!(
            update_legacy_squid(&mut state, constants, DT, &[]),
            LegacyEnemyUpdate::Keep
        );
        assert_eq!(state.speed_y, -2.0);
        assert_eq!(state.x, start_x - 0.4);
        assert_eq!(state.y, start_y - 0.2);
    }

    #[test]
    fn squid_rotation_aligns_and_collisions_suppress_default_response() {
        let constants = LegacySquidConstants::default();
        let mut state = LegacySquidState::spawn(1.0, 2.0);

        state.rotation = -1.0;
        let _ = update_legacy_squid(&mut state, constants, 0.05, &[]);
        assert_eq!(state.rotation, -0.25);

        assert_eq!(
            legacy_squid_collision(),
            super::LegacyEnemyCollisionResponse {
                suppress_default: true
            }
        );
    }

    #[test]
    fn lakito_spawn_matches_legacy_offsets_and_initial_state() {
        let state = LegacyLakitoState::spawn(10.0, 5.0);

        assert_eq!(state.start_y, 5.0);
        assert_eq!(state.x, 10.0 - 1.0 + 2.0 / 16.0);
        assert_eq!(state.y, 5.0 - 12.0 / 16.0);
        assert_eq!(state.width, 12.0 / 16.0);
        assert_eq!(state.height, 12.0 / 16.0);
        assert_eq!(state.speed_x, 0.0);
        assert_eq!(state.speed_y, 0.0);
        assert!(state.active);
        assert_eq!(state.gravity, 0.0);
        assert!(!state.passive);
        assert_eq!(state.direction, LegacyEnemyDirection::Left);
        assert_eq!(state.timer, 0.0);
        assert_eq!(state.lifecycle, LegacyLakitoLifecycle::Active);
    }

    #[test]
    fn lakito_active_update_throws_spikey_when_timer_strictly_exceeds_threshold() {
        let constants = LegacyLakitoConstants::default();
        let mut state = LegacyLakitoState::spawn(10.0, 5.0);
        state.timer = constants.throw_time;

        assert_eq!(
            update_legacy_lakito(&mut state, constants, 0.0, 0, &[], false, 30.0),
            LegacyLakitoUpdate::Idle
        );
        assert_eq!(state.timer, constants.throw_time);

        let throw_x = state.x + 6.0 / 16.0;
        let throw_y = state.y;
        assert_eq!(
            update_legacy_lakito(&mut state, constants, 0.01, 0, &[], false, 30.0),
            LegacyLakitoUpdate::ThrowSpikey {
                x: throw_x,
                y: throw_y
            }
        );
        assert_eq!(state.timer, 0.0);
    }

    #[test]
    fn lakito_rejects_throw_at_max_spikey_count_but_still_tracks_player() {
        let constants = LegacyLakitoConstants::default();
        let mut state = LegacyLakitoState::spawn(10.0, 5.0);
        state.x = 10.0;
        state.timer = constants.throw_time + 0.01;
        let players = [LegacyLakitoPlayerTarget {
            x: 20.0,
            speed_x: 0.0,
            dead: false,
        }];

        assert_eq!(
            update_legacy_lakito(
                &mut state,
                constants,
                0.0,
                constants.max_spikey_count,
                &players,
                false,
                30.0
            ),
            LegacyLakitoUpdate::Idle
        );
        assert_eq!(state.timer, constants.throw_time + 0.01);
        assert_eq!(state.direction, LegacyEnemyDirection::Right);
        assert_eq!(state.speed_x, 14.0);
    }

    #[test]
    fn lakito_movement_preserves_projected_player_selection_bug() {
        let constants = LegacyLakitoConstants::default();
        let mut state = LegacyLakitoState::spawn(10.0, 5.0);
        state.x = 10.0;
        let players = [
            LegacyLakitoPlayerTarget {
                x: 0.1,
                speed_x: 0.0,
                dead: false,
            },
            LegacyLakitoPlayerTarget {
                x: 20.0,
                speed_x: 0.0,
                dead: false,
            },
        ];

        let _ = update_legacy_lakito(
            &mut state,
            constants,
            0.0,
            constants.max_spikey_count,
            &players,
            false,
            30.0,
        );

        assert_eq!(state.direction, LegacyEnemyDirection::Left);
        assert_eq!(state.speed_x, -2.0);
    }

    #[test]
    fn lakito_shot_respawns_when_non_passive_after_strict_respawn_time() {
        let constants = LegacyLakitoConstants::default();
        let mut state = LegacyLakitoState::spawn(3.0, 4.0);

        shoot_legacy_lakito(&mut state, constants);
        assert!(!state.active);
        assert_eq!(state.lifecycle, LegacyLakitoLifecycle::Shot);
        assert_eq!(state.direction, LegacyEnemyDirection::Right);
        assert_eq!(state.gravity, constants.shot_gravity);
        assert_eq!(state.speed_x, 0.0);
        assert_eq!(state.speed_y, -constants.shot_jump_force);

        state.timer = constants.respawn_time;
        assert_eq!(
            update_legacy_lakito(&mut state, constants, 0.0, 0, &[], false, 30.0),
            LegacyLakitoUpdate::Idle
        );
        assert_eq!(state.lifecycle, LegacyLakitoLifecycle::Shot);

        assert_eq!(
            update_legacy_lakito(&mut state, constants, 0.01, 0, &[], false, 30.0),
            LegacyLakitoUpdate::Respawned
        );
        assert_eq!(state.x, 30.0);
        assert_eq!(state.y, state.start_y - 12.0 / 16.0);
        assert_eq!(state.timer, 0.0);
        assert_eq!(state.lifecycle, LegacyLakitoLifecycle::Active);
        assert!(state.active);
        assert_eq!(state.gravity, 0.0);
        assert_eq!(state.speed_x, 0.0);
        assert_eq!(state.speed_y, 0.0);
    }

    #[test]
    fn lakito_passive_and_lakito_end_preserve_legacy_update_order() {
        let constants = LegacyLakitoConstants::default();
        let mut state = LegacyLakitoState::spawn(3.0, 4.0);

        assert_eq!(
            update_legacy_lakito(
                &mut state,
                constants,
                0.1,
                constants.max_spikey_count,
                &[],
                true,
                30.0
            ),
            LegacyLakitoUpdate::Idle
        );
        assert!(state.passive);
        assert_eq!(state.speed_x, 0.0);

        let _ = update_legacy_lakito(
            &mut state,
            constants,
            0.0,
            constants.max_spikey_count,
            &[],
            false,
            30.0,
        );
        assert_eq!(state.speed_x, -constants.passive_speed);

        shoot_legacy_lakito(&mut state, constants);
        state.timer = constants.respawn_time + 1.0;
        assert_eq!(
            update_legacy_lakito(&mut state, constants, 0.1, 0, &[], false, 30.0),
            LegacyLakitoUpdate::Idle
        );
        assert_eq!(state.lifecycle, LegacyLakitoLifecycle::Shot);
        assert!(!state.active);
    }

    #[test]
    fn lakito_stomp_and_spikeyfall_collision_enter_shot_state_and_suppress_default() {
        let constants = LegacyLakitoConstants::default();
        let mut stomped = LegacyLakitoState::spawn(3.0, 4.0);

        stomp_legacy_lakito(&mut stomped, constants);
        assert_eq!(stomped.lifecycle, LegacyLakitoLifecycle::Shot);
        assert_eq!(stomped.direction, LegacyEnemyDirection::Right);
        assert_eq!(stomped.speed_y, 0.0);
        assert!(!stomped.active);

        let mut collided = LegacyLakitoState::spawn(3.0, 4.0);
        assert_eq!(
            legacy_lakito_spikeyfall_collision(&mut collided, constants),
            super::LegacyEnemyCollisionResponse {
                suppress_default: true
            }
        );
        assert_eq!(collided.lifecycle, LegacyLakitoLifecycle::Shot);
        assert_eq!(collided.speed_y, -constants.shot_jump_force);

        assert_eq!(
            legacy_lakito_collision(),
            super::LegacyEnemyCollisionResponse {
                suppress_default: true
            }
        );
    }

    #[test]
    fn hammer_spawn_matches_legacy_offsets_direction_and_initial_mask_slot() {
        let constants = LegacyHammerConstants::default();
        let left = LegacyHammerState::spawn(5.0, 7.0, LegacyEnemyDirection::Left, constants);
        let right = LegacyHammerState::spawn(5.0, 7.0, LegacyEnemyDirection::Right, constants);

        assert_eq!(left.x, 5.0);
        assert_eq!(left.y, 6.0);
        assert_eq!(left.start_y, 6.0);
        assert_eq!(left.speed_x, -constants.speed);
        assert_eq!(right.speed_x, constants.speed);
        assert_eq!(left.speed_y, -constants.start_y_speed);
        assert_eq!(left.width, 12.0 / 16.0);
        assert_eq!(left.height, 12.0 / 16.0);
        assert!(left.active);
        assert_eq!(left.gravity, constants.gravity);
        assert_eq!(left.animation_direction, LegacyEnemyDirection::Left);
        assert_eq!(right.animation_direction, LegacyEnemyDirection::Right);
        assert_eq!(left.animation_timer, 0.0);
        assert_eq!(left.frame, LegacyHammerFrame::One);
        assert!(!left.kill_stuff);
        assert!(!left.hammer_bro_collision_mask_active);
    }

    #[test]
    fn hammer_animation_uses_strict_timer_threshold_and_wraps_four_frames() {
        let constants = LegacyHammerConstants::default();
        let mut state = LegacyHammerState::spawn(5.0, 7.0, LegacyEnemyDirection::Left, constants);

        assert_eq!(
            update_legacy_hammer(&mut state, constants, constants.animation_speed),
            LegacyEnemyUpdate::Keep
        );
        assert_eq!(state.frame, LegacyHammerFrame::One);
        assert_eq!(state.animation_timer, constants.animation_speed);

        let _ = update_legacy_hammer(&mut state, constants, 0.01);
        assert_eq!(state.frame, LegacyHammerFrame::Two);
        assert_close(state.animation_timer, 0.01);

        let _ = update_legacy_hammer(&mut state, constants, constants.animation_speed * 3.0);
        assert_eq!(state.frame, LegacyHammerFrame::One);
        assert_close(state.animation_timer, 0.01);
    }

    #[test]
    fn hammer_mask_slot_twenty_decay_uses_strict_y_threshold_when_active() {
        let constants = LegacyHammerConstants::default();
        let mut state = LegacyHammerState::spawn(5.0, 7.0, LegacyEnemyDirection::Left, constants);
        state.hammer_bro_collision_mask_active = true;

        state.y = state.start_y + 1.0;
        let _ = update_legacy_hammer(&mut state, constants, 0.0);
        assert!(state.hammer_bro_collision_mask_active);

        state.y = state.start_y + 1.01;
        let _ = update_legacy_hammer(&mut state, constants, 0.0);
        assert!(!state.hammer_bro_collision_mask_active);
    }

    #[test]
    fn hammer_portal_sets_killstuff_and_collisions_suppress_default_response() {
        let constants = LegacyHammerConstants::default();
        let mut state = LegacyHammerState::spawn(5.0, 7.0, LegacyEnemyDirection::Right, constants);

        portal_legacy_hammer(&mut state);
        assert!(state.kill_stuff);

        assert_eq!(
            legacy_hammer_collision(),
            super::LegacyEnemyCollisionResponse {
                suppress_default: true
            }
        );
    }

    #[test]
    fn hammer_bro_spawn_matches_legacy_offsets_and_initial_state() {
        let constants = LegacyHammerBroConstants::default();
        let state = LegacyHammerBroState::spawn(10.0, 5.0, constants.throw_time_long, constants);

        assert_eq!(state.start_x, 10.0);
        assert_eq!(state.start_y, 5.0);
        assert_eq!(state.x, 10.0 - 6.0 / 16.0);
        assert_eq!(state.y, 5.0 - 12.0 / 16.0);
        assert_eq!(state.speed_x, -constants.speed);
        assert_eq!(state.speed_y, 0.0);
        assert_eq!(state.width, 12.0 / 16.0);
        assert_eq!(state.height, 12.0 / 16.0);
        assert!(state.active);
        assert_eq!(state.gravity, constants.gravity);
        assert_eq!(state.rotation, 0.0);
        assert_eq!(state.direction, LegacyEnemyDirection::Left);
        assert_eq!(state.animation_direction, LegacyEnemyDirection::Left);
        assert_eq!(state.animation_timer, 0.0);
        assert_eq!(state.frame, LegacyHammerBroFrame::One);
        assert!(!state.preparing_throw);
        assert!(!state.falling);
        assert_eq!(state.throw_timer, constants.throw_time_long);
        assert_eq!(state.jump_timer, 0.0);
        assert_eq!(state.jumping, LegacyHammerBroJumpState::None);
        assert_eq!(state.jumping_y, 0.0);
        assert!(!state.floor_collision_mask_active);
        assert_eq!(state.lifecycle, LegacyHammerBroLifecycle::Active);
    }

    #[test]
    fn hammer_bro_active_throw_timer_uses_lua_less_or_equal_gate_and_injected_next_timer() {
        let constants = LegacyHammerBroConstants::default();
        let mut state = LegacyHammerBroState::spawn(3.0, 4.0, 0.1, constants);

        assert_eq!(
            update_legacy_hammer_bro_active(
                &mut state,
                constants,
                0.09,
                &[],
                LegacyHammerBroJumpDecision::Up,
                || panic!("throw timer should remain positive")
            ),
            LegacyHammerBroUpdate::Idle
        );
        assert_close(state.throw_timer, 0.01);

        assert_eq!(
            update_legacy_hammer_bro_active(
                &mut state,
                constants,
                0.01,
                &[],
                LegacyHammerBroJumpDecision::Up,
                || constants.throw_time_long
            ),
            LegacyHammerBroUpdate::ThrowHammer {
                direction: LegacyEnemyDirection::Left
            }
        );
        assert_eq!(state.throw_timer, constants.throw_time_long);
    }

    #[test]
    fn hammer_bro_active_patrol_bounds_and_turns_toward_nearest_player() {
        let constants = LegacyHammerBroConstants::default();
        let mut state =
            LegacyHammerBroState::spawn(10.0, 5.0, constants.throw_time_long, constants);
        state.x = state.start_x - 1.01;
        state.speed_x = -constants.speed;
        let right_player = state.x + 10.0;

        let _ = update_legacy_hammer_bro_active(
            &mut state,
            constants,
            0.0,
            &[right_player],
            LegacyHammerBroJumpDecision::Up,
            || panic!("no throw should happen"),
        );
        assert_eq!(state.speed_x, constants.speed);
        assert_eq!(state.direction, LegacyEnemyDirection::Right);
        assert_eq!(state.animation_direction, LegacyEnemyDirection::Right);

        state.x = state.start_x + 0.01;
        state.speed_x = constants.speed;
        let left_player = state.x - 2.0;
        let far_right_player = state.x + 10.0;
        let _ = update_legacy_hammer_bro_active(
            &mut state,
            constants,
            0.0,
            &[left_player, far_right_player],
            LegacyHammerBroJumpDecision::Up,
            || panic!("no throw should happen"),
        );
        assert_eq!(state.speed_x, -constants.speed);
        assert_eq!(state.direction, LegacyEnemyDirection::Left);
        assert_eq!(state.animation_direction, LegacyEnemyDirection::Left);
    }

    #[test]
    fn hammer_bro_active_jump_timer_uses_strict_gate_and_injected_middle_band_choice() {
        let constants = LegacyHammerBroConstants::default();
        let mut state = LegacyHammerBroState::spawn(3.0, 8.0, constants.throw_time_long, constants);
        state.jump_timer = constants.jump_time;

        let _ = update_legacy_hammer_bro_active(
            &mut state,
            constants,
            0.0,
            &[],
            LegacyHammerBroJumpDecision::Down,
            || panic!("no throw should happen"),
        );
        assert_eq!(state.jumping, LegacyHammerBroJumpState::None);

        let _ = update_legacy_hammer_bro_active(
            &mut state,
            constants,
            0.01,
            &[],
            LegacyHammerBroJumpDecision::Down,
            || panic!("no throw should happen"),
        );
        assert_eq!(state.jumping, LegacyHammerBroJumpState::Down);
        assert_eq!(state.speed_y, -constants.jump_force_down);
        assert_eq!(state.jumping_y, state.y);
        assert!(state.floor_collision_mask_active);
        assert_close(state.jump_timer, 0.01);

        state.y = state.jumping_y + 2.01;
        let _ = update_legacy_hammer_bro_active(
            &mut state,
            constants,
            0.0,
            &[],
            LegacyHammerBroJumpDecision::Down,
            || panic!("no throw should happen"),
        );
        assert_eq!(state.jumping, LegacyHammerBroJumpState::None);
        assert!(!state.floor_collision_mask_active);
    }

    #[test]
    fn hammer_bro_active_jump_direction_uses_y_bounds_before_random_choice() {
        let constants = LegacyHammerBroConstants::default();
        let mut high = LegacyHammerBroState::spawn(3.0, 13.0, constants.throw_time_long, constants);
        high.jump_timer = constants.jump_time + 0.01;

        let _ = update_legacy_hammer_bro_active(
            &mut high,
            constants,
            0.0,
            &[],
            LegacyHammerBroJumpDecision::Down,
            || panic!("no throw should happen"),
        );
        assert_eq!(high.jumping, LegacyHammerBroJumpState::Up);
        assert_eq!(high.speed_y, -constants.jump_force);

        let mut low = LegacyHammerBroState::spawn(3.0, 5.0, constants.throw_time_long, constants);
        low.jump_timer = constants.jump_time + 0.01;
        let _ = update_legacy_hammer_bro_active(
            &mut low,
            constants,
            0.0,
            &[],
            LegacyHammerBroJumpDecision::Up,
            || panic!("no throw should happen"),
        );
        assert_eq!(low.jumping, LegacyHammerBroJumpState::Down);
        assert_eq!(low.speed_y, -constants.jump_force_down);
    }

    #[test]
    fn hammer_bro_active_animation_preserves_strict_timer_and_prepare_pose_update() {
        let constants = LegacyHammerBroConstants::default();
        let mut state =
            LegacyHammerBroState::spawn(3.0, 8.0, constants.prepare_time - 0.1, constants);

        let _ = update_legacy_hammer_bro_active(
            &mut state,
            constants,
            constants.animation_speed,
            &[],
            LegacyHammerBroJumpDecision::Up,
            || panic!("no throw should happen"),
        );
        assert_eq!(state.frame, LegacyHammerBroFrame::One);
        assert!(!state.preparing_throw);
        assert_eq!(state.animation_timer, constants.animation_speed);

        let _ = update_legacy_hammer_bro_active(
            &mut state,
            constants,
            0.01,
            &[],
            LegacyHammerBroJumpDecision::Up,
            || panic!("no throw should happen"),
        );
        assert_eq!(state.frame, LegacyHammerBroFrame::Two);
        assert!(state.preparing_throw);
        assert_close(state.animation_timer, 0.01);
    }

    #[test]
    fn hammer_bro_active_speed_easing_preserves_negative_overshoot_quirk() {
        let constants = LegacyHammerBroConstants::default();
        let mut positive =
            LegacyHammerBroState::spawn(3.0, 8.0, constants.throw_time_long, constants);
        positive.speed_x = 2.0;

        let _ = update_legacy_hammer_bro_active(
            &mut positive,
            constants,
            0.1,
            &[],
            LegacyHammerBroJumpDecision::Up,
            || panic!("no throw should happen"),
        );
        assert_eq!(positive.speed_x, constants.speed);

        let mut negative =
            LegacyHammerBroState::spawn(3.0, 8.0, constants.throw_time_long, constants);
        negative.speed_x = -2.0;

        let _ = update_legacy_hammer_bro_active(
            &mut negative,
            constants,
            0.1,
            &[],
            LegacyHammerBroJumpDecision::Up,
            || panic!("no throw should happen"),
        );
        assert_close(negative.speed_x, -0.6);
    }

    #[test]
    fn hammer_bro_shot_preserves_legacy_nil_direction_and_direct_motion_step() {
        let constants = LegacyHammerBroConstants::default();
        let mut state =
            LegacyHammerBroState::spawn(3.0, 4.0, constants.throw_time_short, constants);

        shoot_legacy_hammer_bro(&mut state, constants);
        assert_eq!(state.lifecycle, LegacyHammerBroLifecycle::Shot);
        assert_eq!(state.direction, LegacyEnemyDirection::Right);
        assert!(!state.active);
        assert_eq!(state.gravity, constants.shot_gravity);
        assert_eq!(state.speed_x, 0.0);
        assert_eq!(state.speed_y, -constants.shot_jump_force);

        state.rotation = 1.0;
        let start_x = state.x;
        let start_y = state.y;
        assert_eq!(
            update_legacy_hammer_bro_shot(&mut state, constants, DT),
            LegacyEnemyUpdate::Keep
        );
        assert_eq!(state.rotation, 0.0);
        assert_eq!(state.speed_y, -2.0);
        assert_eq!(state.x, start_x);
        assert_eq!(state.y, start_y - 0.2);

        state.y = constants.shot_removal_y;
        state.speed_y = 0.0;
        assert_eq!(
            update_legacy_hammer_bro_shot(&mut state, constants, 0.0),
            LegacyEnemyUpdate::Keep
        );

        state.speed_y = 1.0;
        assert_eq!(
            update_legacy_hammer_bro_shot(&mut state, constants, 0.01),
            LegacyEnemyUpdate::Remove
        );
    }

    #[test]
    fn hammer_bro_stomp_emancipate_and_portal_update_legacy_state() {
        let constants = LegacyHammerBroConstants::default();
        let mut stomped =
            LegacyHammerBroState::spawn(3.0, 4.0, constants.throw_time_short, constants);

        stomp_legacy_hammer_bro(&mut stomped, constants);
        assert_eq!(stomped.lifecycle, LegacyHammerBroLifecycle::Shot);
        assert_eq!(stomped.speed_y, 0.0);
        assert!(!stomped.active);

        let mut portaled =
            LegacyHammerBroState::spawn(3.0, 4.0, constants.throw_time_short, constants);
        portaled.jumping = LegacyHammerBroJumpState::Down;
        portaled.floor_collision_mask_active = true;
        portal_legacy_hammer_bro(&mut portaled);
        assert_eq!(portaled.jumping, LegacyHammerBroJumpState::None);
        assert!(!portaled.floor_collision_mask_active);

        emancipate_legacy_hammer_bro(&mut portaled, constants);
        assert_eq!(portaled.lifecycle, LegacyHammerBroLifecycle::Shot);
        assert_eq!(portaled.speed_y, -constants.shot_jump_force);
    }

    #[test]
    fn hammer_bro_side_collisions_set_legacy_goomba_speed_and_suppress_default() {
        let constants = LegacyHammerBroConstants::default();
        let mut left = LegacyHammerBroState::spawn(3.0, 4.0, constants.throw_time_short, constants);

        assert_eq!(
            legacy_hammer_bro_left_collision(
                &mut left,
                constants,
                LegacyHammerBroCollisionActor::Other
            ),
            super::LegacyEnemyCollisionResponse {
                suppress_default: true
            }
        );
        assert_eq!(left.speed_x, constants.side_collision_speed);
        assert_eq!(left.lifecycle, LegacyHammerBroLifecycle::Active);

        let mut right =
            LegacyHammerBroState::spawn(3.0, 4.0, constants.throw_time_short, constants);
        assert_eq!(
            legacy_hammer_bro_right_collision(
                &mut right,
                constants,
                LegacyHammerBroCollisionActor::BulletBill
            ),
            super::LegacyEnemyCollisionResponse {
                suppress_default: true
            }
        );
        assert_eq!(right.lifecycle, LegacyHammerBroLifecycle::Shot);
        assert_eq!(right.direction, LegacyEnemyDirection::Right);
        assert_eq!(right.speed_x, 0.0);
    }

    #[test]
    fn hammer_bro_ceil_and_floor_collisions_preserve_default_resolution_contract() {
        let constants = LegacyHammerBroConstants::default();
        let mut ceiling =
            LegacyHammerBroState::spawn(3.0, 4.0, constants.throw_time_short, constants);

        assert_eq!(
            legacy_hammer_bro_ceil_collision(
                &mut ceiling,
                constants,
                LegacyHammerBroCollisionActor::PlayerOrBox
            ),
            super::LegacyEnemyCollisionResponse {
                suppress_default: false
            }
        );
        assert_eq!(ceiling.lifecycle, LegacyHammerBroLifecycle::Shot);
        assert_eq!(ceiling.speed_y, 0.0);

        let mut floor =
            LegacyHammerBroState::spawn(3.0, 4.0, constants.throw_time_short, constants);
        assert_eq!(
            legacy_hammer_bro_floor_collision(
                &mut floor,
                constants,
                LegacyHammerBroCollisionActor::KillHammer
            ),
            super::LegacyEnemyCollisionResponse {
                suppress_default: false
            }
        );
        assert_eq!(floor.lifecycle, LegacyHammerBroLifecycle::Shot);
        assert_eq!(floor.speed_y, -constants.shot_jump_force);
    }

    #[test]
    fn hammer_bro_moving_koopa_shell_kills_on_ceiling_collision() {
        let constants = LegacyHammerBroConstants::default();
        let mut state =
            LegacyHammerBroState::spawn(3.0, 4.0, constants.throw_time_short, constants);

        let _ = legacy_hammer_bro_ceil_collision(
            &mut state,
            constants,
            LegacyHammerBroCollisionActor::MovingKoopaShell,
        );

        assert_eq!(state.lifecycle, LegacyHammerBroLifecycle::Shot);
        assert_eq!(state.direction, LegacyEnemyDirection::Right);
    }

    #[test]
    fn flying_fish_spawn_uses_injected_speed_and_zero_speed_fix() {
        let constants = LegacyFlyingFishConstants::default();
        let state = LegacyFlyingFishState::spawn(12.0, 3.0, -3.0, constants);

        assert_eq!(state.x, 12.0);
        assert_eq!(state.y, 15.0);
        assert_eq!(state.width, 12.0 / 16.0);
        assert_eq!(state.height, 12.0 / 16.0);
        assert_eq!(state.speed_x, 1.0);
        assert_eq!(state.speed_y, -constants.jump_force);
        assert!(state.active);
        assert_eq!(state.gravity, constants.gravity);
        assert_eq!(state.animation_direction, LegacyEnemyDirection::Left);
        assert_eq!(state.frame, None);
        assert_eq!(state.lifecycle, LegacyFlyingFishLifecycle::Flying);

        let leftward = LegacyFlyingFishState::spawn(10.0, -2.0, -1.0, constants);
        assert_eq!(leftward.speed_x, -3.0);
        assert_eq!(leftward.animation_direction, LegacyEnemyDirection::Right);
    }

    #[test]
    fn flying_fish_animation_uses_strict_timer_and_uninitialized_frame_quirk() {
        let constants = LegacyFlyingFishConstants::default();
        let mut state = LegacyFlyingFishState::spawn(1.0, 0.0, 0.0, constants);

        assert_eq!(
            update_legacy_flying_fish(&mut state, constants, constants.animation_speed),
            LegacyEnemyUpdate::Keep
        );
        assert_eq!(state.frame, None);
        assert_eq!(state.animation_timer, constants.animation_speed);

        let _ = update_legacy_flying_fish(&mut state, constants, 0.01);
        assert_eq!(state.frame, Some(LegacyFlyingFishFrame::One));
        assert_close(state.animation_timer, 0.01);

        let _ = update_legacy_flying_fish(&mut state, constants, constants.animation_speed + 0.01);
        assert_eq!(state.frame, Some(LegacyFlyingFishFrame::Two));
        assert_close(state.animation_timer, 0.02);
    }

    #[test]
    fn flying_fish_shot_uses_legacy_impulse_gravity_and_direct_motion_step() {
        let constants = LegacyFlyingFishConstants::default();
        let mut state = LegacyFlyingFishState::spawn(4.0, 1.0, 0.0, constants);

        shoot_legacy_flying_fish(&mut state, constants, Some(LegacyEnemyDirection::Left));

        assert_eq!(state.lifecycle, LegacyFlyingFishLifecycle::Shot);
        assert!(!state.active);
        assert_eq!(state.gravity, constants.shot_gravity);
        assert_eq!(state.direction, LegacyEnemyDirection::Left);
        assert_eq!(state.speed_x, -constants.shot_speed_x);
        assert_eq!(state.speed_y, -constants.shot_jump_force);

        let start_x = state.x;
        let start_y = state.y;
        assert_eq!(
            update_legacy_flying_fish(&mut state, constants, DT),
            LegacyEnemyUpdate::Keep
        );
        assert_close(
            state.speed_y,
            -constants.shot_jump_force + constants.shot_gravity * DT,
        );
        assert_close(state.x, start_x - constants.shot_speed_x * DT);
        assert_close(state.y, start_y + state.speed_y * DT);
    }

    #[test]
    fn flying_fish_rotation_recovers_without_modulo() {
        let constants = LegacyFlyingFishConstants::default();
        let mut positive = LegacyFlyingFishState::spawn(1.0, 1.0, 0.0, constants);
        positive.rotation = core::f32::consts::TAU + 1.0;

        let _ = update_legacy_flying_fish(&mut positive, constants, 0.05);
        assert_close(
            positive.rotation,
            core::f32::consts::TAU + 1.0 - constants.rotation_alignment_speed * 0.05,
        );

        let mut negative = LegacyFlyingFishState::spawn(1.0, 1.0, 0.0, constants);
        negative.rotation = -1.0;

        let _ = update_legacy_flying_fish(&mut negative, constants, 0.05);
        assert_close(
            negative.rotation,
            -1.0 + constants.rotation_alignment_speed * 0.05,
        );
    }

    #[test]
    fn flying_fish_stomp_defaults_to_right_shot_and_collisions_suppress_default() {
        let constants = LegacyFlyingFishConstants::default();
        let mut state = LegacyFlyingFishState::spawn(4.0, -2.0, 0.0, constants);

        stomp_legacy_flying_fish(&mut state, constants);

        assert_eq!(state.lifecycle, LegacyFlyingFishLifecycle::Shot);
        assert_eq!(state.direction, LegacyEnemyDirection::Right);
        assert_eq!(state.speed_x, constants.shot_speed_x);
        assert!(!state.active);
        assert_eq!(
            legacy_flying_fish_collision(),
            super::LegacyEnemyCollisionResponse {
                suppress_default: true
            }
        );
    }

    #[test]
    fn goomba_spawn_matches_legacy_offsets_and_initial_motion() {
        let state = LegacyGoombaState::spawn(10.0, 5.0, LegacyGoombaConstants::default());

        assert_eq!(state.x, 10.0 - 6.0 / 16.0);
        assert_eq!(state.y, 5.0 - 11.0 / 16.0);
        assert_eq!(state.width, 12.0 / 16.0);
        assert_eq!(state.height, 12.0 / 16.0);
        assert_eq!(state.speed_x, -2.0);
        assert_eq!(state.speed_y, 0.0);
        assert!(state.active);
        assert_eq!(state.variant, LegacyGoombaVariant::Goomba);
        assert_eq!(state.animation_direction, LegacyEnemyDirection::Left);
        assert_eq!(state.frame, LegacyGoombaFrame::Goomba);
        assert_eq!(state.lifecycle, LegacyGoombaLifecycle::Walking);
    }

    #[test]
    fn spikey_spawn_reuses_goomba_body_and_motion_with_spikey_frame() {
        let constants = LegacyGoombaConstants::default();
        let state = LegacyGoombaState::spawn_spikey(10.0, 5.0, constants);

        assert_eq!(state.x, 10.0 - 6.0 / 16.0);
        assert_eq!(state.y, 5.0 - 11.0 / 16.0);
        assert_eq!(state.width, 12.0 / 16.0);
        assert_eq!(state.height, 12.0 / 16.0);
        assert_eq!(state.speed_x, -constants.speed);
        assert_eq!(state.speed_y, 0.0);
        assert!(state.active);
        assert_eq!(state.variant, LegacyGoombaVariant::Spikey);
        assert_eq!(state.frame, LegacyGoombaFrame::SpikeyOne);
        assert_eq!(state.lifecycle, LegacyGoombaLifecycle::Walking);
    }

    #[test]
    fn falling_spikey_spawn_matches_lakito_thrown_state() {
        let constants = LegacyGoombaConstants::default();
        let state = LegacyGoombaState::spawn_spikey_falling(10.0, 5.0, constants);

        assert_eq!(state.x, 10.0 - 6.0 / 16.0);
        assert_eq!(state.y, 5.0 - 11.0 / 16.0);
        assert_eq!(state.start_y, state.y);
        assert_eq!(state.speed_x, 0.0);
        assert_eq!(state.speed_y, -10.0);
        assert_eq!(state.variant, LegacyGoombaVariant::SpikeyFalling);
        assert_eq!(state.frame, LegacyGoombaFrame::SpikeyFallingOne);
        assert_eq!(state.gravity, Some(30.0));
        assert!(state.lakito_collision_mask_active);
        assert!(legacy_spikey_falling_suppresses_lakito_collision(&state));
    }

    #[test]
    fn walking_goomba_animation_uses_strict_timer_threshold_and_flips_direction() {
        let constants = LegacyGoombaConstants::default();
        let mut state = LegacyGoombaState::spawn(1.0, 1.0, constants);

        assert_eq!(
            update_legacy_goomba(&mut state, constants, 0.2),
            LegacyEnemyUpdate::Keep
        );
        assert_eq!(state.animation_direction, LegacyEnemyDirection::Left);
        assert_eq!(state.animation_timer, 0.2);

        let _ = update_legacy_goomba(&mut state, constants, 0.01);
        assert_eq!(state.animation_direction, LegacyEnemyDirection::Right);
        assert_close(state.animation_timer, 0.01);
    }

    #[test]
    fn walking_spikey_animation_uses_strict_timer_threshold_and_toggles_frames() {
        let constants = LegacyGoombaConstants::default();
        let mut state = LegacyGoombaState::spawn_spikey(1.0, 1.0, constants);

        assert_eq!(
            update_legacy_goomba(&mut state, constants, 0.2),
            LegacyEnemyUpdate::Keep
        );
        assert_eq!(state.frame, LegacyGoombaFrame::SpikeyOne);
        assert_eq!(state.animation_timer, 0.2);

        let _ = update_legacy_goomba(&mut state, constants, 0.01);
        assert_eq!(state.frame, LegacyGoombaFrame::SpikeyTwo);
        assert_eq!(state.animation_direction, LegacyEnemyDirection::Left);
        assert_close(state.animation_timer, 0.01);
    }

    #[test]
    fn falling_spikey_animation_resets_rotation_and_disables_lakito_mask_after_drop() {
        let constants = LegacyGoombaConstants::default();
        let mut state = LegacyGoombaState::spawn_spikey_falling(1.0, 1.0, constants);

        state.rotation = 1.0;
        state.speed_x = 5.0;
        state.y = state.start_y + 2.01;
        assert_eq!(
            update_legacy_goomba(&mut state, constants, 0.2),
            LegacyEnemyUpdate::Keep
        );
        assert_eq!(state.rotation, 0.0);
        assert_eq!(state.speed_x, 5.0);
        assert_eq!(state.frame, LegacyGoombaFrame::SpikeyFallingOne);
        assert!(state.lakito_collision_mask_active);

        let _ = update_legacy_goomba(&mut state, constants, 0.01);
        assert_eq!(state.frame, LegacyGoombaFrame::SpikeyFallingTwo);
        assert!(!state.lakito_collision_mask_active);
    }

    #[test]
    fn walking_goomba_speed_eases_toward_legacy_speed_with_global_friction() {
        let constants = LegacyGoombaConstants::default();
        let mut state = LegacyGoombaState::spawn(1.0, 1.0, constants);

        state.speed_x = -5.0;
        let _ = update_legacy_goomba(&mut state, constants, DT);
        assert_eq!(state.speed_x, -2.2);

        let _ = update_legacy_goomba(&mut state, constants, DT);
        assert_eq!(state.speed_x, -2.0);

        state.speed_x = 0.0;
        let _ = update_legacy_goomba(&mut state, constants, DT);
        assert_eq!(state.speed_x, -2.0);
    }

    #[test]
    fn side_collisions_turn_walking_goomba_around_and_suppress_default_resolution() {
        let constants = LegacyGoombaConstants::default();
        let mut state = LegacyGoombaState::spawn(1.0, 1.0, constants);

        let response = legacy_goomba_left_collision(&mut state, constants);
        assert_eq!(
            response,
            super::LegacyEnemyCollisionResponse {
                suppress_default: true
            }
        );
        assert_eq!(state.speed_x, constants.speed);

        let response = legacy_goomba_right_collision(&mut state, constants);
        assert_eq!(
            response,
            super::LegacyEnemyCollisionResponse {
                suppress_default: true
            }
        );
        assert_eq!(state.speed_x, -constants.speed);
    }

    #[test]
    fn side_collisions_do_not_reactivate_or_turn_non_walking_goomba() {
        let constants = LegacyGoombaConstants::default();
        let mut state = LegacyGoombaState::spawn(1.0, 1.0, constants);

        stomp_legacy_goomba(&mut state);
        assert_eq!(
            legacy_goomba_left_collision(&mut state, constants),
            super::LegacyEnemyCollisionResponse {
                suppress_default: true
            }
        );
        assert_eq!(state.speed_x, -constants.speed);
        assert!(!state.active);
    }

    #[test]
    fn spikey_side_collisions_turn_and_set_legacy_animation_direction() {
        let constants = LegacyGoombaConstants::default();
        let mut state = LegacyGoombaState::spawn_spikey(1.0, 1.0, constants);

        let response = legacy_goomba_left_collision(&mut state, constants);
        assert_eq!(
            response,
            super::LegacyEnemyCollisionResponse {
                suppress_default: true
            }
        );
        assert_eq!(state.speed_x, constants.speed);
        assert_eq!(state.animation_direction, LegacyEnemyDirection::Left);

        let response = legacy_goomba_right_collision(&mut state, constants);
        assert_eq!(
            response,
            super::LegacyEnemyCollisionResponse {
                suppress_default: true
            }
        );
        assert_eq!(state.speed_x, -constants.speed);
        assert_eq!(state.animation_direction, LegacyEnemyDirection::Right);
    }

    #[test]
    fn falling_spikey_floor_collision_becomes_walking_spikey_and_aims_from_player_position() {
        let constants = LegacyGoombaConstants::default();
        let mut state = LegacyGoombaState::spawn_spikey_falling(5.0, 2.0, constants);
        let left_player = state.x - 1.0;

        assert_eq!(
            legacy_spikey_falling_floor_collision(&mut state, constants, &[left_player]),
            super::LegacyEnemyCollisionResponse {
                suppress_default: false
            }
        );
        assert_eq!(state.variant, LegacyGoombaVariant::Spikey);
        assert_eq!(state.frame, LegacyGoombaFrame::SpikeyOne);
        assert_eq!(state.gravity, None);
        assert!(!state.lakito_collision_mask_active);
        assert_eq!(state.speed_x, -constants.speed);

        let mut state = LegacyGoombaState::spawn_spikey_falling(5.0, 2.0, constants);
        let right_player = state.x + 1.0;
        assert_eq!(
            legacy_spikey_falling_floor_collision(&mut state, constants, &[right_player]),
            super::LegacyEnemyCollisionResponse {
                suppress_default: false
            }
        );
        assert_eq!(state.speed_x, constants.speed);
        assert_eq!(state.animation_direction, LegacyEnemyDirection::Left);
    }

    #[test]
    fn falling_spikey_floor_collision_preserves_legacy_nearest_player_bug() {
        let constants = LegacyGoombaConstants::default();
        let mut state = LegacyGoombaState::spawn_spikey_falling(10.0, 2.0, constants);
        state.x = 4.0;

        let _ = legacy_spikey_falling_floor_collision(&mut state, constants, &[1.0, 6.0]);

        assert_eq!(state.speed_x, -constants.speed);
    }

    #[test]
    fn stomped_goomba_becomes_inactive_and_expires_after_legacy_death_time() {
        let constants = LegacyGoombaConstants::default();
        let mut state = LegacyGoombaState::spawn(1.0, 1.0, constants);

        stomp_legacy_goomba(&mut state);
        assert!(!state.active);
        assert_eq!(
            state.lifecycle,
            LegacyGoombaLifecycle::Stomped { death_timer: 0.0 }
        );

        assert_eq!(
            update_legacy_goomba(&mut state, constants, 0.5),
            LegacyEnemyUpdate::Keep
        );
        assert_eq!(
            update_legacy_goomba(&mut state, constants, 0.01),
            LegacyEnemyUpdate::Remove
        );
    }

    #[test]
    fn shot_goomba_uses_legacy_impulse_gravity_and_direct_position_step() {
        let constants = LegacyGoombaConstants::default();
        let mut state = LegacyGoombaState::spawn(3.0, 4.0, constants);

        shoot_legacy_goomba(&mut state, constants, LegacyEnemyDirection::Left);
        assert!(!state.active);
        assert_eq!(state.lifecycle, LegacyGoombaLifecycle::Shot);
        assert_eq!(state.speed_x, -4.0);
        assert_eq!(state.speed_y, -8.0);

        let start_x = state.x;
        let start_y = state.y;
        assert_eq!(
            update_legacy_goomba(&mut state, constants, 0.1),
            LegacyEnemyUpdate::Keep
        );
        assert_eq!(state.speed_y, -2.0);
        assert_eq!(state.x, start_x - 0.4);
        assert_eq!(state.y, start_y - 0.2);
    }

    #[test]
    fn goomba_rotation_aligns_toward_zero_like_portal_recovery() {
        let constants = LegacyGoombaConstants::default();
        let mut state = LegacyGoombaState::spawn(1.0, 1.0, constants);

        state.rotation = 1.0;
        let _ = update_legacy_goomba(&mut state, constants, 0.05);
        assert_eq!(state.rotation, 0.25);
        let _ = update_legacy_goomba(&mut state, constants, 0.05);
        assert_eq!(state.rotation, 0.0);

        state.rotation = -1.0;
        let _ = update_legacy_goomba(&mut state, constants, 0.05);
        assert_eq!(state.rotation, -0.25);
    }

    #[test]
    fn koopa_spawn_matches_legacy_offsets_and_initial_motion() {
        let constants = LegacyKoopaConstants::default();
        let state = LegacyKoopaState::spawn(10.0, 5.0, LegacyKoopaVariant::Green, constants);

        assert_eq!(state.x, 10.0 - 6.0 / 16.0);
        assert_eq!(state.y, 5.0 - 11.0 / 16.0);
        assert_eq!(state.width, 12.0 / 16.0);
        assert_eq!(state.height, 12.0 / 16.0);
        assert_eq!(state.speed_x, -2.0);
        assert_eq!(state.speed_y, 0.0);
        assert_eq!(state.variant, LegacyKoopaVariant::Green);
        assert!(state.active);
        assert!(!state.small);
        assert!(!state.flying);
        assert_eq!(state.combo, 1);
        assert_eq!(state.animation_direction, LegacyEnemyDirection::Right);
        assert_eq!(state.frame, LegacyKoopaFrame::WalkingOne);
        assert_eq!(state.gravity, None);
        assert_eq!(state.lifecycle, LegacyKoopaLifecycle::Normal);
    }

    #[test]
    fn flying_koopa_spawn_sets_flying_frame_and_gravity() {
        let constants = LegacyKoopaConstants::default();
        let state = LegacyKoopaState::spawn(3.0, 4.0, LegacyKoopaVariant::Flying, constants);

        assert!(state.flying);
        assert_eq!(state.frame, LegacyKoopaFrame::FlyingOne);
        assert_eq!(state.gravity, Some(constants.flying_gravity));
        assert_eq!(state.speed_x, -constants.speed);
    }

    #[test]
    fn red_flying_koopa_spawn_matches_legacy_hovering_start() {
        let constants = LegacyKoopaConstants::default();
        let state = LegacyKoopaState::spawn(3.0, 4.0, LegacyKoopaVariant::RedFlying, constants);

        assert!(state.flying);
        assert_eq!(state.frame, LegacyKoopaFrame::FlyingOne);
        assert_eq!(state.gravity, Some(0.0));
        assert_eq!(state.speed_x, 0.0);
    }

    #[test]
    fn beetle_koopa_spawn_reuses_walking_shell_physics() {
        let constants = LegacyKoopaConstants::default();
        let state = LegacyKoopaState::spawn(3.0, 4.0, LegacyKoopaVariant::Beetle, constants);

        assert_eq!(state.x, 3.0 - 6.0 / 16.0);
        assert_eq!(state.y, 4.0 - 11.0 / 16.0);
        assert_eq!(state.speed_x, -constants.speed);
        assert_eq!(state.speed_y, 0.0);
        assert_eq!(state.variant, LegacyKoopaVariant::Beetle);
        assert_eq!(state.frame, LegacyKoopaFrame::WalkingOne);
        assert!(!state.flying);
        assert!(!state.small);
        assert_eq!(state.gravity, None);
        assert_eq!(state.lifecycle, LegacyKoopaLifecycle::Normal);
    }

    #[test]
    fn beetle_koopa_resists_fireballs_but_other_koopas_do_not() {
        let constants = LegacyKoopaConstants::default();
        let beetle = LegacyKoopaState::spawn(1.0, 1.0, LegacyKoopaVariant::Beetle, constants);
        let green = LegacyKoopaState::spawn(1.0, 1.0, LegacyKoopaVariant::Green, constants);
        let red = LegacyKoopaState::spawn(1.0, 1.0, LegacyKoopaVariant::Red, constants);

        assert!(legacy_koopa_resists_fireball(&beetle));
        assert!(!legacy_koopa_resists_fireball(&green));
        assert!(!legacy_koopa_resists_fireball(&red));
    }

    #[test]
    fn koopa_update_sets_animation_direction_and_uses_strict_animation_threshold() {
        let constants = LegacyKoopaConstants::default();
        let mut state = LegacyKoopaState::spawn(1.0, 1.0, LegacyKoopaVariant::Green, constants);

        state.speed_x = constants.speed;
        assert_eq!(
            update_legacy_koopa(&mut state, constants, 0.2),
            LegacyEnemyUpdate::Keep
        );
        assert_eq!(state.animation_direction, LegacyEnemyDirection::Left);
        assert_eq!(state.frame, LegacyKoopaFrame::WalkingOne);

        let _ = update_legacy_koopa(&mut state, constants, 0.01);
        assert_eq!(state.frame, LegacyKoopaFrame::WalkingTwo);
        assert_close(state.animation_timer, 0.01);
    }

    #[test]
    fn koopa_update_eases_walking_and_shell_speeds_toward_legacy_targets() {
        let constants = LegacyKoopaConstants::default();
        let mut state = LegacyKoopaState::spawn(1.0, 1.0, LegacyKoopaVariant::Green, constants);

        state.speed_x = -5.0;
        let _ = update_legacy_koopa(&mut state, constants, DT);
        assert_eq!(state.speed_x, -2.2);

        let _ = stomp_legacy_koopa(&mut state, constants, 0.0, DT);
        state.speed_x = 6.0;
        let _ = update_legacy_koopa(&mut state, constants, DT);
        assert_eq!(state.speed_x, 8.8);

        state.speed_x = 0.0;
        let _ = update_legacy_koopa(&mut state, constants, DT);
        assert_eq!(state.speed_x, 0.0);
    }

    #[test]
    fn koopa_stomp_transitions_from_walking_to_shell_then_starts_or_stops_shell() {
        let constants = LegacyKoopaConstants::default();
        let mut state = LegacyKoopaState::spawn(5.0, 2.0, LegacyKoopaVariant::Green, constants);

        assert_eq!(
            stomp_legacy_koopa(&mut state, constants, 4.0, 1.0 / 60.0),
            LegacyKoopaStompOutcome::EnteredShell
        );
        assert!(state.small);
        assert_eq!(state.frame, LegacyKoopaFrame::Shell);
        assert_eq!(state.speed_x, 0.0);

        state.x = 6.0;
        assert_eq!(
            stomp_legacy_koopa(&mut state, constants, 4.0, 1.0 / 60.0),
            LegacyKoopaStompOutcome::StartedShell
        );
        assert_eq!(state.speed_x, constants.shell_speed);
        assert_close(state.x, 4.0 + 12.0 / 16.0 + constants.shell_speed / 60.0);

        assert_eq!(
            stomp_legacy_koopa(&mut state, constants, 4.0, 1.0 / 60.0),
            LegacyKoopaStompOutcome::StoppedShell
        );
        assert_eq!(state.speed_x, 0.0);
        assert_eq!(state.combo, 1);
    }

    #[test]
    fn koopa_stomp_starts_shell_left_when_player_is_right_of_shell() {
        let constants = LegacyKoopaConstants::default();
        let mut state = LegacyKoopaState::spawn(5.0, 2.0, LegacyKoopaVariant::Green, constants);

        let _ = stomp_legacy_koopa(&mut state, constants, 0.0, 1.0 / 60.0);
        state.x = 3.0;
        assert_eq!(
            stomp_legacy_koopa(&mut state, constants, 4.0, 1.0 / 60.0),
            LegacyKoopaStompOutcome::StartedShell
        );
        assert_eq!(state.speed_x, -constants.shell_speed);
        assert_close(state.x, 4.0 - state.width - constants.shell_speed / 60.0);
    }

    #[test]
    fn stomped_flying_koopa_loses_flight_and_restores_normal_gravity() {
        let constants = LegacyKoopaConstants::default();
        let mut state = LegacyKoopaState::spawn(1.0, 1.0, LegacyKoopaVariant::Flying, constants);

        state.speed_x = 0.0;
        assert_eq!(
            stomp_legacy_koopa(&mut state, constants, 0.0, DT),
            LegacyKoopaStompOutcome::ClearedFlying
        );
        assert!(!state.flying);
        assert!(!state.small);
        assert_eq!(state.frame, LegacyKoopaFrame::WalkingOne);
        assert_eq!(state.speed_x, -constants.speed);
        assert_eq!(state.gravity, Some(constants.normal_gravity));
    }

    #[test]
    fn koopa_side_collision_bounces_shells_but_default_resolves_walking_koopa() {
        let constants = LegacyKoopaConstants::default();
        let mut state = LegacyKoopaState::spawn(1.0, 1.0, LegacyKoopaVariant::Green, constants);

        state.speed_x = -constants.speed;
        let response =
            legacy_koopa_left_collision(&mut state, LegacyKoopaSideCollisionTarget::Solid);
        assert_eq!(
            response,
            super::LegacyKoopaSideCollisionResponse {
                suppress_default: false,
                hit_solid_target: false
            }
        );
        assert_eq!(state.speed_x, constants.speed);
        assert_eq!(state.animation_direction, LegacyEnemyDirection::Left);

        let _ = stomp_legacy_koopa(&mut state, constants, 0.0, DT);
        state.speed_x = constants.shell_speed;
        let response =
            legacy_koopa_right_collision(&mut state, LegacyKoopaSideCollisionTarget::Solid);
        assert_eq!(
            response,
            super::LegacyKoopaSideCollisionResponse {
                suppress_default: true,
                hit_solid_target: true
            }
        );
        assert_eq!(state.speed_x, -constants.shell_speed);
    }

    #[test]
    fn koopa_floor_and_startfall_preserve_flying_jump_contract() {
        let constants = LegacyKoopaConstants::default();
        let mut state = LegacyKoopaState::spawn(1.0, 1.0, LegacyKoopaVariant::Flying, constants);

        legacy_koopa_start_fall(&mut state);
        assert!(state.falling);

        assert_eq!(
            legacy_koopa_floor_collision(&mut state, constants),
            super::LegacyEnemyCollisionResponse {
                suppress_default: false
            }
        );
        assert!(!state.falling);
        assert_eq!(state.speed_y, -constants.jump_force);
    }

    #[test]
    fn shot_koopa_enters_shell_state_and_uses_legacy_motion_step() {
        let constants = LegacyKoopaConstants::default();
        let mut state = LegacyKoopaState::spawn(3.0, 4.0, LegacyKoopaVariant::Green, constants);

        shoot_legacy_koopa(&mut state, constants, LegacyEnemyDirection::Right);
        assert!(!state.active);
        assert!(state.small);
        assert!(!state.flying);
        assert_eq!(state.frame, LegacyKoopaFrame::Shell);
        assert_eq!(state.lifecycle, LegacyKoopaLifecycle::Shot);
        assert_eq!(state.gravity, Some(constants.shot_gravity));
        assert_eq!(state.speed_x, constants.shot_speed_x);
        assert_eq!(state.speed_y, -constants.shot_jump_force);

        let start_x = state.x;
        let start_y = state.y;
        assert_eq!(
            update_legacy_koopa(&mut state, constants, 0.1),
            LegacyEnemyUpdate::Keep
        );
        assert_eq!(state.speed_y, -2.0);
        assert_eq!(state.x, start_x + 0.4);
        assert_eq!(state.y, start_y - 0.2);
    }

    #[test]
    fn red_koopa_edge_turns_left_mover_when_gap_has_solid_neighbor() {
        let constants = LegacyKoopaConstants::default();
        let mut state = LegacyKoopaState::spawn(5.0, 2.0, LegacyKoopaVariant::Red, constants);

        let outcome = apply_legacy_red_koopa_edge_turn(&mut state, |x, y| match (x, y) {
            (6, 3) => Some(false),
            (5, 3) => Some(true),
            _ => None,
        });

        assert_eq!(
            outcome,
            LegacyKoopaEdgeTurn::Turned {
                probe: LegacyEnemyTileProbe { x: 6, y: 3 }
            }
        );
        assert_eq!(state.animation_direction, LegacyEnemyDirection::Left);
        assert_eq!(state.speed_x, constants.speed);
        assert_eq!(state.x, 6.0 - state.width / 2.0);
    }

    #[test]
    fn red_koopa_edge_turns_right_mover_and_snaps_to_left_edge() {
        let constants = LegacyKoopaConstants::default();
        let mut state = LegacyKoopaState::spawn(5.0, 2.0, LegacyKoopaVariant::Red, constants);

        state.x = 4.0;
        state.speed_x = constants.speed;
        let outcome = apply_legacy_red_koopa_edge_turn(&mut state, |x, y| match (x, y) {
            (5, 3) => Some(false),
            (6, 3) => Some(true),
            _ => None,
        });

        assert_eq!(
            outcome,
            LegacyKoopaEdgeTurn::Turned {
                probe: LegacyEnemyTileProbe { x: 5, y: 3 }
            }
        );
        assert_eq!(state.animation_direction, LegacyEnemyDirection::Right);
        assert_eq!(state.speed_x, -constants.speed);
        assert_eq!(state.x, 5.0 - 1.0 - state.width / 2.0);
    }

    #[test]
    fn red_koopa_edge_turn_requires_gap_and_solid_neighbor() {
        let constants = LegacyKoopaConstants::default();
        let mut state = LegacyKoopaState::spawn(5.0, 2.0, LegacyKoopaVariant::Red, constants);
        let original = state;

        let outcome = apply_legacy_red_koopa_edge_turn(&mut state, |x, y| match (x, y) {
            (6, 3) => Some(false),
            _ => None,
        });

        assert_eq!(
            outcome,
            LegacyKoopaEdgeTurn::NoTurn {
                probe: LegacyEnemyTileProbe { x: 6, y: 3 }
            }
        );
        assert_eq!(state, original);

        let outcome = apply_legacy_red_koopa_edge_turn(&mut state, |x, y| match (x, y) {
            (6, 3) => Some(true),
            (5, 3) => Some(true),
            _ => None,
        });

        assert_eq!(
            outcome,
            LegacyKoopaEdgeTurn::NoTurn {
                probe: LegacyEnemyTileProbe { x: 6, y: 3 }
            }
        );
        assert_eq!(state, original);
    }

    #[test]
    fn red_koopa_edge_turn_is_skipped_for_green_flying_shell_or_falling_states() {
        let constants = LegacyKoopaConstants::default();

        let mut green = LegacyKoopaState::spawn(5.0, 2.0, LegacyKoopaVariant::Green, constants);
        assert_eq!(
            apply_legacy_red_koopa_edge_turn(&mut green, |_, _| panic!("should not query map")),
            LegacyKoopaEdgeTurn::Ineligible
        );

        let mut beetle = LegacyKoopaState::spawn(5.0, 2.0, LegacyKoopaVariant::Beetle, constants);
        assert_eq!(
            apply_legacy_red_koopa_edge_turn(&mut beetle, |_, _| panic!("should not query map")),
            LegacyKoopaEdgeTurn::Ineligible
        );

        let mut red_flying =
            LegacyKoopaState::spawn(5.0, 2.0, LegacyKoopaVariant::RedFlying, constants);
        assert_eq!(
            apply_legacy_red_koopa_edge_turn(&mut red_flying, |_, _| panic!(
                "should not query map"
            )),
            LegacyKoopaEdgeTurn::Ineligible
        );

        let mut red = LegacyKoopaState::spawn(5.0, 2.0, LegacyKoopaVariant::Red, constants);
        red.falling = true;
        assert_eq!(
            apply_legacy_red_koopa_edge_turn(&mut red, |_, _| panic!("should not query map")),
            LegacyKoopaEdgeTurn::Ineligible
        );

        red.falling = false;
        red.small = true;
        assert_eq!(
            apply_legacy_red_koopa_edge_turn(&mut red, |_, _| panic!("should not query map")),
            LegacyKoopaEdgeTurn::Ineligible
        );
    }

    #[test]
    fn plant_spawn_matches_legacy_offsets_and_cycle_delay() {
        let constants = LegacyPlantConstants::default();
        let state = LegacyPlantState::spawn(10.0, 5.0, constants);

        assert_eq!(state.x, 10.0 - 8.0 / 16.0);
        assert_eq!(state.y, 5.0 + 9.0 / 16.0);
        assert_eq!(state.start_y, state.y);
        assert_eq!(state.width, 1.0);
        assert_eq!(state.height, 14.0 / 16.0);
        assert!(state.active);
        assert!(!state.destroy);
        assert_eq!(state.animation_frame, LegacyPlantFrame::One);
        assert_eq!(state.animation_timer, 0.0);
        assert_eq!(state.cycle_timer, constants.out_time + 1.5);
    }

    #[test]
    fn plant_animation_uses_strict_timer_threshold_and_toggles_frames() {
        let constants = LegacyPlantConstants::default();
        let mut state = LegacyPlantState::spawn(1.0, 1.0, constants);

        assert_eq!(
            update_legacy_plant(&mut state, constants, constants.animation_delay, |_, _| {
                true
            }),
            LegacyEnemyUpdate::Keep
        );
        assert_eq!(state.animation_frame, LegacyPlantFrame::One);
        assert_eq!(state.animation_timer, constants.animation_delay);

        let _ = update_legacy_plant(&mut state, constants, 0.01, |_, _| true);
        assert_eq!(state.animation_frame, LegacyPlantFrame::Two);
        assert_close(state.animation_timer, 0.01);
    }

    #[test]
    fn plant_moves_out_then_in_and_clamps_to_legacy_travel_distance() {
        let constants = LegacyPlantConstants::default();
        let mut state = LegacyPlantState::spawn(1.0, 1.0, constants);
        state.cycle_timer = 0.0;

        assert_eq!(
            update_legacy_plant(&mut state, constants, 1.0, |_, _| {
                panic!("moving plant should not query player range")
            }),
            LegacyEnemyUpdate::Keep
        );
        assert_close(state.y, state.start_y - constants.move_distance);

        state.cycle_timer = constants.out_time;
        state.y = state.start_y - constants.move_distance;
        assert_eq!(
            update_legacy_plant(&mut state, constants, 1.0, |_, _| {
                panic!("moving plant should not query player range")
            }),
            LegacyEnemyUpdate::Keep
        );
        assert_eq!(state.y, state.start_y);
    }

    #[test]
    fn plant_cycle_waits_while_player_is_near_and_resets_when_clear() {
        let constants = LegacyPlantConstants::default();
        let mut state = LegacyPlantState::spawn(6.0, 3.0, constants);
        state.cycle_timer = constants.out_time + constants.in_time;
        let expected_left = state.x + state.width / 2.0 - 3.0;
        let expected_right = state.x + state.width / 2.0 + 3.0;

        assert_eq!(
            update_legacy_plant(&mut state, constants, DT, |left, right| {
                assert_close(left, expected_left);
                assert_close(right, expected_right);
                true
            }),
            LegacyEnemyUpdate::Keep
        );
        assert_close(
            state.cycle_timer,
            constants.out_time + constants.in_time + DT,
        );

        assert_eq!(
            update_legacy_plant(&mut state, constants, DT, |left, right| {
                assert_close(left, expected_left);
                assert_close(right, expected_right);
                false
            }),
            LegacyEnemyUpdate::Keep
        );
        assert_eq!(state.cycle_timer, 0.0);
    }

    #[test]
    fn shot_plant_becomes_inactive_and_update_removes_it() {
        let constants = LegacyPlantConstants::default();
        let mut state = LegacyPlantState::spawn(1.0, 1.0, constants);

        shoot_legacy_plant(&mut state);
        assert!(!state.active);
        assert!(state.destroy);
        assert_eq!(
            update_legacy_plant(&mut state, constants, DT, |_, _| false),
            LegacyEnemyUpdate::Remove
        );
    }

    #[test]
    fn bullet_bill_spawn_matches_legacy_offsets_and_direction() {
        let constants = LegacyBulletBillConstants::default();
        let left = LegacyBulletBillState::spawn(10.0, 5.0, LegacyEnemyDirection::Left, constants);
        let right = LegacyBulletBillState::spawn(10.0, 5.0, LegacyEnemyDirection::Right, constants);

        assert_eq!(left.start_x, 10.0 - 14.0 / 16.0);
        assert_eq!(left.x, left.start_x);
        assert_eq!(left.y, 5.0 - 14.0 / 16.0);
        assert_eq!(left.width, 12.0 / 16.0);
        assert_eq!(left.height, 12.0 / 16.0);
        assert_eq!(left.speed_x, -constants.speed);
        assert_eq!(right.speed_x, constants.speed);
        assert_eq!(left.speed_y, 0.0);
        assert!(left.active);
        assert_eq!(left.gravity, 0.0);
        assert_eq!(left.rotation, 0.0);
        assert_eq!(left.timer, 0.0);
        assert_eq!(left.animation_direction, LegacyEnemyDirection::Left);
        assert!(left.custom_scissor_active);
        assert!(!left.kill_stuff);
        assert_eq!(left.lifecycle, LegacyBulletBillLifecycle::Flying);
    }

    #[test]
    fn flying_bullet_bill_lifetime_removal_uses_greater_or_equal_timer() {
        let constants = LegacyBulletBillConstants::default();
        let mut state =
            LegacyBulletBillState::spawn(1.0, 1.0, LegacyEnemyDirection::Right, constants);

        assert_eq!(
            update_legacy_bullet_bill(&mut state, constants, constants.lifetime - 0.01),
            LegacyEnemyUpdate::Keep
        );
        assert_close(state.timer, constants.lifetime - 0.01);

        assert_eq!(
            update_legacy_bullet_bill(&mut state, constants, 0.01),
            LegacyEnemyUpdate::Remove
        );
        assert_eq!(state.timer, constants.lifetime);
    }

    #[test]
    fn flying_bullet_bill_clears_scissor_after_one_tile_and_snaps_rotation() {
        let constants = LegacyBulletBillConstants::default();
        let mut state =
            LegacyBulletBillState::spawn(1.0, 1.0, LegacyEnemyDirection::Right, constants);

        state.x = state.start_x + 1.0;
        state.rotation = core::f32::consts::FRAC_PI_2 + 0.05;
        assert_eq!(
            update_legacy_bullet_bill(&mut state, constants, 0.0),
            LegacyEnemyUpdate::Keep
        );
        assert!(state.custom_scissor_active);
        assert_eq!(state.rotation, -core::f32::consts::FRAC_PI_2);

        state.x = state.start_x + 1.01;
        state.rotation = 0.2;
        let _ = update_legacy_bullet_bill(&mut state, constants, 0.0);
        assert!(!state.custom_scissor_active);
        assert_eq!(state.rotation, 0.0);
    }

    #[test]
    fn flying_bullet_bill_animation_direction_follows_legacy_speed_priority() {
        let constants = LegacyBulletBillConstants::default();
        let mut state =
            LegacyBulletBillState::spawn(1.0, 1.0, LegacyEnemyDirection::Right, constants);

        state.speed_x = -constants.speed;
        state.speed_y = 20.0;
        let _ = update_legacy_bullet_bill(&mut state, constants, 0.0);
        assert_eq!(state.animation_direction, LegacyEnemyDirection::Left);

        state.speed_x = 0.0;
        state.speed_y = -1.0;
        let _ = update_legacy_bullet_bill(&mut state, constants, 0.0);
        assert_eq!(state.animation_direction, LegacyEnemyDirection::Right);

        state.speed_y = 1.0;
        let _ = update_legacy_bullet_bill(&mut state, constants, 0.0);
        assert_eq!(state.animation_direction, LegacyEnemyDirection::Left);
    }

    #[test]
    fn stomped_bullet_bill_enters_shot_state_and_uses_direct_motion_step() {
        let constants = LegacyBulletBillConstants::default();
        let mut state =
            LegacyBulletBillState::spawn(3.0, 4.0, LegacyEnemyDirection::Left, constants);

        stomp_legacy_bullet_bill(&mut state, constants, Some(LegacyEnemyDirection::Right));
        assert!(!state.active);
        assert_eq!(state.lifecycle, LegacyBulletBillLifecycle::Shot);
        assert_eq!(state.gravity, constants.shot_gravity);
        assert_eq!(state.speed_x, constants.shot_speed_x);
        assert_eq!(state.speed_y, 0.0);

        let start_x = state.x;
        let start_y = state.y;
        assert_eq!(
            update_legacy_bullet_bill(&mut state, constants, DT),
            LegacyEnemyUpdate::Keep
        );
        assert_eq!(state.speed_y, constants.shot_gravity * DT);
        assert_eq!(state.x, start_x + constants.shot_speed_x * DT);
        assert_eq!(state.y, start_y + constants.shot_gravity * DT * DT);
    }

    #[test]
    fn shot_bullet_bill_defaults_right_and_removes_after_falling_below_legacy_y() {
        let constants = LegacyBulletBillConstants::default();
        let mut state =
            LegacyBulletBillState::spawn(3.0, 4.0, LegacyEnemyDirection::Left, constants);

        shoot_legacy_bullet_bill(&mut state, constants, None);
        assert_eq!(state.speed_x, constants.shot_speed_x);

        state.y = constants.removal_y;
        assert_eq!(
            update_legacy_bullet_bill(&mut state, constants, 0.0),
            LegacyEnemyUpdate::Keep
        );

        state.speed_y = 1.0;
        assert_eq!(
            update_legacy_bullet_bill(&mut state, constants, 0.01),
            LegacyEnemyUpdate::Remove
        );
    }

    #[test]
    fn portaled_bullet_bill_sets_legacy_killstuff_flag() {
        let constants = LegacyBulletBillConstants::default();
        let mut state =
            LegacyBulletBillState::spawn(1.0, 1.0, LegacyEnemyDirection::Right, constants);

        portal_legacy_bullet_bill(&mut state);

        assert!(state.kill_stuff);
    }

    #[test]
    fn bullet_bill_launcher_spawn_sets_initial_timer_from_random_time() {
        let state = LegacyBulletBillLauncherState::spawn(10.0, 5.0, 2.7);

        assert_eq!(state.x, 10.0);
        assert_eq!(state.y, 5.0);
        assert_eq!(state.time, 2.7);
        assert_eq!(state.timer, 2.2);
        assert!(state.autodelete);
    }

    #[test]
    fn bullet_bill_launcher_update_uses_strict_timer_and_viewport_gates() {
        let constants = LegacyBulletBillConstants::default();
        let viewport = LegacyBulletBillLauncherViewport {
            left: 0.0,
            width: 20.0,
        };
        let mut state = LegacyBulletBillLauncherState::spawn(10.0, 5.0, 1.0);

        assert_eq!(
            update_legacy_bullet_bill_launcher(
                &mut state,
                constants,
                0.5,
                viewport,
                0,
                &[14.0],
                || panic!("strict equal timer should not consume random time")
            ),
            LegacyBulletBillLauncherUpdate::Idle
        );
        assert_eq!(state.timer, state.time);

        assert_eq!(
            update_legacy_bullet_bill_launcher(
                &mut state,
                constants,
                0.01,
                viewport,
                0,
                &[14.0],
                || { 3.1 }
            ),
            LegacyBulletBillLauncherUpdate::Fired {
                direction: LegacyEnemyDirection::Right
            }
        );
        assert_eq!(state.timer, 0.0);
        assert_eq!(state.time, 3.1);

        let mut left_edge = LegacyBulletBillLauncherState::spawn(0.0, 5.0, 1.0);
        assert_eq!(
            update_legacy_bullet_bill_launcher(
                &mut left_edge,
                constants,
                1.0,
                viewport,
                0,
                &[14.0],
                || panic!("offscreen launcher should not consume random time")
            ),
            LegacyBulletBillLauncherUpdate::Idle
        );

        let mut right_edge = LegacyBulletBillLauncherState::spawn(22.0, 5.0, 1.0);
        assert_eq!(
            update_legacy_bullet_bill_launcher(
                &mut right_edge,
                constants,
                1.0,
                viewport,
                0,
                &[14.0],
                || panic!("offscreen launcher should not consume random time")
            ),
            LegacyBulletBillLauncherUpdate::Idle
        );
    }

    #[test]
    fn bullet_bill_launcher_fire_decision_uses_nearest_player_probe_and_range() {
        let constants = LegacyBulletBillConstants::default();
        let state = LegacyBulletBillLauncherState::spawn(10.0, 5.0, 1.0);

        assert_eq!(
            fire_legacy_bullet_bill_launcher(&state, constants, 0, &[14.0]),
            Some(LegacyEnemyDirection::Right)
        );
        assert_eq!(
            fire_legacy_bullet_bill_launcher(&state, constants, 0, &[5.0]),
            Some(LegacyEnemyDirection::Left)
        );
        assert_eq!(
            fire_legacy_bullet_bill_launcher(&state, constants, 0, &[9.0]),
            None
        );

        assert_eq!(
            fire_legacy_bullet_bill_launcher(&state, constants, 0, &[5.0, 9.0]),
            None
        );
    }

    #[test]
    fn bullet_bill_launcher_preserves_first_player_on_distance_ties() {
        let constants = LegacyBulletBillConstants::default();
        let state = LegacyBulletBillLauncherState::spawn(10.0, 5.0, 1.0);
        let left_player_x = 10.0 - constants.range - 14.0 / 16.0 - 0.5;
        let right_player_x = 10.0 + constants.range - 14.0 / 16.0 + 0.5;

        assert_eq!(
            fire_legacy_bullet_bill_launcher(
                &state,
                constants,
                0,
                &[left_player_x, right_player_x]
            ),
            Some(LegacyEnemyDirection::Left)
        );
        assert_eq!(
            fire_legacy_bullet_bill_launcher(
                &state,
                constants,
                0,
                &[right_player_x, left_player_x]
            ),
            Some(LegacyEnemyDirection::Right)
        );
    }

    #[test]
    fn bullet_bill_launcher_does_not_fire_at_max_count_or_without_players() {
        let constants = LegacyBulletBillConstants::default();
        let viewport = LegacyBulletBillLauncherViewport {
            left: 0.0,
            width: 20.0,
        };
        let mut state = LegacyBulletBillLauncherState::spawn(10.0, 5.0, 1.0);

        assert_eq!(
            update_legacy_bullet_bill_launcher(
                &mut state,
                constants,
                1.0,
                viewport,
                constants.max_count,
                &[14.0],
                || panic!("max-count launcher should not consume random time")
            ),
            LegacyBulletBillLauncherUpdate::Idle
        );
        assert!(state.timer > state.time);

        assert_eq!(
            fire_legacy_bullet_bill_launcher(&state, constants, 0, &[]),
            None
        );
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.0001,
            "expected {actual} to be close to {expected}"
        );
    }
}
