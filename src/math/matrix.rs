//! Dense matrix type and basic linear algebra operations.
//!
//! # Generic scalar type
//!
//! The matrix is parameterized by a scalar type `K`:
//!
//! - By default, `Matrix` means `Matrix<f32`.
//! - For the bonus exercise, the same type can be instantiated as
//!     `Matrix<Complex>` to represent matrix over the complex field.
//!
//! Operations are implemented under progressively stronger trait bounds,
//! ranking from basic algebraic operations (`Add`, `Mul`) to full field
//! requirements (`Field`) for elimination-based algorithms.
//!
//! # Storage layout
//!
//! Matrices are stored in **column-major** order (Fortran / BLAS style).
//! For an element at row `r` and column `c`, the linear index is:
//!
//! ```text
//! index = c * rows + r
//! ```
//!
//! This layout is cache-friendly for column-wise traversals and naturally
//! matches matrix-vector and matrix-matrix multiplication patterns.
//!
//! # Numerical notes
//!
//! Algorithms base on Gaussian elimination (row echelon form, rank,
//! determinant, inverse) rely on a small numerical tolerance (`EPS`) to decide
//! whether a value should be considered zero.
//!
//! This tolerance is applied to the scalar magnitude with the `Abs` trait,
//! allowing the code to work for both real and complex scalars.
//!
//! # Panics and errors
//!
//! Many methods use `debug_assert!` / `debug_assert_eq!` to validate
//! preconditions (dimension checks, bounds).
//!
//! These checks are enabled in debug builds and removes in release builds.
//!
//! The `inverse()` method returns a `Result` because non-invertibility is a
//! normal outcome that must be handled by callers.

use crate::math::vector::Vector;
use crate::math::scalar::{One, Zero, Field};
use core::ops::{Add, Mul, Sub};

/// Numerical tolerance used to treat small values as zero in elimination-based algorithms.
const EPS: f32 = 1e-6;

/// A dense matrix over a scalar type `K`, stored in column-major order.
///
/// The underlying storage is a contiguous `Vec<K>` with length `rows * cols`.
///
/// # Type parameters
/// - `K`: scalar type (defaults to `f32`).
///
/// # Invariants
/// - `data.len() == rows * cols`
/// - Elements are indexed as `(r, c) -> c * rows + r` (column-major).
///
/// # Examples
/// ```
/// use matrix::Matrix;
///
/// // 2x3 matrix filled with zeros
/// let m: Matrix<f32> = Matrix::zeros(2, 3);
/// assert_eq!(m.rows(), 2);
/// assert_eq!(m.cols(), 3);
/// ```
///
/// ```
/// use matrix::{Matrix, Complex};
///
/// // Identity matrix over the complex field
/// let i: Matrix<Complex> = Matrix::identity(2);
///```
#[derive(Debug, Clone, PartialEq)]
pub struct Matrix<K = f32> {
    /// Elements stored in column-major order.
    data: Vec<K>,
    rows: usize,
    cols: usize,
}

impl<K> Matrix<K> {
    /// Creates a new matrix from raw column-major data.
    ///
    /// # Parameters
    /// - `data`: Elements in column-major order.
    /// - `rows`: Number of rows.
    /// - `cols`: Number of columns.
    ///
    /// # Panics (debug)
    /// Panics in debug builds if `rows == 0`, `cols == 0`, `data.len() != rows * cols`.
    pub fn new(data: Vec<K>, rows: usize, cols: usize) -> Self {
        debug_assert!(rows > 0 && cols > 0, "matrix dimensions must be non-zero");
        debug_assert_eq!(data.len(), rows * cols, "data size mismatch");
        Self { data, rows, cols }
    }

    pub fn rows(&self) -> usize {
        self.rows
    }
    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn is_square(&self) -> bool {
        self.rows == self.cols
    }

    /// Returns the underlying storage is a slice in column-major order.
    ///
    /// This exposes the raw layout. Prefer `get()`/`set()` for index-based access.
    pub fn as_slice(&self) -> &[K] {
        &self.data
    }

    #[inline]
    fn index(&self, r: usize, c: usize) -> usize {
        debug_assert!(r < self.rows, "row out of bounds");
        debug_assert!(c < self.cols, "col out of bounds");
        c * self.rows + r // column-major
    }

    /// Sets the elements at row `r`, column `c` to `value`.
    ///
    /// # Panics (debug)
    /// Panics in debug builds if `r` or `c` is out of bounds.
    pub fn set(&mut self, r: usize, c: usize, value: K) {
        let i = self.index(r, c);
        self.data[i] = value;
    }
}

impl<K: Copy> Matrix<K> {
    /// Returns the element at row `r`, column `c`.
    ///
    /// # Panics (debug)
    /// Panics in debug builds if `r` or `c` is out of bounds.
    pub fn get(&self, r: usize, c: usize) -> K {
        let i = self.index(r, c);
        self.data[i]
    }
}

impl<K: Zero + Copy> Matrix<K> {
    /// Creates a `rows`x`cols` matrix filled with zeros.
    pub fn zeros(rows: usize, cols: usize) -> Self {
        Self::new(vec![K::zero(); rows * cols], rows, cols)
    }
}

impl<K: Zero + One + Copy> Matrix<K> {
    /// Creates an identity matrix of size `n`x`n`.
    ///
    /// # Examples
    /// ```
    /// use matrix::Matrix;
    /// let i: Matrix<f32> = Matrix::identity(3);
    /// assert_eq!(i.get(0, 0), 1.0);
    /// assert_eq!(i.get(0, 1), 0.0);
    /// ```
    pub fn identity(n: usize) -> Self {
        let mut m = Self::zeros(n, n);
        for i in 0..n {
            m.set(i, i, K::one());
        }
        m
    }
}

impl<K: Copy + Add<Output = K> + Sub<Output = K>> Matrix<K> {
    pub fn add(&mut self, other: &Matrix<K>) {
        debug_assert_eq!(self.rows, other.rows, "row mismatch");
        debug_assert_eq!(self.cols, other.cols, "col mismatch");

        for i in 0..self.data.len() {
            self.data[i] = self.data[i] + other.data[i];
        }
    }

    pub fn sub(&mut self, other: &Matrix<K>) {
        debug_assert_eq!(self.rows, other.rows, "row mismatch");
        debug_assert_eq!(self.cols, other.cols, "col mismatch");

        for i in 0..self.data.len() {
            self.data[i] = self.data[i] - other.data[i];
        }
    }
}

impl<K: Copy + Mul<Output = K>> Matrix<K> {
    pub fn scl(&mut self, a: K) {
        for x in &mut self.data {
            *x =  *x * a;
        }
    }

    pub fn scale(&mut self, a: K) {
        self.scl(a);
    }
}

impl<K: Copy + Zero + Add<Output = K> + Mul<Output = K>> Matrix<K> {
    /// Multiplies this matrix (mxn) by a vector of length `n`,
    /// returning a vector of length `m`.
    ///
    /// This implementation is optimized for colum-major storage.
    ///
    /// # Panics (debug)
    /// Panics in debug builds if `self.cols() != v.len()`.
    pub fn mul_vec(&self, v: &Vector<K>) -> Vector<K> {
        debug_assert_eq!(
            self.cols,
            v.len(),
            "dimension mismatch: matrix cols vs vector len"
        );

        let rows = self.rows;
        let cols = self.cols;

        let mut out = vec![K::zero(); rows];

        // Column-major friendly traversal:
        // data index for (r, c) is c * rows + r
        let vx = v.as_slice();

        for c in 0..cols {
            let a = vx[c];
            let col_base = c * rows;

            for r in 0..rows {
                out[r] = out[r] + self.data[col_base + r] * a;
            }
        }

        Vector::new(out)
    }

    /// Multiplies this matrix (mxn) by another matrix (nxp),
    /// returning a matrix of size (mxp).
    ///
    /// Both matrices are assumed to be stored in column-major order.
    ///
    /// # Panics (debug)
    /// Panics in debug builds if `self.cols() != other.rows()`
    pub fn mul_mat(&self, other: &Matrix<K>) -> Matrix<K> {
        debug_assert_eq!(
            self.cols, other.rows,
            "dimension mismatch: self.cols vs other.rows"
        );

        let m = self.rows;
        let n = self.cols;
        let p = other.cols;

        let mut out = vec![K::zero(); m * p];

        for c in 0..p {
            let out_col_base = c * m;
            let b_col_base = c * n;

            for k in 0..n {
                let b_kc = other.data[b_col_base + k];
                let a_col_base = k * m;

                for r in 0..m {
                    out[out_col_base + r] = out[out_col_base + r] + self.data[a_col_base + r] * b_kc;
                }
            }
        }

        Matrix::new(out, m, p)
    }
}

impl<K: Copy + Zero + Add<Output = K>> Matrix<K> {
    /// Returns the trace of the matrix.
    ///
    /// The trace is defined only for square matrices and is equal to the sum
    /// of diagonal elements.
    ///
    /// # Panics (debug)
    /// Panics in debug builds if the matrix is not square.
    pub fn trace(&self) -> K {
        debug_assert_eq!(self.rows, self.cols, "trace requires a square matrix");

        let mut sum = K::zero();
        for i in 0..self.rows {
            sum = sum + self.data[i * self.rows + i];
        }

        sum
    }
} 
impl<K: Copy + Zero> Matrix<K> {
    /// Returns the transpose of the matrix.
    ///
    /// If the matrix has size mxn, the result has size nxm.
    ///
    /// The transpose is computed by reindexing elements while preserving
    /// column-major storage in the output matrix.
    pub fn transpose(&self) -> Matrix<K>
    {
        let mut out = vec![K::zero(); self.rows * self.cols];

        for r in 0..self.rows {
            for c in 0..self.cols {
                let src = c * self.rows + r;
                let dst = r * self.cols + c;
                out[dst] = self.data[src];
            }
        }

        Matrix::new(out, self.cols, self.rows)
    }
}

impl<K: Field> Matrix<K> {
    fn row_echelon_with_swaps(&self) -> (Matrix<K>, usize) {
        let mut m = self.clone();

        let rows = m.rows;
        let cols = m.cols;
        let mut pivot_row = 0;
        let mut swaps = 0;

        for col in 0..cols {
            if pivot_row >= rows {
                break;
            }

            let mut pivot = None;
            for r in pivot_row..rows {
                if m.data[col * rows + r].abs() > EPS {
                    pivot = Some(r);
                    break;
                }
            }

            let Some(pivot) = pivot else {
                continue;
            };

            if pivot != pivot_row {
                for c in 0..cols {
                    let i1 = c * rows + pivot_row;
                    let i2 = c * rows + pivot;
                    m.data.swap(i1, i2);
                }
                swaps += 1;
            }

            let pivot_val = m.data[col * rows + pivot_row];

            for r in (pivot_row + 1)..rows {
                let factor = m.data[col * rows + r] / pivot_val;

                if factor.abs() <= EPS {
                    continue;
                }

                for c in col..cols {
                    let i_r = c * rows + r;
                    let i_p = c * rows + pivot_row;
                    m.data[i_r] = m.data[i_r] - factor * m.data[i_p];
                }
            }

            pivot_row += 1;
        }

        (m, swaps)
    }

    /// Returns the row echelon form (REF) of the matrix using Gaussian elimination.
    ///
    /// The returned matrix has zeros below each pivot (up to numerical tolerance `EPS`).
    /// Row swaps are performed as needed to select valid pivots.
    pub fn row_echelon(&self) -> Matrix<K> {
        self.row_echelon_with_swaps().0
    }

    /// Returns the rank of the matrix.
    ///
    /// The rank is computed as the number of non-zero rows in the row echelon form,
    /// where a row is considered non-zero if at least one element has magnitude
    /// greated than `EPS`.
    pub fn rank(&self) -> usize {
        let ref_m = self.row_echelon();
        let rows = ref_m.rows;
        let cols = ref_m.cols;

        let mut rank = 0;

        for r in 0..rows {
            let mut non_zero = false;

            for c in 0..cols {
                if ref_m.data[c * rows + r].abs() > EPS {
                    non_zero = true;
                    break;
                }
            }

            if non_zero {
                rank += 1;
            }
        }

        rank
    }

    /// Returns the determinant of the matrix.
    ///
    /// The determinant is computed using Gaussian elimination.
    /// Row swaps are tracked to adjust the sign of the result.
    ///
    /// The result belongs to the same scalar field as the matrix coefficients.
    ///
    /// # Panics (debug)
    /// Panics in debug builds if the matrix is not square.
    pub fn determinant(&self) -> K {
        debug_assert_eq!(self.rows, self.cols, "determinant requires a square matrix");

        let (u, swaps) = self.row_echelon_with_swaps();

        let n = u.rows;
        let mut det = K::one();

        for i in 0..n {
            let diag = u.data[i * n + i];

            if diag.abs() <= EPS {
                return K::zero();
            }

            det = det * diag;
        }

        if swaps % 2 == 1 {
            det = -det;
        }

        det
    }

    /// Returns the inverse matrix using Gauss-Jordan elimination.
    ///
    /// The matrix must be square and non-singular.
    ///
    /// # Errors
    /// Returns `Err` if the matrix is not square or is singular
    /// (i.e. has determinant zero within numerical tolerance).
    pub fn inverse(&self) -> Result<Matrix<K>, &'static str> {
        if self.rows != self.cols {
            return Err("inverse requires a square matrix");
        }

        let n = self.rows;
        let rows = n;
        let cols = 2 * n;

        // Build augmented matrix [A | I]
        let mut aug = vec![K::zero(); rows * cols];

        // Copy A into the left block
        for c in 0..n {
            let src_base = c * n;
            let dst_base = c * rows;
            for r in 0..n {
                aug[dst_base + r] = self.data[src_base + r];
            }
        }

        // Put the identity into the right block
        for i in 0..n {
            let c = n + i;
            let idx = c * rows + i;
            aug[idx] = K::one();
        }

        // Gauss-Jordan elimination on the left block
        for col in 0..n {
            // 1) Find pivot row
            let mut pivot = None;
            for r in col..n {
                let v = aug[col * rows + r];
                if v.abs() > EPS {
                    pivot = Some(r);
                    break;
                }
            }

            let Some(pivot_row) = pivot else {
                return Err("matrix is singular");
            };

            // 2) Swap current row with pivot row
            if pivot_row != col {
                for c in 0..cols {
                    let i1 = c * rows + col;
                    let i2 = c * rows + pivot_row;
                    aug.swap(i1, i2);
                }
            }

            // 3) Normalize pivot row so that pivot becomes 1
            let pivot_val = aug[col * rows + col];
            if pivot_val.abs() <= EPS {
                return Err("matrix is singular");
            }

            for c in 0..cols {
                let i = c * rows + col;
                aug[i] = aug[i] / pivot_val;
            }

            // 4) Eliminate all other rows in this column
            for r in 0..n {
                if r == col {
                    continue;
                }

                let factor = aug[col * rows + r];
                if factor.abs() <= EPS {
                    continue;
                }

                for c in 0..cols {
                    let i_r = c * rows + r;
                    let i_p = c * rows + col;
                    aug[i_r] = aug[i_r] - factor * aug[i_p];
                }
            }
        }

        // Extract the right block as the inverse
        let mut inv = vec![K::zero(); n * n];
        for c in 0..n {
            let aug_c = n + c;
            let src_base = aug_c * rows;
            let dst_base = c * n;
            for r in 0..n {
                inv[dst_base + r] = aug[src_base + r];
            }
        }

        Ok(Matrix::new(inv, n, n))
    }
}

#[cfg(test)]
mod tests {

    fn assert_f32_approx_eq(a: f32, b: f32, eps: f32) {
        assert!(
            (a - b).abs() <= eps,
            "expected approx equal: a={} b={} (diff={})",
            a,
            b,
            (a - b).abs()
        );
    }

    fn assert_matrix_approx_eq(a: &super::Matrix, b: &super::Matrix, eps: f32) {
        assert_eq!(a.rows(), b.rows(), "row mismatch");
        assert_eq!(a.cols(), b.cols(), "cols mismatch");

        let rows = a.rows();
        let cols = a.cols();

        for c in 0..cols {
            for r in 0..rows {
                let av = a.get(r, c);
                let bv = b.get(r, c);
                assert_f32_approx_eq(av, bv, eps);
            }
        }
    }

    use super::Matrix;

    #[test]
    fn get_set_column_major() {
        let mut m = Matrix::zeros(2, 3);

        m.set(0, 0, 1.0);
        m.set(1, 0, 2.0);
        m.set(0, 1, 3.0);
        m.set(1, 1, 4.0);

        assert_f32_approx_eq(m.get(0, 0), 1.0, 1e-5);
        assert_f32_approx_eq(m.get(1, 0), 2.0, 1e-5);
        assert_f32_approx_eq(m.get(0, 1), 3.0, 1e-5);
        assert_f32_approx_eq(m.get(1, 1), 4.0, 1e-5);
    }

    #[test]
    fn add_sub_scl_work() {
        let mut a = Matrix::new(vec![1.0, 2.0, 3.0, 4.0], 2, 2);
        let b = Matrix::new(vec![10.0, 20.0, 30.0, 40.0], 2, 2);

        a.add(&b);
        assert_matrix_approx_eq(&a, &Matrix::new(vec![11.0, 22.0, 33.0, 44.0], 2, 2), 1e-5);

        a.sub(&b);
        assert_matrix_approx_eq(&a, &Matrix::new(vec![1.0, 2.0, 3.0, 4.0], 2, 2), 1e-5);

        a.scl(2.0);
        assert_matrix_approx_eq(&a, &Matrix::new(vec![2.0, 4.0, 6.0, 8.0], 2, 2), 1e-5);
    }

    #[test]
    fn mul_vec_identity() {
        use super::Matrix;
        use crate::math::vector::{Vector};

        let m = Matrix::identity(3);

        let v = Vector::new(vec![5.0, -2.0, 7.0]);
        let r = m.mul_vec(&v);

        assert_eq!(r, v);
    }

    #[test]
    fn mul_vec_general_case() {
        use super::Matrix;
        use crate::math::vector::{Vector};

        let a = Matrix::new(vec![1.0, 4.0, 2.0, 5.0, 3.0, 6.0], 2, 3);
        let x = Vector::new(vec![10.0, 20.0, 30.0]);

        let y = a.mul_vec(&x);

        assert_eq!(y, Vector::new(vec![140.0, 320.0]));
    }

    #[test]
    fn mul_mat_identity() {
        use super::Matrix;

        let i = Matrix::identity(3);

        let a = Matrix::new(vec![1.0, 4.0, 7.0, 2.0, 5.0, 8.0, 3.0, 6.0, 9.0], 3, 3);

        assert_matrix_approx_eq(&a.mul_mat(&i), &a, 1e-5);
        assert_matrix_approx_eq(&i.mul_mat(&a), &a, 1e-5);
    }

    #[test]
    fn mul_mat_rectangular() {
        use super::Matrix;

        let a = Matrix::new(vec![1.0, 4.0, 2.0, 5.0, 3.0, 6.0], 2, 3);
        let b = Matrix::new(vec![7.0, 9.0, 11.0, 8.0, 10.0, 12.0], 3, 2);

        let c = a.mul_mat(&b);

        assert_matrix_approx_eq(&c, &Matrix::new(vec![58.0, 139.0, 64.0, 154.0], 2, 2), 1e-5);
    }

    #[test]
    fn trace_works() {
        use super::Matrix;

        let m = Matrix::new(vec![1.0, 4.0, 7.0, 2.0, 5.0, 8.0, 3.0, 6.0, 9.0], 3, 3);

        assert_f32_approx_eq(m.trace(), 15.0, 1e-5);
    }

    #[test]
    fn tranpose_works() {
        use super::Matrix;

        let a = Matrix::new(vec![1.0, 4.0, 2.0, 5.0, 3.0, 6.0], 2, 3);

        let at = a.transpose();

        let expected = Matrix::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], 3, 2);

        assert_matrix_approx_eq(&at, &expected, 1e-5);
    }

    #[test]
    fn row_echelon_basic() {
        use super::Matrix;

        let a = Matrix::new(vec![1.0, 4.0, 7.0, 2.0, 5.0, 8.0, 3.0, 6.0, 9.0], 3, 3);

        let r = a.row_echelon();

        assert_f32_approx_eq(r.get(2, 0), 0.0, 1e-5);
        assert_f32_approx_eq(r.get(2, 1), 0.0, 1e-5);
        assert_f32_approx_eq(r.get(2, 2), 0.0, 1e-5);
    }

    #[test]
    fn rank_full() {
        use super::Matrix;

        let m = Matrix::new(vec![1.0, 4.0, 7.0, 2.0, 5.0, 8.0, 3.0, 6.0, 10.0], 3, 3);

        assert_eq!(m.rank(), 3);
    }

    #[test]
    fn rank_deficient() {
        use super::Matrix;

        let m = Matrix::new(vec![1.0, 2.0, 3.0, 2.0, 4.0, 6.0, 3.0, 6.0, 9.0], 3, 3);

        assert_eq!(m.rank(), 1);
    }

    #[test]
    fn rank_rectangular() {
        use super::Matrix;

        let m = Matrix::new(vec![1.0, 4.0, 2.0, 5.0, 3.0, 6.0], 3, 2);

        assert_eq!(m.rank(), 2);
    }

    #[test]
    fn determinant_2x2() {
        use super::Matrix;

        let m = Matrix::new(vec![1.0, 3.0, 2.0, 4.0], 2, 2);
        assert_f32_approx_eq(m.determinant(), -2.0, 1e-5);
    }

    #[test]
    fn determinant_identity() {
        use super::Matrix;

        let i: Matrix<f32> = Matrix::identity(3);
        assert_f32_approx_eq(i.determinant(), 1.0, 1e-5);
    }

    #[test]
    fn determinant_singular() {
        use super::Matrix;

        let m = Matrix::new(vec![1.0, 2.0, 2.0, 4.0], 2, 2);
        assert_f32_approx_eq(m.determinant(), 0.0, 1e-5);
    }

    #[test]
    fn inverse_2x2() {
        use super::Matrix;

        let a = Matrix::new(vec![1.0, 3.0, 2.0, 4.0], 2, 2);
        let inv = a.inverse().unwrap();

        let expected = Matrix::new(vec![-2.0, 1.5, 1.0, -0.5], 2, 2);
        assert_matrix_approx_eq(&inv, &expected, 1e-5);
    }

    #[test]
    fn inverse_singular_fails() {
        use super::Matrix;

        let a = Matrix::new(vec![1.0, 2.0, 2.0, 4.0], 2, 2);
        assert!(a.inverse().is_err());
    }

    #[test]
    fn inverse_multiplies_to_identity() {
        use super::Matrix;

        let a = Matrix::new(vec![1.0, 3.0, 2.0, 4.0], 2, 2);
        let inv = a.inverse().unwrap();

        let prod = a.mul_mat(&inv);

        let i = Matrix::identity(2);
        assert_matrix_approx_eq(&prod, &i, 1e-5);
    }

    #[test]
    fn transpose_is_involution() {
        use super::Matrix;

        let a = Matrix::new(vec![1.0, 4.0, 7.0, 2.0, 5.0, 8.0, 3.0, 6.0, 10.0], 3, 3);

        let att = a.transpose().transpose();
        assert_matrix_approx_eq(&att, &a, 1e-5);
    }

    #[test]
    fn inverse_multiplies_to_identity_approx() {
        use super::Matrix;

        let a = Matrix::new(vec![1.0, 3.0, 2.0, 4.0], 2, 2);
        let inv = a.inverse().unwrap();

        let prod = a.mul_mat(&inv);
        let i = Matrix::identity(2);

        assert_matrix_approx_eq(&prod, &i, 1e-5);
    }

    #[test]
    fn inverse_right_multiplies_to_identity_approx() {
        use super::Matrix;

        let a = Matrix::new(vec![1.0, 3.0, 2.0, 4.0], 2, 2);
        let inv = a.inverse().unwrap();

        let prod = inv.mul_mat(&a);
        let i = Matrix::identity(2);

        assert_matrix_approx_eq(&prod, &i, 1e-5);
    }

    #[test]
    fn rank_deficient_implies_zero_determinant() {
        use super::Matrix;

        let a = Matrix::new(vec![1.0, 2.0, 3.0, 2.0, 4.0, 6.0, 3.0, 6.0, 9.0], 3, 3);

        assert!(a.rank() < 3);
        assert_f32_approx_eq(a.determinant(), 0.0, 1e-5);
    }

    #[test]
    fn transpose_of_product() {
        use super::Matrix;

        let a = Matrix::new(vec![1.0, 4.0, 2.0, 5.0, 3.0, 6.0], 2, 3);
        let b = Matrix::new(vec![7.0, 9.0, 11.0, 8.0, 10.0, 12.0], 3, 2);

        let left = a.mul_mat(&b).transpose();
        let right = b.transpose().mul_mat(&a.transpose());

        assert_matrix_approx_eq(&left, &right, 1e-5);
    }
}
