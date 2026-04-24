//! Engine-neutral spring object rules ported from `spring.lua`.

use crate::config::SpringConstants;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacySpringState {
    pub anchor_x: f32,
    pub anchor_y: f32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub is_static: bool,
    pub active: bool,
    pub drawable: bool,
    pub timer: f32,
    pub frame: u8,
}

impl LegacySpringState {
    #[must_use]
    pub fn spawn(anchor_x: f32, anchor_y: f32, constants: SpringConstants) -> Self {
        Self {
            anchor_x,
            anchor_y,
            x: anchor_x - 1.0,
            y: anchor_y - constants.player_y_offset,
            width: 1.0,
            height: constants.player_y_offset,
            is_static: true,
            active: true,
            drawable: false,
            timer: constants.duration,
            frame: 1,
        }
    }
}

pub fn apply_legacy_spring_hit(state: &mut LegacySpringState) {
    state.timer = 0.0;
}

pub fn update_legacy_spring(state: &mut LegacySpringState, constants: SpringConstants, dt: f32) {
    if state.timer < constants.duration {
        state.timer += dt;
        if state.timer > constants.duration {
            state.timer = constants.duration;
        }

        let third = constants.duration / 3.0;
        let mut frame = ((state.timer / third + 0.001).ceil() as i32) + 1;
        if frame > 3 {
            frame = 6 - frame;
        }
        state.frame = frame as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::{LegacySpringState, apply_legacy_spring_hit, update_legacy_spring};
    use crate::config::SpringConstants;

    fn assert_close(actual: f32, expected: f32) {
        let diff = (actual - expected).abs();
        assert!(
            diff <= 1.0e-6,
            "expected {expected}, got {actual} (diff {diff})"
        );
    }

    #[test]
    fn spring_spawn_matches_spring_lua_init() {
        let constants = SpringConstants::default();
        let state = LegacySpringState::spawn(12.0, 8.0, constants);

        assert_close(state.anchor_x, 12.0);
        assert_close(state.anchor_y, 8.0);
        assert_close(state.x, 11.0);
        assert_close(state.y, 8.0 - 31.0 / 16.0);
        assert_close(state.width, 1.0);
        assert_close(state.height, 31.0 / 16.0);
        assert!(state.is_static);
        assert!(state.active);
        assert!(!state.drawable);
        assert_close(state.timer, constants.duration);
        assert_eq!(state.frame, 1);
    }

    #[test]
    fn spring_hit_resets_timer_without_touching_frame() {
        let constants = SpringConstants::default();
        let mut state = LegacySpringState::spawn(12.0, 8.0, constants);
        state.frame = 3;

        apply_legacy_spring_hit(&mut state);

        assert_close(state.timer, 0.0);
        assert_eq!(state.frame, 3);
    }

    #[test]
    fn spring_update_is_noop_when_fully_extended() {
        let constants = SpringConstants::default();
        let mut state = LegacySpringState::spawn(12.0, 8.0, constants);

        update_legacy_spring(&mut state, constants, 0.05);

        assert_close(state.timer, constants.duration);
        assert_eq!(state.frame, 1);
    }

    #[test]
    fn spring_update_advances_through_compression_frames() {
        let constants = SpringConstants::default();
        let mut state = LegacySpringState::spawn(12.0, 8.0, constants);
        apply_legacy_spring_hit(&mut state);

        update_legacy_spring(&mut state, constants, constants.duration / 6.0);
        assert_eq!(state.frame, 2);

        update_legacy_spring(&mut state, constants, constants.duration / 6.0);
        assert_eq!(state.frame, 3);

        update_legacy_spring(&mut state, constants, constants.duration / 3.0);
        assert_eq!(state.frame, 2);
    }

    #[test]
    fn spring_update_clamps_timer_and_returns_to_frame_one() {
        let constants = SpringConstants::default();
        let mut state = LegacySpringState::spawn(12.0, 8.0, constants);
        apply_legacy_spring_hit(&mut state);

        update_legacy_spring(&mut state, constants, constants.duration * 2.0);

        assert_close(state.timer, constants.duration);
        assert_eq!(state.frame, 1);
    }
}
