//! Engine-neutral one-shot visual effect rules ported from Lua objects.

use crate::config::{
    LegacyBlockDebrisConstants, LegacyCoinBlockAnimationConstants, LegacyFireworkConstants,
    LegacyScrollingScoreConstants,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyScrollingScoreLabel {
    Points(u32),
    OneUp,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyScrollingScoreState {
    pub x: f32,
    pub y: f32,
    pub label: LegacyScrollingScoreLabel,
    pub timer: f32,
}

impl LegacyScrollingScoreState {
    #[must_use]
    pub fn spawn(label: LegacyScrollingScoreLabel, x: f32, y: f32, x_scroll: f32) -> Self {
        Self {
            x: x - x_scroll,
            y,
            label,
            timer: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyScrollingScoreUpdate {
    pub remove: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyScrollingScorePresentation {
    pub x: f32,
    pub y: f32,
    pub label: LegacyScrollingScoreLabel,
}

#[must_use]
pub fn update_legacy_scrolling_score(
    state: &mut LegacyScrollingScoreState,
    constants: LegacyScrollingScoreConstants,
    dt: f32,
) -> LegacyScrollingScoreUpdate {
    state.timer += dt;

    LegacyScrollingScoreUpdate {
        remove: state.timer > constants.lifetime,
    }
}

#[must_use]
pub fn legacy_scrolling_score_presentation(
    state: &LegacyScrollingScoreState,
    constants: LegacyScrollingScoreConstants,
) -> LegacyScrollingScorePresentation {
    let x = match state.label {
        LegacyScrollingScoreLabel::Points(_) => state.x - constants.numeric_x_offset,
        LegacyScrollingScoreLabel::OneUp => state.x,
    };
    let y = state.y
        - constants.y_base_offset
        - constants.rise_height * (state.timer / constants.lifetime);

    LegacyScrollingScorePresentation {
        x,
        y,
        label: state.label,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyBlockDebrisState {
    pub x: f32,
    pub y: f32,
    pub speed_x: f32,
    pub speed_y: f32,
    pub timer: f32,
    pub frame: u8,
}

impl LegacyBlockDebrisState {
    #[must_use]
    pub const fn spawn(x: f32, y: f32, speed_x: f32, speed_y: f32) -> Self {
        Self {
            x,
            y,
            speed_x,
            speed_y,
            timer: 0.0,
            frame: 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyBlockDebrisUpdate {
    pub remove: bool,
}

#[must_use]
pub fn update_legacy_block_debris(
    state: &mut LegacyBlockDebrisState,
    constants: LegacyBlockDebrisConstants,
    dt: f32,
) -> LegacyBlockDebrisUpdate {
    state.timer += dt;
    while state.timer > constants.animation_time {
        state.timer -= constants.animation_time;
        state.frame = if state.frame == 1 { 2 } else { 1 };
    }

    state.speed_y += constants.gravity * dt;
    state.x += state.speed_x * dt;
    state.y += state.speed_y * dt;

    LegacyBlockDebrisUpdate {
        remove: state.y > constants.removal_y,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyCoinBlockAnimationState {
    pub x: f32,
    pub y: f32,
    pub timer: f32,
    pub frame: u32,
}

impl LegacyCoinBlockAnimationState {
    #[must_use]
    pub const fn spawn(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            timer: 0.0,
            frame: 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyCoinBlockAnimationScore {
    pub score_delta: i32,
    pub floating_score: u32,
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyCoinBlockAnimationUpdate {
    pub remove: bool,
    pub score: Option<LegacyCoinBlockAnimationScore>,
}

#[must_use]
pub fn update_legacy_coin_block_animation(
    state: &mut LegacyCoinBlockAnimationState,
    constants: LegacyCoinBlockAnimationConstants,
    dt: f32,
) -> LegacyCoinBlockAnimationUpdate {
    state.timer += dt;

    while state.timer > constants.animation_delay {
        state.frame += 1;
        state.timer -= constants.animation_delay;
    }

    let score = (state.frame >= constants.remove_frame).then_some(LegacyCoinBlockAnimationScore {
        score_delta: 0,
        floating_score: constants.floating_score,
        x: state.x,
        y: state.y,
    });

    LegacyCoinBlockAnimationUpdate {
        remove: score.is_some(),
        score,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyFireworkFrame {
    One,
    Two,
    Three,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyFireworkBoomState {
    pub x: f32,
    pub y: f32,
    pub timer: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyFireworkBoomSpawn {
    pub state: LegacyFireworkBoomState,
    pub score_delta: u32,
}

impl LegacyFireworkBoomState {
    #[must_use]
    pub fn spawn(
        x: f32,
        random_x_delta: f32,
        random_y: f32,
        constants: LegacyFireworkConstants,
    ) -> LegacyFireworkBoomSpawn {
        LegacyFireworkBoomSpawn {
            state: Self {
                x: x + random_x_delta,
                y: random_y,
                timer: 0.0,
            },
            score_delta: constants.score_delta,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyFireworkBoomUpdate {
    pub remove: bool,
    pub play_sound: bool,
}

#[must_use]
pub fn update_legacy_firework_boom(
    state: &mut LegacyFireworkBoomState,
    constants: LegacyFireworkConstants,
    dt: f32,
) -> LegacyFireworkBoomUpdate {
    let previous_timer = state.timer;
    state.timer += dt;

    LegacyFireworkBoomUpdate {
        remove: state.timer > constants.duration,
        play_sound: state.timer >= constants.sound_time && previous_timer < constants.sound_time,
    }
}

#[must_use]
pub fn legacy_firework_boom_frame(
    state: &LegacyFireworkBoomState,
    constants: LegacyFireworkConstants,
) -> LegacyFireworkFrame {
    let frame_length = constants.duration / 3.0;

    if state.timer > frame_length * 2.0 {
        LegacyFireworkFrame::Three
    } else if state.timer > frame_length {
        LegacyFireworkFrame::Two
    } else {
        LegacyFireworkFrame::One
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LegacyBlockDebrisState, LegacyBlockDebrisUpdate, LegacyCoinBlockAnimationScore,
        LegacyCoinBlockAnimationState, LegacyCoinBlockAnimationUpdate, LegacyFireworkBoomState,
        LegacyFireworkBoomUpdate, LegacyFireworkFrame, LegacyScrollingScoreLabel,
        LegacyScrollingScorePresentation, LegacyScrollingScoreState, LegacyScrollingScoreUpdate,
        legacy_firework_boom_frame, legacy_scrolling_score_presentation,
        update_legacy_block_debris, update_legacy_coin_block_animation,
        update_legacy_firework_boom, update_legacy_scrolling_score,
    };
    use crate::config::{
        LegacyBlockDebrisConstants, LegacyCoinBlockAnimationConstants, LegacyFireworkConstants,
        LegacyScrollingScoreConstants,
    };

    #[test]
    fn scrolling_score_spawn_captures_scroll_relative_x_and_label() {
        let state = LegacyScrollingScoreState::spawn(
            LegacyScrollingScoreLabel::Points(200),
            12.0,
            6.0,
            3.25,
        );

        assert_eq!(state.x, 8.75);
        assert_eq!(state.y, 6.0);
        assert_eq!(state.label, LegacyScrollingScoreLabel::Points(200));
        assert_eq!(state.timer, 0.0);
    }

    #[test]
    fn scrolling_score_update_removes_after_strict_lifetime() {
        let constants = LegacyScrollingScoreConstants::default();
        let mut state =
            LegacyScrollingScoreState::spawn(LegacyScrollingScoreLabel::OneUp, 12.0, 6.0, 3.25);

        assert_eq!(
            update_legacy_scrolling_score(&mut state, constants, constants.lifetime),
            LegacyScrollingScoreUpdate { remove: false }
        );
        assert_eq!(state.timer, constants.lifetime);

        assert_eq!(
            update_legacy_scrolling_score(&mut state, constants, 0.001),
            LegacyScrollingScoreUpdate { remove: true }
        );
        assert_eq!(state.timer, constants.lifetime + 0.001);
    }

    #[test]
    fn scrolling_score_presentation_derives_numeric_draw_position() {
        let constants = LegacyScrollingScoreConstants::default();
        let mut state = LegacyScrollingScoreState::spawn(
            LegacyScrollingScoreLabel::Points(800),
            12.0,
            6.0,
            3.25,
        );
        state.timer = constants.lifetime / 2.0;

        assert_eq!(
            legacy_scrolling_score_presentation(&state, constants),
            LegacyScrollingScorePresentation {
                x: 8.75 - constants.numeric_x_offset,
                y: 6.0
                    - constants.y_base_offset
                    - constants.rise_height * (state.timer / constants.lifetime),
                label: LegacyScrollingScoreLabel::Points(800),
            }
        );
    }

    #[test]
    fn scrolling_score_presentation_keeps_one_up_x_without_numeric_offset() {
        let constants = LegacyScrollingScoreConstants::default();
        let mut state =
            LegacyScrollingScoreState::spawn(LegacyScrollingScoreLabel::OneUp, 12.0, 6.0, 3.25);
        state.timer = constants.lifetime;

        assert_eq!(
            legacy_scrolling_score_presentation(&state, constants),
            LegacyScrollingScorePresentation {
                x: 8.75,
                y: 6.0 - constants.y_base_offset - constants.rise_height,
                label: LegacyScrollingScoreLabel::OneUp,
            }
        );
    }

    #[test]
    fn block_debris_spawn_matches_legacy_initial_state() {
        let state = LegacyBlockDebrisState::spawn(9.5, 4.0, 3.5, -23.0);

        assert_eq!(state.x, 9.5);
        assert_eq!(state.y, 4.0);
        assert_eq!(state.speed_x, 3.5);
        assert_eq!(state.speed_y, -23.0);
        assert_eq!(state.timer, 0.0);
        assert_eq!(state.frame, 1);
    }

    #[test]
    fn block_debris_animation_uses_strict_toggle_delay() {
        let constants = LegacyBlockDebrisConstants::default();
        let mut state = LegacyBlockDebrisState::spawn(9.5, 4.0, 0.0, 0.0);

        assert_eq!(
            update_legacy_block_debris(&mut state, constants, constants.animation_time),
            LegacyBlockDebrisUpdate { remove: false }
        );
        assert_eq!(state.frame, 1);
        assert_eq!(state.timer, constants.animation_time);

        let _ = update_legacy_block_debris(&mut state, constants, 0.001);
        assert_eq!(state.frame, 2);
        assert!((state.timer - 0.001).abs() < f32::EPSILON);
    }

    #[test]
    fn block_debris_animation_can_toggle_multiple_times() {
        let constants = LegacyBlockDebrisConstants::default();
        let mut state = LegacyBlockDebrisState::spawn(9.5, 4.0, 0.0, 0.0);

        let _ = update_legacy_block_debris(
            &mut state,
            constants,
            constants.animation_time * 3.0 + 0.001,
        );

        assert_eq!(state.frame, 2);
        assert!((state.timer - 0.001).abs() < 0.000_001);
    }

    #[test]
    fn block_debris_applies_gravity_before_motion() {
        let constants = LegacyBlockDebrisConstants::default();
        let mut state = LegacyBlockDebrisState::spawn(1.0, 2.0, 3.0, -4.0);

        assert_eq!(
            update_legacy_block_debris(&mut state, constants, 0.5),
            LegacyBlockDebrisUpdate { remove: false }
        );
        assert_eq!(state.speed_y, -4.0 + constants.gravity * 0.5);
        assert_eq!(state.x, 2.5);
        assert_eq!(state.y, 15.0);
    }

    #[test]
    fn block_debris_removal_uses_strict_y_threshold_after_motion() {
        let constants = LegacyBlockDebrisConstants::default();
        let mut state = LegacyBlockDebrisState::spawn(1.0, constants.removal_y, 0.0, 0.0);

        assert_eq!(
            update_legacy_block_debris(&mut state, constants, 0.0),
            LegacyBlockDebrisUpdate { remove: false }
        );

        assert_eq!(
            update_legacy_block_debris(&mut state, constants, 0.001),
            LegacyBlockDebrisUpdate { remove: true }
        );
        assert!(state.y > constants.removal_y);
    }

    #[test]
    fn coin_block_animation_spawn_matches_legacy_initial_state() {
        let state = LegacyCoinBlockAnimationState::spawn(9.5, 4.0);

        assert_eq!(state.x, 9.5);
        assert_eq!(state.y, 4.0);
        assert_eq!(state.timer, 0.0);
        assert_eq!(state.frame, 1);
    }

    #[test]
    fn coin_block_animation_uses_strict_frame_delay() {
        let constants = LegacyCoinBlockAnimationConstants::default();
        let mut state = LegacyCoinBlockAnimationState::spawn(9.5, 4.0);

        assert_eq!(
            update_legacy_coin_block_animation(&mut state, constants, constants.animation_delay),
            LegacyCoinBlockAnimationUpdate {
                remove: false,
                score: None
            }
        );
        assert_eq!(state.frame, 1);
        assert_eq!(state.timer, constants.animation_delay);

        let _ = update_legacy_coin_block_animation(&mut state, constants, 0.001);
        assert_eq!(state.frame, 2);
        assert!((state.timer - 0.001).abs() < f32::EPSILON);
    }

    #[test]
    fn coin_block_animation_advances_multiple_frames_before_remove_score() {
        let constants = LegacyCoinBlockAnimationConstants::default();
        let mut state = LegacyCoinBlockAnimationState::spawn(9.5, 4.0);
        state.frame = constants.remove_frame - 2;

        assert_eq!(
            update_legacy_coin_block_animation(
                &mut state,
                constants,
                constants.animation_delay * 2.0 + 0.001,
            ),
            LegacyCoinBlockAnimationUpdate {
                remove: true,
                score: Some(LegacyCoinBlockAnimationScore {
                    score_delta: 0,
                    floating_score: 200,
                    x: 9.5,
                    y: 4.0,
                })
            }
        );
        assert_eq!(state.frame, constants.remove_frame);
        assert!((state.timer - 0.001).abs() < f32::EPSILON);
    }

    #[test]
    fn coin_block_animation_remove_threshold_is_inclusive() {
        let constants = LegacyCoinBlockAnimationConstants::default();
        let mut state = LegacyCoinBlockAnimationState::spawn(9.5, 4.0);
        state.frame = constants.remove_frame;

        assert_eq!(
            update_legacy_coin_block_animation(&mut state, constants, 0.0),
            LegacyCoinBlockAnimationUpdate {
                remove: true,
                score: Some(LegacyCoinBlockAnimationScore {
                    score_delta: 0,
                    floating_score: 200,
                    x: 9.5,
                    y: 4.0,
                })
            }
        );
        assert_eq!(state.frame, constants.remove_frame);
    }

    #[test]
    fn firework_boom_spawn_uses_injected_random_offsets_and_scores_immediately() {
        let constants = LegacyFireworkConstants::default();
        let spawn = LegacyFireworkBoomState::spawn(10.0, -4.0, 3.0, constants);

        assert_eq!(spawn.state.x, 6.0);
        assert_eq!(spawn.state.y, 3.0);
        assert_eq!(spawn.state.timer, 0.0);
        assert_eq!(spawn.score_delta, 200);
    }

    #[test]
    fn firework_boom_update_plays_sound_on_crossing_sound_time_only() {
        let constants = LegacyFireworkConstants::default();
        let mut state = LegacyFireworkBoomState::spawn(10.0, 0.0, 3.0, constants).state;

        assert_eq!(
            update_legacy_firework_boom(&mut state, constants, constants.sound_time),
            LegacyFireworkBoomUpdate {
                remove: false,
                play_sound: true
            }
        );

        assert_eq!(
            update_legacy_firework_boom(&mut state, constants, 0.0),
            LegacyFireworkBoomUpdate {
                remove: false,
                play_sound: false
            }
        );

        let mut crossing = LegacyFireworkBoomState::spawn(10.0, 0.0, 3.0, constants).state;
        crossing.timer = constants.sound_time - 0.01;
        assert_eq!(
            update_legacy_firework_boom(&mut crossing, constants, 0.02),
            LegacyFireworkBoomUpdate {
                remove: false,
                play_sound: true
            }
        );
    }

    #[test]
    fn firework_boom_update_removes_after_strict_duration() {
        let constants = LegacyFireworkConstants::default();
        let mut state = LegacyFireworkBoomState::spawn(10.0, 0.0, 3.0, constants).state;

        assert_eq!(
            update_legacy_firework_boom(&mut state, constants, constants.duration),
            LegacyFireworkBoomUpdate {
                remove: false,
                play_sound: true
            }
        );

        assert_eq!(
            update_legacy_firework_boom(&mut state, constants, 0.01),
            LegacyFireworkBoomUpdate {
                remove: true,
                play_sound: false
            }
        );
    }

    #[test]
    fn firework_boom_frame_uses_strict_thirds_of_duration() {
        let constants = LegacyFireworkConstants::default();
        let mut state = LegacyFireworkBoomState::spawn(10.0, 0.0, 3.0, constants).state;
        let frame_length = constants.duration / 3.0;

        state.timer = frame_length;
        assert_eq!(
            legacy_firework_boom_frame(&state, constants),
            LegacyFireworkFrame::One
        );

        state.timer = frame_length + 0.01;
        assert_eq!(
            legacy_firework_boom_frame(&state, constants),
            LegacyFireworkFrame::Two
        );

        state.timer = frame_length * 2.0;
        assert_eq!(
            legacy_firework_boom_frame(&state, constants),
            LegacyFireworkFrame::Two
        );

        state.timer = frame_length * 2.0 + 0.01;
        assert_eq!(
            legacy_firework_boom_frame(&state, constants),
            LegacyFireworkFrame::Three
        );
    }
}
