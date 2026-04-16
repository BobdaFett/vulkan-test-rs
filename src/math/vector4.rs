use std::ops::{Add, Div, Mul, Sub};
use crate::math::traits::Vector;
use crate::math::{Vector2, Vector3};

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vector4 {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn xyzw(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.z, self.w)
    }

    pub fn xyz(&self) -> Vector3 {
        Vector3::from([self.x, self.y, self.z])
    }
    
    pub fn xy(&self) -> Vector2 {
        Vector2::from([self.x, self.y])
    }
    
    pub fn yz(&self) -> Vector2 {
        Vector2::from([self.y, self.z])
    }
    
    pub fn xz(&self) -> Vector2 {
        Vector2::from([self.x, self.z])
    }
}

impl Add for Vector4 {
    type Output = Self;
    
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w,
        }
    }
}

impl Sub for Vector4 {
    type Output = Self;
    
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
            w: self.w - other.w,
        }
    }
}

impl Mul<f32> for Vector4 {
    type Output = Self;
    
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
            w: self.w * rhs,
        }
    }
}

impl Mul<Vector4> for f32 {
    type Output = Vector4;
    
    fn mul(self, rhs: Vector4) -> Self::Output {
        rhs * self
    }
}

impl Div<f32> for Vector4 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
            w: self.w / rhs,
        }
    }
}

impl Div<Vector4> for f32 {
    type Output = Vector4;
    
    fn div(self, rhs: Vector4) -> Self::Output {
        rhs / self
    }
}

impl Vector for Vector4 {
    fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    fn length(&self) -> f32 {
        self.x.hypot(self.y.hypot(self.z.hypot(self.w)))
    }

    fn normalize(self) -> Self {
        self / self.length()
    }
}

impl From<[f32; 4]> for Vector4 {
    fn from(v: [f32; 4]) -> Self {
        Self {
            x: v[0],
            y: v[1],
            z: v[2],
            w: v[3],
        }
    }
}

impl From<(Vector3, f32)> for Vector4 {
    fn from((v, n): (Vector3, f32)) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
            w: n,
        }
    }
}

impl From<(Vector2, f32, f32)> for Vector4 {
    fn from((v, n1, n2): (Vector2, f32, f32)) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: n1,
            w: n2,
        }
    }
}
