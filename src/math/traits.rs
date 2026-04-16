use std::ops::{Add, Div, Mul, Sub};

pub trait Vector: Copy + Default + PartialEq + PartialOrd + Add + Sub + Div<f32, Output = Self> + Mul<f32, Output = Self> {
    /// Gets the dot product of the two vectors.
    fn dot(self, other: Self) -> f32;

    /// Gets the signed length of the vector.
    fn length(&self) -> f32;

    /// Creates a normalized vector.
    fn normalize(self) -> Self;

    /// Projects the given vector onto the current vector. For example:
    /// ```
    /// let v = <some vector>;
    /// let u = <another vector>;
    /// let vu_projection = v.project(u);  // projects u onto v
    /// let uv_projection = u.project(v);  // projects v onto u
    /// ```
    fn project(self, other: Self) -> Self {
        let dot = self.dot(other);
        let length = self.length();
        let full_length = length * length;

        self * (dot / full_length)
    }
}

pub trait Matrix: Copy + Default + PartialEq + PartialOrd + Add + Sub
    + Div<f32, Output = Self> + Div
    + Mul<f32, Output = Self> + Mul {
    /// Creates an identity matrix.
    fn identity() -> Self;
    
    /// Gets the transpose of this matrix.
    fn transpose(self) -> Self;

    /// Gets the determinant of this matrix.
    fn determinant(self) -> f32;

    /// Finds the minor of this matrix. The element at location (`i`, `j`) of the minor is the
    /// determinant of the submatrix obtained by removing row `i` and column `j` in the original
    /// matrix.
    fn minor(self) -> Self;

    /// Finds the cofactor of this matrix. The element at location (`i`, `j`) of the cofactor is
    /// defined as -1^ij * Mij, where Mij is the element at (`i`, `j`) in the minor of the matrix.
    /// This is effectively flipping the sign of every other element in the minor of this matrix.
    fn cofactor(self) -> Self;

    /// Gets the adjugate of this matrix. This is the transpose of the cofactor of this matrix.
    fn adjugate(self) -> Self;

    /// Gets the inverse of this matrix. Note that this is a very expensive operation and should
    /// be used as sparingly as possible.
    fn inverse(self) -> Self;
}
