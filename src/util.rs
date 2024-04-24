use std::ops::{Add, Div, Mul, Neg, Sub};

pub type GenericResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Vec2I(pub i32, pub i32);
impl Add for Vec2I {
    type Output = Self;
    fn add(self, Self(x1, y1): Self) -> Self::Output {
        let Self(x0, y0) = self;
        Self(x0 + x1, y0 + y1)
    }
}
impl Sub for Vec2I {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
    }
}
impl Mul<i32> for Vec2I {
    type Output = Self;
    fn mul(self, rhs: i32) -> Self::Output {
        let Self(x, y) = self;

        Self(x * rhs, y * rhs)
    }
}
impl Neg for Vec2I {
    type Output = Self;
    fn neg(self) -> Self::Output {
        let Self(x, y) = self;
        Self(-x, -y)
    }
}
impl Div<i32> for Vec2I {
    type Output = Self;
    fn div(self, rhs: i32) -> Self::Output {
        let Self(x, y) = self;
        Self(x / rhs, y / rhs)
    }
}
impl From<(i32, i32)> for Vec2I {
    fn from((x, y): (i32, i32)) -> Self {
        Self(x, y)
    }
}

#[derive(Debug, Clone, Copy, Hash)]
pub struct RectI {
    pub pos: Vec2I,
    pub dim: Vec2I,
}
impl RectI {
    pub fn aspect(&self) -> f32 {
        let Vec2I(width, height) = self.dim;
        return height as f32 / width as f32;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vec2(f64, f64);
impl Add for Vec2 {
    type Output = Self;
    fn add(self, Self(x1, y1): Self) -> Self::Output {
        let Self(x0, y0) = self;
        Self(x0 + x1, y0 + y1)
    }
}
impl Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
    }
}
impl Mul<f64> for Vec2 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        let Self(x, y) = self;

        Self(x * rhs, y * rhs)
    }
}
impl Neg for Vec2 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        let Self(x, y) = self;
        Self(-x, -y)
    }
}
impl Div<f64> for Vec2 {
    type Output = Self;
    fn div(self, rhs: f64) -> Self::Output {
        let Self(x, y) = self;
        Self(x / rhs, y / rhs)
    }
}
