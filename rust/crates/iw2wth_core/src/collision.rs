//! Collision helpers ported from `physics.lua` and `game.lua`.

use crate::math::{Aabb, Vec2};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CollisionBody {
    pub bounds: Aabb,
    pub velocity: Vec2,
    pub gravity: f32,
}

impl CollisionBody {
    #[must_use]
    pub fn new(x: f32, y: f32, width: f32, height: f32, speed_x: f32, speed_y: f32) -> Self {
        Self {
            bounds: Aabb::from_xywh(x, y, width, height),
            velocity: Vec2::new(speed_x, speed_y),
            gravity: 80.0,
        }
    }

    #[must_use]
    pub fn with_gravity(mut self, gravity: f32) -> Self {
        self.gravity = gravity;
        self
    }

    #[must_use]
    fn future_bounds(self, dt: f32) -> Aabb {
        self.bounds.translated(self.velocity * dt)
    }

    #[must_use]
    fn horizontal_future_bounds(self, dt: f32) -> Aabb {
        self.bounds.translated(Vec2::new(self.velocity.x * dt, 0.0))
    }

    #[must_use]
    fn vertical_future_bounds(self, dt: f32) -> Aabb {
        self.bounds.translated(Vec2::new(0.0, self.velocity.y * dt))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CollisionKind {
    None,
    Passive,
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyCollisionHandlerResult {
    ApplyDefault,
    SuppressDefault,
}

impl LegacyCollisionHandlerResult {
    #[must_use]
    pub const fn applies_default(self) -> bool {
        matches!(self, Self::ApplyDefault)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyCollisionActor {
    pub bounds: Aabb,
    pub velocity: Vec2,
}

impl LegacyCollisionActor {
    #[must_use]
    pub fn new(x: f32, y: f32, width: f32, height: f32, speed_x: f32, speed_y: f32) -> Self {
        Self {
            bounds: Aabb::from_xywh(x, y, width, height),
            velocity: Vec2::new(speed_x, speed_y),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyCollisionTarget {
    pub bounds: Aabb,
    pub velocity: Option<Vec2>,
}

impl LegacyCollisionTarget {
    #[must_use]
    pub fn new(x: f32, y: f32, width: f32, height: f32, velocity: Option<Vec2>) -> Self {
        Self {
            bounds: Aabb::from_xywh(x, y, width, height),
            velocity,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyCollisionResponse {
    pub moving: LegacyCollisionActor,
    pub target_velocity: Option<Vec2>,
    pub resolved: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyPassiveCollisionResponse {
    pub moving: LegacyCollisionActor,
    pub target_velocity: Option<Vec2>,
    pub resolved: bool,
    pub moving_passive_called: bool,
    pub target_passive_called: bool,
}

#[must_use]
pub fn lua_aabb_overlap(a: Aabb, b: Aabb) -> bool {
    a.intersects(b)
}

#[must_use]
pub fn in_range(value: f32, mut a: f32, mut b: f32, include_edges: bool) -> bool {
    if a > b {
        core::mem::swap(&mut a, &mut b);
    }

    if include_edges {
        value >= a && value <= b
    } else {
        value > a && value < b
    }
}

#[must_use]
pub fn collision_kind(
    moving: CollisionBody,
    target: Aabb,
    dt: f32,
    passed_portal: bool,
) -> CollisionKind {
    if !broad_phase_near(moving.bounds, target) {
        return CollisionKind::None;
    }

    if !passed_portal && moving.bounds.intersects(target) {
        return CollisionKind::Passive;
    }

    if !moving.future_bounds(dt).intersects(target) {
        return CollisionKind::None;
    }

    if moving.horizontal_future_bounds(dt).intersects(target) {
        return CollisionKind::Horizontal;
    }

    if moving.vertical_future_bounds(dt).intersects(target) {
        return CollisionKind::Vertical;
    }

    if (moving.velocity.y - moving.gravity * dt).abs() < moving.velocity.x.abs() {
        CollisionKind::Vertical
    } else {
        CollisionKind::Horizontal
    }
}

#[must_use]
pub fn legacy_horizontal_collision_response(
    moving: LegacyCollisionActor,
    target: LegacyCollisionTarget,
    moving_handler: LegacyCollisionHandlerResult,
    target_handler: LegacyCollisionHandlerResult,
) -> LegacyCollisionResponse {
    let mut response = LegacyCollisionResponse {
        moving,
        target_velocity: target.velocity,
        resolved: false,
    };

    if moving.velocity.x < 0.0 {
        if target_handler.applies_default() {
            zero_target_x_if(&mut response.target_velocity, |speed_x| speed_x > 0.0);
        }

        if moving_handler.applies_default() {
            if response.moving.velocity.x < 0.0 {
                response.moving.velocity.x = 0.0;
            }
            response.moving.bounds = with_min_x(response.moving.bounds, target.bounds.max.x);
            response.resolved = true;
        }
    } else {
        if target_handler.applies_default() {
            zero_target_x_if(&mut response.target_velocity, |speed_x| speed_x < 0.0);
        }

        if moving_handler.applies_default() {
            if response.moving.velocity.x > 0.0 {
                response.moving.velocity.x = 0.0;
            }
            response.moving.bounds = with_min_x(
                response.moving.bounds,
                target.bounds.min.x - moving.bounds.width(),
            );
            response.resolved = true;
        }
    }

    response
}

#[must_use]
pub fn legacy_passive_collision_response(
    moving: LegacyCollisionActor,
    target: LegacyCollisionTarget,
    moving_passive_handler: bool,
    target_passive_handler: bool,
    floor_handler: Option<LegacyCollisionHandlerResult>,
) -> LegacyPassiveCollisionResponse {
    let mut response = LegacyPassiveCollisionResponse {
        moving,
        target_velocity: target.velocity,
        resolved: false,
        moving_passive_called: false,
        target_passive_called: false,
    };

    if moving_passive_handler {
        response.moving_passive_called = true;
        response.target_passive_called = target_passive_handler;
        return response;
    }

    if floor_handler.is_none_or(LegacyCollisionHandlerResult::applies_default) {
        if response.moving.velocity.y > 0.0 {
            response.moving.velocity.y = 0.0;
        }
        response.moving.bounds = with_min_y(
            response.moving.bounds,
            target.bounds.min.y - response.moving.bounds.height(),
        );
        response.resolved = true;
    }

    response
}

#[must_use]
pub fn legacy_vertical_collision_response(
    moving: LegacyCollisionActor,
    target: LegacyCollisionTarget,
    moving_handler: LegacyCollisionHandlerResult,
    target_handler: LegacyCollisionHandlerResult,
) -> LegacyCollisionResponse {
    let mut response = LegacyCollisionResponse {
        moving,
        target_velocity: target.velocity,
        resolved: false,
    };

    if moving.velocity.y < 0.0 {
        if target_handler.applies_default() {
            zero_target_y_if(&mut response.target_velocity, |speed_y| speed_y > 0.0);
        }

        if moving_handler.applies_default() {
            if response.moving.velocity.y < 0.0 {
                response.moving.velocity.y = 0.0;
            }
            response.moving.bounds = with_min_y(response.moving.bounds, target.bounds.max.y);
            response.resolved = true;
        }
    } else {
        if target_handler.applies_default() {
            zero_target_y_if(&mut response.target_velocity, |speed_y| speed_y < 0.0);
        }

        if moving_handler.applies_default() {
            if response.moving.velocity.y > 0.0 {
                response.moving.velocity.y = 0.0;
            }
            response.moving.bounds = with_min_y(
                response.moving.bounds,
                target.bounds.min.y - moving.bounds.height(),
            );
            response.resolved = true;
        }
    }

    response
}

#[must_use]
fn broad_phase_near(a: Aabb, b: Aabb) -> bool {
    (a.min.x - b.min.x).abs() < a.width().max(b.width()) + 1.0
        && (a.min.y - b.min.y).abs() < a.height().max(b.height()) + 1.0
}

fn zero_target_x_if(velocity: &mut Option<Vec2>, predicate: impl FnOnce(f32) -> bool) {
    match velocity {
        Some(velocity) if predicate(velocity.x) => velocity.x = 0.0,
        _ => {}
    }
}

fn zero_target_y_if(velocity: &mut Option<Vec2>, predicate: impl FnOnce(f32) -> bool) {
    match velocity {
        Some(velocity) if predicate(velocity.y) => velocity.y = 0.0,
        _ => {}
    }
}

#[must_use]
fn with_min_x(bounds: Aabb, x: f32) -> Aabb {
    Aabb::from_xywh(x, bounds.min.y, bounds.width(), bounds.height())
}

#[must_use]
fn with_min_y(bounds: Aabb, y: f32) -> Aabb {
    Aabb::from_xywh(bounds.min.x, y, bounds.width(), bounds.height())
}

#[cfg(test)]
mod tests {
    use super::{
        CollisionBody, CollisionKind, LegacyCollisionActor, LegacyCollisionHandlerResult,
        LegacyCollisionTarget, collision_kind, in_range, legacy_horizontal_collision_response,
        legacy_passive_collision_response, legacy_vertical_collision_response, lua_aabb_overlap,
    };
    use crate::math::{Aabb, Vec2};

    #[test]
    fn lua_aabb_overlap_matches_edge_exclusive_semantics() {
        assert!(lua_aabb_overlap(
            Aabb::from_xywh(0.0, 0.0, 1.0, 1.0),
            Aabb::from_xywh(0.5, 0.5, 1.0, 1.0)
        ));
        assert!(!lua_aabb_overlap(
            Aabb::from_xywh(0.0, 0.0, 1.0, 1.0),
            Aabb::from_xywh(1.0, 0.0, 1.0, 1.0)
        ));
    }

    #[test]
    fn in_range_matches_lua_order_and_edge_behavior() {
        assert!(in_range(2.0, 3.0, 1.0, true));
        assert!(in_range(2.0, 1.0, 3.0, false));
        assert!(!in_range(1.0, 1.0, 3.0, false));
        assert!(in_range(1.0, 1.0, 3.0, true));
    }

    #[test]
    fn collision_kind_reports_passive_overlap_before_sweep() {
        let moving = CollisionBody::new(0.0, 0.0, 1.0, 1.0, 5.0, 0.0);
        let target = Aabb::from_xywh(0.5, 0.0, 1.0, 1.0);

        assert_eq!(
            collision_kind(moving, target, 0.1, false),
            CollisionKind::Passive
        );
    }

    #[test]
    fn collision_kind_respects_portal_passed_flag() {
        let moving = CollisionBody::new(0.0, 0.0, 1.0, 1.0, 5.0, 0.0);
        let target = Aabb::from_xywh(0.5, 0.0, 1.0, 1.0);

        assert_eq!(
            collision_kind(moving, target, 0.1, true),
            CollisionKind::Horizontal
        );
    }

    #[test]
    fn collision_kind_separates_horizontal_and_vertical_sweeps() {
        let horizontal = CollisionBody::new(0.0, 0.0, 1.0, 1.0, 10.0, 0.0);
        let vertical = CollisionBody::new(0.0, 0.0, 1.0, 1.0, 0.0, 10.0);

        assert_eq!(
            collision_kind(horizontal, Aabb::from_xywh(1.5, 0.0, 1.0, 1.0), 0.1, false),
            CollisionKind::Horizontal
        );
        assert_eq!(
            collision_kind(vertical, Aabb::from_xywh(0.0, 1.5, 1.0, 1.0), 0.1, false),
            CollisionKind::Vertical
        );
    }

    #[test]
    fn diagonal_collision_uses_mari0_axis_tiebreak() {
        let mostly_horizontal = CollisionBody::new(0.0, 0.0, 1.0, 1.0, 10.0, 6.0);
        let mostly_vertical = CollisionBody::new(0.0, 0.0, 1.0, 1.0, 6.0, 20.0);
        let target = Aabb::from_xywh(1.5, 1.5, 1.0, 1.0);

        assert_eq!(
            collision_kind(mostly_horizontal, target, 0.1, false),
            CollisionKind::Vertical
        );
        assert_eq!(
            collision_kind(mostly_vertical, target, 0.1, false),
            CollisionKind::Horizontal
        );
    }

    #[test]
    fn horizontal_response_snaps_left_mover_to_target_right_and_zeros_inward_speeds() {
        let moving = LegacyCollisionActor::new(4.0, 2.0, 1.0, 1.5, -6.0, 3.0);
        let target = LegacyCollisionTarget::new(2.0, 2.0, 1.25, 2.0, Some(Vec2::new(4.0, 9.0)));

        let response = legacy_horizontal_collision_response(
            moving,
            target,
            LegacyCollisionHandlerResult::ApplyDefault,
            LegacyCollisionHandlerResult::ApplyDefault,
        );

        assert!(response.resolved);
        assert_eq!(response.moving.bounds.min.x, 3.25);
        assert_eq!(response.moving.bounds.min.y, 2.0);
        assert_eq!(response.moving.velocity, Vec2::new(0.0, 3.0));
        assert_eq!(response.target_velocity, Some(Vec2::new(0.0, 9.0)));
    }

    #[test]
    fn horizontal_response_snaps_right_mover_to_target_left_and_preserves_noninward_speeds() {
        let moving = LegacyCollisionActor::new(2.0, 2.0, 1.0, 1.5, 0.0, 3.0);
        let target = LegacyCollisionTarget::new(4.0, 2.0, 1.25, 2.0, Some(Vec2::new(4.0, 9.0)));

        let response = legacy_horizontal_collision_response(
            moving,
            target,
            LegacyCollisionHandlerResult::ApplyDefault,
            LegacyCollisionHandlerResult::ApplyDefault,
        );

        assert!(response.resolved);
        assert_eq!(response.moving.bounds.min.x, 3.0);
        assert_eq!(response.moving.velocity, Vec2::new(0.0, 3.0));
        assert_eq!(response.target_velocity, Some(Vec2::new(4.0, 9.0)));
    }

    #[test]
    fn horizontal_response_respects_lua_false_handler_results_independently() {
        let moving = LegacyCollisionActor::new(2.0, 2.0, 1.0, 1.5, 6.0, 3.0);
        let target = LegacyCollisionTarget::new(4.0, 2.0, 1.25, 2.0, Some(Vec2::new(-4.0, 9.0)));

        let moving_suppressed = legacy_horizontal_collision_response(
            moving,
            target,
            LegacyCollisionHandlerResult::SuppressDefault,
            LegacyCollisionHandlerResult::ApplyDefault,
        );
        let target_suppressed = legacy_horizontal_collision_response(
            moving,
            target,
            LegacyCollisionHandlerResult::ApplyDefault,
            LegacyCollisionHandlerResult::SuppressDefault,
        );

        assert!(!moving_suppressed.resolved);
        assert_eq!(moving_suppressed.moving.bounds, moving.bounds);
        assert_eq!(moving_suppressed.moving.velocity, moving.velocity);
        assert_eq!(moving_suppressed.target_velocity, Some(Vec2::new(0.0, 9.0)));

        assert!(target_suppressed.resolved);
        assert_eq!(target_suppressed.moving.bounds.min.x, 3.0);
        assert_eq!(target_suppressed.moving.velocity.x, 0.0);
        assert_eq!(
            target_suppressed.target_velocity,
            Some(Vec2::new(-4.0, 9.0))
        );
    }

    #[test]
    fn passive_response_calls_passive_handlers_without_default_resolution() {
        let moving = LegacyCollisionActor::new(2.0, 4.0, 1.0, 1.5, 6.0, 3.0);
        let target = LegacyCollisionTarget::new(2.0, 4.5, 1.25, 2.0, Some(Vec2::new(4.0, 9.0)));

        let response = legacy_passive_collision_response(moving, target, true, true, None);

        assert!(!response.resolved);
        assert!(response.moving_passive_called);
        assert!(response.target_passive_called);
        assert_eq!(response.moving, moving);
        assert_eq!(response.target_velocity, Some(Vec2::new(4.0, 9.0)));
    }

    #[test]
    fn passive_response_defaults_to_floor_snap_when_no_passive_or_floor_handler_exists() {
        let moving = LegacyCollisionActor::new(2.0, 4.0, 1.0, 1.5, 6.0, 3.0);
        let target = LegacyCollisionTarget::new(2.0, 4.5, 1.25, 2.0, None);

        let response = legacy_passive_collision_response(moving, target, false, true, None);

        assert!(response.resolved);
        assert!(!response.moving_passive_called);
        assert!(!response.target_passive_called);
        assert_eq!(response.moving.bounds.min.y, 3.0);
        assert_eq!(response.moving.velocity, Vec2::new(6.0, 0.0));
        assert_eq!(response.target_velocity, None);
    }

    #[test]
    fn passive_response_respects_floor_handler_result_and_only_zeros_downward_speed() {
        let moving_up = LegacyCollisionActor::new(2.0, 4.0, 1.0, 1.5, 6.0, -3.0);
        let target = LegacyCollisionTarget::new(2.0, 4.5, 1.25, 2.0, None);

        let applied = legacy_passive_collision_response(
            moving_up,
            target,
            false,
            false,
            Some(LegacyCollisionHandlerResult::ApplyDefault),
        );
        let suppressed = legacy_passive_collision_response(
            moving_up,
            target,
            false,
            false,
            Some(LegacyCollisionHandlerResult::SuppressDefault),
        );

        assert!(applied.resolved);
        assert_eq!(applied.moving.bounds.min.y, 3.0);
        assert_eq!(applied.moving.velocity, Vec2::new(6.0, -3.0));

        assert!(!suppressed.resolved);
        assert_eq!(suppressed.moving, moving_up);
    }

    #[test]
    fn vertical_response_snaps_upward_mover_to_target_bottom_and_zeros_inward_speeds() {
        let moving = LegacyCollisionActor::new(2.0, 4.0, 1.0, 1.5, 6.0, -3.0);
        let target = LegacyCollisionTarget::new(2.0, 2.0, 1.25, 2.0, Some(Vec2::new(4.0, 9.0)));

        let response = legacy_vertical_collision_response(
            moving,
            target,
            LegacyCollisionHandlerResult::ApplyDefault,
            LegacyCollisionHandlerResult::ApplyDefault,
        );

        assert!(response.resolved);
        assert_eq!(response.moving.bounds.min.y, 4.0);
        assert_eq!(response.moving.bounds.min.x, 2.0);
        assert_eq!(response.moving.velocity, Vec2::new(6.0, 0.0));
        assert_eq!(response.target_velocity, Some(Vec2::new(4.0, 0.0)));
    }

    #[test]
    fn vertical_response_snaps_downward_mover_to_target_top_and_handles_static_targets() {
        let moving = LegacyCollisionActor::new(2.0, 1.0, 1.0, 1.5, 6.0, 3.0);
        let target = LegacyCollisionTarget::new(2.0, 4.0, 1.25, 2.0, None);

        let response = legacy_vertical_collision_response(
            moving,
            target,
            LegacyCollisionHandlerResult::ApplyDefault,
            LegacyCollisionHandlerResult::ApplyDefault,
        );

        assert!(response.resolved);
        assert_eq!(response.moving.bounds.min.y, 2.5);
        assert_eq!(response.moving.velocity, Vec2::new(6.0, 0.0));
        assert_eq!(response.target_velocity, None);
    }
}
