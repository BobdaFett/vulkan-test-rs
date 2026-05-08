use crate::math::Vector2;
use crate::math::traits::{Matrix, Vector};
use std::ops::{Add, Index, IndexMut, Mul, Sub};

#[derive(Clone, Debug, PartialEq, PartialOrd, Copy, Default)]
pub struct Matrix2 {
    row1: Vector2,
    row2: Vector2,
}

impl Matrix2 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn row_vector(&self, row: usize) -> Vector2 {
        Vector2::from([self[row][0], self[row][1]])
    }

    pub fn column_vector(&self, col: usize) -> Vector2 {
        Vector2::from([self[0][col], self[1][col]])
    }
}

impl Index<usize> for Matrix2 {
    type Output = Vector2;

    fn index(&self, idx: usize) -> &Self::Output {
        match idx {
            0 => &self.row1,
            1 => &self.row2,
            _ => panic!("Index out of bounds"),
        }
    }
}

impl IndexMut<usize> for Matrix2 {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        match idx {
            0 => &mut self.row1,
            1 => &mut self.row2,
            _ => panic!("Index out of bounds"),
        }
    }
}

impl Add for Matrix2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            row1: [self[0][0] + rhs[0][0], self[0][1] + rhs[0][1]].into(),
            row2: [self[1][0] + rhs[1][0], self[1][1] + rhs[1][1]].into(),
        }
    }
}

impl Sub for Matrix2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            row1: [self[0][0] - rhs[0][0], self[0][1] - rhs[0][1]].into(),
            row2: [self[1][0] - rhs[1][0], self[1][1] - rhs[1][1]].into(),
        }
    }
}

impl Mul<f32> for Matrix2 {
    type Output = Matrix2;
    fn mul(self, rhs: f32) -> Matrix2 {
        Self {
            row1: self.row1 * rhs,
            row2: self.row2 * rhs,
        }
    }
}

impl Mul<Vector2> for Matrix2 {
    type Output = Vector2;

    fn mul(self, rhs: Vector2) -> Self::Output {
        Vector2 {
            x: self[0].dot(rhs),
            y: self[1].dot(rhs),
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
        let other = rhs.transpose();
        Self {
            row1: [self[0].dot(other[0]), self[0].dot(other[1])].into(),
            row2: [self[1].dot(other[0]), self[1].dot(other[1])].into(),
        }
    }
}

impl Matrix<Vector2> for Matrix2 {
    fn identity() -> Self {
        Self {
            row1: [1.0, 0.0].into(),
            row2: [0.0, 1.0].into(),
        }
    }

    fn transpose(&self) -> Self {
        Self {
            row1: [self[0][0], self[1][0]].into(),
            row2: [self[0][1], self[1][1]].into(),
        }
    }

    fn determinant(&self) -> f32 {
        self[0][0] * self[1][1] - self[0][1] * self[1][0]
    }

    fn minor(&self) -> Self {
        Self {
            row1: [self[1][1], self[1][0]].into(),
            row2: [self[0][1], self[0][0]].into(),
        }
    }

    fn cofactor(&self) -> Self {
        let mut minor = self.minor();
        for row in 0..2 {
            for col in 0..2 {
                let even = (row + col) % 2 == 0;
                if even {
                    minor[row][col] = -minor[row][col];
                }
            }
        }

        minor
    }

    fn adjugate(&self) -> Self {
        todo!()
    }

    fn inverse(&self) -> Self {
        let adj = self.adjugate();
        let det = self.determinant();

        (1.0 / det) * adj
    }
}
