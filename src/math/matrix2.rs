use std::ops::{Add, Div, Index, IndexMut, Mul, Sub};
use crate::math::traits::Matrix;
use crate::math::Vector2;

#[derive(Clone, Debug, PartialEq, PartialOrd, Copy, Default)]
pub struct Matrix2 {
    row1: [f32; 2],
    row2: [f32; 2],
}

impl Matrix2 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn row_vector(&self, row: usize) -> Vector2 {
        Vector2::from([
            self[row][0],
            self[row][1],
        ])
    }

    pub fn column_vector(&self, col: usize) -> Vector2 {
        Vector2::from([
            self[0][col],
            self[1][col],
        ])
    }
}

impl Index<usize> for Matrix2 {
    type Output = [f32; 2];

    fn index(&self, index: usize) -> &Self::Output {
        if index == 0 {
            &self.row1
        } else {
            &self.row2
        }
    }
}

impl IndexMut<usize> for Matrix2 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index == 0 {
            &mut self.row1
        } else {
            &mut self.row2
        }
    }
}

impl Add for Matrix2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            row1: [self[0][0] + rhs[0][0], self[0][1] + rhs[0][1]],
            row2: [self[1][0] + rhs[1][0], self[1][1] + rhs[1][1]],
        }
    }
}

impl Sub for Matrix2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            row1: [self[0][0] - rhs[0][0], self[0][1] - rhs[0][1]],
            row2: [self[1][0] - rhs[1][0], self[1][1] - rhs[1][1]],
        }
    }
}

impl Mul<f32> for Matrix2 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            row1: [self[0][0] * rhs, self[0][1] * rhs],
            row2: [self[1][0] * rhs, self[1][1] * rhs],
        }
    }
}

impl Mul<Matrix2> for f32 {
    type Output = Matrix2;

    fn mul(self, rhs: Matrix2) -> Self::Output {
        rhs * self
    }
}

impl Mul for Matrix2 {
    type Output = Self;

    fn mul(self, rhs: Matrix2) -> Self::Output {
        Self {
            row1: [self[0][0] * rhs[0][0] + self[0][1] * rhs[1][0], self[0][0] * rhs[0][1] + self[0][1] * rhs[1][1]],
            row2: [self[1][0] * rhs[0][0] + self[1][1] * rhs[1][0], self[1][0] * rhs[0][1] + self[1][1] * rhs[1][1]],
        }
    }
}

impl Div<f32> for Matrix2 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        todo!()
    }
}

impl Div for Matrix2 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl Matrix for Matrix2 {
    fn identity() -> Self {
        Self {
            row1: [1.0, 0.0],
            row2: [0.0, 1.0],
        }
    }

    fn transpose(self) -> Self {
        todo!()
    }

    fn determinant(self) -> f32 {
        todo!()
    }

    fn minor(self) -> Self {
        todo!()
    }

    fn cofactor(self) -> Self {
        todo!()
    }

    fn adjugate(self) -> Self {
        todo!()
    }

    fn inverse(self) -> Self {
        todo!()
    }
}
