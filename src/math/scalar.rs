//! Scalar traits used by the project.
//!
//! This module defines small, focused traits that allow generic linear algebra
//! code without pulling external dependencies.

use core::ops::{Add, Sub, Mul, Div, Neg};
/// Provides an additive identity.
pub trait Zero {
	fn zero() -> Self;
}

/// Provides a multiplicative identity.
pub trait One {
	fn one() -> Self;
}

/// Complex conjugation (identity for real scalars).
pub trait Conj {
	fn conj(self) -> Self;
}

/// Squared magnitude as an `f32`.
///
/// - For real numbers: `abs2(x) = x*x`.
/// - For complex numbers: `abs2(a+bi) = a*a + b*b`.
pub trait Abs2 {
	fn abs2(self) -> f32;
}

/// Magnitude as an `f32`  (derived from `Abs2`).
pub trait Abs: Abs2 {
	fn abs(self) -> f32 where Self: Sized {
		self.abs2().sqrt()
	}
}

// Blanket impl: any Abs2 automatically gets Abs.
impl<T: Abs2> Abs for T {}

pub trait Field:
	Copy
	+ Zero
	+ One
	+ Abs
	+ Add<Output = Self>
	+ Sub<Output = Self>
	+ Mul<Output = Self>
	+ Div<Output = Self>
	+ Neg<Output = Self>
{
}

impl Zero for f32 {
	fn zero() -> Self {
		0.0
	}
}

impl One for f32 {
	fn one() -> Self {
		1.0
	}
}

impl Conj for f32 {
	fn conj(self) -> Self {
		self
	}
}

impl Abs2 for f32 {
	fn abs2(self) -> f32 {
		self * self
	}
}

impl<T> Field for T
where
	T: Copy
		+ Zero
		+ One
		+ Abs
		+ Add<Output = T>
		+ Sub<Output = T>
		+ Mul<Output = T>
		+ Div<Output = T>
		+ Neg<Output = T>,
{
}