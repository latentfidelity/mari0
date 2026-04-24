use crate::math::Vec2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Facing {
    Up,
    Right,
    Down,
    Left,
}

impl Facing {
    #[must_use]
    pub const fn normal(self) -> Vec2 {
        match self {
            Self::Up => Vec2::new(0.0, -1.0),
            Self::Right => Vec2::new(1.0, 0.0),
            Self::Down => Vec2::new(0.0, 1.0),
            Self::Left => Vec2::new(-1.0, 0.0),
        }
    }

    #[must_use]
    pub const fn tangent(self) -> Vec2 {
        match self {
            Self::Up => Vec2::new(1.0, 0.0),
            Self::Right => Vec2::new(0.0, 1.0),
            Self::Down => Vec2::new(-1.0, 0.0),
            Self::Left => Vec2::new(0.0, -1.0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WormholeEndpoint {
    pub center: Vec2,
    pub facing: Facing,
}

impl WormholeEndpoint {
    #[must_use]
    pub const fn new(center: Vec2, facing: Facing) -> Self {
        Self { center, facing }
    }

    #[must_use]
    fn local_components(self, value: Vec2) -> (f32, f32) {
        let relative = value - self.center;
        (
            relative.dot(self.facing.tangent()),
            relative.dot(self.facing.normal()),
        )
    }

    #[must_use]
    fn local_velocity(self, velocity: Vec2) -> (f32, f32) {
        (
            velocity.dot(self.facing.tangent()),
            velocity.dot(self.facing.normal()),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WormholePair {
    pub a: WormholeEndpoint,
    pub b: WormholeEndpoint,
}

impl WormholePair {
    #[must_use]
    pub const fn new(a: WormholeEndpoint, b: WormholeEndpoint) -> Self {
        Self { a, b }
    }

    #[must_use]
    pub fn transit_a_to_b(
        self,
        position: Vec2,
        velocity: Vec2,
        exit_offset: f32,
    ) -> WormholeTransit {
        transit(self.a, self.b, position, velocity, exit_offset)
    }

    #[must_use]
    pub fn transit_b_to_a(
        self,
        position: Vec2,
        velocity: Vec2,
        exit_offset: f32,
    ) -> WormholeTransit {
        transit(self.b, self.a, position, velocity, exit_offset)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WormholeTransit {
    pub position: Vec2,
    pub velocity: Vec2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnimationDirection {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyPortalEndpoint {
    pub x: f32,
    pub y: f32,
    pub facing: Facing,
}

impl LegacyPortalEndpoint {
    #[must_use]
    pub const fn new(x: f32, y: f32, facing: Facing) -> Self {
        Self { x, y, facing }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyPortalTransitInput {
    pub position: Vec2,
    pub velocity: Vec2,
    pub size: Vec2,
    pub rotation: f32,
    pub animation_direction: Option<AnimationDirection>,
    pub entry: LegacyPortalEndpoint,
    pub exit: LegacyPortalEndpoint,
    pub live: bool,
    pub gravity: f32,
    pub frame_dt: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LegacyPortalTransit {
    pub position: Vec2,
    pub velocity: Vec2,
    pub rotation: f32,
    pub animation_direction: Option<AnimationDirection>,
}

#[must_use]
fn transit(
    source: WormholeEndpoint,
    destination: WormholeEndpoint,
    position: Vec2,
    velocity: Vec2,
    exit_offset: f32,
) -> WormholeTransit {
    let (tangent_position, _) = source.local_components(position);
    let (tangent_velocity, normal_velocity) = source.local_velocity(velocity);

    WormholeTransit {
        position: destination.center
            + destination.facing.tangent() * tangent_position
            + destination.facing.normal() * exit_offset,
        velocity: destination.facing.tangent() * tangent_velocity
            + destination.facing.normal() * normal_velocity.abs(),
    }
}

#[must_use]
pub fn legacy_portal_coords(input: LegacyPortalTransitInput) -> LegacyPortalTransit {
    let mut x = input.position.x + input.size.x / 2.0;
    let mut y = input.position.y + input.size.y / 2.0;
    let mut speed_x = input.velocity.x;
    let mut speed_y = input.velocity.y;
    let mut rotation = input.rotation;
    let mut animation_direction = input.animation_direction;

    let (direct_range, relative_range) =
        legacy_portal_ranges(x, y, input.size.x, input.size.y, input.entry);

    let new_x;
    let mut new_y;

    match (input.entry.facing, input.exit.facing) {
        (Facing::Up, Facing::Up) => {
            new_x = x + (input.exit.x - input.entry.x);
            new_y = input.exit.y + direct_range - 1.0;
            speed_y = -speed_y;
            rotation -= core::f32::consts::PI;
            apply_up_exit_min_speed(&mut speed_y, input);
        }
        (Facing::Down, Facing::Down) => {
            new_x = x + (input.exit.x - input.entry.x);
            new_y = input.exit.y - direct_range;
            speed_y = -speed_y;
            rotation -= core::f32::consts::PI;
        }
        (Facing::Up, Facing::Right) => {
            new_y = input.exit.y - relative_range * (2.0 - input.size.y) - input.size.y / 2.0 + 1.0;
            new_x = input.exit.x - direct_range;
            (speed_x, speed_y) = (speed_y, -speed_x);
            rotation -= core::f32::consts::FRAC_PI_2;
        }
        (Facing::Up, Facing::Left) => {
            new_y = input.exit.y + relative_range * (2.0 - input.size.y) + input.size.y / 2.0 - 2.0;
            new_x = input.exit.x + direct_range - 1.0;
            (speed_x, speed_y) = (-speed_y, speed_x);
            rotation += core::f32::consts::FRAC_PI_2;
        }
        (Facing::Up, Facing::Down) => {
            let mut clamped_x = x + (input.exit.x - input.entry.x) - 1.0;
            new_y = input.exit.y - direct_range;

            if input.entry.y > input.exit.y {
                while new_y + 0.5 + speed_y * input.frame_dt > input.entry.y {
                    new_y -= 0.01;
                }

                while new_y + 0.5 < input.exit.y {
                    new_y += 0.01;
                }
            }

            let min_x = input.exit.x - 2.0 + input.size.x / 2.0;
            let max_x = input.exit.x - input.size.x / 2.0;
            if clamped_x <= min_x {
                clamped_x = min_x;
            } else if clamped_x > max_x {
                clamped_x = max_x;
            }
            new_x = clamped_x;
        }
        (Facing::Down, Facing::Up) => {
            new_x = x + (input.exit.x - input.entry.x) + 1.0;
            new_y = input.exit.y + direct_range - 1.0;
        }
        (Facing::Down, Facing::Left) => {
            new_y = input.exit.y - relative_range * (2.0 - input.size.y) - input.size.y / 2.0;
            new_x = input.exit.x + direct_range - 1.0;
            (speed_x, speed_y) = (speed_y, -speed_x);
            rotation -= core::f32::consts::FRAC_PI_2;
        }
        (Facing::Down, Facing::Right) => {
            new_y = input.exit.y + relative_range * (2.0 - input.size.y) + input.size.y / 2.0 - 1.0;
            new_x = input.exit.x - direct_range;
            (speed_x, speed_y) = (-speed_y, speed_x);
            rotation += core::f32::consts::FRAC_PI_2;
        }
        (Facing::Left, Facing::Right) => {
            new_x = input.exit.x - direct_range;
            new_y = y + (input.exit.y - input.entry.y) + 1.0;
        }
        (Facing::Right, Facing::Left) => {
            new_x = input.exit.x + direct_range - 1.0;
            new_y = y + (input.exit.y - input.entry.y) - 1.0;
        }
        (Facing::Right, Facing::Right) => {
            new_x = input.exit.x - direct_range;
            new_y = y + (input.exit.y - input.entry.y);
            speed_x = -speed_x;
            animation_direction = flip_animation_direction(animation_direction);
        }
        (Facing::Left, Facing::Left) => {
            new_x = input.exit.x + direct_range - 1.0;
            new_y = y + (input.exit.y - input.entry.y);
            speed_x = -speed_x;
            animation_direction = flip_animation_direction(animation_direction);
        }
        (Facing::Left, Facing::Up) => {
            new_x = input.exit.x + relative_range * (2.0 - input.size.x) + input.size.x / 2.0 - 1.0;
            new_y = input.exit.y + direct_range - 1.0;
            (speed_x, speed_y) = (speed_y, -speed_x);
            rotation -= core::f32::consts::FRAC_PI_2;
            apply_up_exit_min_speed(&mut speed_y, input);
        }
        (Facing::Right, Facing::Up) => {
            new_x = input.exit.x - relative_range * (2.0 - input.size.x) - input.size.x / 2.0 + 1.0;
            new_y = input.exit.y + direct_range - 1.0;
            (speed_x, speed_y) = (-speed_y, speed_x);
            rotation += core::f32::consts::FRAC_PI_2;
            apply_up_exit_min_speed(&mut speed_y, input);
        }
        (Facing::Left, Facing::Down) => {
            new_x = input.exit.x - relative_range * (2.0 - input.size.x) - input.size.x / 2.0;
            new_y = input.exit.y - direct_range;
            (speed_x, speed_y) = (-speed_y, speed_x);
            rotation += core::f32::consts::FRAC_PI_2;
        }
        (Facing::Right, Facing::Down) => {
            new_x = input.exit.x + relative_range * (2.0 - input.size.x) + input.size.x / 2.0 - 2.0;
            new_y = input.exit.y - direct_range;
            (speed_x, speed_y) = (speed_y, -speed_x);
            rotation -= core::f32::consts::FRAC_PI_2;
        }
    }

    x = new_x - input.size.x / 2.0;
    y = new_y - input.size.y / 2.0;

    LegacyPortalTransit {
        position: Vec2::new(x, y),
        velocity: Vec2::new(speed_x, speed_y),
        rotation,
        animation_direction,
    }
}

#[must_use]
fn legacy_portal_ranges(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    entry: LegacyPortalEndpoint,
) -> (f32, f32) {
    match entry.facing {
        Facing::Up => {
            let direct_range = entry.y - y - 1.0;
            let relative_range = if width == 2.0 {
                0.0
            } else {
                ((x - width / 2.0) - entry.x + 1.0) / (2.0 - width)
            };
            (direct_range, relative_range)
        }
        Facing::Right => {
            let direct_range = x - entry.x;
            let relative_range = if height == 2.0 {
                0.0
            } else {
                ((y - height / 2.0) - entry.y + 1.0) / (2.0 - height)
            };
            (direct_range, relative_range)
        }
        Facing::Down => {
            let direct_range = y - entry.y;
            let relative_range = if width == 2.0 {
                0.0
            } else {
                ((x - width / 2.0) - entry.x + 2.0) / (2.0 - width)
            };
            (direct_range, relative_range)
        }
        Facing::Left => {
            let direct_range = entry.x - x - 1.0;
            let relative_range = if height == 2.0 {
                0.0
            } else {
                ((y - height / 2.0) - entry.y + 2.0) / (2.0 - height)
            };
            (direct_range, relative_range)
        }
    }
}

fn apply_up_exit_min_speed(speed_y: &mut f32, input: LegacyPortalTransitInput) {
    if !input.live {
        return;
    }

    let min_speed = (2.0 * input.gravity * input.size.y).sqrt();
    if *speed_y > -min_speed {
        *speed_y = -min_speed;
    }
}

#[must_use]
fn flip_animation_direction(direction: Option<AnimationDirection>) -> Option<AnimationDirection> {
    match direction {
        Some(AnimationDirection::Left) => Some(AnimationDirection::Right),
        Some(AnimationDirection::Right) => Some(AnimationDirection::Left),
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AnimationDirection, Facing, LegacyPortalEndpoint, LegacyPortalTransitInput,
        WormholeEndpoint, WormholePair, legacy_portal_coords,
    };
    use crate::math::Vec2;

    #[test]
    fn facing_basis_uses_unit_axes() {
        assert_eq!(Facing::Up.normal(), Vec2::new(0.0, -1.0));
        assert_eq!(Facing::Right.tangent(), Vec2::new(0.0, 1.0));
    }

    #[test]
    fn transit_preserves_tangent_offset_and_exits_destination_normal() {
        let pair = WormholePair::new(
            WormholeEndpoint::new(Vec2::new(10.0, 10.0), Facing::Right),
            WormholeEndpoint::new(Vec2::new(20.0, 20.0), Facing::Up),
        );

        let transit = pair.transit_a_to_b(Vec2::new(10.0, 12.0), Vec2::new(5.0, 2.0), 0.25);

        assert_eq!(transit.position, Vec2::new(22.0, 19.75));
        assert_eq!(transit.velocity, Vec2::new(2.0, -5.0));
    }

    #[test]
    fn legacy_portal_up_to_right_matches_mari0_formula() {
        let transit = legacy_portal_coords(LegacyPortalTransitInput {
            position: Vec2::new(4.0, 8.0),
            velocity: Vec2::new(2.0, 5.0),
            size: Vec2::new(1.0, 1.0),
            rotation: 0.0,
            animation_direction: None,
            entry: LegacyPortalEndpoint::new(4.0, 9.0, Facing::Up),
            exit: LegacyPortalEndpoint::new(20.0, 3.0, Facing::Right),
            live: false,
            gravity: 80.0,
            frame_dt: 1.0 / 60.0,
        });

        assert_vec_close(transit.position, Vec2::new(20.0, 2.0));
        assert_vec_close(transit.velocity, Vec2::new(5.0, -2.0));
        assert_close(transit.rotation, -core::f32::consts::FRAC_PI_2);
    }

    #[test]
    fn legacy_portal_same_horizontal_facing_flips_velocity_and_animation() {
        let transit = legacy_portal_coords(LegacyPortalTransitInput {
            position: Vec2::new(9.25, 4.0),
            velocity: Vec2::new(3.0, 0.0),
            size: Vec2::new(1.0, 1.0),
            rotation: 0.25,
            animation_direction: Some(AnimationDirection::Left),
            entry: LegacyPortalEndpoint::new(10.0, 4.0, Facing::Right),
            exit: LegacyPortalEndpoint::new(30.0, 8.0, Facing::Right),
            live: false,
            gravity: 80.0,
            frame_dt: 1.0 / 60.0,
        });

        assert_vec_close(transit.position, Vec2::new(29.75, 8.0));
        assert_vec_close(transit.velocity, Vec2::new(-3.0, 0.0));
        assert_close(transit.rotation, 0.25);
        assert_eq!(transit.animation_direction, Some(AnimationDirection::Right));
    }

    #[test]
    fn legacy_portal_live_up_exit_applies_minimum_speed() {
        let transit = legacy_portal_coords(LegacyPortalTransitInput {
            position: Vec2::new(5.5, 6.0),
            velocity: Vec2::new(2.0, 0.0),
            size: Vec2::new(1.0, 1.0),
            rotation: 0.0,
            animation_direction: None,
            entry: LegacyPortalEndpoint::new(5.0, 6.0, Facing::Left),
            exit: LegacyPortalEndpoint::new(12.0, 3.0, Facing::Up),
            live: true,
            gravity: 80.0,
            frame_dt: 1.0 / 60.0,
        });

        assert_vec_close(transit.position, Vec2::new(13.0, -0.5));
        assert_close(transit.velocity.x, 0.0);
        assert_close(transit.velocity.y, -(160.0_f32).sqrt());
        assert_close(transit.rotation, -core::f32::consts::FRAC_PI_2);
    }

    fn assert_vec_close(actual: Vec2, expected: Vec2) {
        assert_close(actual.x, expected.x);
        assert_close(actual.y, expected.y);
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.0001,
            "expected {actual} to be close to {expected}"
        );
    }
}
