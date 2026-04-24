//! Engine-neutral hazard rules ported from Lua gameplay objects.

use crate::config::{LegacyCastleFireConstants, LegacyFireConstants, LegacyUpFireConstants};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyCastleFireDirection {
    Clockwise,
    CounterClockwise,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyCastleFireFrame {
    One,
    Two,
    Three,
    Four,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyCastleFireState {
    pub x: f32,
    pub y: f32,
    pub length: usize,
    pub direction: LegacyCastleFireDirection,
    pub angle_degrees: f32,
    pub angle_timer: f32,
    pub animation_timer: f32,
    pub frame: LegacyCastleFireFrame,
}

impl LegacyCastleFireState {
    #[must_use]
    pub fn spawn(
        x: f32,
        y: f32,
        length: Option<usize>,
        direction: Option<LegacyCastleFireDirection>,
    ) -> Self {
        Self {
            x,
            y: y + 1.0 / 16.0,
            length: length.unwrap_or(6),
            direction: direction.unwrap_or(LegacyCastleFireDirection::Clockwise),
            angle_degrees: 0.0,
            angle_timer: 0.0,
            animation_timer: 0.0,
            frame: LegacyCastleFireFrame::One,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyCastleFireSegment {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub active: bool,
    pub static_body: bool,
    pub frame: LegacyCastleFireFrame,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyCastleFireUpdate {
    pub remove: bool,
}

#[must_use]
pub fn update_legacy_castle_fire(
    state: &mut LegacyCastleFireState,
    constants: LegacyCastleFireConstants,
    dt: f32,
) -> LegacyCastleFireUpdate {
    state.angle_timer += dt;
    while state.angle_timer > constants.angle_delay {
        state.angle_timer -= constants.angle_delay;
        match state.direction {
            LegacyCastleFireDirection::Clockwise => {
                state.angle_degrees = (state.angle_degrees + constants.angle_step_degrees) % 360.0;
            }
            LegacyCastleFireDirection::CounterClockwise => {
                state.angle_degrees -= constants.angle_step_degrees;
                while state.angle_degrees < 0.0 {
                    state.angle_degrees += 360.0;
                }
            }
        }
    }

    state.animation_timer += dt;
    while state.animation_timer > constants.animation_delay {
        state.animation_timer -= constants.animation_delay;
        state.frame = match state.frame {
            LegacyCastleFireFrame::One => LegacyCastleFireFrame::Two,
            LegacyCastleFireFrame::Two => LegacyCastleFireFrame::Three,
            LegacyCastleFireFrame::Three => LegacyCastleFireFrame::Four,
            LegacyCastleFireFrame::Four => LegacyCastleFireFrame::One,
        };
    }

    LegacyCastleFireUpdate { remove: false }
}

#[must_use]
pub fn legacy_castle_fire_segments(state: &LegacyCastleFireState) -> Vec<LegacyCastleFireSegment> {
    let x = state.x - 0.5;
    let y = state.y - 0.5;
    let radians = state.angle_degrees.to_radians();
    let cos = radians.cos();
    let sin = radians.sin();

    (0..state.length)
        .map(|index| {
            let distance = index as f32 * 0.5;
            LegacyCastleFireSegment {
                x: x + cos * distance - 0.25,
                y: y + sin * distance - 0.25,
                width: 8.0 / 16.0,
                height: 8.0 / 16.0,
                active: true,
                static_body: true,
                frame: state.frame,
            }
        })
        .collect()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyFireFrame {
    One,
    Two,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyFireSource {
    None,
    Bowser,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyFireState {
    pub x: f32,
    pub y: f32,
    pub target_y: f32,
    pub speed_x: f32,
    pub speed_y: f32,
    pub width: f32,
    pub height: f32,
    pub active: bool,
    pub static_body: bool,
    pub autodelete: bool,
    pub gravity: f32,
    pub rotation: f32,
    pub timer: f32,
    pub frame: LegacyFireFrame,
    pub source: LegacyFireSource,
}

impl LegacyFireState {
    #[must_use]
    pub fn spawn_static(x: f32, y: f32, constants: LegacyFireConstants) -> Self {
        let y = y - 1.0 + 1.0 / 16.0;

        Self {
            x: x + 6.0 / 16.0,
            y,
            target_y: y,
            speed_x: -constants.speed,
            speed_y: 0.0,
            width: 24.0 / 16.0,
            height: 8.0 / 16.0,
            active: true,
            static_body: true,
            autodelete: true,
            gravity: 0.0,
            rotation: 0.0,
            timer: 0.0,
            frame: LegacyFireFrame::One,
            source: LegacyFireSource::None,
        }
    }

    #[must_use]
    pub fn spawn_bowser(
        bowser_x: f32,
        bowser_y: f32,
        bowser_start_y: f32,
        random_target_drop: f32,
        constants: LegacyFireConstants,
    ) -> Self {
        Self {
            x: bowser_x - 0.750,
            y: bowser_y + 0.25,
            target_y: bowser_start_y - random_target_drop + 2.0 / 16.0,
            speed_x: -constants.speed,
            speed_y: 0.0,
            width: 24.0 / 16.0,
            height: 8.0 / 16.0,
            active: true,
            static_body: true,
            autodelete: true,
            gravity: 0.0,
            rotation: 0.0,
            timer: 0.0,
            frame: LegacyFireFrame::One,
            source: LegacyFireSource::Bowser,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyFireUpdate {
    pub remove: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyFireCollisionResponse {
    pub suppress_default: bool,
}

#[must_use]
pub fn update_legacy_fire(
    state: &mut LegacyFireState,
    constants: LegacyFireConstants,
    dt: f32,
) -> LegacyFireUpdate {
    state.timer += dt;
    while state.timer > constants.animation_delay {
        state.frame = match state.frame {
            LegacyFireFrame::One => LegacyFireFrame::Two,
            LegacyFireFrame::Two => LegacyFireFrame::One,
        };
        state.timer -= constants.animation_delay;
    }

    state.x += state.speed_x * dt;

    if state.y > state.target_y {
        state.y -= constants.vertical_speed * dt;
        if state.y < state.target_y {
            state.y = state.target_y;
        }
    } else if state.y < state.target_y {
        state.y += constants.vertical_speed * dt;
        if state.y > state.target_y {
            state.y = state.target_y;
        }
    }

    LegacyFireUpdate { remove: false }
}

#[must_use]
pub const fn legacy_fire_collision() -> LegacyFireCollisionResponse {
    LegacyFireCollisionResponse {
        suppress_default: true,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyUpFireState {
    pub coord_x: f32,
    pub coord_y: f32,
    pub x: f32,
    pub y: f32,
    pub speed_x: f32,
    pub speed_y: f32,
    pub width: f32,
    pub height: f32,
    pub active: bool,
    pub gravity: f32,
    pub delay: f32,
    pub rotation: f32,
    pub timer: f32,
}

impl LegacyUpFireState {
    #[must_use]
    pub fn spawn(x: f32, y: f32, constants: LegacyUpFireConstants) -> Self {
        Self {
            coord_x: x,
            coord_y: y,
            x: x - 14.0 / 16.0,
            y: constants.hide_y,
            speed_x: 0.0,
            speed_y: 0.0,
            width: 12.0 / 16.0,
            height: 12.0 / 16.0,
            active: true,
            gravity: 0.0,
            delay: 0.0,
            rotation: 0.0,
            timer: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyUpFireUpdate {
    pub launched: bool,
    pub remove: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyUpFireCollisionResponse {
    pub suppress_default: bool,
}

#[must_use]
pub fn update_legacy_up_fire(
    state: &mut LegacyUpFireState,
    constants: LegacyUpFireConstants,
    dt: f32,
    mut next_delay: impl FnMut() -> f32,
) -> LegacyUpFireUpdate {
    let mut launched = false;

    if state.y > constants.hide_y && state.speed_y > 0.0 {
        state.timer += dt;
        while state.timer > state.delay {
            state.y = state.coord_y + constants.start_y_offset;
            state.speed_y = -constants.force;
            state.y = constants.hide_y;
            state.delay = next_delay();
            state.timer -= state.delay;
            launched = true;

            if state.delay <= 0.0 {
                break;
            }
        }
    }

    state.speed_y += constants.gravity * dt;
    state.y += state.speed_y * dt;

    LegacyUpFireUpdate {
        launched,
        remove: false,
    }
}

#[must_use]
pub const fn legacy_up_fire_collision() -> LegacyUpFireCollisionResponse {
    LegacyUpFireCollisionResponse {
        suppress_default: true,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LegacyCastleFireDirection, LegacyCastleFireFrame, LegacyCastleFireState,
        LegacyCastleFireUpdate, LegacyFireCollisionResponse, LegacyFireFrame, LegacyFireSource,
        LegacyFireState, LegacyFireUpdate, LegacyUpFireCollisionResponse, LegacyUpFireState,
        LegacyUpFireUpdate, legacy_castle_fire_segments, legacy_fire_collision,
        legacy_up_fire_collision, update_legacy_castle_fire, update_legacy_fire,
        update_legacy_up_fire,
    };
    use crate::config::{LegacyCastleFireConstants, LegacyFireConstants, LegacyUpFireConstants};

    #[test]
    fn castle_fire_spawn_defaults_length_direction_and_initial_segments() {
        let state = LegacyCastleFireState::spawn(5.0, 6.0, None, None);

        assert_eq!(state.x, 5.0);
        assert_eq!(state.y, 6.0 + 1.0 / 16.0);
        assert_eq!(state.length, 6);
        assert_eq!(state.direction, LegacyCastleFireDirection::Clockwise);
        assert_eq!(state.angle_degrees, 0.0);
        assert_eq!(state.angle_timer, 0.0);
        assert_eq!(state.animation_timer, 0.0);
        assert_eq!(state.frame, LegacyCastleFireFrame::One);

        let segments = legacy_castle_fire_segments(&state);
        assert_eq!(segments.len(), 6);
        assert_close(segments[0].x, 5.0 - 0.75);
        assert_close(segments[0].y, 6.0 + 1.0 / 16.0 - 0.75);
        assert_eq!(segments[0].width, 8.0 / 16.0);
        assert_eq!(segments[0].height, 8.0 / 16.0);
        assert!(segments[0].active);
        assert!(segments[0].static_body);
        assert_eq!(segments[0].frame, LegacyCastleFireFrame::One);
        assert_close(segments[1].x, segments[0].x + 0.5);
        assert_close(segments[1].y, segments[0].y);
    }

    #[test]
    fn castle_fire_spawn_preserves_explicit_length_and_counter_clockwise_direction() {
        let state = LegacyCastleFireState::spawn(
            5.0,
            6.0,
            Some(3),
            Some(LegacyCastleFireDirection::CounterClockwise),
        );

        assert_eq!(state.length, 3);
        assert_eq!(state.direction, LegacyCastleFireDirection::CounterClockwise);
        assert_eq!(legacy_castle_fire_segments(&state).len(), 3);
    }

    #[test]
    fn castle_fire_angle_update_uses_strict_timer_and_wraps_by_direction() {
        let constants = LegacyCastleFireConstants::default();
        let mut clockwise = LegacyCastleFireState::spawn(5.0, 6.0, None, None);

        assert_eq!(
            update_legacy_castle_fire(&mut clockwise, constants, constants.angle_delay),
            LegacyCastleFireUpdate { remove: false }
        );
        assert_eq!(clockwise.angle_degrees, 0.0);
        assert_close(clockwise.angle_timer, constants.angle_delay);

        let _ = update_legacy_castle_fire(&mut clockwise, constants, 0.01);
        assert_close(clockwise.angle_degrees, constants.angle_step_degrees);
        assert_close(clockwise.angle_timer, 0.01);

        let mut counter_clockwise = LegacyCastleFireState::spawn(
            5.0,
            6.0,
            None,
            Some(LegacyCastleFireDirection::CounterClockwise),
        );
        let _ = update_legacy_castle_fire(
            &mut counter_clockwise,
            constants,
            constants.angle_delay + 0.01,
        );
        assert_close(
            counter_clockwise.angle_degrees,
            360.0 - constants.angle_step_degrees,
        );
    }

    #[test]
    fn castle_fire_animation_uses_strict_timer_and_wraps_four_frames() {
        let constants = LegacyCastleFireConstants::default();
        let mut state = LegacyCastleFireState::spawn(5.0, 6.0, None, None);

        let _ = update_legacy_castle_fire(&mut state, constants, constants.animation_delay);
        assert_eq!(state.frame, LegacyCastleFireFrame::One);
        assert_close(state.animation_timer, constants.animation_delay);

        let _ = update_legacy_castle_fire(&mut state, constants, 0.01);
        assert_eq!(state.frame, LegacyCastleFireFrame::Two);
        assert_close(state.animation_timer, 0.01);

        let _ = update_legacy_castle_fire(
            &mut state,
            constants,
            constants.animation_delay * 3.0 + 0.01,
        );
        assert_eq!(state.frame, LegacyCastleFireFrame::One);
        assert_close(state.animation_timer, 0.02);
    }

    #[test]
    fn castle_fire_segments_follow_current_angle_and_frame() {
        let mut state = LegacyCastleFireState::spawn(5.0, 6.0, Some(2), None);
        state.angle_degrees = 90.0;
        state.frame = LegacyCastleFireFrame::Three;

        let segments = legacy_castle_fire_segments(&state);

        assert_close(segments[1].x, segments[0].x);
        assert_close(segments[1].y, segments[0].y + 0.5);
        assert_eq!(segments[1].frame, LegacyCastleFireFrame::Three);
    }

    #[test]
    fn fire_static_spawn_matches_legacy_offsets_and_initial_state() {
        let constants = LegacyFireConstants::default();
        let state = LegacyFireState::spawn_static(8.0, 12.0, constants);

        assert_eq!(state.x, 8.0 + 6.0 / 16.0);
        assert_eq!(state.y, 12.0 - 1.0 + 1.0 / 16.0);
        assert_eq!(state.target_y, state.y);
        assert_eq!(state.speed_x, -constants.speed);
        assert_eq!(state.speed_y, 0.0);
        assert_eq!(state.width, 24.0 / 16.0);
        assert_eq!(state.height, 8.0 / 16.0);
        assert!(state.active);
        assert!(state.static_body);
        assert!(state.autodelete);
        assert_eq!(state.gravity, 0.0);
        assert_eq!(state.rotation, 0.0);
        assert_eq!(state.timer, 0.0);
        assert_eq!(state.frame, LegacyFireFrame::One);
        assert_eq!(state.source, LegacyFireSource::None);
    }

    #[test]
    fn fire_bowser_spawn_uses_injected_random_target_drop() {
        let constants = LegacyFireConstants::default();
        let state = LegacyFireState::spawn_bowser(9.0, 4.0, 6.0, 2.0, constants);

        assert_eq!(state.x, 9.0 - 0.750);
        assert_eq!(state.y, 4.0 + 0.25);
        assert_eq!(state.target_y, 6.0 - 2.0 + 2.0 / 16.0);
        assert_eq!(state.speed_x, -constants.speed);
        assert_eq!(state.source, LegacyFireSource::Bowser);
    }

    #[test]
    fn fire_animation_uses_strict_timer_and_toggles_two_frames() {
        let constants = LegacyFireConstants::default();
        let mut state = LegacyFireState::spawn_static(8.0, 12.0, constants);

        assert_eq!(
            update_legacy_fire(&mut state, constants, constants.animation_delay),
            LegacyFireUpdate { remove: false }
        );
        assert_eq!(state.frame, LegacyFireFrame::One);
        assert_close(state.timer, constants.animation_delay);

        let _ = update_legacy_fire(&mut state, constants, 0.01);
        assert_eq!(state.frame, LegacyFireFrame::Two);
        assert_close(state.timer, 0.01);

        let _ = update_legacy_fire(&mut state, constants, constants.animation_delay + 0.01);
        assert_eq!(state.frame, LegacyFireFrame::One);
        assert_close(state.timer, 0.02);
    }

    #[test]
    fn fire_update_moves_left_and_seeks_target_y_with_clamp() {
        let constants = LegacyFireConstants::default();
        let mut down = LegacyFireState::spawn_bowser(9.0, 4.0, 6.0, 1.0, constants);
        let start_x = down.x;

        let _ = update_legacy_fire(&mut down, constants, 0.5);
        assert_close(down.x, start_x - constants.speed * 0.5);
        assert_close(down.y, down.target_y);

        let mut up = LegacyFireState::spawn_bowser(9.0, 8.0, 6.0, 3.0, constants);
        let _ = update_legacy_fire(&mut up, constants, 10.0);
        assert_close(up.y, up.target_y);
    }

    #[test]
    fn fire_collisions_suppress_default_resolution() {
        assert_eq!(
            legacy_fire_collision(),
            LegacyFireCollisionResponse {
                suppress_default: true
            }
        );
    }

    #[test]
    fn up_fire_spawn_matches_legacy_hidden_position_and_body() {
        let constants = LegacyUpFireConstants::default();
        let state = LegacyUpFireState::spawn(8.0, 12.0, constants);

        assert_eq!(state.coord_x, 8.0);
        assert_eq!(state.coord_y, 12.0);
        assert_eq!(state.x, 8.0 - 14.0 / 16.0);
        assert_eq!(state.y, constants.hide_y);
        assert_eq!(state.speed_x, 0.0);
        assert_eq!(state.speed_y, 0.0);
        assert_eq!(state.width, 12.0 / 16.0);
        assert_eq!(state.height, 12.0 / 16.0);
        assert!(state.active);
        assert_eq!(state.gravity, 0.0);
        assert_eq!(state.delay, 0.0);
        assert_eq!(state.rotation, 0.0);
        assert_eq!(state.timer, 0.0);
    }

    #[test]
    fn up_fire_update_applies_manual_gravity_and_motion_without_removal() {
        let constants = LegacyUpFireConstants::default();
        let mut state = LegacyUpFireState::spawn(8.0, 12.0, constants);

        assert_eq!(
            update_legacy_up_fire(&mut state, constants, 0.1, || 1.0),
            LegacyUpFireUpdate {
                launched: false,
                remove: false
            }
        );
        assert_close(state.speed_y, constants.gravity * 0.1);
        assert_close(state.y, constants.hide_y + state.speed_y * 0.1);
        assert_eq!(state.timer, 0.0);
    }

    #[test]
    fn up_fire_launch_uses_strict_delay_and_preserves_new_delay_timer_subtraction() {
        let constants = LegacyUpFireConstants::default();
        let mut state = LegacyUpFireState::spawn(8.0, 12.0, constants);
        state.y = constants.hide_y + 0.1;
        state.speed_y = 1.0;
        state.delay = 0.2;

        assert_eq!(
            update_legacy_up_fire(&mut state, constants, 0.2, || 1.0),
            LegacyUpFireUpdate {
                launched: false,
                remove: false
            }
        );
        assert_close(state.timer, 0.2);
        assert_close(state.speed_y, 1.0 + constants.gravity * 0.2);

        state.delay = 0.0;
        state.timer = 0.0;
        state.y = constants.hide_y + 0.2;
        state.speed_y = 3.0;

        assert_eq!(
            update_legacy_up_fire(&mut state, constants, 0.1, || 2.5),
            LegacyUpFireUpdate {
                launched: true,
                remove: false
            }
        );
        assert_eq!(state.delay, 2.5);
        assert_close(state.timer, -2.4);
        assert_close(state.speed_y, -constants.force + constants.gravity * 0.1);
        assert_close(state.y, constants.hide_y + state.speed_y * 0.1);
    }

    #[test]
    fn up_fire_zero_next_delay_launch_guard_keeps_update_finite() {
        let constants = LegacyUpFireConstants::default();
        let mut state = LegacyUpFireState::spawn(8.0, 12.0, constants);
        state.y = constants.hide_y + 0.1;
        state.speed_y = 1.0;

        assert_eq!(
            update_legacy_up_fire(&mut state, constants, 0.1, || 0.0),
            LegacyUpFireUpdate {
                launched: true,
                remove: false
            }
        );
        assert_eq!(state.delay, 0.0);
        assert_close(state.timer, 0.1);
    }

    #[test]
    fn up_fire_collisions_suppress_default_resolution() {
        assert_eq!(
            legacy_up_fire_collision(),
            LegacyUpFireCollisionResponse {
                suppress_default: true
            }
        );
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.0001,
            "expected {actual} to be close to {expected}"
        );
    }
}
