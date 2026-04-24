use core::ops::{Add, Mul, Sub};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    pub min: Vec2,
    pub max: Vec2,
}

impl Aabb {
    #[must_use]
    pub const fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    #[must_use]
    pub fn from_xywh(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self::new(Vec2::new(x, y), Vec2::new(x + width, y + height))
    }

    #[must_use]
    pub fn translated(self, delta: Vec2) -> Self {
        Self::new(self.min + delta, self.max + delta)
    }

    #[must_use]
    pub fn width(self) -> f32 {
        self.max.x - self.min.x
    }

    #[must_use]
    pub fn height(self) -> f32 {
        self.max.y - self.min.y
    }

    #[must_use]
    pub fn intersects(self, other: Self) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
    }
}

#[cfg(test)]
mod tests {
    use super::{Aabb, Vec2};

    #[test]
    fn aabb_intersections_exclude_touching_edges() {
        let left = Aabb::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0));
        let touching = Aabb::new(Vec2::new(1.0, 0.0), Vec2::new(2.0, 1.0));
        let overlapping = Aabb::new(Vec2::new(0.5, 0.5), Vec2::new(1.5, 1.5));

        assert!(!left.intersects(touching));
        assert!(left.intersects(overlapping));
    }
}
