//! Engine-neutral player movement rules ported from `mario.lua`.

use core::f32::consts::FRAC_PI_2;

use crate::collision::in_range;
use crate::config::{
    BlueGelBounceConstants, OrangeGelMovementConstants, PhysicsConstants, PlayerAnimationConstants,
    PlayerMovementConstants, SpringConstants, UnderwaterMovementConstants,
};
use crate::math::Vec2;
use crate::wormhole::{
    AnimationDirection, Facing, LegacyPortalEndpoint, LegacyPortalTransitInput,
    legacy_portal_coords,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HorizontalDirection {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerAnimationState {
    Idle,
    Running,
    Jumping,
    Falling,
    Swimming,
    Sliding,
    Climbing,
    Dead,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerEnvironment {
    Normal,
    Underwater,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerSideCollision {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyVineSide {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyPlayerHazard {
    Fire,
    CastleFire,
    UpFire,
    Hammer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyPlayerHazardCollision {
    Floor,
    Side(PlayerSideCollision),
    Ceiling,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyPlayerDeathCause {
    CastleFireFire,
    EnemyFloorCollision,
    EnemyRightCollision,
    EnemyLeftCollision,
    EnemyCeilingCollision,
}

impl LegacyPlayerDeathCause {
    #[must_use]
    pub const fn legacy_label(self) -> &'static str {
        match self {
            Self::CastleFireFire => "castlefirefire",
            Self::EnemyFloorCollision => "Enemy (floorcollide)",
            Self::EnemyRightCollision => "Enemy (rightcollide)",
            Self::EnemyLeftCollision => "Enemy (leftcollide)",
            Self::EnemyCeilingCollision => "Enemy (Ceilcollided)",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyPlayerHazardOutcome {
    Ignored,
    Dies(LegacyPlayerDeathCause),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyPlayerCollisionSnapshot {
    pub jumping: bool,
    pub falling: bool,
    pub animation_state: PlayerAnimationState,
}

impl LegacyPlayerCollisionSnapshot {
    #[must_use]
    pub const fn new(jumping: bool, falling: bool, animation_state: PlayerAnimationState) -> Self {
        Self {
            jumping,
            falling,
            animation_state,
        }
    }

    #[must_use]
    pub const fn from_state(state: &PlayerMovementState) -> Self {
        Self {
            jumping: state.jumping,
            falling: state.falling,
            animation_state: state.animation_state,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacySideBoxResponse {
    pub box_speed_x: Option<f32>,
    pub suppress_default: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyPipeDirection {
    Right,
    Down,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyPipeCandidate {
    pub sublevel: Option<i32>,
}

impl LegacyPipeCandidate {
    #[must_use]
    pub const fn new(sublevel: Option<i32>) -> Self {
        Self { sublevel }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyPipeEntry {
    pub coord: LegacyMapTileCoord,
    pub direction: LegacyPipeDirection,
    pub sublevel: Option<i32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyCeilingTileContext {
    pub tile: LegacyMapTileCoord,
    pub big_mario: bool,
    pub map_width: i32,
    pub left_neighbor_solid: bool,
    pub right_neighbor_solid: bool,
    pub push_left_clear: bool,
    pub push_right_clear: bool,
    pub physics: PhysicsConstants,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LegacyCeilingTileResponse {
    BreakBlock { coord: LegacyMapTileCoord },
    PushPlayer { x: f32 },
    HitBlock { coord: LegacyMapTileCoord },
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PlayerMovementInput {
    pub left: bool,
    pub right: bool,
    pub run: bool,
}

impl PlayerMovementInput {
    #[must_use]
    pub const fn new(left: bool, right: bool, run: bool) -> Self {
        Self { left, right, run }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerMovementState {
    pub speed_x: f32,
    pub speed_y: f32,
    pub gravity: f32,
    pub jumping: bool,
    pub falling: bool,
    pub ducking: bool,
    pub run_frame: u8,
    pub swim_frame: u8,
    pub run_animation_progress: f32,
    pub swim_animation_progress: f32,
    pub animation_state: PlayerAnimationState,
    pub animation_direction: HorizontalDirection,
    pub spring: bool,
    pub spring_high: bool,
    pub spring_timer: f32,
}

impl Default for PlayerMovementState {
    fn default() -> Self {
        Self {
            speed_x: 0.0,
            speed_y: 0.0,
            gravity: 80.0,
            jumping: false,
            falling: false,
            ducking: false,
            run_frame: 3,
            swim_frame: 1,
            run_animation_progress: 1.0,
            swim_animation_progress: 1.0,
            animation_state: PlayerAnimationState::Idle,
            animation_direction: HorizontalDirection::Right,
            spring: false,
            spring_high: false,
            spring_timer: 0.0,
        }
    }
}

impl PlayerMovementState {
    #[must_use]
    pub const fn airborne(self) -> bool {
        self.jumping || self.falling
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerVerticalBounds {
    pub y: f32,
    pub height: f32,
}

impl PlayerVerticalBounds {
    #[must_use]
    pub const fn new(y: f32, height: f32) -> Self {
        Self { y, height }
    }

    #[must_use]
    pub const fn bottom(self) -> f32 {
        self.y + self.height
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerBodyBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl PlayerBodyBounds {
    #[must_use]
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    #[must_use]
    pub const fn bottom(self) -> f32 {
        self.y + self.height
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyHeldSpringContext {
    pub spring_x: f32,
    pub spring_y: f32,
    pub spring_frame: u8,
    pub player_height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyHeldSpringUpdate {
    pub x: f32,
    pub y: f32,
    pub auto_release: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyGrabVineContext {
    pub inside_portal: bool,
    pub player: PlayerBodyBounds,
    pub vine_coord: LegacyMapTileCoord,
    pub vine_x: f32,
    pub vine_width: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyGrabVineOutcome {
    pub x: f32,
    pub pointing_angle: f32,
    pub side: LegacyVineSide,
    pub climb_frame: u8,
    pub move_timer: f32,
    pub vine_coord: LegacyMapTileCoord,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyOnVineDirection {
    Up,
    Down,
    Idle,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyOnVineContext {
    pub y: f32,
    pub height: f32,
    pub move_timer: f32,
    pub direction: LegacyOnVineDirection,
    pub blocking_collision: Option<PlayerVerticalBounds>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyOnVineUpdate {
    pub y: f32,
    pub move_timer: f32,
    pub climb_frame: u8,
    pub trigger_animation: bool,
    pub portal_probe_y: Option<f32>,
    pub blocked_by_solid: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyOnVineHorizontalPortalPair {
    pub portal1: LegacyPortalEndpoint,
    pub portal2: LegacyPortalEndpoint,
}

impl LegacyOnVineHorizontalPortalPair {
    #[must_use]
    pub const fn new(portal1: LegacyPortalEndpoint, portal2: LegacyPortalEndpoint) -> Self {
        Self { portal1, portal2 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyOnVineHorizontalPortalContext {
    pub body: PlayerBodyBounds,
    pub next_y: f32,
    pub portals: LegacyOnVineHorizontalPortalPair,
    pub rotation: f32,
    pub exit_blocked: bool,
    pub frame_dt: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyOnVineHorizontalPortalOutcome {
    pub x: f32,
    pub y: f32,
    pub speed_x: f32,
    pub speed_y: f32,
    pub rotation: f32,
    pub animation_direction: HorizontalDirection,
    pub jumping: bool,
    pub falling: bool,
    pub entry_facing: Facing,
    pub exit_facing: Facing,
    pub exit_blocked: bool,
    pub portaled_exit_facing: Facing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyDropVineContext {
    pub side: LegacyVineSide,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyDropVineOutcome {
    pub x: f32,
    pub vine_active: bool,
    pub vine_mask_enabled: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyOnVineAttachmentLossContext {
    pub side: LegacyVineSide,
    pub has_vine_overlap: bool,
}

impl From<PlayerBodyBounds> for PlayerVerticalBounds {
    fn from(bounds: PlayerBodyBounds) -> Self {
        Self::new(bounds.y, bounds.height)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyMapBounds {
    pub width: i32,
    pub height: i32,
}

impl LegacyMapBounds {
    #[must_use]
    pub const fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }

    #[must_use]
    pub const fn mari0(width: i32) -> Self {
        Self { width, height: 15 }
    }

    #[must_use]
    pub const fn contains(self, coord: LegacyMapTileCoord) -> bool {
        coord.x >= 1 && coord.x <= self.width && coord.y >= 1 && coord.y <= self.height
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyMapTileCoord {
    pub x: i32,
    pub y: i32,
}

impl LegacyMapTileCoord {
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyGelKind {
    Blue,
    Orange,
    White,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacySurfaceMovementContext {
    pub movement: PlayerMovementConstants,
    pub orange_gel: OrangeGelMovementConstants,
    pub body: PlayerBodyBounds,
    pub map_bounds: LegacyMapBounds,
}

impl LegacySurfaceMovementContext {
    #[must_use]
    pub const fn new(
        movement: PlayerMovementConstants,
        orange_gel: OrangeGelMovementConstants,
        body: PlayerBodyBounds,
        map_bounds: LegacyMapBounds,
    ) -> Self {
        Self {
            movement,
            orange_gel,
            body,
            map_bounds,
        }
    }
}

pub fn apply_legacy_player_gravity_selection(
    state: &mut PlayerMovementState,
    environment: PlayerEnvironment,
    physics: PhysicsConstants,
    underwater: UnderwaterMovementConstants,
) {
    if state.jumping {
        state.gravity = match environment {
            PlayerEnvironment::Normal => physics.jumping_gravity,
            PlayerEnvironment::Underwater => underwater.jumping_gravity,
        };

        if state.speed_y > 0.0 {
            state.jumping = false;
            state.falling = true;
        }
    } else {
        state.gravity = match environment {
            PlayerEnvironment::Normal => physics.gravity,
            PlayerEnvironment::Underwater => underwater.gravity,
        };
    }
}

pub fn apply_legacy_player_gravity_velocity(
    state: &mut PlayerMovementState,
    dt: f32,
    physics: PhysicsConstants,
) {
    state.speed_y += state.gravity * dt;

    if state.speed_y > physics.max_y_speed {
        state.speed_y = physics.max_y_speed;
    }
}

pub fn apply_legacy_start_fall_after_vertical_move(
    state: &mut PlayerMovementState,
    dt: f32,
) -> bool {
    if state.speed_y == state.gravity * dt && !state.falling {
        state.falling = true;
        state.animation_state = PlayerAnimationState::Falling;
        return true;
    }

    false
}

pub fn apply_legacy_floor_landing_state(state: &mut PlayerMovementState) {
    if state.speed_x == 0.0 {
        state.animation_state = PlayerAnimationState::Idle;
    } else if state.animation_state != PlayerAnimationState::Sliding {
        state.animation_state = PlayerAnimationState::Running;
    }

    state.falling = false;
    state.jumping = false;
}

pub fn apply_legacy_head_bump_state(state: &mut PlayerMovementState, physics: PhysicsConstants) {
    state.jumping = false;
    state.falling = true;
    state.speed_y = physics.head_force;
}

pub fn apply_legacy_non_invisible_ceiling_tile_response(
    state: &mut PlayerMovementState,
    body: &mut PlayerBodyBounds,
    context: LegacyCeilingTileContext,
) -> LegacyCeilingTileResponse {
    if context.big_mario {
        return LegacyCeilingTileResponse::BreakBlock {
            coord: context.tile,
        };
    }

    let tile_x = context.tile.x as f32;
    let mut hit_coord = context.tile;

    if body.x < tile_x - LEGACY_CEILING_LEFT_RETARGET_THRESHOLD {
        if context.tile.x > 1 && context.left_neighbor_solid {
            hit_coord = LegacyMapTileCoord::new(context.tile.x - 1, context.tile.y);
        } else if context.push_left_clear {
            body.x = tile_x - LEGACY_CEILING_LEFT_PUSH_OFFSET;
            if state.speed_x > 0.0 {
                state.speed_x = 0.0;
            }
            return LegacyCeilingTileResponse::PushPlayer { x: body.x };
        }
    } else if body.x > tile_x - LEGACY_CEILING_RIGHT_RETARGET_THRESHOLD {
        if context.tile.x < context.map_width && context.right_neighbor_solid {
            hit_coord = LegacyMapTileCoord::new(context.tile.x + 1, context.tile.y);
        } else if context.push_right_clear {
            body.x = tile_x;
            if state.speed_x < 0.0 {
                state.speed_x = 0.0;
            }
            return LegacyCeilingTileResponse::PushPlayer { x: body.x };
        }
    }

    apply_legacy_head_bump_state(state, context.physics);
    LegacyCeilingTileResponse::HitBlock { coord: hit_coord }
}

const LEGACY_CEILING_LEFT_RETARGET_THRESHOLD: f32 = 22.0 / 16.0;
const LEGACY_CEILING_RIGHT_RETARGET_THRESHOLD: f32 = 6.0 / 16.0;
const LEGACY_CEILING_LEFT_PUSH_OFFSET: f32 = 28.0 / 16.0;

pub fn apply_legacy_floor_invisible_tile_suppression(
    state: &mut PlayerMovementState,
    snapshot: LegacyPlayerCollisionSnapshot,
    tile_invisible: bool,
) -> bool {
    if !tile_invisible {
        return false;
    }

    state.jumping = snapshot.jumping;
    state.falling = snapshot.falling;
    state.animation_state = snapshot.animation_state;
    true
}

#[must_use]
pub const fn legacy_side_invisible_tile_suppresses_default(tile_invisible: bool) -> bool {
    tile_invisible
}

#[must_use]
pub fn legacy_ceiling_invisible_tile_suppresses_default(
    body: PlayerBodyBounds,
    state: PlayerMovementState,
    tile: PlayerBodyBounds,
    tile_invisible: bool,
) -> bool {
    tile_invisible && body.y - state.speed_y <= tile.y
}

pub fn apply_legacy_side_tile_gap_run(
    state: &mut PlayerMovementState,
    body: &mut PlayerBodyBounds,
    tile: PlayerBodyBounds,
    side: PlayerSideCollision,
    above_tile_collision: Option<bool>,
    physics: PhysicsConstants,
) -> bool {
    if above_tile_collision != Some(false)
        || state.speed_y <= 0.0
        || body.bottom() >= tile.y + physics.space_run_room
    {
        return false;
    }

    body.y = tile.y - body.height;
    state.speed_y = 0.0;
    body.x = match side {
        PlayerSideCollision::Right => tile.x - body.width + LEGACY_SIDE_TILE_NUDGE,
        PlayerSideCollision::Left => tile.x + tile.width - LEGACY_SIDE_TILE_NUDGE,
    };
    state.falling = false;
    state.animation_state = PlayerAnimationState::Running;
    true
}

const LEGACY_SIDE_TILE_NUDGE: f32 = 0.0001;

pub fn apply_legacy_side_box_response(
    state: &mut PlayerMovementState,
    side: PlayerSideCollision,
    box_can_move: bool,
    dt: f32,
    movement: PlayerMovementConstants,
) -> LegacySideBoxResponse {
    let half_walk_speed = movement.max_walk_speed / 2.0;

    match side {
        PlayerSideCollision::Right if state.speed_x > half_walk_speed => {
            state.speed_x -= state.speed_x * LEGACY_SIDE_BOX_DAMPING * dt;
        }
        PlayerSideCollision::Left if state.speed_x < -half_walk_speed => {
            state.speed_x -= state.speed_x * LEGACY_SIDE_BOX_DAMPING * dt;
        }
        _ => {}
    }

    if box_can_move {
        LegacySideBoxResponse {
            box_speed_x: Some(state.speed_x),
            suppress_default: true,
        }
    } else {
        LegacySideBoxResponse {
            box_speed_x: None,
            suppress_default: false,
        }
    }
}

const LEGACY_SIDE_BOX_DAMPING: f32 = 6.0;

#[must_use]
pub fn legacy_right_side_pipe_entry(
    state: &PlayerMovementState,
    right_input: bool,
    intermission: bool,
    tile: LegacyMapTileCoord,
    current_tile_pipe: Option<LegacyPipeCandidate>,
    below_tile_pipe: Option<LegacyPipeCandidate>,
) -> Option<LegacyPipeEntry> {
    if state.airborne() || (!right_input && !intermission) {
        return None;
    }

    if let Some(pipe) = current_tile_pipe {
        return Some(LegacyPipeEntry {
            coord: tile,
            direction: LegacyPipeDirection::Right,
            sublevel: pipe.sublevel,
        });
    }

    below_tile_pipe.map(|pipe| LegacyPipeEntry {
        coord: LegacyMapTileCoord::new(tile.x, tile.y + 1),
        direction: LegacyPipeDirection::Right,
        sublevel: pipe.sublevel,
    })
}

pub fn apply_legacy_side_button_response(
    state: &mut PlayerMovementState,
    body: &mut PlayerBodyBounds,
    button: PlayerBodyBounds,
    side: PlayerSideCollision,
) -> bool {
    body.y = button.y - body.height;
    body.x = match side {
        PlayerSideCollision::Right => button.x - body.width + LEGACY_BUTTON_SIDE_NUDGE,
        PlayerSideCollision::Left => button.x + button.width - LEGACY_BUTTON_SIDE_NUDGE,
    };

    if state.speed_y > 0.0 {
        state.speed_y = 0.0;
    }

    true
}

const LEGACY_BUTTON_SIDE_NUDGE: f32 = 0.001;

#[must_use]
pub const fn legacy_player_hazard_collision_outcome(
    hazard: LegacyPlayerHazard,
    collision: LegacyPlayerHazardCollision,
    invincible: bool,
    star_or_big_mario: bool,
) -> LegacyPlayerHazardOutcome {
    if invincible || star_or_big_mario {
        return LegacyPlayerHazardOutcome::Ignored;
    }

    match collision {
        LegacyPlayerHazardCollision::Floor => match hazard {
            LegacyPlayerHazard::Fire | LegacyPlayerHazard::CastleFire => {
                LegacyPlayerHazardOutcome::Dies(LegacyPlayerDeathCause::CastleFireFire)
            }
            LegacyPlayerHazard::UpFire | LegacyPlayerHazard::Hammer => {
                LegacyPlayerHazardOutcome::Dies(LegacyPlayerDeathCause::EnemyFloorCollision)
            }
        },
        LegacyPlayerHazardCollision::Side(PlayerSideCollision::Right) => {
            LegacyPlayerHazardOutcome::Dies(LegacyPlayerDeathCause::EnemyRightCollision)
        }
        LegacyPlayerHazardCollision::Side(PlayerSideCollision::Left) => {
            LegacyPlayerHazardOutcome::Dies(LegacyPlayerDeathCause::EnemyLeftCollision)
        }
        LegacyPlayerHazardCollision::Ceiling => {
            LegacyPlayerHazardOutcome::Dies(LegacyPlayerDeathCause::EnemyCeilingCollision)
        }
    }
}

pub fn apply_legacy_floor_blue_gel_bounce(
    state: &mut PlayerMovementState,
    down_held: bool,
    frame_dt: f32,
    physics: PhysicsConstants,
    constants: BlueGelBounceConstants,
) -> bool {
    if down_held
        || state.speed_y
            <= frame_dt * physics.gravity * constants.floor_threshold_gravity_multiplier
    {
        return false;
    }

    state.speed_y = -state.speed_y;
    state.falling = true;
    state.animation_state = PlayerAnimationState::Jumping;
    state.speed_y += state.gravity * frame_dt;
    true
}

pub fn apply_legacy_side_blue_gel_bounce(
    state: &mut PlayerMovementState,
    side: PlayerSideCollision,
    down_held: bool,
    constants: BlueGelBounceConstants,
) -> bool {
    if down_held || !state.airborne() {
        return false;
    }

    match side {
        PlayerSideCollision::Right => {
            if state.speed_x <= constants.horizontal_min_speed_x {
                return false;
            }

            state.speed_x = (-constants.horizontal_max_speed_x)
                .min(-state.speed_x * constants.horizontal_multiplier);
        }
        PlayerSideCollision::Left => {
            if state.speed_x >= -constants.horizontal_min_speed_x {
                return false;
            }

            state.speed_x = constants
                .horizontal_max_speed_x
                .min(-state.speed_x * constants.horizontal_multiplier);
        }
    }

    state.speed_y = state.speed_y.min(-constants.horizontal_vertical_speed);
    true
}

#[must_use]
pub fn legacy_enemy_stomp_bounce_speed(gravity: f32, physics: PhysicsConstants) -> f32 {
    (2.0 * gravity * physics.bounce_height).sqrt()
}

pub fn apply_legacy_enemy_stomp_bounce_state(
    state: &mut PlayerMovementState,
    physics: PhysicsConstants,
) {
    state.animation_state = PlayerAnimationState::Jumping;
    state.falling = true;
    state.speed_y = -legacy_enemy_stomp_bounce_speed(state.gravity, physics);
}

pub fn apply_legacy_hit_spring_state(state: &mut PlayerMovementState) {
    state.speed_y = 0.0;
    state.spring = true;
    state.spring_high = false;
    state.spring_timer = 0.0;
    state.gravity = 0.0;
    state.animation_state = PlayerAnimationState::Idle;
}

#[must_use]
pub fn legacy_leave_spring_y(spring_y: f32, player_height: f32, constants: SpringConstants) -> f32 {
    spring_y - player_height - constants.player_y_offset
}

#[must_use]
pub fn update_legacy_held_spring(
    state: &mut PlayerMovementState,
    context: LegacyHeldSpringContext,
    dt: f32,
    constants: SpringConstants,
) -> LegacyHeldSpringUpdate {
    state.spring_timer += dt;

    LegacyHeldSpringUpdate {
        x: context.spring_x,
        y: legacy_leave_spring_y(context.spring_y, context.player_height, constants)
            + legacy_spring_frame_y_offset(context.spring_frame),
        auto_release: state.spring_timer > constants.duration,
    }
}

pub fn apply_legacy_leave_spring_state(
    state: &mut PlayerMovementState,
    physics: PhysicsConstants,
    constants: SpringConstants,
) {
    state.speed_y = if state.spring_high {
        -constants.high_force
    } else {
        -constants.force
    };
    state.animation_state = PlayerAnimationState::Falling;
    state.gravity = physics.gravity;
    state.falling = true;
    state.spring = false;
}

#[must_use]
pub fn apply_legacy_grab_vine(
    state: &mut PlayerMovementState,
    context: LegacyGrabVineContext,
) -> Option<LegacyGrabVineOutcome> {
    state.ducking = false;

    if context.inside_portal {
        return None;
    }

    state.gravity = 0.0;
    state.speed_x = 0.0;
    state.speed_y = 0.0;
    state.animation_state = PlayerAnimationState::Climbing;

    let (x, pointing_angle, side) = if context.vine_x > context.player.x {
        (
            context.vine_x + context.vine_width / 2.0 - context.player.width + 2.0 / 16.0,
            -FRAC_PI_2,
            LegacyVineSide::Left,
        )
    } else {
        (
            context.vine_x + context.vine_width / 2.0 - 2.0 / 16.0,
            FRAC_PI_2,
            LegacyVineSide::Right,
        )
    };

    Some(LegacyGrabVineOutcome {
        x,
        pointing_angle,
        side,
        climb_frame: 2,
        move_timer: 0.0,
        vine_coord: context.vine_coord,
    })
}

pub fn update_legacy_on_vine_motion(
    state: &mut PlayerMovementState,
    context: LegacyOnVineContext,
    constants: crate::config::LegacyVineConstants,
    dt: f32,
) -> LegacyOnVineUpdate {
    state.animation_state = PlayerAnimationState::Climbing;

    let mut portal_probe_y = None;
    let (mut y, move_timer, mut climb_frame) = match context.direction {
        LegacyOnVineDirection::Up => {
            let move_timer = context.move_timer + dt;
            (
                context.y - constants.move_speed * dt,
                move_timer,
                legacy_vine_climb_frame(move_timer, constants.frame_delay),
            )
        }
        LegacyOnVineDirection::Down => {
            let move_timer = context.move_timer + dt;
            let next_y = context.y + constants.move_down_speed * dt;
            portal_probe_y = Some(next_y);
            (
                next_y,
                move_timer,
                legacy_vine_climb_frame(move_timer, constants.frame_delay_down),
            )
        }
        LegacyOnVineDirection::Idle => (context.y, 0.0, 2),
    };

    let mut blocked_by_solid = false;
    if let Some(blocking_collision) = context.blocking_collision {
        match context.direction {
            LegacyOnVineDirection::Up => {
                y = blocking_collision.y + blocking_collision.height;
                climb_frame = 2;
                blocked_by_solid = true;
            }
            LegacyOnVineDirection::Down => {
                y = blocking_collision.y - context.height;
                climb_frame = 2;
                blocked_by_solid = true;
            }
            LegacyOnVineDirection::Idle => {}
        }
    }

    LegacyOnVineUpdate {
        y,
        move_timer,
        climb_frame,
        trigger_animation: y + context.height <= constants.animation_start_y,
        portal_probe_y,
        blocked_by_solid,
    }
}

#[must_use]
pub fn apply_legacy_on_vine_horizontal_portal(
    state: &mut PlayerMovementState,
    context: LegacyOnVineHorizontalPortalContext,
) -> Option<LegacyOnVineHorizontalPortalOutcome> {
    let (entry, exit) =
        legacy_horizontal_portal_entry(context.body, context.next_y, context.portals)?;

    if (entry.facing == Facing::Up && state.speed_y < 0.0)
        || (entry.facing == Facing::Down && state.speed_y > 0.0)
        || matches!(entry.facing, Facing::Left | Facing::Right)
    {
        return None;
    }

    let transit = legacy_portal_coords(LegacyPortalTransitInput {
        position: Vec2::new(context.body.x, context.body.y),
        velocity: Vec2::new(state.speed_x, state.speed_y),
        size: Vec2::new(context.body.width, context.body.height),
        rotation: context.rotation,
        animation_direction: Some(legacy_wormhole_animation_direction(
            state.animation_direction,
        )),
        entry,
        exit,
        live: true,
        gravity: state.gravity,
        frame_dt: context.frame_dt,
    });

    let (x, y, speed_x, speed_y, rotation, animation_direction) = if context.exit_blocked {
        let mut bounced_speed_y = -state.speed_y * 0.95;
        if bounced_speed_y.abs() < 2.0 {
            bounced_speed_y = if bounced_speed_y > 0.0 { 2.0 } else { -2.0 };
        }
        (
            context.body.x,
            context.body.y,
            state.speed_x,
            bounced_speed_y,
            context.rotation,
            state.animation_direction,
        )
    } else {
        (
            transit.position.x,
            transit.position.y,
            transit.velocity.x,
            transit.velocity.y,
            transit.rotation,
            legacy_player_animation_direction(
                transit.animation_direction,
                state.animation_direction,
            ),
        )
    };

    state.speed_x = speed_x;
    state.speed_y = speed_y;
    state.animation_direction = animation_direction;

    if !matches!(
        (entry.facing, exit.facing),
        (Facing::Down, Facing::Up) | (Facing::Up, Facing::Down)
    ) {
        state.jumping = false;
        state.falling = true;
    }

    Some(LegacyOnVineHorizontalPortalOutcome {
        x,
        y,
        speed_x,
        speed_y,
        rotation,
        animation_direction,
        jumping: state.jumping,
        falling: state.falling,
        entry_facing: entry.facing,
        exit_facing: exit.facing,
        exit_blocked: context.exit_blocked,
        portaled_exit_facing: exit.facing,
    })
}

#[must_use]
pub fn apply_legacy_drop_vine(
    state: &mut PlayerMovementState,
    x: f32,
    context: LegacyDropVineContext,
    physics: PhysicsConstants,
    constants: crate::config::LegacyVineConstants,
) -> LegacyDropVineOutcome {
    state.animation_state = PlayerAnimationState::Falling;
    state.gravity = physics.gravity;

    let x = match context.side {
        LegacyVineSide::Right => x + constants.drop_x_offset,
        LegacyVineSide::Left => x - constants.drop_x_offset,
    };

    LegacyDropVineOutcome {
        x,
        vine_active: false,
        vine_mask_enabled: false,
    }
}

#[must_use]
pub fn apply_legacy_on_vine_attachment_loss(
    state: &mut PlayerMovementState,
    x: f32,
    context: LegacyOnVineAttachmentLossContext,
    physics: PhysicsConstants,
    constants: crate::config::LegacyVineConstants,
) -> Option<LegacyDropVineOutcome> {
    if context.has_vine_overlap {
        return None;
    }

    Some(apply_legacy_drop_vine(
        state,
        x,
        LegacyDropVineContext { side: context.side },
        physics,
        constants,
    ))
}

pub fn apply_legacy_spring_high_request(state: &mut PlayerMovementState) {
    if state.spring {
        state.spring_high = true;
    }
}

pub fn apply_legacy_faithplate_state(state: &mut PlayerMovementState) {
    state.animation_state = PlayerAnimationState::Jumping;
    state.falling = true;
}

pub fn apply_legacy_player_movement(
    state: &mut PlayerMovementState,
    input: PlayerMovementInput,
    dt: f32,
    constants: PlayerMovementConstants,
) {
    if input.run {
        apply_running_movement(state, input, dt, constants);
    } else {
        apply_walking_movement(state, input, dt, constants);
    }
}

pub fn advance_legacy_player_animation(
    state: &mut PlayerMovementState,
    dt: f32,
    constants: PlayerAnimationConstants,
) {
    if state.animation_state == PlayerAnimationState::Running {
        advance_legacy_run_animation(state, dt, constants);
    }
}

pub fn advance_legacy_underwater_animation(
    state: &mut PlayerMovementState,
    dt: f32,
    constants: PlayerAnimationConstants,
) {
    if state.airborne() {
        if matches!(
            state.animation_state,
            PlayerAnimationState::Jumping | PlayerAnimationState::Falling
        ) {
            advance_legacy_swim_animation(state, dt, constants);
        }
    } else if state.animation_state == PlayerAnimationState::Running {
        advance_legacy_run_animation(state, dt, constants);
    }
}

pub fn apply_legacy_player_movement_with_surface_query(
    state: &mut PlayerMovementState,
    input: PlayerMovementInput,
    dt: f32,
    context: LegacySurfaceMovementContext,
    top_gel_at: impl FnOnce(LegacyMapTileCoord) -> Option<LegacyGelKind>,
) {
    let top_gel =
        legacy_top_ground_probe(state, context.body, context.map_bounds).and_then(top_gel_at);
    let constants =
        legacy_surface_movement_constants(context.movement, context.orange_gel, top_gel);

    apply_legacy_player_movement(state, input, dt, constants);
}

pub fn apply_legacy_underwater_movement(
    state: &mut PlayerMovementState,
    input: PlayerMovementInput,
    dt: f32,
    constants: UnderwaterMovementConstants,
    bounds: Option<PlayerVerticalBounds>,
) {
    if input.right {
        if state.airborne() {
            if state.speed_x < constants.max_air_walk_speed {
                accelerate_underwater_air_right(state, dt, constants);
                if state.speed_x > constants.max_air_walk_speed {
                    state.speed_x = constants.max_air_walk_speed;
                }
            }
        } else if !state.ducking {
            if state.speed_x < constants.max_walk_speed {
                if state.speed_x < 0.0 {
                    if state.speed_x < -constants.max_run_speed {
                        state.speed_x +=
                            constants.super_friction * dt + constants.run_acceleration * dt;
                    } else {
                        state.speed_x += constants.friction * dt + constants.run_acceleration * dt;
                    }
                    state.animation_state = PlayerAnimationState::Sliding;
                    state.animation_direction = HorizontalDirection::Right;
                } else {
                    state.speed_x += constants.walk_acceleration * dt;
                    state.animation_state = PlayerAnimationState::Running;
                    state.animation_direction = HorizontalDirection::Right;
                }

                if state.speed_x > constants.max_walk_speed {
                    state.speed_x = constants.max_walk_speed;
                }
            } else {
                state.speed_x -= constants.friction * dt;
                if state.speed_x < constants.max_walk_speed {
                    state.speed_x = constants.max_walk_speed;
                }
            }
        }
    } else if input.left {
        if state.airborne() {
            if state.speed_x > -constants.max_air_walk_speed {
                accelerate_underwater_air_left(state, dt, constants);
                if state.speed_x < -constants.max_air_walk_speed {
                    state.speed_x = -constants.max_air_walk_speed;
                }
            }
        } else if !state.ducking {
            if state.speed_x > -constants.max_walk_speed {
                if state.speed_x > 0.0 {
                    if state.speed_x > constants.max_run_speed {
                        state.speed_x -=
                            constants.super_friction * dt + constants.run_acceleration * dt;
                    } else {
                        state.speed_x -= constants.friction * dt + constants.run_acceleration * dt;
                    }
                    state.animation_state = PlayerAnimationState::Sliding;
                    state.animation_direction = HorizontalDirection::Left;
                } else {
                    state.speed_x -= constants.walk_acceleration * dt;
                    state.animation_state = PlayerAnimationState::Running;
                    state.animation_direction = HorizontalDirection::Left;
                }

                if state.speed_x < -constants.max_walk_speed {
                    state.speed_x = -constants.max_walk_speed;
                }
            } else {
                state.speed_x += constants.friction * dt;
                if state.speed_x > -constants.max_walk_speed {
                    state.speed_x = -constants.max_walk_speed;
                }
            }
        }
    } else {
        apply_underwater_no_movement(state, dt, constants);
    }

    if bounds.is_some_and(|bounds| bounds.bottom() < constants.max_height) {
        state.speed_y = constants.push_down_speed;
    }
}

fn advance_legacy_run_animation(
    state: &mut PlayerMovementState,
    dt: f32,
    constants: PlayerAnimationConstants,
) {
    state.run_animation_progress +=
        (state.speed_x.abs() + 4.0) / 5.0 * dt * constants.run_animation_speed;

    while state.run_animation_progress >= 4.0 {
        state.run_animation_progress -= 3.0;
    }

    state.run_frame = state.run_animation_progress.floor() as u8;
}

fn advance_legacy_swim_animation(
    state: &mut PlayerMovementState,
    dt: f32,
    constants: PlayerAnimationConstants,
) {
    state.swim_animation_progress += constants.run_animation_speed * dt;

    while state.swim_animation_progress >= 3.0 {
        state.swim_animation_progress -= 2.0;
    }

    state.swim_frame = state.swim_animation_progress.floor() as u8;
}

#[must_use]
pub fn legacy_top_ground_probe(
    state: &PlayerMovementState,
    body: PlayerBodyBounds,
    map_bounds: LegacyMapBounds,
) -> Option<LegacyMapTileCoord> {
    if state.airborne() || body.bottom() % 1.0 != 0.0 {
        return None;
    }

    let coord = LegacyMapTileCoord::new(
        lua_round(body.x + body.width / 2.0 + 0.5),
        body.bottom() as i32 + 1,
    );
    map_bounds.contains(coord).then_some(coord)
}

#[must_use]
pub fn legacy_surface_movement_constants(
    mut movement: PlayerMovementConstants,
    orange_gel: OrangeGelMovementConstants,
    top_gel: Option<LegacyGelKind>,
) -> PlayerMovementConstants {
    if top_gel == Some(LegacyGelKind::Orange) {
        movement.max_run_speed = orange_gel.max_run_speed;
        movement.max_walk_speed = orange_gel.max_walk_speed;
        movement.run_acceleration = orange_gel.run_acceleration;
        movement.walk_acceleration = orange_gel.walk_acceleration;
    }

    movement
}

#[must_use]
pub fn legacy_jump_velocity(
    speed_x: f32,
    physics: PhysicsConstants,
    movement: PlayerMovementConstants,
) -> f32 {
    let force =
        -physics.jump_force - (speed_x.abs() / movement.max_run_speed) * physics.jump_force_add;
    force.max(-physics.jump_force - physics.jump_force_add)
}

pub fn try_legacy_jump(
    state: &mut PlayerMovementState,
    physics: PhysicsConstants,
    movement: PlayerMovementConstants,
) -> bool {
    if state.falling {
        return false;
    }

    state.speed_y = legacy_jump_velocity(state.speed_x, physics, movement);
    state.jumping = true;
    state.animation_state = PlayerAnimationState::Jumping;
    true
}

#[must_use]
fn lua_round(value: f32) -> i32 {
    (value + 0.5).floor() as i32
}

#[must_use]
pub fn legacy_underwater_jump_velocity(
    speed_x: f32,
    underwater: UnderwaterMovementConstants,
    movement: PlayerMovementConstants,
) -> f32 {
    -underwater.jump_force - (speed_x.abs() / movement.max_run_speed) * underwater.jump_force_add
}

pub fn apply_legacy_underwater_jump(
    state: &mut PlayerMovementState,
    underwater: UnderwaterMovementConstants,
    movement: PlayerMovementConstants,
) {
    state.ducking = false;
    state.speed_y = legacy_underwater_jump_velocity(state.speed_x, underwater, movement);
    state.jumping = true;
    state.animation_state = PlayerAnimationState::Jumping;
}

pub fn stop_legacy_jump(state: &mut PlayerMovementState) {
    if state.jumping {
        state.jumping = false;
        state.falling = true;
    }
}

fn apply_running_movement(
    state: &mut PlayerMovementState,
    input: PlayerMovementInput,
    dt: f32,
    constants: PlayerMovementConstants,
) {
    if input.right {
        if state.airborne() {
            if state.speed_x < constants.max_walk_speed {
                accelerate_air_right(state, constants.air_run_acceleration, dt, constants);
                if state.speed_x > constants.max_walk_speed {
                    state.speed_x = constants.max_walk_speed;
                }
            } else if state.speed_x > constants.max_walk_speed
                && state.speed_x < constants.max_run_speed
            {
                accelerate_air_right(state, constants.air_run_acceleration, dt, constants);
                if state.speed_x > constants.max_run_speed {
                    state.speed_x = constants.max_run_speed;
                }
            }
        } else if !state.ducking {
            if state.speed_x < 0.0 {
                // Preserve Mari0's sign check exactly; tests lock the legacy quirk.
                if state.speed_x > constants.max_run_speed {
                    state.speed_x +=
                        constants.super_friction * dt + constants.run_acceleration * dt;
                } else {
                    state.speed_x += constants.friction * dt + constants.run_acceleration * dt;
                }
                state.animation_state = PlayerAnimationState::Sliding;
                state.animation_direction = HorizontalDirection::Right;
            } else {
                state.speed_x += constants.run_acceleration * dt;
                state.animation_state = PlayerAnimationState::Running;
                state.animation_direction = HorizontalDirection::Right;
            }

            if state.speed_x > constants.max_run_speed {
                state.speed_x = constants.max_run_speed;
            }
        }
    } else if input.left {
        if state.airborne() {
            if state.speed_x > -constants.max_walk_speed {
                accelerate_air_left(state, constants.air_run_acceleration, dt, constants);
                if state.speed_x < -constants.max_walk_speed {
                    state.speed_x = -constants.max_walk_speed;
                }
            } else if state.speed_x < -constants.max_walk_speed
                && state.speed_x > -constants.max_run_speed
            {
                accelerate_air_left(state, constants.air_run_acceleration, dt, constants);
                if state.speed_x < -constants.max_run_speed {
                    state.speed_x = -constants.max_run_speed;
                }
            }
        } else if !state.ducking {
            if state.speed_x > 0.0 {
                // Preserve Mari0's sign check exactly; tests lock the legacy quirk.
                if state.speed_x < -constants.max_run_speed {
                    state.speed_x -=
                        constants.super_friction * dt + constants.run_acceleration * dt;
                } else {
                    state.speed_x -= constants.friction * dt + constants.run_acceleration * dt;
                }
                state.animation_state = PlayerAnimationState::Sliding;
                state.animation_direction = HorizontalDirection::Left;
            } else {
                state.speed_x -= constants.run_acceleration * dt;
                state.animation_state = PlayerAnimationState::Running;
                state.animation_direction = HorizontalDirection::Left;
            }

            if state.speed_x < -constants.max_run_speed {
                state.speed_x = -constants.max_run_speed;
            }
        }
    }

    if no_direction(input) || grounded_ducking(state) {
        apply_no_movement_running(state, dt, constants);
    }
}

fn apply_walking_movement(
    state: &mut PlayerMovementState,
    input: PlayerMovementInput,
    dt: f32,
    constants: PlayerMovementConstants,
) {
    if input.right {
        if state.airborne() {
            if state.speed_x < constants.max_walk_speed {
                accelerate_air_right(state, constants.air_walk_acceleration, dt, constants);
                if state.speed_x > constants.max_walk_speed {
                    state.speed_x = constants.max_walk_speed;
                }
            }
        } else if !state.ducking {
            if state.speed_x < constants.max_walk_speed {
                if state.speed_x < 0.0 {
                    if state.speed_x < -constants.max_run_speed {
                        state.speed_x +=
                            constants.super_friction * dt + constants.run_acceleration * dt;
                    } else {
                        state.speed_x += constants.friction * dt + constants.run_acceleration * dt;
                    }
                    state.animation_state = PlayerAnimationState::Sliding;
                    state.animation_direction = HorizontalDirection::Right;
                } else {
                    state.speed_x += constants.walk_acceleration * dt;
                    state.animation_state = PlayerAnimationState::Running;
                    state.animation_direction = HorizontalDirection::Right;
                }

                if state.speed_x > constants.max_walk_speed {
                    state.speed_x = constants.max_walk_speed;
                }
            } else {
                state.speed_x -= constants.friction * dt;
                if state.speed_x < constants.max_walk_speed {
                    state.speed_x = constants.max_walk_speed;
                }
            }
        }
    } else if input.left {
        if state.airborne() {
            if state.speed_x > -constants.max_walk_speed {
                accelerate_air_left(state, constants.air_walk_acceleration, dt, constants);
                if state.speed_x < -constants.max_walk_speed {
                    state.speed_x = -constants.max_walk_speed;
                }
            }
        } else if !state.ducking {
            if state.speed_x > -constants.max_walk_speed {
                if state.speed_x > 0.0 {
                    if state.speed_x > constants.max_run_speed {
                        state.speed_x -=
                            constants.super_friction * dt + constants.run_acceleration * dt;
                    } else {
                        state.speed_x -= constants.friction * dt + constants.run_acceleration * dt;
                    }
                    state.animation_state = PlayerAnimationState::Sliding;
                    state.animation_direction = HorizontalDirection::Left;
                } else {
                    state.speed_x -= constants.walk_acceleration * dt;
                    state.animation_state = PlayerAnimationState::Running;
                    state.animation_direction = HorizontalDirection::Left;
                }

                if state.speed_x < -constants.max_walk_speed {
                    state.speed_x = -constants.max_walk_speed;
                }
            } else {
                state.speed_x += constants.friction * dt;
                if state.speed_x > -constants.max_walk_speed {
                    state.speed_x = -constants.max_walk_speed;
                }
            }
        }
    }

    if no_direction(input) || grounded_ducking(state) {
        apply_no_movement_walking(state, dt, constants);
    }
}

fn accelerate_air_right(
    state: &mut PlayerMovementState,
    acceleration: f32,
    dt: f32,
    constants: PlayerMovementConstants,
) {
    if state.speed_x < 0.0 {
        state.speed_x += acceleration * dt * constants.air_slide_factor;
    } else {
        state.speed_x += acceleration * dt;
    }
}

fn accelerate_air_left(
    state: &mut PlayerMovementState,
    acceleration: f32,
    dt: f32,
    constants: PlayerMovementConstants,
) {
    if state.speed_x > 0.0 {
        state.speed_x -= acceleration * dt * constants.air_slide_factor;
    } else {
        state.speed_x -= acceleration * dt;
    }
}

fn accelerate_underwater_air_right(
    state: &mut PlayerMovementState,
    dt: f32,
    constants: UnderwaterMovementConstants,
) {
    if state.speed_x < 0.0 {
        state.speed_x += constants.air_walk_acceleration * dt * constants.air_slide_factor;
    } else {
        state.speed_x += constants.air_walk_acceleration * dt;
    }
}

fn accelerate_underwater_air_left(
    state: &mut PlayerMovementState,
    dt: f32,
    constants: UnderwaterMovementConstants,
) {
    if state.speed_x > 0.0 {
        state.speed_x -= constants.air_walk_acceleration * dt * constants.air_slide_factor;
    } else {
        state.speed_x -= constants.air_walk_acceleration * dt;
    }
}

fn apply_underwater_no_movement(
    state: &mut PlayerMovementState,
    dt: f32,
    constants: UnderwaterMovementConstants,
) {
    if state.airborne() {
        if state.speed_x > 0.0 {
            state.speed_x -= constants.air_friction * dt;
            if state.speed_x < 0.0 {
                stop_horizontal(state);
            }
        } else {
            state.speed_x += constants.air_friction * dt;
            if state.speed_x > 0.0 {
                stop_horizontal(state);
            }
        }
    } else if state.speed_x > 0.0 {
        if state.speed_x > constants.max_run_speed {
            state.speed_x -= constants.super_friction * dt;
        } else {
            state.speed_x -= constants.friction * dt;
        }
        if state.speed_x < 0.0 {
            stop_horizontal_grounded(state);
        }
    } else {
        if state.speed_x < -constants.max_run_speed {
            state.speed_x += constants.super_friction * dt;
        } else {
            state.speed_x += constants.friction * dt;
        }
        if state.speed_x > 0.0 {
            stop_horizontal_grounded(state);
        }
    }
}

fn apply_no_movement_running(
    state: &mut PlayerMovementState,
    dt: f32,
    constants: PlayerMovementConstants,
) {
    if state.airborne() {
        if state.speed_x > 0.0 {
            state.speed_x -= constants.air_friction * dt;
            if state.speed_x < constants.min_speed {
                stop_horizontal(state);
            }
        } else {
            state.speed_x += constants.air_friction * dt;
            if state.speed_x > -constants.min_speed {
                stop_horizontal(state);
            }
        }
    } else if state.speed_x > 0.0 {
        if state.speed_x > constants.max_run_speed {
            state.speed_x -= constants.super_friction * dt;
        } else {
            state.speed_x -= constants.friction * dt;
        }
        if state.speed_x < constants.min_speed {
            stop_horizontal_grounded(state);
        }
    } else {
        if state.speed_x < -constants.max_run_speed {
            state.speed_x += constants.super_friction * dt;
        } else {
            state.speed_x += constants.friction * dt;
        }
        if state.speed_x > -constants.min_speed {
            stop_horizontal_grounded(state);
        }
    }
}

fn apply_no_movement_walking(
    state: &mut PlayerMovementState,
    dt: f32,
    constants: PlayerMovementConstants,
) {
    if state.airborne() {
        if state.speed_x > 0.0 {
            state.speed_x -= constants.air_friction * dt;
            if state.speed_x < 0.0 {
                stop_horizontal(state);
            }
        } else {
            state.speed_x += constants.air_friction * dt;
            if state.speed_x > 0.0 {
                stop_horizontal(state);
            }
        }
    } else if state.speed_x > 0.0 {
        if state.speed_x > constants.max_run_speed {
            state.speed_x -= constants.super_friction * dt;
        } else {
            state.speed_x -= constants.friction * dt;
        }
        if state.speed_x < 0.0 {
            stop_horizontal_grounded(state);
        }
    } else {
        if state.speed_x < -constants.max_run_speed {
            state.speed_x += constants.super_friction * dt;
        } else {
            state.speed_x += constants.friction * dt;
        }
        if state.speed_x > 0.0 {
            stop_horizontal_grounded(state);
        }
    }
}

fn stop_horizontal(state: &mut PlayerMovementState) {
    state.speed_x = 0.0;
    state.run_frame = 1;
}

fn stop_horizontal_grounded(state: &mut PlayerMovementState) {
    stop_horizontal(state);
    state.animation_state = PlayerAnimationState::Idle;
}

#[must_use]
fn legacy_spring_frame_y_offset(frame: u8) -> f32 {
    match frame {
        2 => 0.5,
        3 => 1.0,
        _ => 0.0,
    }
}

#[must_use]
fn legacy_vine_climb_frame(timer: f32, frame_delay: f32) -> u8 {
    (((timer % (frame_delay * 2.0)) / frame_delay).ceil() as u8).max(1)
}

#[must_use]
fn legacy_horizontal_portal_entry(
    body: PlayerBodyBounds,
    next_y: f32,
    portals: LegacyOnVineHorizontalPortalPair,
) -> Option<(LegacyPortalEndpoint, LegacyPortalEndpoint)> {
    if legacy_horizontal_portal_crossed(body, next_y, portals.portal1) {
        Some((portals.portal1, portals.portal2))
    } else if legacy_horizontal_portal_crossed(body, next_y, portals.portal2) {
        Some((portals.portal2, portals.portal1))
    } else {
        None
    }
}

#[must_use]
fn legacy_horizontal_portal_crossed(
    body: PlayerBodyBounds,
    next_y: f32,
    portal: LegacyPortalEndpoint,
) -> bool {
    let (x_plus, portal_y) = match portal.facing {
        Facing::Up => (1.0, portal.y - 1.0),
        Facing::Down => (-1.0, portal.y),
        Facing::Left | Facing::Right => (0.0, portal.y),
    };
    let probe_x = (body.x + 1.0).floor();
    let current_center_y = body.y + body.height / 2.0;
    let next_center_y = next_y + body.height / 2.0;

    (portal.x == probe_x || portal.x + x_plus == probe_x)
        && in_range(portal_y, current_center_y, next_center_y, false)
}

#[must_use]
fn legacy_wormhole_animation_direction(direction: HorizontalDirection) -> AnimationDirection {
    match direction {
        HorizontalDirection::Left => AnimationDirection::Left,
        HorizontalDirection::Right => AnimationDirection::Right,
    }
}

#[must_use]
fn legacy_player_animation_direction(
    direction: Option<AnimationDirection>,
    fallback: HorizontalDirection,
) -> HorizontalDirection {
    match direction {
        Some(AnimationDirection::Left) => HorizontalDirection::Left,
        Some(AnimationDirection::Right) => HorizontalDirection::Right,
        None => fallback,
    }
}

#[must_use]
fn no_direction(input: PlayerMovementInput) -> bool {
    !input.right && !input.left
}

#[must_use]
fn grounded_ducking(state: &PlayerMovementState) -> bool {
    state.ducking && !state.airborne()
}

#[cfg(test)]
mod tests {
    use core::f32::consts::FRAC_PI_2;

    use super::{
        HorizontalDirection, LegacyCeilingTileContext, LegacyCeilingTileResponse,
        LegacyDropVineContext, LegacyGelKind, LegacyGrabVineContext, LegacyGrabVineOutcome,
        LegacyHeldSpringContext, LegacyHeldSpringUpdate, LegacyMapBounds, LegacyMapTileCoord,
        LegacyOnVineAttachmentLossContext, LegacyOnVineContext, LegacyOnVineDirection,
        LegacyOnVineHorizontalPortalContext, LegacyOnVineHorizontalPortalPair, LegacyPipeCandidate,
        LegacyPipeDirection, LegacyPipeEntry, LegacyPlayerCollisionSnapshot,
        LegacyPlayerDeathCause, LegacyPlayerHazard, LegacyPlayerHazardCollision,
        LegacyPlayerHazardOutcome, LegacySideBoxResponse, LegacySurfaceMovementContext,
        LegacyVineSide, PlayerAnimationState, PlayerBodyBounds, PlayerEnvironment,
        PlayerMovementInput, PlayerMovementState, PlayerSideCollision, PlayerVerticalBounds,
        advance_legacy_player_animation, advance_legacy_underwater_animation,
        apply_legacy_drop_vine, apply_legacy_enemy_stomp_bounce_state,
        apply_legacy_faithplate_state, apply_legacy_floor_blue_gel_bounce,
        apply_legacy_floor_invisible_tile_suppression, apply_legacy_floor_landing_state,
        apply_legacy_grab_vine, apply_legacy_head_bump_state, apply_legacy_hit_spring_state,
        apply_legacy_leave_spring_state, apply_legacy_non_invisible_ceiling_tile_response,
        apply_legacy_on_vine_attachment_loss, apply_legacy_on_vine_horizontal_portal,
        apply_legacy_player_gravity_selection, apply_legacy_player_gravity_velocity,
        apply_legacy_player_movement, apply_legacy_player_movement_with_surface_query,
        apply_legacy_side_blue_gel_bounce, apply_legacy_side_box_response,
        apply_legacy_side_button_response, apply_legacy_side_tile_gap_run,
        apply_legacy_spring_high_request, apply_legacy_start_fall_after_vertical_move,
        apply_legacy_underwater_jump, apply_legacy_underwater_movement,
        legacy_ceiling_invisible_tile_suppresses_default, legacy_enemy_stomp_bounce_speed,
        legacy_jump_velocity, legacy_leave_spring_y, legacy_player_hazard_collision_outcome,
        legacy_right_side_pipe_entry, legacy_side_invisible_tile_suppresses_default,
        legacy_surface_movement_constants, legacy_top_ground_probe,
        legacy_underwater_jump_velocity, stop_legacy_jump, try_legacy_jump,
        update_legacy_held_spring, update_legacy_on_vine_motion,
    };
    use crate::config::{
        BlueGelBounceConstants, LegacyVineConstants, OrangeGelMovementConstants, PhysicsConstants,
        PlayerAnimationConstants, PlayerMovementConstants, SpringConstants,
        UnderwaterMovementConstants,
    };
    use crate::wormhole::{Facing, LegacyPortalEndpoint};

    const EPSILON: f32 = 0.000_01;

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < EPSILON,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn default_player_gravity_matches_normal_mario_gravity() {
        let state = PlayerMovementState::default();

        assert_close(state.gravity, 80.0);
    }

    #[test]
    fn gravity_selection_uses_jump_gravity_before_jump_to_fall_transition() {
        let physics = PhysicsConstants::default();
        let underwater = UnderwaterMovementConstants::default();
        let mut ascending = PlayerMovementState {
            speed_y: -3.0,
            jumping: true,
            animation_state: PlayerAnimationState::Jumping,
            ..PlayerMovementState::default()
        };
        let mut descending = PlayerMovementState {
            speed_y: 0.5,
            jumping: true,
            animation_state: PlayerAnimationState::Jumping,
            ..PlayerMovementState::default()
        };

        apply_legacy_player_gravity_selection(
            &mut ascending,
            PlayerEnvironment::Normal,
            physics,
            underwater,
        );
        apply_legacy_player_gravity_selection(
            &mut descending,
            PlayerEnvironment::Normal,
            physics,
            underwater,
        );

        assert_close(ascending.gravity, 30.0);
        assert!(ascending.jumping);
        assert!(!ascending.falling);
        assert_close(descending.gravity, 30.0);
        assert!(!descending.jumping);
        assert!(descending.falling);
        assert_eq!(descending.animation_state, PlayerAnimationState::Jumping);
    }

    #[test]
    fn gravity_selection_uses_underwater_gravity_values() {
        let physics = PhysicsConstants::default();
        let underwater = UnderwaterMovementConstants::default();
        let mut swimming = PlayerMovementState::default();
        let mut jumping = PlayerMovementState {
            jumping: true,
            speed_y: -1.0,
            ..PlayerMovementState::default()
        };

        apply_legacy_player_gravity_selection(
            &mut swimming,
            PlayerEnvironment::Underwater,
            physics,
            underwater,
        );
        apply_legacy_player_gravity_selection(
            &mut jumping,
            PlayerEnvironment::Underwater,
            physics,
            underwater,
        );

        assert_close(swimming.gravity, 9.0);
        assert_close(jumping.gravity, 12.0);
    }

    #[test]
    fn gravity_velocity_applies_current_gravity_and_clamps_fall_speed() {
        let physics = PhysicsConstants::default();
        let mut state = PlayerMovementState {
            speed_y: 95.0,
            gravity: 80.0,
            ..PlayerMovementState::default()
        };

        apply_legacy_player_gravity_velocity(&mut state, 0.1, physics);

        assert_close(state.speed_y, 100.0);
    }

    #[test]
    fn startfall_after_vertical_move_matches_physics_exact_velocity_check() {
        let mut state = PlayerMovementState {
            speed_y: 8.0,
            gravity: 80.0,
            animation_state: PlayerAnimationState::Idle,
            ..PlayerMovementState::default()
        };

        assert!(apply_legacy_start_fall_after_vertical_move(&mut state, 0.1));
        assert!(state.falling);
        assert_eq!(state.animation_state, PlayerAnimationState::Falling);

        let mut already_falling = PlayerMovementState {
            speed_y: 8.0,
            gravity: 80.0,
            falling: true,
            animation_state: PlayerAnimationState::Running,
            ..PlayerMovementState::default()
        };
        assert!(!apply_legacy_start_fall_after_vertical_move(
            &mut already_falling,
            0.1
        ));
        assert_eq!(
            already_falling.animation_state,
            PlayerAnimationState::Running
        );
    }

    #[test]
    fn floor_landing_sets_idle_for_zero_horizontal_speed() {
        let mut state = PlayerMovementState {
            speed_x: 0.0,
            jumping: true,
            falling: true,
            animation_state: PlayerAnimationState::Jumping,
            ..PlayerMovementState::default()
        };

        apply_legacy_floor_landing_state(&mut state);

        assert_eq!(state.animation_state, PlayerAnimationState::Idle);
        assert!(!state.jumping);
        assert!(!state.falling);
    }

    #[test]
    fn floor_landing_sets_running_for_nonzero_speed_unless_sliding() {
        let mut running = PlayerMovementState {
            speed_x: -2.0,
            falling: true,
            animation_state: PlayerAnimationState::Falling,
            ..PlayerMovementState::default()
        };
        let mut sliding = PlayerMovementState {
            speed_x: 2.0,
            falling: true,
            animation_state: PlayerAnimationState::Sliding,
            ..PlayerMovementState::default()
        };

        apply_legacy_floor_landing_state(&mut running);
        apply_legacy_floor_landing_state(&mut sliding);

        assert_eq!(running.animation_state, PlayerAnimationState::Running);
        assert_eq!(sliding.animation_state, PlayerAnimationState::Sliding);
        assert!(!running.falling);
        assert!(!sliding.falling);
    }

    #[test]
    fn head_bump_state_sets_head_force_without_changing_animation() {
        let mut state = PlayerMovementState {
            speed_y: -12.0,
            jumping: true,
            falling: false,
            animation_state: PlayerAnimationState::Jumping,
            ..PlayerMovementState::default()
        };

        apply_legacy_head_bump_state(&mut state, PhysicsConstants::default());

        assert_close(state.speed_y, 2.0);
        assert!(!state.jumping);
        assert!(state.falling);
        assert_eq!(state.animation_state, PlayerAnimationState::Jumping);
    }

    #[test]
    fn floor_invisible_tile_suppression_restores_pre_floor_collision_flags() {
        let snapshot =
            LegacyPlayerCollisionSnapshot::new(true, false, PlayerAnimationState::Jumping);
        let mut state = PlayerMovementState {
            jumping: false,
            falling: false,
            speed_x: 4.0,
            speed_y: 3.0,
            animation_state: PlayerAnimationState::Running,
            ..PlayerMovementState::default()
        };

        assert!(apply_legacy_floor_invisible_tile_suppression(
            &mut state, snapshot, true,
        ));
        assert!(state.jumping);
        assert!(!state.falling);
        assert_eq!(state.animation_state, PlayerAnimationState::Jumping);
        assert_close(state.speed_y, 3.0);

        let unchanged = state;
        assert!(!apply_legacy_floor_invisible_tile_suppression(
            &mut state, snapshot, false,
        ));
        assert_eq!(state, unchanged);
    }

    #[test]
    fn collision_snapshot_captures_only_legacy_restored_floor_fields() {
        let state = PlayerMovementState {
            jumping: true,
            falling: false,
            animation_state: PlayerAnimationState::Sliding,
            speed_x: 7.0,
            speed_y: 9.0,
            ..PlayerMovementState::default()
        };

        assert_eq!(
            LegacyPlayerCollisionSnapshot::from_state(&state),
            LegacyPlayerCollisionSnapshot::new(true, false, PlayerAnimationState::Sliding)
        );
    }

    #[test]
    fn side_invisible_tile_suppresses_default_resolution() {
        assert!(legacy_side_invisible_tile_suppresses_default(true));
        assert!(!legacy_side_invisible_tile_suppresses_default(false));
    }

    #[test]
    fn ceiling_invisible_tile_suppression_preserves_legacy_position_speed_check() {
        let tile = PlayerBodyBounds::new(4.0, 8.0, 1.0, 1.0);
        let below_or_at_previous_top = PlayerBodyBounds::new(4.0, 8.25, 0.75, 0.75);
        let past_previous_top = PlayerBodyBounds::new(4.0, 8.25, 0.75, 0.75);

        assert!(legacy_ceiling_invisible_tile_suppresses_default(
            below_or_at_previous_top,
            PlayerMovementState {
                speed_y: 0.25,
                ..PlayerMovementState::default()
            },
            tile,
            true,
        ));
        assert!(!legacy_ceiling_invisible_tile_suppresses_default(
            past_previous_top,
            PlayerMovementState {
                speed_y: 0.20,
                ..PlayerMovementState::default()
            },
            tile,
            true,
        ));
        assert!(!legacy_ceiling_invisible_tile_suppresses_default(
            below_or_at_previous_top,
            PlayerMovementState {
                speed_y: 0.25,
                ..PlayerMovementState::default()
            },
            tile,
            false,
        ));
    }

    #[test]
    fn non_invisible_ceiling_tile_breaks_block_for_big_mario_without_head_bump() {
        let mut state = PlayerMovementState {
            speed_x: 3.0,
            speed_y: -6.0,
            jumping: true,
            animation_state: PlayerAnimationState::Jumping,
            ..PlayerMovementState::default()
        };
        let original_state = state;
        let mut body = PlayerBodyBounds::new(3.7, 7.8, 0.75, 0.75);
        let original_body = body;

        let response = apply_legacy_non_invisible_ceiling_tile_response(
            &mut state,
            &mut body,
            LegacyCeilingTileContext {
                tile: LegacyMapTileCoord::new(5, 8),
                big_mario: true,
                map_width: 10,
                left_neighbor_solid: true,
                right_neighbor_solid: true,
                push_left_clear: true,
                push_right_clear: true,
                physics: PhysicsConstants::default(),
            },
        );

        assert_eq!(
            response,
            LegacyCeilingTileResponse::BreakBlock {
                coord: LegacyMapTileCoord::new(5, 8),
            }
        );
        assert_eq!(state, original_state);
        assert_eq!(body, original_body);
    }

    #[test]
    fn non_invisible_ceiling_tile_retargets_solid_neighbors_and_applies_head_bump() {
        let physics = PhysicsConstants::default();
        let mut left_state = PlayerMovementState {
            speed_y: -6.0,
            jumping: true,
            ..PlayerMovementState::default()
        };
        let mut right_state = left_state;
        let mut left_body = PlayerBodyBounds::new(3.5, 7.8, 0.75, 0.75);
        let mut right_body = PlayerBodyBounds::new(4.7, 7.8, 0.75, 0.75);

        let left_response = apply_legacy_non_invisible_ceiling_tile_response(
            &mut left_state,
            &mut left_body,
            LegacyCeilingTileContext {
                tile: LegacyMapTileCoord::new(5, 8),
                big_mario: false,
                map_width: 10,
                left_neighbor_solid: true,
                right_neighbor_solid: false,
                push_left_clear: true,
                push_right_clear: true,
                physics,
            },
        );
        let right_response = apply_legacy_non_invisible_ceiling_tile_response(
            &mut right_state,
            &mut right_body,
            LegacyCeilingTileContext {
                tile: LegacyMapTileCoord::new(5, 8),
                big_mario: false,
                map_width: 10,
                left_neighbor_solid: false,
                right_neighbor_solid: true,
                push_left_clear: true,
                push_right_clear: true,
                physics,
            },
        );

        assert_eq!(
            left_response,
            LegacyCeilingTileResponse::HitBlock {
                coord: LegacyMapTileCoord::new(4, 8),
            }
        );
        assert_eq!(
            right_response,
            LegacyCeilingTileResponse::HitBlock {
                coord: LegacyMapTileCoord::new(6, 8),
            }
        );
        assert_close(left_state.speed_y, physics.head_force);
        assert_close(right_state.speed_y, physics.head_force);
        assert!(!left_state.jumping);
        assert!(left_state.falling);
    }

    #[test]
    fn non_invisible_ceiling_tile_pushes_player_when_side_space_is_clear() {
        let physics = PhysicsConstants::default();
        let mut left_state = PlayerMovementState {
            speed_x: 3.0,
            speed_y: -6.0,
            jumping: true,
            ..PlayerMovementState::default()
        };
        let mut right_state = PlayerMovementState {
            speed_x: -3.0,
            speed_y: -6.0,
            jumping: true,
            ..PlayerMovementState::default()
        };
        let mut left_body = PlayerBodyBounds::new(3.5, 7.8, 0.75, 0.75);
        let mut right_body = PlayerBodyBounds::new(4.7, 7.8, 0.75, 0.75);

        let left_response = apply_legacy_non_invisible_ceiling_tile_response(
            &mut left_state,
            &mut left_body,
            LegacyCeilingTileContext {
                tile: LegacyMapTileCoord::new(5, 8),
                big_mario: false,
                map_width: 10,
                left_neighbor_solid: false,
                right_neighbor_solid: false,
                push_left_clear: true,
                push_right_clear: false,
                physics,
            },
        );
        let right_response = apply_legacy_non_invisible_ceiling_tile_response(
            &mut right_state,
            &mut right_body,
            LegacyCeilingTileContext {
                tile: LegacyMapTileCoord::new(5, 8),
                big_mario: false,
                map_width: 10,
                left_neighbor_solid: false,
                right_neighbor_solid: false,
                push_left_clear: false,
                push_right_clear: true,
                physics,
            },
        );

        assert_eq!(
            left_response,
            LegacyCeilingTileResponse::PushPlayer { x: 3.25 }
        );
        assert_eq!(
            right_response,
            LegacyCeilingTileResponse::PushPlayer { x: 5.0 }
        );
        assert_close(left_body.x, 3.25);
        assert_close(right_body.x, 5.0);
        assert_close(left_state.speed_x, 0.0);
        assert_close(right_state.speed_x, 0.0);
        assert_close(left_state.speed_y, -6.0);
        assert!(left_state.jumping);
    }

    #[test]
    fn non_invisible_ceiling_tile_hits_original_tile_when_side_push_is_blocked() {
        let physics = PhysicsConstants::default();
        let mut state = PlayerMovementState {
            speed_x: 3.0,
            speed_y: -6.0,
            jumping: true,
            ..PlayerMovementState::default()
        };
        let mut body = PlayerBodyBounds::new(3.5, 7.8, 0.75, 0.75);

        let response = apply_legacy_non_invisible_ceiling_tile_response(
            &mut state,
            &mut body,
            LegacyCeilingTileContext {
                tile: LegacyMapTileCoord::new(5, 8),
                big_mario: false,
                map_width: 10,
                left_neighbor_solid: false,
                right_neighbor_solid: false,
                push_left_clear: false,
                push_right_clear: true,
                physics,
            },
        );

        assert_eq!(
            response,
            LegacyCeilingTileResponse::HitBlock {
                coord: LegacyMapTileCoord::new(5, 8),
            }
        );
        assert_close(state.speed_y, physics.head_force);
        assert!(!state.jumping);
        assert!(state.falling);
        assert_close(body.x, 3.5);
    }

    #[test]
    fn side_tile_gap_run_snaps_to_tile_top_and_side_edge() {
        let physics = PhysicsConstants::default();
        let tile = PlayerBodyBounds::new(8.0, 4.0, 1.0, 1.0);
        let mut right_state = PlayerMovementState {
            speed_y: 2.5,
            falling: true,
            animation_state: PlayerAnimationState::Falling,
            ..PlayerMovementState::default()
        };
        let mut left_state = right_state;
        let mut right_body = PlayerBodyBounds::new(7.2, 3.44, 0.75, 0.625);
        let mut left_body = PlayerBodyBounds::new(8.2, 3.44, 0.75, 0.625);

        assert!(apply_legacy_side_tile_gap_run(
            &mut right_state,
            &mut right_body,
            tile,
            PlayerSideCollision::Right,
            Some(false),
            physics,
        ));
        assert!(apply_legacy_side_tile_gap_run(
            &mut left_state,
            &mut left_body,
            tile,
            PlayerSideCollision::Left,
            Some(false),
            physics,
        ));

        assert_close(right_body.y, tile.y - right_body.height);
        assert_close(right_body.x, tile.x - right_body.width + 0.0001);
        assert_close(left_body.x, tile.x + tile.width - 0.0001);
        assert_close(right_state.speed_y, 0.0);
        assert!(!right_state.falling);
        assert_eq!(right_state.animation_state, PlayerAnimationState::Running);
        assert_close(left_state.speed_y, 0.0);
        assert!(!left_state.falling);
    }

    #[test]
    fn side_tile_gap_run_requires_open_above_tile_falling_speed_and_room() {
        let physics = PhysicsConstants::default();
        let tile = PlayerBodyBounds::new(8.0, 4.0, 1.0, 1.0);
        let body = PlayerBodyBounds::new(7.2, 3.44, 0.75, 0.625);

        for (mut state, above_tile_collision, mut candidate) in [
            (
                PlayerMovementState {
                    speed_y: 2.5,
                    ..PlayerMovementState::default()
                },
                None,
                body,
            ),
            (
                PlayerMovementState {
                    speed_y: 2.5,
                    ..PlayerMovementState::default()
                },
                Some(true),
                body,
            ),
            (
                PlayerMovementState {
                    speed_y: 0.0,
                    ..PlayerMovementState::default()
                },
                Some(false),
                body,
            ),
            (
                PlayerMovementState {
                    speed_y: 2.5,
                    ..PlayerMovementState::default()
                },
                Some(false),
                PlayerBodyBounds::new(7.2, 3.45, 0.75, 0.625),
            ),
        ] {
            let original = candidate;

            assert!(!apply_legacy_side_tile_gap_run(
                &mut state,
                &mut candidate,
                tile,
                PlayerSideCollision::Right,
                above_tile_collision,
                physics,
            ));
            assert_eq!(candidate, original);
        }
    }

    #[test]
    fn side_box_response_damps_inward_speed_and_pushes_box_when_unblocked() {
        let movement = PlayerMovementConstants::default();
        let mut right = PlayerMovementState {
            speed_x: 8.0,
            ..PlayerMovementState::default()
        };
        let mut left = PlayerMovementState {
            speed_x: -8.0,
            ..PlayerMovementState::default()
        };

        let right_response = apply_legacy_side_box_response(
            &mut right,
            PlayerSideCollision::Right,
            true,
            0.1,
            movement,
        );
        let left_response = apply_legacy_side_box_response(
            &mut left,
            PlayerSideCollision::Left,
            true,
            0.1,
            movement,
        );

        assert_close(right.speed_x, 3.2);
        assert_close(left.speed_x, -3.2);
        assert!(right_response.suppress_default);
        assert!(left_response.suppress_default);
        assert_close(right_response.box_speed_x.unwrap_or_default(), 3.2);
        assert_close(left_response.box_speed_x.unwrap_or_default(), -3.2);
    }

    #[test]
    fn side_box_response_uses_strict_walk_half_threshold() {
        let movement = PlayerMovementConstants::default();
        let half_walk_speed = movement.max_walk_speed / 2.0;
        let mut right = PlayerMovementState {
            speed_x: half_walk_speed,
            ..PlayerMovementState::default()
        };
        let mut left = PlayerMovementState {
            speed_x: -half_walk_speed,
            ..PlayerMovementState::default()
        };

        let right_response = apply_legacy_side_box_response(
            &mut right,
            PlayerSideCollision::Right,
            true,
            0.1,
            movement,
        );
        let left_response = apply_legacy_side_box_response(
            &mut left,
            PlayerSideCollision::Left,
            true,
            0.1,
            movement,
        );

        assert_close(right.speed_x, half_walk_speed);
        assert_close(left.speed_x, -half_walk_speed);
        assert_eq!(
            right_response,
            LegacySideBoxResponse {
                box_speed_x: Some(half_walk_speed),
                suppress_default: true,
            }
        );
        assert_eq!(
            left_response,
            LegacySideBoxResponse {
                box_speed_x: Some(-half_walk_speed),
                suppress_default: true,
            }
        );
    }

    #[test]
    fn side_box_response_keeps_default_collision_when_box_is_blocked() {
        let mut state = PlayerMovementState {
            speed_x: 8.0,
            ..PlayerMovementState::default()
        };

        let response = apply_legacy_side_box_response(
            &mut state,
            PlayerSideCollision::Right,
            false,
            0.1,
            PlayerMovementConstants::default(),
        );

        assert_close(state.speed_x, 3.2);
        assert_eq!(
            response,
            LegacySideBoxResponse {
                box_speed_x: None,
                suppress_default: false,
            }
        );
    }

    #[test]
    fn right_side_pipe_entry_prefers_current_tile_pipe() {
        let state = PlayerMovementState::default();
        let tile = LegacyMapTileCoord::new(5, 7);

        assert_eq!(
            legacy_right_side_pipe_entry(
                &state,
                true,
                false,
                tile,
                Some(LegacyPipeCandidate::new(Some(3))),
                Some(LegacyPipeCandidate::new(Some(9))),
            ),
            Some(LegacyPipeEntry {
                coord: tile,
                direction: LegacyPipeDirection::Right,
                sublevel: Some(3),
            })
        );
    }

    #[test]
    fn right_side_pipe_entry_uses_below_tile_pipe_as_fallback() {
        let state = PlayerMovementState::default();
        let tile = LegacyMapTileCoord::new(5, 7);

        assert_eq!(
            legacy_right_side_pipe_entry(
                &state,
                true,
                false,
                tile,
                None,
                Some(LegacyPipeCandidate::new(None)),
            ),
            Some(LegacyPipeEntry {
                coord: LegacyMapTileCoord::new(5, 8),
                direction: LegacyPipeDirection::Right,
                sublevel: None,
            })
        );
    }

    #[test]
    fn right_side_pipe_entry_requires_grounded_right_input_or_intermission() {
        let tile = LegacyMapTileCoord::new(5, 7);
        let pipe = Some(LegacyPipeCandidate::new(Some(3)));
        let grounded = PlayerMovementState::default();
        let falling = PlayerMovementState {
            falling: true,
            ..PlayerMovementState::default()
        };
        let jumping = PlayerMovementState {
            jumping: true,
            ..PlayerMovementState::default()
        };

        assert_eq!(
            legacy_right_side_pipe_entry(&grounded, false, true, tile, pipe, None),
            Some(LegacyPipeEntry {
                coord: tile,
                direction: LegacyPipeDirection::Right,
                sublevel: Some(3),
            })
        );
        assert_eq!(
            legacy_right_side_pipe_entry(&grounded, false, false, tile, pipe, None),
            None
        );
        assert_eq!(
            legacy_right_side_pipe_entry(&falling, true, false, tile, pipe, None),
            None
        );
        assert_eq!(
            legacy_right_side_pipe_entry(&jumping, true, false, tile, pipe, None),
            None
        );
    }

    #[test]
    fn side_button_response_snaps_to_button_top_and_side_edge() {
        let button = PlayerBodyBounds::new(8.0, 4.0, 1.0, 0.25);
        let mut right_state = PlayerMovementState {
            speed_y: 2.5,
            falling: true,
            animation_state: PlayerAnimationState::Falling,
            ..PlayerMovementState::default()
        };
        let mut left_state = right_state;
        let mut right_body = PlayerBodyBounds::new(7.4, 3.8, 0.75, 0.625);
        let mut left_body = PlayerBodyBounds::new(8.4, 3.8, 0.75, 0.625);

        assert!(apply_legacy_side_button_response(
            &mut right_state,
            &mut right_body,
            button,
            PlayerSideCollision::Right,
        ));
        assert!(apply_legacy_side_button_response(
            &mut left_state,
            &mut left_body,
            button,
            PlayerSideCollision::Left,
        ));

        assert_close(right_body.y, button.y - right_body.height);
        assert_close(right_body.x, button.x - right_body.width + 0.001);
        assert_close(left_body.x, button.x + button.width - 0.001);
        assert_close(right_state.speed_y, 0.0);
        assert_close(left_state.speed_y, 0.0);
        assert!(right_state.falling);
        assert_eq!(right_state.animation_state, PlayerAnimationState::Falling);
    }

    #[test]
    fn side_button_response_preserves_upward_vertical_speed() {
        let button = PlayerBodyBounds::new(8.0, 4.0, 1.0, 0.25);
        let mut state = PlayerMovementState {
            speed_y: -2.5,
            jumping: true,
            animation_state: PlayerAnimationState::Jumping,
            ..PlayerMovementState::default()
        };
        let mut body = PlayerBodyBounds::new(7.4, 3.8, 0.75, 0.625);

        assert!(apply_legacy_side_button_response(
            &mut state,
            &mut body,
            button,
            PlayerSideCollision::Right,
        ));

        assert_close(state.speed_y, -2.5);
        assert!(state.jumping);
        assert_eq!(state.animation_state, PlayerAnimationState::Jumping);
    }

    #[test]
    fn player_hazard_floor_outcome_preserves_fire_and_generic_enemy_death_reasons() {
        assert_eq!(
            legacy_player_hazard_collision_outcome(
                LegacyPlayerHazard::Fire,
                LegacyPlayerHazardCollision::Floor,
                false,
                false,
            ),
            LegacyPlayerHazardOutcome::Dies(LegacyPlayerDeathCause::CastleFireFire)
        );
        assert_eq!(
            legacy_player_hazard_collision_outcome(
                LegacyPlayerHazard::CastleFire,
                LegacyPlayerHazardCollision::Floor,
                false,
                false,
            ),
            LegacyPlayerHazardOutcome::Dies(LegacyPlayerDeathCause::CastleFireFire)
        );
        assert_eq!(
            legacy_player_hazard_collision_outcome(
                LegacyPlayerHazard::UpFire,
                LegacyPlayerHazardCollision::Floor,
                false,
                false,
            ),
            LegacyPlayerHazardOutcome::Dies(LegacyPlayerDeathCause::EnemyFloorCollision)
        );
        assert_eq!(
            legacy_player_hazard_collision_outcome(
                LegacyPlayerHazard::Hammer,
                LegacyPlayerHazardCollision::Floor,
                false,
                false,
            ),
            LegacyPlayerHazardOutcome::Dies(LegacyPlayerDeathCause::EnemyFloorCollision)
        );
    }

    #[test]
    fn player_hazard_side_and_ceiling_outcomes_use_handler_specific_death_reasons() {
        assert_eq!(
            legacy_player_hazard_collision_outcome(
                LegacyPlayerHazard::Fire,
                LegacyPlayerHazardCollision::Side(PlayerSideCollision::Right),
                false,
                false,
            ),
            LegacyPlayerHazardOutcome::Dies(LegacyPlayerDeathCause::EnemyRightCollision)
        );
        assert_eq!(
            legacy_player_hazard_collision_outcome(
                LegacyPlayerHazard::CastleFire,
                LegacyPlayerHazardCollision::Side(PlayerSideCollision::Left),
                false,
                false,
            ),
            LegacyPlayerHazardOutcome::Dies(LegacyPlayerDeathCause::EnemyLeftCollision)
        );
        assert_eq!(
            legacy_player_hazard_collision_outcome(
                LegacyPlayerHazard::Hammer,
                LegacyPlayerHazardCollision::Ceiling,
                false,
                false,
            ),
            LegacyPlayerHazardOutcome::Dies(LegacyPlayerDeathCause::EnemyCeilingCollision)
        );
    }

    #[test]
    fn player_hazard_outcome_ignores_invincible_and_star_or_big_mario_collisions() {
        assert_eq!(
            legacy_player_hazard_collision_outcome(
                LegacyPlayerHazard::Fire,
                LegacyPlayerHazardCollision::Floor,
                true,
                false,
            ),
            LegacyPlayerHazardOutcome::Ignored
        );
        assert_eq!(
            legacy_player_hazard_collision_outcome(
                LegacyPlayerHazard::UpFire,
                LegacyPlayerHazardCollision::Side(PlayerSideCollision::Right),
                false,
                true,
            ),
            LegacyPlayerHazardOutcome::Ignored
        );
    }

    #[test]
    fn player_death_cause_legacy_labels_match_lua_die_strings() {
        assert_eq!(
            LegacyPlayerDeathCause::CastleFireFire.legacy_label(),
            "castlefirefire"
        );
        assert_eq!(
            LegacyPlayerDeathCause::EnemyFloorCollision.legacy_label(),
            "Enemy (floorcollide)"
        );
        assert_eq!(
            LegacyPlayerDeathCause::EnemyRightCollision.legacy_label(),
            "Enemy (rightcollide)"
        );
        assert_eq!(
            LegacyPlayerDeathCause::EnemyLeftCollision.legacy_label(),
            "Enemy (leftcollide)"
        );
        assert_eq!(
            LegacyPlayerDeathCause::EnemyCeilingCollision.legacy_label(),
            "Enemy (Ceilcollided)"
        );
    }

    #[test]
    fn floor_blue_gel_bounce_reverses_fast_fall_and_reapplies_frame_gravity() {
        let mut state = PlayerMovementState {
            speed_y: 90.0,
            gravity: 80.0,
            falling: false,
            jumping: false,
            animation_state: PlayerAnimationState::Idle,
            ..PlayerMovementState::default()
        };

        assert!(apply_legacy_floor_blue_gel_bounce(
            &mut state,
            false,
            0.1,
            PhysicsConstants::default(),
            BlueGelBounceConstants::default(),
        ));

        assert_close(state.speed_y, -82.0);
        assert!(state.falling);
        assert!(!state.jumping);
        assert_eq!(state.animation_state, PlayerAnimationState::Jumping);
    }

    #[test]
    fn floor_blue_gel_bounce_requires_threshold_and_no_down_input() {
        let mut slow = PlayerMovementState {
            speed_y: 80.0,
            gravity: 80.0,
            ..PlayerMovementState::default()
        };
        let mut down_held = PlayerMovementState {
            speed_y: 90.0,
            gravity: 80.0,
            ..PlayerMovementState::default()
        };

        assert!(!apply_legacy_floor_blue_gel_bounce(
            &mut slow,
            false,
            0.1,
            PhysicsConstants::default(),
            BlueGelBounceConstants::default(),
        ));
        assert!(!apply_legacy_floor_blue_gel_bounce(
            &mut down_held,
            true,
            0.1,
            PhysicsConstants::default(),
            BlueGelBounceConstants::default(),
        ));

        assert_close(slow.speed_y, 80.0);
        assert_close(down_held.speed_y, 90.0);
    }

    #[test]
    fn right_side_blue_gel_bounce_matches_legacy_asymmetric_min_call() {
        let mut state = PlayerMovementState {
            speed_x: 20.0,
            speed_y: 3.0,
            falling: true,
            ..PlayerMovementState::default()
        };

        assert!(apply_legacy_side_blue_gel_bounce(
            &mut state,
            PlayerSideCollision::Right,
            false,
            BlueGelBounceConstants::default(),
        ));

        assert_close(state.speed_x, -30.0);
        assert_close(state.speed_y, -20.0);
    }

    #[test]
    fn left_side_blue_gel_bounce_caps_positive_exit_speed() {
        let mut state = PlayerMovementState {
            speed_x: -20.0,
            speed_y: -25.0,
            jumping: true,
            ..PlayerMovementState::default()
        };

        assert!(apply_legacy_side_blue_gel_bounce(
            &mut state,
            PlayerSideCollision::Left,
            false,
            BlueGelBounceConstants::default(),
        ));

        assert_close(state.speed_x, 15.0);
        assert_close(state.speed_y, -25.0);
    }

    #[test]
    fn side_blue_gel_bounce_requires_airborne_speed_and_no_down_input() {
        let mut grounded = PlayerMovementState {
            speed_x: 3.0,
            ..PlayerMovementState::default()
        };
        let mut slow = PlayerMovementState {
            speed_x: 2.0,
            falling: true,
            ..PlayerMovementState::default()
        };
        let mut down_held = PlayerMovementState {
            speed_x: 3.0,
            falling: true,
            ..PlayerMovementState::default()
        };

        assert!(!apply_legacy_side_blue_gel_bounce(
            &mut grounded,
            PlayerSideCollision::Right,
            false,
            BlueGelBounceConstants::default(),
        ));
        assert!(!apply_legacy_side_blue_gel_bounce(
            &mut slow,
            PlayerSideCollision::Right,
            false,
            BlueGelBounceConstants::default(),
        ));
        assert!(!apply_legacy_side_blue_gel_bounce(
            &mut down_held,
            PlayerSideCollision::Right,
            true,
            BlueGelBounceConstants::default(),
        ));

        assert_close(grounded.speed_x, 3.0);
        assert_close(slow.speed_x, 2.0);
        assert_close(down_held.speed_x, 3.0);
    }

    #[test]
    fn enemy_stomp_bounce_speed_uses_current_player_gravity_and_bounce_height() {
        assert_close(
            legacy_enemy_stomp_bounce_speed(80.0, PhysicsConstants::default()),
            11.832_16,
        );
        assert_close(
            legacy_enemy_stomp_bounce_speed(20.0, PhysicsConstants::default()),
            5.916_08,
        );
    }

    #[test]
    fn enemy_stomp_bounce_sets_jump_animation_and_preserves_jump_flag() {
        let mut state = PlayerMovementState {
            speed_y: 3.0,
            gravity: 80.0,
            jumping: true,
            falling: false,
            animation_state: PlayerAnimationState::Running,
            ..PlayerMovementState::default()
        };

        apply_legacy_enemy_stomp_bounce_state(&mut state, PhysicsConstants::default());

        assert_close(state.speed_y, -11.832_16);
        assert!(state.falling);
        assert!(state.jumping);
        assert_eq!(state.animation_state, PlayerAnimationState::Jumping);
    }

    #[test]
    fn default_spring_state_matches_mario_init() {
        let state = PlayerMovementState::default();

        assert!(!state.spring);
        assert!(!state.spring_high);
        assert_close(state.spring_timer, 0.0);
    }

    #[test]
    fn hit_spring_state_latches_player_on_spring() {
        let mut state = PlayerMovementState {
            speed_y: -5.0,
            gravity: 80.0,
            spring_high: true,
            spring_timer: 1.5,
            animation_state: PlayerAnimationState::Falling,
            ..PlayerMovementState::default()
        };

        apply_legacy_hit_spring_state(&mut state);

        assert_close(state.speed_y, 0.0);
        assert!(state.spring);
        assert!(!state.spring_high);
        assert_close(state.spring_timer, 0.0);
        assert_close(state.gravity, 0.0);
        assert_eq!(state.animation_state, PlayerAnimationState::Idle);
    }

    #[test]
    fn held_spring_update_pins_x_advances_timer_and_uses_frame_table_y_offset() {
        let mut state = PlayerMovementState {
            spring: true,
            spring_timer: 0.05,
            ..PlayerMovementState::default()
        };

        let update = update_legacy_held_spring(
            &mut state,
            LegacyHeldSpringContext {
                spring_x: 4.25,
                spring_y: 12.0,
                spring_frame: 3,
                player_height: 0.75,
            },
            0.1,
            SpringConstants::default(),
        );

        assert_close(state.spring_timer, 0.15);
        assert_eq!(
            update,
            LegacyHeldSpringUpdate {
                x: 4.25,
                y: 10.3125,
                auto_release: false,
            }
        );
    }

    #[test]
    fn held_spring_update_requires_strict_threshold_crossing_for_auto_release() {
        let mut exact = PlayerMovementState {
            spring: true,
            spring_timer: 0.1,
            ..PlayerMovementState::default()
        };
        let mut crossed = exact;
        let constants = SpringConstants::default();

        let exact_update = update_legacy_held_spring(
            &mut exact,
            LegacyHeldSpringContext {
                spring_x: 2.0,
                spring_y: 8.0,
                spring_frame: 1,
                player_height: 0.75,
            },
            0.1,
            constants,
        );
        let crossed_update = update_legacy_held_spring(
            &mut crossed,
            LegacyHeldSpringContext {
                spring_x: 2.0,
                spring_y: 8.0,
                spring_frame: 2,
                player_height: 0.75,
            },
            0.1001,
            constants,
        );

        assert_close(exact.spring_timer, constants.duration);
        assert!(!exact_update.auto_release);
        assert_close(crossed.spring_timer, 0.2001);
        assert!(crossed_update.auto_release);
    }

    #[test]
    fn grab_vine_clears_ducking_even_when_portal_guard_suppresses_grab() {
        let mut state = PlayerMovementState {
            ducking: true,
            gravity: 80.0,
            speed_x: 3.0,
            speed_y: -2.0,
            animation_state: PlayerAnimationState::Running,
            ..PlayerMovementState::default()
        };

        let outcome = apply_legacy_grab_vine(
            &mut state,
            LegacyGrabVineContext {
                inside_portal: true,
                player: PlayerBodyBounds::new(4.0, 7.0, 0.75, 0.75),
                vine_coord: LegacyMapTileCoord::new(5, 8),
                vine_x: 5.0,
                vine_width: 10.0 / 16.0,
            },
        );

        assert_eq!(outcome, None);
        assert!(!state.ducking);
        assert_close(state.gravity, 80.0);
        assert_close(state.speed_x, 3.0);
        assert_close(state.speed_y, -2.0);
        assert_eq!(state.animation_state, PlayerAnimationState::Running);
    }

    #[test]
    fn grab_vine_sets_climbing_state_and_left_side_snap_when_vine_is_right_of_player() {
        let mut state = PlayerMovementState {
            ducking: true,
            gravity: 80.0,
            speed_x: 3.0,
            speed_y: -2.0,
            animation_state: PlayerAnimationState::Running,
            ..PlayerMovementState::default()
        };

        let outcome = apply_legacy_grab_vine(
            &mut state,
            LegacyGrabVineContext {
                inside_portal: false,
                player: PlayerBodyBounds::new(4.0, 7.0, 0.75, 0.75),
                vine_coord: LegacyMapTileCoord::new(5, 8),
                vine_x: 5.0,
                vine_width: 10.0 / 16.0,
            },
        );

        assert_eq!(
            outcome,
            Some(LegacyGrabVineOutcome {
                x: 4.6875,
                pointing_angle: -FRAC_PI_2,
                side: LegacyVineSide::Left,
                climb_frame: 2,
                move_timer: 0.0,
                vine_coord: LegacyMapTileCoord::new(5, 8),
            })
        );
        assert!(!state.ducking);
        assert_close(state.gravity, 0.0);
        assert_close(state.speed_x, 0.0);
        assert_close(state.speed_y, 0.0);
        assert_eq!(state.animation_state, PlayerAnimationState::Climbing);
    }

    #[test]
    fn grab_vine_uses_right_side_snap_when_vine_is_not_right_of_player() {
        let mut state = PlayerMovementState::default();

        let outcome = apply_legacy_grab_vine(
            &mut state,
            LegacyGrabVineContext {
                inside_portal: false,
                player: PlayerBodyBounds::new(5.0, 7.0, 0.75, 0.75),
                vine_coord: LegacyMapTileCoord::new(6, 8),
                vine_x: 5.0,
                vine_width: 10.0 / 16.0,
            },
        );

        assert_eq!(
            outcome,
            Some(LegacyGrabVineOutcome {
                x: 5.1875,
                pointing_angle: FRAC_PI_2,
                side: LegacyVineSide::Right,
                climb_frame: 2,
                move_timer: 0.0,
                vine_coord: LegacyMapTileCoord::new(6, 8),
            })
        );
    }

    #[test]
    fn on_vine_motion_up_advances_timer_frame_and_triggers_animation_at_threshold() {
        let mut state = PlayerMovementState::default();

        let update = update_legacy_on_vine_motion(
            &mut state,
            LegacyOnVineContext {
                y: 3.1,
                height: 0.75,
                move_timer: 0.10,
                direction: LegacyOnVineDirection::Up,
                blocking_collision: None,
            },
            LegacyVineConstants::default(),
            0.05,
        );

        assert_eq!(state.animation_state, PlayerAnimationState::Climbing);
        assert_close(update.y, 2.9395);
        assert_close(update.move_timer, 0.15);
        assert_eq!(update.climb_frame, 1);
        assert!(update.trigger_animation);
        assert_eq!(update.portal_probe_y, None);
        assert!(!update.blocked_by_solid);
    }

    #[test]
    fn on_vine_motion_down_uses_legacy_down_speed_and_frame_delay() {
        let mut state = PlayerMovementState::default();

        let update = update_legacy_on_vine_motion(
            &mut state,
            LegacyOnVineContext {
                y: 5.0,
                height: 0.75,
                move_timer: 0.04,
                direction: LegacyOnVineDirection::Down,
                blocking_collision: None,
            },
            LegacyVineConstants::default(),
            0.04,
        );

        assert_close(update.y, 5.2568);
        assert_close(update.move_timer, 0.08);
        assert_eq!(update.climb_frame, 2);
        assert!(!update.trigger_animation);
        assert_eq!(update.portal_probe_y, Some(5.2568));
        assert!(!update.blocked_by_solid);
    }

    #[test]
    fn on_vine_motion_idle_resets_timer_and_holds_neutral_frame() {
        let mut state = PlayerMovementState {
            animation_state: PlayerAnimationState::Running,
            ..PlayerMovementState::default()
        };

        let update = update_legacy_on_vine_motion(
            &mut state,
            LegacyOnVineContext {
                y: 6.0,
                height: 0.75,
                move_timer: 1.2,
                direction: LegacyOnVineDirection::Idle,
                blocking_collision: Some(PlayerVerticalBounds::new(5.5, 1.0)),
            },
            LegacyVineConstants::default(),
            0.2,
        );

        assert_eq!(state.animation_state, PlayerAnimationState::Climbing);
        assert_close(update.y, 6.0);
        assert_close(update.move_timer, 0.0);
        assert_eq!(update.climb_frame, 2);
        assert!(!update.trigger_animation);
        assert_eq!(update.portal_probe_y, None);
        assert!(!update.blocked_by_solid);
    }

    #[test]
    fn on_vine_motion_up_clamps_below_first_tile_or_portalwall_collision() {
        let mut state = PlayerMovementState::default();

        let update = update_legacy_on_vine_motion(
            &mut state,
            LegacyOnVineContext {
                y: 5.0,
                height: 0.75,
                move_timer: 0.12,
                direction: LegacyOnVineDirection::Up,
                blocking_collision: Some(PlayerVerticalBounds::new(4.0, 1.0)),
            },
            LegacyVineConstants::default(),
            0.05,
        );

        assert_close(update.y, 5.0);
        assert_close(update.move_timer, 0.17);
        assert_eq!(update.climb_frame, 2);
        assert_eq!(update.portal_probe_y, None);
        assert!(update.blocked_by_solid);
    }

    #[test]
    fn on_vine_motion_down_probes_portal_before_clamping_above_solid() {
        let mut state = PlayerMovementState::default();

        let update = update_legacy_on_vine_motion(
            &mut state,
            LegacyOnVineContext {
                y: 5.0,
                height: 0.75,
                move_timer: 0.02,
                direction: LegacyOnVineDirection::Down,
                blocking_collision: Some(PlayerVerticalBounds::new(5.5, 1.0)),
            },
            LegacyVineConstants::default(),
            0.04,
        );

        assert_close(
            update.portal_probe_y.expect("down-vine portal probe"),
            5.2568,
        );
        assert_close(update.y, 4.75);
        assert_close(update.move_timer, 0.06);
        assert_eq!(update.climb_frame, 2);
        assert!(update.blocked_by_solid);
    }

    #[test]
    fn on_vine_horizontal_portal_success_applies_checkportalhor_transit_outputs() {
        let mut state = PlayerMovementState {
            speed_x: 1.25,
            jumping: true,
            animation_direction: HorizontalDirection::Right,
            ..PlayerMovementState::default()
        };

        let outcome = apply_legacy_on_vine_horizontal_portal(
            &mut state,
            LegacyOnVineHorizontalPortalContext {
                body: PlayerBodyBounds::new(4.0, 5.0, 0.75, 0.75),
                next_y: 6.0,
                portals: LegacyOnVineHorizontalPortalPair::new(
                    LegacyPortalEndpoint::new(5.0, 6.0, Facing::Down),
                    LegacyPortalEndpoint::new(10.0, 3.0, Facing::Right),
                ),
                rotation: 0.25,
                exit_blocked: false,
                frame_dt: 1.0 / 60.0,
            },
        )
        .expect("down-vine portal crossing");

        assert_close(outcome.x, 10.25);
        assert_close(outcome.y, 3.0);
        assert_close(outcome.speed_x, 0.0);
        assert_close(outcome.speed_y, 1.25);
        assert_close(outcome.rotation, 0.25 + FRAC_PI_2);
        assert_eq!(outcome.entry_facing, Facing::Down);
        assert_eq!(outcome.exit_facing, Facing::Right);
        assert_eq!(outcome.portaled_exit_facing, Facing::Right);
        assert!(!outcome.exit_blocked);
        assert_eq!(state.animation_direction, HorizontalDirection::Right);
        assert!(!state.jumping);
        assert!(state.falling);
    }

    #[test]
    fn on_vine_horizontal_portal_blocked_exit_uses_legacy_vertical_bounce() {
        let mut state = PlayerMovementState {
            speed_x: 1.25,
            speed_y: 0.0,
            jumping: true,
            falling: false,
            ..PlayerMovementState::default()
        };

        let outcome = apply_legacy_on_vine_horizontal_portal(
            &mut state,
            LegacyOnVineHorizontalPortalContext {
                body: PlayerBodyBounds::new(4.0, 5.0, 0.75, 0.75),
                next_y: 6.0,
                portals: LegacyOnVineHorizontalPortalPair::new(
                    LegacyPortalEndpoint::new(5.0, 6.0, Facing::Down),
                    LegacyPortalEndpoint::new(10.0, 3.0, Facing::Right),
                ),
                rotation: 0.25,
                exit_blocked: true,
                frame_dt: 1.0 / 60.0,
            },
        )
        .expect("blocked down-vine portal crossing");

        assert_close(outcome.x, 4.0);
        assert_close(outcome.y, 5.0);
        assert_close(outcome.speed_x, 1.25);
        assert_close(outcome.speed_y, -2.0);
        assert_close(outcome.rotation, 0.25);
        assert!(outcome.exit_blocked);
        assert_eq!(outcome.portaled_exit_facing, Facing::Right);
        assert_close(state.speed_y, -2.0);
        assert!(!state.jumping);
        assert!(state.falling);
    }

    #[test]
    fn on_vine_horizontal_portal_opposite_vertical_pair_preserves_jump_flags() {
        let mut state = PlayerMovementState {
            speed_x: 1.25,
            jumping: true,
            falling: false,
            ..PlayerMovementState::default()
        };

        let outcome = apply_legacy_on_vine_horizontal_portal(
            &mut state,
            LegacyOnVineHorizontalPortalContext {
                body: PlayerBodyBounds::new(4.0, 5.0, 0.75, 0.75),
                next_y: 6.0,
                portals: LegacyOnVineHorizontalPortalPair::new(
                    LegacyPortalEndpoint::new(5.0, 6.0, Facing::Down),
                    LegacyPortalEndpoint::new(8.0, 2.0, Facing::Up),
                ),
                rotation: 0.25,
                exit_blocked: false,
                frame_dt: 1.0 / 60.0,
            },
        )
        .expect("vertical-opposite down-vine portal crossing");

        assert_close(outcome.x, 8.0);
        assert_close(outcome.y, 0.0);
        assert_close(outcome.speed_x, 1.25);
        assert_close(outcome.speed_y, 0.0);
        assert!(outcome.jumping);
        assert!(!outcome.falling);
        assert!(state.jumping);
        assert!(!state.falling);
    }

    #[test]
    fn on_vine_horizontal_portal_respects_entry_direction_speed_gate() {
        let mut state = PlayerMovementState {
            speed_y: 3.0,
            jumping: true,
            falling: false,
            ..PlayerMovementState::default()
        };

        let outcome = apply_legacy_on_vine_horizontal_portal(
            &mut state,
            LegacyOnVineHorizontalPortalContext {
                body: PlayerBodyBounds::new(4.0, 5.0, 0.75, 0.75),
                next_y: 6.0,
                portals: LegacyOnVineHorizontalPortalPair::new(
                    LegacyPortalEndpoint::new(5.0, 6.0, Facing::Down),
                    LegacyPortalEndpoint::new(8.0, 2.0, Facing::Up),
                ),
                rotation: 0.25,
                exit_blocked: false,
                frame_dt: 1.0 / 60.0,
            },
        );

        assert_eq!(outcome, None);
        assert_close(state.speed_y, 3.0);
        assert!(state.jumping);
        assert!(!state.falling);
    }

    #[test]
    fn drop_vine_applies_right_side_offset_and_detach_outputs() {
        let mut state = PlayerMovementState {
            gravity: 0.0,
            falling: false,
            animation_state: PlayerAnimationState::Climbing,
            ..PlayerMovementState::default()
        };

        let outcome = apply_legacy_drop_vine(
            &mut state,
            5.0,
            LegacyDropVineContext {
                side: LegacyVineSide::Right,
            },
            PhysicsConstants::default(),
            LegacyVineConstants::default(),
        );

        assert_eq!(state.animation_state, PlayerAnimationState::Falling);
        assert_close(state.gravity, 80.0);
        assert!(!state.falling);
        assert_eq!(
            outcome,
            super::LegacyDropVineOutcome {
                x: 5.4375,
                vine_active: false,
                vine_mask_enabled: false,
            }
        );
    }

    #[test]
    fn drop_vine_uses_left_side_offset_and_preserves_existing_falling_flag() {
        let mut state = PlayerMovementState {
            gravity: 0.0,
            falling: true,
            animation_state: PlayerAnimationState::Climbing,
            ..PlayerMovementState::default()
        };

        let outcome = apply_legacy_drop_vine(
            &mut state,
            5.0,
            LegacyDropVineContext {
                side: LegacyVineSide::Left,
            },
            PhysicsConstants::default(),
            LegacyVineConstants::default(),
        );

        assert_eq!(state.animation_state, PlayerAnimationState::Falling);
        assert_close(state.gravity, 80.0);
        assert!(state.falling);
        assert_eq!(
            outcome,
            super::LegacyDropVineOutcome {
                x: 4.5625,
                vine_active: false,
                vine_mask_enabled: false,
            }
        );
    }

    #[test]
    fn on_vine_attachment_loss_delegates_to_drop_vine_when_overlap_is_empty() {
        let mut state = PlayerMovementState {
            gravity: 0.0,
            falling: false,
            animation_state: PlayerAnimationState::Climbing,
            ..PlayerMovementState::default()
        };

        let outcome = apply_legacy_on_vine_attachment_loss(
            &mut state,
            5.0,
            LegacyOnVineAttachmentLossContext {
                side: LegacyVineSide::Left,
                has_vine_overlap: false,
            },
            PhysicsConstants::default(),
            LegacyVineConstants::default(),
        );

        assert_eq!(state.animation_state, PlayerAnimationState::Falling);
        assert_close(state.gravity, 80.0);
        assert!(!state.falling);
        assert_eq!(
            outcome,
            Some(super::LegacyDropVineOutcome {
                x: 4.5625,
                vine_active: false,
                vine_mask_enabled: false,
            })
        );
    }

    #[test]
    fn on_vine_attachment_loss_is_suppressed_while_the_player_still_overlaps_the_vine() {
        let mut state = PlayerMovementState {
            gravity: 0.0,
            animation_state: PlayerAnimationState::Climbing,
            ..PlayerMovementState::default()
        };

        let outcome = apply_legacy_on_vine_attachment_loss(
            &mut state,
            5.0,
            LegacyOnVineAttachmentLossContext {
                side: LegacyVineSide::Left,
                has_vine_overlap: true,
            },
            PhysicsConstants::default(),
            LegacyVineConstants::default(),
        );

        assert_eq!(outcome, None);
        assert_eq!(state.animation_state, PlayerAnimationState::Climbing);
        assert_close(state.gravity, 0.0);
    }

    #[test]
    fn spring_high_request_only_applies_while_on_spring() {
        let mut on_spring = PlayerMovementState {
            spring: true,
            ..PlayerMovementState::default()
        };
        let mut not_on_spring = PlayerMovementState::default();

        apply_legacy_spring_high_request(&mut on_spring);
        apply_legacy_spring_high_request(&mut not_on_spring);

        assert!(on_spring.spring_high);
        assert!(!not_on_spring.spring_high);
    }

    #[test]
    fn leave_spring_state_uses_normal_or_high_force_and_restores_gravity() {
        let mut normal = PlayerMovementState {
            spring: true,
            spring_high: false,
            gravity: 0.0,
            ..PlayerMovementState::default()
        };
        let mut high = PlayerMovementState {
            spring: true,
            spring_high: true,
            gravity: 0.0,
            ..PlayerMovementState::default()
        };

        apply_legacy_leave_spring_state(
            &mut normal,
            PhysicsConstants::default(),
            SpringConstants::default(),
        );
        apply_legacy_leave_spring_state(
            &mut high,
            PhysicsConstants::default(),
            SpringConstants::default(),
        );

        assert_close(normal.speed_y, -24.0);
        assert_close(high.speed_y, -41.0);
        assert_eq!(normal.animation_state, PlayerAnimationState::Falling);
        assert_close(normal.gravity, 80.0);
        assert!(normal.falling);
        assert!(!normal.spring);
    }

    #[test]
    fn leave_spring_y_matches_legacy_offset_formula() {
        assert_close(
            legacy_leave_spring_y(12.0, 0.75, SpringConstants::default()),
            9.3125,
        );
    }

    #[test]
    fn faithplate_state_sets_jumping_animation_and_falling_flag() {
        let mut state = PlayerMovementState {
            animation_state: PlayerAnimationState::Idle,
            falling: false,
            jumping: false,
            ..PlayerMovementState::default()
        };

        apply_legacy_faithplate_state(&mut state);

        assert_eq!(state.animation_state, PlayerAnimationState::Jumping);
        assert!(state.falling);
        assert!(!state.jumping);
    }

    #[test]
    fn default_animation_frames_match_mario_init() {
        let state = PlayerMovementState::default();

        assert_eq!(state.run_frame, 3);
        assert_eq!(state.swim_frame, 1);
        assert_close(state.run_animation_progress, 1.0);
        assert_close(state.swim_animation_progress, 1.0);
    }

    #[test]
    fn normal_running_animation_advances_from_current_speed() {
        let mut state = PlayerMovementState {
            speed_x: 6.0,
            animation_state: PlayerAnimationState::Running,
            ..PlayerMovementState::default()
        };

        advance_legacy_player_animation(&mut state, 0.1, PlayerAnimationConstants::default());
        assert_close(state.run_animation_progress, 3.0);
        assert_eq!(state.run_frame, 3);

        advance_legacy_player_animation(&mut state, 0.1, PlayerAnimationConstants::default());
        assert_close(state.run_animation_progress, 2.0);
        assert_eq!(state.run_frame, 2);
    }

    #[test]
    fn normal_animation_does_not_advance_when_not_running() {
        let mut state = PlayerMovementState {
            speed_x: 6.0,
            animation_state: PlayerAnimationState::Sliding,
            ..PlayerMovementState::default()
        };

        advance_legacy_player_animation(&mut state, 0.5, PlayerAnimationConstants::default());

        assert_close(state.run_animation_progress, 1.0);
        assert_eq!(state.run_frame, 3);
    }

    #[test]
    fn underwater_grounded_running_uses_run_animation() {
        let mut state = PlayerMovementState {
            speed_x: 1.0,
            animation_state: PlayerAnimationState::Running,
            ..PlayerMovementState::default()
        };

        advance_legacy_underwater_animation(&mut state, 0.2, PlayerAnimationConstants::default());

        assert_close(state.run_animation_progress, 3.0);
        assert_eq!(state.run_frame, 3);
        assert_close(state.swim_animation_progress, 1.0);
    }

    #[test]
    fn underwater_airborne_jumping_uses_legacy_run_animation_speed_for_swim_frames() {
        let mut state = PlayerMovementState {
            jumping: true,
            animation_state: PlayerAnimationState::Jumping,
            ..PlayerMovementState::default()
        };
        let constants = PlayerAnimationConstants {
            run_animation_speed: 2.0,
            swim_animation_speed: 99.0,
        };

        advance_legacy_underwater_animation(&mut state, 0.5, constants);

        assert_close(state.swim_animation_progress, 2.0);
        assert_eq!(state.swim_frame, 2);
        assert_close(state.run_animation_progress, 1.0);
    }

    #[test]
    fn underwater_swim_animation_wraps_between_one_and_two() {
        let mut state = PlayerMovementState {
            falling: true,
            animation_state: PlayerAnimationState::Falling,
            swim_animation_progress: 2.5,
            ..PlayerMovementState::default()
        };

        advance_legacy_underwater_animation(&mut state, 0.1, PlayerAnimationConstants::default());

        assert_close(state.swim_animation_progress, 1.5);
        assert_eq!(state.swim_frame, 1);
    }

    #[test]
    fn underwater_airborne_running_does_not_advance_either_animation() {
        let mut state = PlayerMovementState {
            falling: true,
            animation_state: PlayerAnimationState::Running,
            ..PlayerMovementState::default()
        };

        advance_legacy_underwater_animation(&mut state, 0.2, PlayerAnimationConstants::default());

        assert_close(state.run_animation_progress, 1.0);
        assert_close(state.swim_animation_progress, 1.0);
    }

    #[test]
    fn walking_right_on_ground_uses_walk_acceleration() {
        let mut state = PlayerMovementState::default();

        apply_legacy_player_movement(
            &mut state,
            PlayerMovementInput::new(false, true, false),
            0.1,
            PlayerMovementConstants::default(),
        );

        assert_close(state.speed_x, 0.8);
        assert_eq!(state.animation_state, PlayerAnimationState::Running);
        assert_eq!(state.animation_direction, HorizontalDirection::Right);
    }

    #[test]
    fn running_right_on_ground_caps_at_run_speed() {
        let mut state = PlayerMovementState {
            speed_x: 8.8,
            ..PlayerMovementState::default()
        };

        apply_legacy_player_movement(
            &mut state,
            PlayerMovementInput::new(false, true, true),
            0.1,
            PlayerMovementConstants::default(),
        );

        assert_close(state.speed_x, 9.0);
        assert_eq!(state.animation_state, PlayerAnimationState::Running);
    }

    #[test]
    fn right_key_wins_when_both_directions_are_pressed() {
        let mut state = PlayerMovementState::default();

        apply_legacy_player_movement(
            &mut state,
            PlayerMovementInput::new(true, true, false),
            0.1,
            PlayerMovementConstants::default(),
        );

        assert_close(state.speed_x, 0.8);
        assert_eq!(state.animation_direction, HorizontalDirection::Right);
    }

    #[test]
    fn running_turnaround_preserves_legacy_superfriction_sign_quirk() {
        let mut state = PlayerMovementState {
            speed_x: -12.0,
            ..PlayerMovementState::default()
        };

        apply_legacy_player_movement(
            &mut state,
            PlayerMovementInput::new(false, true, true),
            0.1,
            PlayerMovementConstants::default(),
        );

        assert_close(state.speed_x, -9.0);
        assert_eq!(state.animation_state, PlayerAnimationState::Sliding);
        assert_eq!(state.animation_direction, HorizontalDirection::Right);
    }

    #[test]
    fn walking_turnaround_uses_superfriction_past_run_speed() {
        let mut state = PlayerMovementState {
            speed_x: -12.0,
            ..PlayerMovementState::default()
        };

        apply_legacy_player_movement(
            &mut state,
            PlayerMovementInput::new(false, true, false),
            0.1,
            PlayerMovementConstants::default(),
        );

        assert_close(state.speed_x, -0.4);
        assert_eq!(state.animation_state, PlayerAnimationState::Sliding);
    }

    #[test]
    fn air_running_from_below_walk_speed_sticks_at_walk_threshold() {
        let mut state = PlayerMovementState {
            speed_x: 6.3,
            falling: true,
            animation_state: PlayerAnimationState::Falling,
            ..PlayerMovementState::default()
        };

        apply_legacy_player_movement(
            &mut state,
            PlayerMovementInput::new(false, true, true),
            0.1,
            PlayerMovementConstants::default(),
        );
        apply_legacy_player_movement(
            &mut state,
            PlayerMovementInput::new(false, true, true),
            0.1,
            PlayerMovementConstants::default(),
        );

        assert_close(state.speed_x, 6.4);
    }

    #[test]
    fn air_running_above_walk_speed_can_continue_toward_run_speed() {
        let mut state = PlayerMovementState {
            speed_x: 7.0,
            falling: true,
            animation_state: PlayerAnimationState::Falling,
            ..PlayerMovementState::default()
        };

        apply_legacy_player_movement(
            &mut state,
            PlayerMovementInput::new(false, true, true),
            0.1,
            PlayerMovementConstants::default(),
        );

        assert_close(state.speed_x, 8.6);
    }

    #[test]
    fn run_key_no_direction_uses_min_speed_deadzone() {
        let mut running_state = PlayerMovementState {
            speed_x: 0.8,
            ..PlayerMovementState::default()
        };
        let mut walking_state = running_state;

        apply_legacy_player_movement(
            &mut running_state,
            PlayerMovementInput::new(false, false, true),
            0.01,
            PlayerMovementConstants::default(),
        );
        apply_legacy_player_movement(
            &mut walking_state,
            PlayerMovementInput::new(false, false, false),
            0.01,
            PlayerMovementConstants::default(),
        );

        assert_close(running_state.speed_x, 0.0);
        assert_close(walking_state.speed_x, 0.66);
        assert_eq!(running_state.animation_state, PlayerAnimationState::Idle);
    }

    #[test]
    fn ducking_on_ground_applies_no_movement_friction() {
        let mut state = PlayerMovementState {
            speed_x: 2.0,
            ducking: true,
            animation_state: PlayerAnimationState::Running,
            ..PlayerMovementState::default()
        };

        apply_legacy_player_movement(
            &mut state,
            PlayerMovementInput::new(false, true, false),
            0.1,
            PlayerMovementConstants::default(),
        );

        assert_close(state.speed_x, 0.6);
        assert_eq!(state.animation_state, PlayerAnimationState::Running);
    }

    #[test]
    fn jump_velocity_scales_with_horizontal_speed_and_caps() {
        let physics = PhysicsConstants::default();
        let movement = PlayerMovementConstants::default();

        assert_close(legacy_jump_velocity(0.0, physics, movement), -16.0);
        assert_close(legacy_jump_velocity(20.0, physics, movement), -17.9);
    }

    #[test]
    fn jump_sets_vertical_speed_unless_falling() {
        let physics = PhysicsConstants::default();
        let movement = PlayerMovementConstants::default();
        let mut state = PlayerMovementState {
            speed_x: movement.max_run_speed,
            ..PlayerMovementState::default()
        };

        assert!(try_legacy_jump(&mut state, physics, movement));
        assert_close(state.speed_y, -17.9);
        assert!(state.jumping);
        assert_eq!(state.animation_state, PlayerAnimationState::Jumping);

        let mut falling = PlayerMovementState {
            falling: true,
            speed_y: 4.0,
            ..PlayerMovementState::default()
        };
        assert!(!try_legacy_jump(&mut falling, physics, movement));
        assert_close(falling.speed_y, 4.0);
    }

    #[test]
    fn stop_jump_switches_to_falling() {
        let mut state = PlayerMovementState {
            jumping: true,
            ..PlayerMovementState::default()
        };

        stop_legacy_jump(&mut state);

        assert!(!state.jumping);
        assert!(state.falling);
    }

    #[test]
    fn legacy_top_ground_probe_matches_mari0_grid_math() {
        let state = PlayerMovementState::default();
        let body = PlayerBodyBounds::new(4.25, 7.25, 0.75, 0.75);

        let probe = legacy_top_ground_probe(&state, body, LegacyMapBounds::mari0(20));

        assert_eq!(probe, Some(LegacyMapTileCoord::new(5, 9)));
    }

    #[test]
    fn legacy_top_ground_probe_requires_grounded_grid_aligned_in_map_position() {
        let grounded = PlayerMovementState::default();
        let falling = PlayerMovementState {
            falling: true,
            ..PlayerMovementState::default()
        };

        assert_eq!(
            legacy_top_ground_probe(
                &falling,
                PlayerBodyBounds::new(4.25, 7.25, 0.75, 0.75),
                LegacyMapBounds::mari0(20),
            ),
            None,
        );
        assert_eq!(
            legacy_top_ground_probe(
                &grounded,
                PlayerBodyBounds::new(4.25, 7.2, 0.75, 0.75),
                LegacyMapBounds::mari0(20),
            ),
            None,
        );
        assert_eq!(
            legacy_top_ground_probe(
                &grounded,
                PlayerBodyBounds::new(21.25, 7.25, 0.75, 0.75),
                LegacyMapBounds::mari0(20),
            ),
            None,
        );
    }

    #[test]
    fn orange_top_gel_overrides_only_ground_movement_constants() {
        let constants = legacy_surface_movement_constants(
            PlayerMovementConstants::default(),
            OrangeGelMovementConstants::default(),
            Some(LegacyGelKind::Orange),
        );

        assert_close(constants.max_run_speed, 50.0);
        assert_close(constants.max_walk_speed, 25.0);
        assert_close(constants.run_acceleration, 25.0);
        assert_close(constants.walk_acceleration, 12.5);
        assert_close(
            constants.air_run_acceleration,
            PlayerMovementConstants::default().air_run_acceleration,
        );

        assert_eq!(
            legacy_surface_movement_constants(
                PlayerMovementConstants::default(),
                OrangeGelMovementConstants::default(),
                Some(LegacyGelKind::Blue),
            ),
            PlayerMovementConstants::default(),
        );
    }

    #[test]
    fn orange_gel_surface_query_applies_boosted_ground_acceleration() {
        let mut state = PlayerMovementState::default();
        let body = PlayerBodyBounds::new(4.25, 7.25, 0.75, 0.75);

        apply_legacy_player_movement_with_surface_query(
            &mut state,
            PlayerMovementInput::new(false, true, true),
            0.1,
            LegacySurfaceMovementContext::new(
                PlayerMovementConstants::default(),
                OrangeGelMovementConstants::default(),
                body,
                LegacyMapBounds::mari0(20),
            ),
            |coord| {
                assert_eq!(coord, LegacyMapTileCoord::new(5, 9));
                Some(LegacyGelKind::Orange)
            },
        );

        assert_close(state.speed_x, 2.5);
        assert_eq!(state.animation_state, PlayerAnimationState::Running);
    }

    #[test]
    fn orange_gel_surface_query_is_ignored_when_airborne() {
        let mut state = PlayerMovementState {
            falling: true,
            animation_state: PlayerAnimationState::Falling,
            ..PlayerMovementState::default()
        };
        let body = PlayerBodyBounds::new(4.25, 7.25, 0.75, 0.75);

        apply_legacy_player_movement_with_surface_query(
            &mut state,
            PlayerMovementInput::new(false, true, true),
            0.1,
            LegacySurfaceMovementContext::new(
                PlayerMovementConstants::default(),
                OrangeGelMovementConstants::default(),
                body,
                LegacyMapBounds::mari0(20),
            ),
            |_| panic!("airborne players should not query ground gel"),
        );

        assert_close(state.speed_x, 1.6);
        assert_eq!(state.animation_state, PlayerAnimationState::Falling);
    }

    #[test]
    fn underwater_ground_right_uses_underwater_walk_cap() {
        let mut state = PlayerMovementState {
            speed_x: 3.4,
            ..PlayerMovementState::default()
        };

        apply_legacy_underwater_movement(
            &mut state,
            PlayerMovementInput::new(false, true, false),
            0.1,
            UnderwaterMovementConstants::default(),
            None,
        );

        assert_close(state.speed_x, 3.6);
        assert_eq!(state.animation_state, PlayerAnimationState::Running);
        assert_eq!(state.animation_direction, HorizontalDirection::Right);
    }

    #[test]
    fn underwater_movement_ignores_run_input() {
        let mut walking = PlayerMovementState::default();
        let mut running = PlayerMovementState::default();

        apply_legacy_underwater_movement(
            &mut walking,
            PlayerMovementInput::new(false, true, false),
            0.1,
            UnderwaterMovementConstants::default(),
            None,
        );
        apply_legacy_underwater_movement(
            &mut running,
            PlayerMovementInput::new(false, true, true),
            0.1,
            UnderwaterMovementConstants::default(),
            None,
        );

        assert_close(walking.speed_x, 0.8);
        assert_close(running.speed_x, walking.speed_x);
    }

    #[test]
    fn underwater_air_movement_uses_air_walk_cap() {
        let mut state = PlayerMovementState {
            speed_x: 4.9,
            falling: true,
            animation_state: PlayerAnimationState::Falling,
            ..PlayerMovementState::default()
        };

        apply_legacy_underwater_movement(
            &mut state,
            PlayerMovementInput::new(false, true, false),
            0.1,
            UnderwaterMovementConstants::default(),
            None,
        );

        assert_close(state.speed_x, 5.0);
        assert_eq!(state.animation_state, PlayerAnimationState::Falling);
    }

    #[test]
    fn underwater_no_direction_applies_ground_friction() {
        let mut state = PlayerMovementState {
            speed_x: 0.5,
            animation_state: PlayerAnimationState::Running,
            ..PlayerMovementState::default()
        };

        apply_legacy_underwater_movement(
            &mut state,
            PlayerMovementInput::new(false, false, false),
            0.1,
            UnderwaterMovementConstants::default(),
            None,
        );

        assert_close(state.speed_x, 0.0);
        assert_eq!(state.run_frame, 1);
        assert_eq!(state.animation_state, PlayerAnimationState::Idle);
    }

    #[test]
    fn underwater_direction_while_ducking_does_not_apply_no_movement_friction() {
        let mut state = PlayerMovementState {
            speed_x: 2.0,
            ducking: true,
            animation_state: PlayerAnimationState::Running,
            ..PlayerMovementState::default()
        };

        apply_legacy_underwater_movement(
            &mut state,
            PlayerMovementInput::new(false, true, false),
            0.1,
            UnderwaterMovementConstants::default(),
            None,
        );

        assert_close(state.speed_x, 2.0);
        assert_eq!(state.animation_state, PlayerAnimationState::Running);
    }

    #[test]
    fn underwater_pushes_player_down_above_max_height() {
        let mut high_state = PlayerMovementState {
            speed_y: -1.0,
            ..PlayerMovementState::default()
        };
        let mut threshold_state = high_state;

        apply_legacy_underwater_movement(
            &mut high_state,
            PlayerMovementInput::default(),
            0.1,
            UnderwaterMovementConstants::default(),
            Some(PlayerVerticalBounds::new(1.0, 1.0)),
        );
        apply_legacy_underwater_movement(
            &mut threshold_state,
            PlayerMovementInput::default(),
            0.1,
            UnderwaterMovementConstants::default(),
            Some(PlayerVerticalBounds::new(1.5, 1.0)),
        );

        assert_close(high_state.speed_y, 3.0);
        assert_close(threshold_state.speed_y, -1.0);
    }

    #[test]
    fn underwater_jump_clears_ducking_and_does_not_require_grounded_state() {
        let mut state = PlayerMovementState {
            speed_x: 4.0,
            speed_y: 2.0,
            falling: true,
            ducking: true,
            ..PlayerMovementState::default()
        };

        apply_legacy_underwater_jump(
            &mut state,
            UnderwaterMovementConstants::default(),
            PlayerMovementConstants::default(),
        );

        assert_close(state.speed_y, -5.9);
        assert!(state.jumping);
        assert!(state.falling);
        assert!(!state.ducking);
        assert_eq!(state.animation_state, PlayerAnimationState::Jumping);
    }

    #[test]
    fn underwater_jump_velocity_uses_normal_max_run_speed_for_scaling() {
        let underwater = UnderwaterMovementConstants {
            jump_force_add: 1.8,
            ..UnderwaterMovementConstants::default()
        };

        assert_close(
            legacy_underwater_jump_velocity(9.0, underwater, PlayerMovementConstants::default()),
            -7.7,
        );
    }
}
