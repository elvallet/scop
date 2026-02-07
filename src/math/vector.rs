//! Vector math primitives.
//!
//! This module provides a generic dense vector type `Vector<K>` and common
//! linear algebra operations.
//!
//! # Generic scalar type
//!
//! The vector is parameterized by a scalar type `K`:
//!
//! - By default, `Vector` means `Vector<f32>`.
//! - For the bonus exercise, the same type can be instantiated as
//!     `Vector<Complex>` to represent vector spaces over the complex field.
//!
//! The scalar type `K` is constrained on a per-method basis using small,
//! purpose-built traits (see [`crate::scalar`]) instead of relying on a single
//! monolithic numeric trait.
//!
//! # Storage layout
//!
//! The underlying storage is a contiguous `Vec<K>` storing vector components
//! in logical order:
//!
//! ```text
//! [x₀, x₁, x₂, …, xₙ₋₁]
//! ```
//!
//! # Algebraic conventions
//!
//! - The dot product is implemented as a **Hermitian inner product**:
//!
//!     \[ ⟨u, v⟩ = Σ conj(uᵢ) · vᵢ \]
//!
//!     - For real scalars (`f32`), `conj(x) = x`, so this reduces to the usual
//!       Euclidean dot product.
//!     - For complex scalars, this ensures that ⟨u, u⟩ is real and non-negative.
//!
//! - Norms are derived from the scalar magnitude (`abs` / `abs2`), and therefore
//!   behave correctly for both real and complex vector spaces.
//!
//! # Panics and errors
//!
//! Many operations use `debug_assert!` / `debug_assert_eq!` to validate
//! preconditions (dimensions checks, non-empty inputs, etc.).
//!
//! These checks are enabled in debug builds and removed in release builds.

use std::{fmt};
use crate::math::scalar::{Abs, Conj, One, Zero};
use core::ops::{Add, Mul, Sub};

/// A dense mathematical vector over a scalar type `K`.
///
/// The underlying storage is a contiguous `Vec<K>`.
///
/// # Type parameter
/// - `K`: Scalar type (defaults to `f32`).
///
/// # Invariants
/// - The vector length is `data.len()`.
/// - Operations that combine multiple vectors assume matching lengths.
///
/// # Examples
/// ```
/// use matrix::Vector;
///
/// let mut u = Vector::new(vec![2.0, 3.0]);
/// let v = Vector::new(vec![2.0, 1.0]);
///
/// u.add(&v);
/// assert_eq!(u, Vector::new(vec![4.0, 4.0]));
///```
#[derive(Debug, Clone, PartialEq)]
pub struct Vector<K = f32> {
    data: Vec<K>,
}

impl <K> Vector<K> {
    /// Creates a new vector from the given components.
    ///
    /// # Parameters
    /// - `data`: Components of the vector.
    ///
    /// # Notes
    /// This constructor does not enforce non-emptiness.
    pub fn new(data: Vec<K>) -> Self {
        Self { data }
    }

    /// Returns the number of components in the vector.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns the vector data as a read-only slice.
    ///
    /// This is useful to interoperate with APIs expecting `&[f32]`.
    pub fn as_slice(&self) -> &[K] {
        &self.data
    }
}

impl<K: Zero + Copy> Vector<K> {
   /// Creates a vector filled with zeros.
    pub fn zeros(n: usize) -> Self {
        Self::new(vec![K::zero(); n])
    }
}

impl<K> Vector<K>
where
    K: Copy + Add<Output = K> + Sub<Output = K>,
{
    /// Adds `other` to `self` in-place.
    ///
    /// This performs: `self[i] += other[i]` for all components.
    ///
    /// # Panics (debug)
    /// Panics in debug builds if the vectors have different lengths.
    pub fn add(&mut self, other: &Vector<K>) {
        debug_assert_eq!(self.len(), other.len(), "Vector size mismatch");
        for i in 0..self.len() {
            self.data[i] = self.data[i] + other.data[i];
        }
    }

    /// Substracts `other` from `self` in-place.
    ///
    /// This performs: `self[i] -= other[i]` for all components.
    ///
    /// # Panics (debug)
    /// Panics in debug builds if the vectors have different lengths.
    pub fn sub(&mut self, other: &Vector<K>) {
        debug_assert_eq!(self.len(), other.len(), "Vector size mismatch");
        for i in 0..self.len() {
            self.data[i] = self.data[i] - other.data[i];
        }
    }
}

impl<K> Vector<K>
where
    K: Copy + Mul<Output = K>,
{
    /// Scales the vector by `a` in-place.
    ///
    /// This performs: `self[i] *= a` for all components.
    pub fn scl(&mut self, a: K) {
        for x in &mut self.data {
            *x = *x * a;
        }
    }
}

impl<K> Vector<K>
where
    K: Copy + Zero + Conj + Add<Output = K> + Mul<Output = K>,
{
    /// Computes the dot (inner) product between `self` and `other`.
    ///
    /// \[ ⟨u, v⟩ = Σ conj(uᵢ) · vᵢ \]
    ///
    /// - For real scalars (`f32`), this is the standard Euclidean dot product.
    /// - For complex scalars, this ensures positive definiteness:
    ///   \[ ⟨u, u⟩ ≥ 0 \]
    ///
    /// # Returns
    /// The scalar dot product as a value of type `K`.
    ///
    /// # Panics (debug)
    /// Panics in debug builds if the vectors have different lengths.
    pub fn dot(&self, other: &Vector<K>) -> K {
        debug_assert_eq!(self.len(), other.len(), "dimension mismatch");

        let mut sum = K::zero();
        for i in 0..self.len() {
            sum = sum + self.data[i].conj() * other.data[i];
        }
        sum
    }
}

impl<K> Vector<K>
where
    K: One + Zero + Add<Output = K> + Mul<Output = K> + Sub<Output = K> + From<f32> + Copy + Zero + Abs + Mul<Output = K>,
{
    /// Returns a normalized copy of the vector (unit length).
    ///
    /// # Panics (debug)
    /// Panics if the vector has zero norm.
    pub fn normalize(&self) -> Vector<K> 
    {
        let n = self.norm();
        debug_assert!(n > 0.0, "cannot normalize a zero vector");
        
        let inv_norm = K::from(1.0 / n);
        let mut result = self.clone();
        result.scl(inv_norm);
        result
    }
}

impl<K> Vector<K>
where
    K: Copy + Sub<Output = K>,
{
    /// Returns a new vector: `self - other`
    pub fn sub_vec(&self, other: &Vector<K>) -> Vector<K> {
        debug_assert_eq!(self.len(), other.len(), "Vector size mismatch");
        
        let data: Vec<K> = self.data.iter()
            .zip(other.data.iter())
            .map(|(&a, &b)| a - b)
            .collect();
        
        Vector::new(data)
    }
}

impl<K> Vector<K>
where
    K: Copy + Zero + Abs + Mul<Output = K>,  {
    /// Computes the L1 norm (Manhattan norm).
    ///
    /// \[ ||u||₁ = Σ |uᵢ| \]
    ///
    /// The absolute value is taken using the scalar magnitude (`abs`), so this
    /// method behaves correctly for both real and complex scalars.
    pub fn norm_1(&self) -> f32 {
        let mut sum = 0.0;

        for &x in &self.data {
            sum += x.abs();
        }

        sum
    }

    /// Computes the L2 norm (Euclidean norm).
    ///
    /// \[ ||u||₂ = sqrt(Σ uᵢ²) \]
    ///
    /// For complex vectors, this is derived from the Hermitian inner product:
    /// \[ ||u||₂ = sqrt(⟨u, u⟩) \]
    pub fn norm(&self) -> f32 {
        let mut sum = 0.0;

        for &x in &self.data {
            sum += x.abs2();
        }
        sum.sqrt()
    }

    /// Computes the L∞ norm (maximum norm).
    ///
    /// \[ ||u||∞ = max |uᵢ| \]
    pub fn norm_inf(&self) -> f32 {
        let mut max = 0.0;

        for &x in &self.data {
            let v = x.abs();
            if v > max {
                max = v;
            }
        }

        max
    }
}

impl Vector<f32> {
    /// Computes the cosine similarity between `self` and `other`.
    ///
    /// Cosine similarity is defined as:
    /// \[ (u · v) / (||u|| * ||v||) \]
    ///
    /// This operation is only defined for real vector spaces and is therefore
    /// restricted to `Vector<f32>`
    ///
    /// # Panics (debug)
    /// Panics in debug builds if:
    /// - vectors have different lengths,
    /// - either vector has zero L2 norm (division by zero).
    pub fn cosine_similarity(&self, other: &Vector<f32>) -> f32 {
        debug_assert_eq!(self.len(), other.len(), "dimension mismatch");

        let denom = self.norm() * other.norm();
        debug_assert!(
            denom != 0.0,
            "cosine similarity is undefined for zero vectors"
        );

        self.dot(other) / denom
    }

    /// Computes the 3D cross product between `self` and `other`.
    ///
    /// The cross product is only defined for real 3-dimensional vectors and is
    /// therefore restricted to `Vector<f32`.
    ///
    /// \[ u x v = (u_y v_z - u_z v_y, u_z v_x - u_x v_z, u_x v_y - u_y v_x) \]
    ///
    /// # Returns
    /// A new vector containing the cross product.
    ///
    /// # Panics (debug)
    /// Panics in debug builds if either vector is not 3-dimensional.
    pub fn cross(&self, other: &Vector<f32>) -> Vector<f32> {
        debug_assert_eq!(self.len(), 3, "cross product requires 3D vectors");
        debug_assert_eq!(other.len(), 3, "cross product requires 3D vectors");

        let u = self.as_slice();
        let v = other.as_slice();

        let ux = u[0];
        let uy = u[1];
        let uz = u[2];

        let vx = v[0];
        let vy = v[1];
        let vz = v[2];

        Vector::new(vec![
            uy * vz - uz * vy,
            uz * vx - ux * vz,
            ux * vy - uy * vx,
        ])
    }
}

/// Computes a linear combination of vectors.
///
/// Given vectors `v₀..vₙ₋₁` and coefficients `a₀..aₙ₋₁`, returns:
///
/// \[ Σ aᵢ vᵢ \]
///
/// This function is generic over the scalar type and works for both real and
/// complex vector spaces.
///
/// # Parameters
/// - `vectors`: Slice of input vectors.
/// - `coeffs`: Slice of coefficients, one per vector.
///
/// #  Returns
/// A new vector holding the linear combination.
///
/// # Panics (debug)
/// Panics in debug builds if:
/// - `vectors` is empty,
/// - `vectors.len() != coeffs.len()`,
/// - input vectors do not all share the same dimension.
pub fn linear_combination<K>(vectors: &[Vector<K>], coeffs: &[K]) -> Vector<K>
where
    K: Copy + Zero + Add<Output = K> + Mul<Output = K>,
{
    debug_assert!(!vectors.is_empty(), "no vectors");
    debug_assert_eq!(vectors.len(), coeffs.len(), "size mismatch");

    let dim = vectors[0].len();

    for v in vectors {
        debug_assert_eq!(v.len(), dim, "dimensions mismatch");
    }

    let mut result = vec![K::zero(); dim];

    for (v, &a) in vectors.iter().zip(coeffs.iter()) {
        for i in 0..dim {
            result[i] = result[i] + v.as_slice()[i] * a;
        }
    }

    Vector::new(result)
}

/// Trait for types that support linear interpolation.
///
/// This trait is intentionally minimal: it allows the project to define `lerp`
/// for multiple types (scalars, vectors, matrices) without depending on operator
/// overloading (`std::ops::Add`, `Mul`, etc.).
pub trait Lerp {
    /// Returns the linear interpolation between `u` and `v` for parameter `t`.
    ///
    /// The formula is:
    /// \[ (1 - t) u + t v \]
    ///
    /// # Parameters
    /// - `u`: Start value (t = 0).
    /// - `v`: End value (t = 1).
    /// - `t`: Interpolation parameter (commonly in \[0, 1\] but not enforced).
    fn lerp(u: &Self, v: &Self, t: f32) -> Self;
}


impl<K> Lerp for K
where
    K: Copy + One + Zero + Add<Output = K> + Mul<Output = K> + Sub<Output = K> + From<f32>,
{
    fn lerp(u: &Self, v: &Self, t: f32) -> Self {

        let t_k = K::from(t);
        (K::one() - t_k) * *u + t_k * *v
    }
}

/// `Lerp` implementation for `Vector`.
///
/// # Panics (debug)
/// Panics in debug builds if `u` and `v` have different lengths.
impl<K> Lerp for Vector<K> 
where
    K: Copy + One + Zero + Add<Output = K> + Mul<Output = K> + Sub<Output = K> + From<f32>,
{
    fn lerp(u: &Self, v: &Self, t: f32) -> Self {
        debug_assert_eq!(u.len(), v.len(), "dimension mismatch");

        let mut data = vec![K::zero(); u.len()];
        let t_k = K::from(t);

        for i in 0..u.len() {
            data[i] = (K::one() - t_k) * u.as_slice()[i] + t_k * v.as_slice()[i];
        }

        Vector::new(data)
    }
}

/// Generic linear interpolation helper.
///
/// # Type parameters
/// - `V`: Any type implementing [`Lerp`].
pub fn lerp<V: Lerp>(u: &V, v: &V, t: f32) -> V {
    V::lerp(u, v, t)
}

/// User-friendly display formatting.
///
/// This prints vectors like: `[1.0, 2.0, 3.0]`
impl<K: fmt::Display> fmt::Display for Vector<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (i, x) in self.data.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", x)?;
        }
        write!(f, "]")
    }
}

#[cfg(test)]
mod tests {
    use super::Vector;
    use super::linear_combination;

    #[test]
    fn add_works() {
        let mut u = Vector::new(vec![2.0, 3.0]);
        let v = Vector::new(vec![2.0, 1.0]);
        u.add(&v);
        assert_eq!(u, Vector::new(vec![4.0, 4.0]));
    }

    #[test]
    fn sub_works() {
        let mut u = Vector::new(vec![2.0, 3.0]);
        let v = Vector::new(vec![2.0, 1.0]);
        u.sub(&v);
        assert_eq!(u, Vector::new(vec![0.0, 2.0]));
    }

    #[test]
    fn scl_works() {
        let mut u = Vector::new(vec![2.0, 3.0]);
        u.scl(2.0);
        assert_eq!(u, Vector::new(vec![4.0, 6.0]));
    }

    #[test]
    fn linear_combination_works() {
        let v1 = Vector::new(vec![1.0, 0.0]);
        let v2 = Vector::new(vec![0.0, 1.0]);

        let result = linear_combination(&[v1, v2], &[2.0, 3.0]);

        assert_eq!(result, Vector::new(vec![2.0, 3.0]));
    }

    #[test]
    fn dot_product_works() {
        let u = Vector::new(vec![1.0, 2.0, 3.0]);
        let v = Vector::new(vec![4.0, 5.0, 6.0]);

        assert_eq!(u.dot(&v), 32.0);
    }

    #[test]
    fn norms_work() {
        let u = Vector::new(vec![3.0, -4.0]);

        assert_eq!(u.norm_1(), 7.0);
        assert_eq!(u.norm(), 5.0);
        assert_eq!(u.norm_inf(), 4.0);
    }

    #[test]
    fn cosine_similarity_works() {
        let u = Vector::new(vec![1.0, 0.0]);
        let v = Vector::new(vec![0.0, 1.0]);
        assert_eq!(u.cosine_similarity(&v), 0.0);

        let u = Vector::new(vec![1.0, 0.0]);
        let v = Vector::new(vec![1.0, 0.0]);
        assert_eq!(u.cosine_similarity(&v), 1.0);

        let u = Vector::new(vec![1.0, 0.0]);
        let v = Vector::new(vec![-1.0, 0.0]);
        assert_eq!(u.cosine_similarity(&v), -1.0);
    }

    #[test]
    fn cross_product_basis_vectors() {
        let ex = Vector::new(vec![1.0, 0.0, 0.0]);
        let ey = Vector::new(vec![0.0, 1.0, 0.0]);
        let ez = Vector::new(vec![0.0, 0.0, 1.0]);

        assert_eq!(ex.cross(&ey), ez);
        assert_eq!(ey.cross(&ex), Vector::new(vec![0.0, 0.0, -1.0]));
    }

    #[test]
    fn cross_product_example() {
        let u = Vector::new(vec![4.0, 2.0, -3.0]);
        let v = Vector::new(vec![-2.0, -5.0, 16.0]);

        assert_eq!(u.cross(&v), Vector::new(vec![17.0, -58.0, -16.0]));
    }
}
