//! Engine-neutral vine object rules ported from `vine.lua`.

use crate::config::LegacyVineConstants;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyVineVariant {
    Regular,
    Start,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyVineState {
    pub anchor_x: f32,
    pub anchor_y: f32,
    pub variant: LegacyVineVariant,
    pub limit: f32,
    pub timer: f32,
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
    pub is_static: bool,
    pub active: bool,
    pub drawable: bool,
}

impl LegacyVineState {
    #[must_use]
    pub fn spawn(
        anchor_x: f32,
        anchor_y: f32,
        variant: LegacyVineVariant,
        constants: LegacyVineConstants,
    ) -> Self {
        let limit = match variant {
            LegacyVineVariant::Regular => constants.regular_limit,
            LegacyVineVariant::Start => constants.start_limit,
        };

        Self {
            anchor_x,
            anchor_y,
            variant,
            limit,
            timer: 0.0,
            width: constants.width,
            height: 0.0,
            x: anchor_x - 0.5 - constants.width / 2.0,
            y: anchor_y - constants.spawn_y_offset,
            is_static: true,
            active: true,
            drawable: false,
        }
    }
}

pub fn update_legacy_vine(state: &mut LegacyVineState, constants: LegacyVineConstants, dt: f32) {
    if state.y > state.limit {
        state.y -= constants.speed * dt;
        state.height = state.anchor_y - state.y - constants.growth_height_offset;

        if state.y <= state.limit {
            state.y = state.limit;
            state.height = state.anchor_y - state.limit - constants.growth_height_offset;
        }

        state.height = state.height.max(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::{LegacyVineState, LegacyVineVariant, update_legacy_vine};
    use crate::config::LegacyVineConstants;

    fn assert_close(actual: f32, expected: f32) {
        let diff = (actual - expected).abs();
        assert!(
            diff <= 1.0e-6,
            "expected {expected}, got {actual} (diff {diff})"
        );
    }

    #[test]
    fn vine_state_spawn_matches_regular_vine_lua_init() {
        let constants = LegacyVineConstants::default();
        let state = LegacyVineState::spawn(12.0, 8.0, LegacyVineVariant::Regular, constants);

        assert_close(state.anchor_x, 12.0);
        assert_close(state.anchor_y, 8.0);
        assert_eq!(state.variant, LegacyVineVariant::Regular);
        assert_close(state.limit, -1.0);
        assert_close(state.timer, 0.0);
        assert_close(state.width, 10.0 / 16.0);
        assert_close(state.height, 0.0);
        assert_close(state.x, 11.1875);
        assert_close(state.y, 7.0);
        assert!(state.is_static);
        assert!(state.active);
        assert!(!state.drawable);
    }

    #[test]
    fn vine_state_spawn_uses_start_variant_limit() {
        let constants = LegacyVineConstants::default();
        let state = LegacyVineState::spawn(5.0, 16.0, LegacyVineVariant::Start, constants);

        assert_eq!(state.variant, LegacyVineVariant::Start);
        assert_close(state.limit, 9.0 + 1.0 / 16.0);
        assert_close(state.y, 15.0);
    }

    #[test]
    fn vine_state_update_grows_upward_and_updates_height() {
        let constants = LegacyVineConstants::default();
        let mut state = LegacyVineState::spawn(12.0, 8.0, LegacyVineVariant::Regular, constants);

        update_legacy_vine(&mut state, constants, 0.5);

        assert_close(state.y, 5.935);
        assert_close(state.height, 0.365);
    }

    #[test]
    fn vine_state_update_clamps_to_limit_when_growth_crosses_threshold() {
        let constants = LegacyVineConstants::default();
        let mut state = LegacyVineState::spawn(5.0, 11.0, LegacyVineVariant::Start, constants);

        update_legacy_vine(&mut state, constants, 1.0);

        assert_close(state.y, constants.start_limit);
        assert_close(
            state.height,
            11.0 - constants.start_limit - constants.growth_height_offset,
        );
    }

    #[test]
    fn vine_state_update_preserves_zero_height_when_growth_formula_goes_negative() {
        let constants = LegacyVineConstants::default();
        let mut state = LegacyVineState::spawn(2.0, 1.0, LegacyVineVariant::Regular, constants);

        update_legacy_vine(&mut state, constants, 0.1);

        assert_close(state.y, -0.213);
        assert_close(state.height, 0.0);
    }

    #[test]
    fn vine_state_update_is_noop_once_at_limit() {
        let constants = LegacyVineConstants::default();
        let mut state = LegacyVineState::spawn(12.0, 8.0, LegacyVineVariant::Regular, constants);
        state.y = state.limit;
        state.height = 2.5;

        update_legacy_vine(&mut state, constants, 0.5);

        assert_close(state.y, state.limit);
        assert_close(state.height, 2.5);
    }
}
