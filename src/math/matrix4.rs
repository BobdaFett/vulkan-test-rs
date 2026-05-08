use std::ops::{Add, Div, Mul, Sub};
use crate::math::traits::{Matrix, Vector};
use crate::math::Vector4;

#[derive(Debug, Clone, Default, Copy, PartialEq)]
pub struct Matrix4 {
    row1: Vector4,
    row2: Vector4,
    row3: Vector4,
    row4: Vector4,
}

impl Matrix4 {
}

impl Add for Matrix4 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            row1: self.row1 + rhs.row1,
            row2: self.row2 + rhs.row2,
            row3: self.row3 + rhs.row3,
            row4: self.row4 + rhs.row4,
        }
    }
}

impl Sub for Matrix4 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            row1: self.row1 - rhs.row1,
            row2: self.row2 - rhs.row2,
            row3: self.row3 - rhs.row3,
            row4: self.row4 - rhs.row4,
        }
    }
}

impl Div<f32> for Matrix4 {
    type Output = Self;

    /// Equivalent to multiplying by the reciprocal of `rhs`.
    fn div(self, rhs: f32) -> Self::Output {
        let rhs = 1.0 / rhs;
        self * rhs
    }
}

impl Mul<f32> for Matrix4 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            row1: self.row1 * rhs,
            row2: self.row2 * rhs,
            row3: self.row3 * rhs,
            row4: self.row4 * rhs,
        }
    }
}

impl Mul for Matrix4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let rhs = rhs.transpose();
        Self {
            row1: [self.row1.dot(rhs.row1), self.row1.dot(rhs.row2), self.row1.dot(rhs.row3), self.row1.dot(rhs.row4)].into(),
            row2: [self.row2.dot(rhs.row1), self.row2.dot(rhs.row2), self.row2.dot(rhs.row3), self.row2.dot(rhs.row4)].into(),
            row3: [self.row3.dot(rhs.row1), self.row3.dot(rhs.row2), self.row3.dot(rhs.row3), self.row3.dot(rhs.row4)].into(),
            row4: [self.row4.dot(rhs.row1), self.row4.dot(rhs.row2), self.row4.dot(rhs.row3), self.row4.dot(rhs.row4)].into(),
        }
    }
}

impl Mul<Vector4> for Matrix4 {
    type Output = Vector4;

    fn mul(self, rhs: Vector4) -> Vector4 {
        Vector4 {
            x: self.row1.dot(rhs),
            y: self.row2.dot(rhs),
            z: self.row3.dot(rhs),
            w: self.row4.dot(rhs),
        }
    }
}

impl Matrix<Vector4> for Matrix4 {
    fn identity() -> Self {
        Self {
            row1: Vector4::new(1.0, 0.0, 0.0, 0.0),
            row2: Vector4::new(0.0, 1.0, 0.0, 0.0),
            row3: Vector4::new(0.0, 0.0, 1.0, 0.0),
            row4: Vector4::new(0.0, 0.0, 0.0, 1.0),
        }
    }

    fn transpose(&self) -> Self {
        Self {
            row1: [self.row1.x, self.row2.x, self.row3.x, self.row4.x].into(),
            row2: [self.row1.y, self.row2.y, self.row3.y, self.row4.y].into(),
            row3: [self.row1.z, self.row2.z, self.row3.z, self.row4.z].into(),
            row4: [self.row1.w, self.row2.w, self.row3.w, self.row4.w].into(),
        }
    }

    fn determinant(&self) -> f32 {
        todo!()
    }

    fn minor(&self) -> Self {
        todo!()
    }

    fn cofactor(&self) -> Self {
        todo!()
    }

    fn adjugate(&self) -> Self {
        todo!()
    }

    fn inverse(&self) -> Self {
        todo!()
    }
}
