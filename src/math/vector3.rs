use std::ops::{Add, Div, Mul, Sub};
use crate::math::traits::Vector;
use crate::math::vector2::Vector2;

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn xy(&self) -> Vector2 {
        Vector2::from([
            self.x,
            self.y
        ])
    }

    pub fn yz(&self) -> Vector2 {
        Vector2::from([
            self.y,
            self.z
        ])
    }

    pub fn xz(&self) -> Vector2 {
        Vector2::from([
            self.x,
            self.z
        ])
    }

    pub fn xyz(&self) -> (f32, f32, f32) {
        (self.x, self.y, self.z)
    }

    pub fn cross(self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
}

impl Add for Vector3 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Sub for Vector3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl Mul<f32> for Vector3 {
    type Output = Self;

    fn mul(self, other: f32) -> Self {
        Self {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
        }
    }
}

impl Mul<Vector3> for f32 {
    type Output = Vector3;

    fn mul(self, rhs: Vector3) -> Self::Output {
        rhs * self
    }
}

impl Div<f32> for Vector3 {
    type Output = Self;

    fn div(self, other: f32) -> Self {
        Self {
            x: self.x / other,
            y: self.y / other,
            z: self.z / other,
        }
    }
}

impl Div<Vector3> for f32 {
    type Output = Vector3;

    fn div(self, rhs: Vector3) -> Self::Output {
        rhs / self
    }
}

impl Vector for Vector3 {
    fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    fn length(&self) -> f32 {
        self.x.hypot(self.y.hypot(self.z))
    }

    fn normalize(self) -> Self {
        let length = self.length();

        Self {
            x: self.x / length,
            y: self.y / length,
            z: self.z / length,
        }
    }
}

impl From<[f32; 3]> for Vector3 {
    fn from(v: [f32; 3]) -> Self {
        Self {
            x: v[0],
            y: v[1],
            z: v[2],
        }
    }
}

impl From<Vector2> for Vector3 {
    fn from(v: Vector2) -> Self {
        Self {
            z: 0.0,
            ..v.into()
        }
    }
}
