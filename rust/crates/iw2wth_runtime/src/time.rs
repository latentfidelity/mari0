//! Time/frame-step adapter boundary for the legacy `love.update` loop.
//!
//! `main.lua` clamps raw frame time, smooths the runtime speed modifier, scales
//! the update `dt`, mirrors it to global `gdt`, and only then applies
//! frame-advance and skip-update gates. This module keeps that deterministic
//! runtime sequencing outside `iw2wth_core`.

pub const LEGACY_MAX_UPDATE_DT: f32 = 0.016_666_67;
pub const LEGACY_SPEED_SMOOTHING: f32 = 5.0;
pub const LEGACY_SPEED_SNAP_EPSILON: f32 = 0.02;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyFrameAdvance {
    Running,
    Held,
    StepQueued,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyFrameClock {
    pub speed: f32,
    pub speed_target: f32,
    pub frame_advance: LegacyFrameAdvance,
    pub skip_update: bool,
}

impl Default for LegacyFrameClock {
    fn default() -> Self {
        Self {
            speed: 1.0,
            speed_target: 1.0,
            frame_advance: LegacyFrameAdvance::Running,
            skip_update: false,
        }
    }
}

impl LegacyFrameClock {
    #[must_use]
    pub const fn new(speed: f32, speed_target: f32) -> Self {
        Self {
            speed,
            speed_target,
            frame_advance: LegacyFrameAdvance::Running,
            skip_update: false,
        }
    }

    #[must_use]
    pub const fn with_frame_advance(mut self, frame_advance: LegacyFrameAdvance) -> Self {
        self.frame_advance = frame_advance;
        self
    }

    #[must_use]
    pub const fn with_skip_update(mut self, skip_update: bool) -> Self {
        self.skip_update = skip_update;
        self
    }

    pub fn step(&mut self, raw_dt: f32) -> LegacyFrameStep {
        let clamped_dt = raw_dt.min(LEGACY_MAX_UPDATE_DT);
        let previous_speed = self.speed;

        if self.speed != self.speed_target {
            self.speed = if self.speed > self.speed_target {
                self.speed_target.max(
                    self.speed
                        + (self.speed_target - self.speed) * clamped_dt * LEGACY_SPEED_SMOOTHING,
                )
            } else {
                self.speed_target.min(
                    self.speed
                        + (self.speed_target - self.speed) * clamped_dt * LEGACY_SPEED_SMOOTHING,
                )
            };

            if (self.speed - self.speed_target).abs() < LEGACY_SPEED_SNAP_EPSILON {
                self.speed = self.speed_target;
            }
        }

        let update_dt = clamped_dt * self.speed;
        let speed_changed = self.speed != previous_speed;

        let should_update = match self.frame_advance {
            LegacyFrameAdvance::Held => false,
            LegacyFrameAdvance::StepQueued => {
                self.frame_advance = LegacyFrameAdvance::Held;
                self.consume_skip_update_gate()
            }
            LegacyFrameAdvance::Running => self.consume_skip_update_gate(),
        };

        LegacyFrameStep {
            raw_dt,
            clamped_dt,
            update_dt,
            global_dt: update_dt,
            previous_speed,
            speed: self.speed,
            speed_changed,
            should_update,
        }
    }

    fn consume_skip_update_gate(&mut self) -> bool {
        if self.skip_update {
            self.skip_update = false;
            false
        } else {
            true
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyFrameStep {
    pub raw_dt: f32,
    pub clamped_dt: f32,
    pub update_dt: f32,
    pub global_dt: f32,
    pub previous_speed: f32,
    pub speed: f32,
    pub speed_changed: bool,
    pub should_update: bool,
}

#[cfg(test)]
mod tests {
    use super::{LEGACY_MAX_UPDATE_DT, LegacyFrameAdvance, LegacyFrameClock, LegacyFrameStep};

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.000_001,
            "expected {actual} to be close to {expected}",
        );
    }

    #[test]
    fn frame_step_clamps_raw_dt_before_exposing_update_and_global_dt() {
        let mut clock = LegacyFrameClock::default();

        let step = clock.step(0.25);

        assert_close(step.raw_dt, 0.25);
        assert_close(step.clamped_dt, LEGACY_MAX_UPDATE_DT);
        assert_close(step.update_dt, LEGACY_MAX_UPDATE_DT);
        assert_close(step.global_dt, step.update_dt);
        assert!(step.should_update);
    }

    #[test]
    fn speed_easing_uses_clamped_dt_before_scaling_update_dt() {
        let mut clock = LegacyFrameClock::new(0.5, 1.0);

        let step = clock.step(1.0);

        assert_close(step.previous_speed, 0.5);
        assert_close(step.speed, 0.541_666_7);
        assert!(step.speed_changed);
        assert_close(step.update_dt, LEGACY_MAX_UPDATE_DT * step.speed);
        assert!(step.should_update);
    }

    #[test]
    fn speed_easing_snaps_to_target_inside_legacy_epsilon() {
        let mut clock = LegacyFrameClock::new(1.0, 0.99);

        let step = clock.step(0.01);

        assert_close(step.speed, 0.99);
        assert!(step.speed_changed);
        assert_close(step.update_dt, 0.01 * 0.99);
    }

    #[test]
    fn held_frame_advance_suppresses_update_before_skip_update_is_consumed() {
        let mut clock = LegacyFrameClock::default()
            .with_frame_advance(LegacyFrameAdvance::Held)
            .with_skip_update(true);

        let step = clock.step(0.01);

        assert!(!step.should_update);
        assert!(clock.skip_update);
        assert_eq!(clock.frame_advance, LegacyFrameAdvance::Held);
    }

    #[test]
    fn queued_frame_advance_runs_one_update_and_becomes_held() {
        let mut clock =
            LegacyFrameClock::default().with_frame_advance(LegacyFrameAdvance::StepQueued);

        let step = clock.step(0.01);

        assert!(step.should_update);
        assert_eq!(clock.frame_advance, LegacyFrameAdvance::Held);
    }

    #[test]
    fn skip_update_clears_after_dt_and_speed_are_computed() {
        let mut clock = LegacyFrameClock::new(0.5, 1.0).with_skip_update(true);

        let step = clock.step(1.0);

        assert!(!step.should_update);
        assert!(!clock.skip_update);
        assert_close(step.speed, 0.541_666_7);
        assert_close(step.update_dt, LEGACY_MAX_UPDATE_DT * step.speed);
    }

    #[test]
    fn queued_frame_advance_still_allows_skip_update_to_cancel_the_step() {
        let mut clock = LegacyFrameClock::default()
            .with_frame_advance(LegacyFrameAdvance::StepQueued)
            .with_skip_update(true);

        let LegacyFrameStep { should_update, .. } = clock.step(0.01);

        assert!(!should_update);
        assert!(!clock.skip_update);
        assert_eq!(clock.frame_advance, LegacyFrameAdvance::Held);
    }
}
