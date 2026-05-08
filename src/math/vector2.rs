use std::ops::{Add, Div, Mul, Sub, Index, IndexMut};
use crate::math::traits::Vector;

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_values(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Index<usize> for Vector2 {
    type Output = f32;
    fn index(&self, idx: usize) -> &f32 {
        match idx {
            0 => &self.x,
            1 => &self.y,
            _ => panic!("Index out of bounds"),
        }
    }
}

impl IndexMut<usize> for Vector2 {
    fn index_mut(&mut self, idx: usize) -> &mut f32 {
        match idx {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => panic!("Index out of bounds"),
        }
    }
}

impl Add for Vector2 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Vector2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<f32> for Vector2 {
    type Output = Self;

    fn mul(self, other: f32) -> Self {
        Self {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl Mul<Vector2> for f32 {
    type Output = Vector2;

    fn mul(self, rhs: Vector2) -> Self::Output {
        rhs * self
    }
}

impl Div<f32> for Vector2 {
    type Output = Self;

    fn div(self, other: f32) -> Self {
        Self {
            x: self.x / other,
            y: self.y / other,
        }
    }
}

impl Div<Vector2> for f32 {
    type Output = Vector2;

    fn div(self, rhs: Vector2) -> Self::Output {
        rhs / self
    }
}

impl Vector for Vector2 {
    fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    fn length(&self) -> f32 {
        self.x.hypot(self.y)
    }

    fn normalize(self) -> Self {
        let length = self.length();
        Self {
            x: self.x / length,
            y: self.y / length,
        }
    }
}

impl From<[f32; 2]> for Vector2 {
    fn from(v: [f32; 2]) -> Self {
        Self {
            x: v[0],
            y: v[1],
        }
    }
}

impl From<f32> for Vector2 {
    fn from(v: f32) -> Self {
        Self {
            x: v,
            y: 0.0,
        }
    }
}
