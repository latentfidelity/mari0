//! Input adapter boundary for legacy Mari0 control bindings.
//!
//! `game.lua` implements `runkey`, `rightkey`, and `leftkey` by calling
//! `checkkey` over keyboard or joystick bindings. This module preserves that
//! polling contract outside `iw2wth_core` and projects the result into the
//! engine-neutral `PlayerMovementInput` type.

use std::collections::{BTreeMap, BTreeSet};

use iw2wth_core::PlayerMovementInput;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LegacyControlBinding {
    Keyboard(String),
    JoystickHat {
        joystick: u32,
        hat: u32,
        direction: String,
    },
    JoystickButton {
        joystick: u32,
        button: u32,
    },
    JoystickAxis {
        joystick: u32,
        axis: u32,
        direction: LegacyAxisDirection,
    },
}

impl LegacyControlBinding {
    #[must_use]
    pub fn keyboard(key: impl Into<String>) -> Self {
        Self::Keyboard(key.into())
    }

    #[must_use]
    pub fn joystick_hat(joystick: u32, hat: u32, direction: impl Into<String>) -> Self {
        Self::JoystickHat {
            joystick,
            hat,
            direction: direction.into(),
        }
    }

    #[must_use]
    pub const fn joystick_button(joystick: u32, button: u32) -> Self {
        Self::JoystickButton { joystick, button }
    }

    #[must_use]
    pub const fn joystick_axis(joystick: u32, axis: u32, direction: LegacyAxisDirection) -> Self {
        Self::JoystickAxis {
            joystick,
            axis,
            direction,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyAxisDirection {
    Positive,
    Negative,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegacyPlayerControls {
    pub left: LegacyControlBinding,
    pub right: LegacyControlBinding,
    pub run: LegacyControlBinding,
}

impl LegacyPlayerControls {
    #[must_use]
    pub const fn new(
        left: LegacyControlBinding,
        right: LegacyControlBinding,
        run: LegacyControlBinding,
    ) -> Self {
        Self { left, right, run }
    }
}

pub trait LegacyInputSnapshot {
    fn keyboard_is_down(&self, key: &str) -> bool;
    fn joystick_hat(&self, joystick: u32, hat: u32) -> Option<&str>;
    fn joystick_button_is_down(&self, joystick: u32, button: u32) -> bool;
    fn joystick_axis(&self, joystick: u32, axis: u32) -> Option<f32>;
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BufferedLegacyInputSnapshot {
    keyboard: BTreeSet<String>,
    hats: BTreeMap<(u32, u32), String>,
    buttons: BTreeSet<(u32, u32)>,
    axes: BTreeMap<(u32, u32), f32>,
}

impl BufferedLegacyInputSnapshot {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_keyboard_key(mut self, key: impl Into<String>) -> Self {
        self.keyboard.insert(key.into());
        self
    }

    #[must_use]
    pub fn with_joystick_hat(
        mut self,
        joystick: u32,
        hat: u32,
        direction: impl Into<String>,
    ) -> Self {
        self.hats.insert((joystick, hat), direction.into());
        self
    }

    #[must_use]
    pub fn with_joystick_button(mut self, joystick: u32, button: u32) -> Self {
        self.buttons.insert((joystick, button));
        self
    }

    #[must_use]
    pub fn with_joystick_axis(mut self, joystick: u32, axis: u32, value: f32) -> Self {
        self.axes.insert((joystick, axis), value);
        self
    }
}

impl LegacyInputSnapshot for BufferedLegacyInputSnapshot {
    fn keyboard_is_down(&self, key: &str) -> bool {
        self.keyboard.contains(key)
    }

    fn joystick_hat(&self, joystick: u32, hat: u32) -> Option<&str> {
        self.hats.get(&(joystick, hat)).map(String::as_str)
    }

    fn joystick_button_is_down(&self, joystick: u32, button: u32) -> bool {
        self.buttons.contains(&(joystick, button))
    }

    fn joystick_axis(&self, joystick: u32, axis: u32) -> Option<f32> {
        self.axes.get(&(joystick, axis)).copied()
    }
}

#[must_use]
pub fn legacy_check_key(
    binding: &LegacyControlBinding,
    snapshot: &impl LegacyInputSnapshot,
    joystick_deadzone: f32,
) -> bool {
    match binding {
        LegacyControlBinding::Keyboard(key) => snapshot.keyboard_is_down(key),
        LegacyControlBinding::JoystickHat {
            joystick,
            hat,
            direction,
        } => snapshot
            .joystick_hat(*joystick, *hat)
            .is_some_and(|actual| actual == direction),
        LegacyControlBinding::JoystickButton { joystick, button } => {
            snapshot.joystick_button_is_down(*joystick, *button)
        }
        LegacyControlBinding::JoystickAxis {
            joystick,
            axis,
            direction,
        } => {
            let Some(value) = snapshot.joystick_axis(*joystick, *axis) else {
                return false;
            };

            match direction {
                LegacyAxisDirection::Positive => value > joystick_deadzone,
                LegacyAxisDirection::Negative => value < -joystick_deadzone,
            }
        }
    }
}

#[must_use]
pub fn legacy_player_movement_input(
    controls: &LegacyPlayerControls,
    snapshot: &impl LegacyInputSnapshot,
    joystick_deadzone: f32,
) -> PlayerMovementInput {
    PlayerMovementInput::new(
        legacy_check_key(&controls.left, snapshot, joystick_deadzone),
        legacy_check_key(&controls.right, snapshot, joystick_deadzone),
        legacy_check_key(&controls.run, snapshot, joystick_deadzone),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        BufferedLegacyInputSnapshot, LegacyAxisDirection, LegacyControlBinding,
        LegacyPlayerControls, legacy_check_key, legacy_player_movement_input,
    };
    use iw2wth_core::PlayerMovementInput;

    #[test]
    fn keyboard_binding_matches_lua_keyboard_branch() {
        let snapshot = BufferedLegacyInputSnapshot::new().with_keyboard_key("left");

        assert!(legacy_check_key(
            &LegacyControlBinding::keyboard("left"),
            &snapshot,
            0.2,
        ));
        assert!(!legacy_check_key(
            &LegacyControlBinding::keyboard("right"),
            &snapshot,
            0.2,
        ));
    }

    #[test]
    fn joystick_hat_requires_exact_legacy_direction_match() {
        let snapshot = BufferedLegacyInputSnapshot::new().with_joystick_hat(1, 2, "ld");

        assert!(legacy_check_key(
            &LegacyControlBinding::joystick_hat(1, 2, "ld"),
            &snapshot,
            0.2,
        ));
        assert!(!legacy_check_key(
            &LegacyControlBinding::joystick_hat(1, 2, "l"),
            &snapshot,
            0.2,
        ));
        assert!(!legacy_check_key(
            &LegacyControlBinding::joystick_hat(2, 2, "ld"),
            &snapshot,
            0.2,
        ));
    }

    #[test]
    fn joystick_button_returns_false_when_joystick_or_button_is_missing() {
        let snapshot = BufferedLegacyInputSnapshot::new().with_joystick_button(1, 4);

        assert!(legacy_check_key(
            &LegacyControlBinding::joystick_button(1, 4),
            &snapshot,
            0.2,
        ));
        assert!(!legacy_check_key(
            &LegacyControlBinding::joystick_button(1, 3),
            &snapshot,
            0.2,
        ));
        assert!(!legacy_check_key(
            &LegacyControlBinding::joystick_button(2, 4),
            &snapshot,
            0.2,
        ));
    }

    #[test]
    fn joystick_axis_uses_strict_deadzone_thresholds() {
        let at_deadzone = BufferedLegacyInputSnapshot::new()
            .with_joystick_axis(1, 1, 0.2)
            .with_joystick_axis(1, 2, -0.2);
        let past_deadzone = BufferedLegacyInputSnapshot::new()
            .with_joystick_axis(1, 1, 0.201)
            .with_joystick_axis(1, 2, -0.201);

        assert!(!legacy_check_key(
            &LegacyControlBinding::joystick_axis(1, 1, LegacyAxisDirection::Positive),
            &at_deadzone,
            0.2,
        ));
        assert!(!legacy_check_key(
            &LegacyControlBinding::joystick_axis(1, 2, LegacyAxisDirection::Negative),
            &at_deadzone,
            0.2,
        ));
        assert!(legacy_check_key(
            &LegacyControlBinding::joystick_axis(1, 1, LegacyAxisDirection::Positive),
            &past_deadzone,
            0.2,
        ));
        assert!(legacy_check_key(
            &LegacyControlBinding::joystick_axis(1, 2, LegacyAxisDirection::Negative),
            &past_deadzone,
            0.2,
        ));
    }

    #[test]
    fn player_movement_input_preserves_independent_legacy_key_queries() {
        let controls = LegacyPlayerControls::new(
            LegacyControlBinding::keyboard("left"),
            LegacyControlBinding::keyboard("right"),
            LegacyControlBinding::keyboard("lshift"),
        );
        let snapshot = BufferedLegacyInputSnapshot::new()
            .with_keyboard_key("left")
            .with_keyboard_key("right");

        assert_eq!(
            legacy_player_movement_input(&controls, &snapshot, 0.2),
            PlayerMovementInput::new(true, true, false),
        );
    }
}
