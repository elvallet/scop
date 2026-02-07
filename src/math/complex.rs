use core::ops::{Add, Sub, Mul, Div, Neg};
use crate::math::scalar::{Abs2, Conj, One, Zero};

/// A complex number `re + i * im` using `f32`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Complex {
	pub re: f32,
	pub im: f32,
}

impl Complex {
	/// Creates a new complex number.
	pub fn new(re: f32, im: f32) -> Self {
		Self { re, im }
	}
}

impl Zero for Complex {
	fn zero() -> Self {
		Self { re: 0.0, im: 0.0 }
	}
}

impl One for Complex {
	fn one() -> Self {
		Self { re: 1.0, im: 0.0 }
	}
}

impl Conj for Complex {
	fn conj(self) -> Self {
		Self { re: self.re, im: -self.im }
	}
}

impl Abs2 for Complex {
	fn abs2(self) -> f32 {
		self.re * self.re + self.im * self.im
	}
}

impl From<f32> for Complex {
	fn from(x: f32) -> Self {
		Self { re: x, im: 0.0 }
	}
}

impl Add for Complex {
	type Output = Self;
	fn add(self, rhs: Self) -> Self {
		Self { re: self.re + rhs.re, im: self.im + rhs.im }
	}
}

impl Sub for Complex {
	type Output = Self;
	fn sub(self, rhs: Self) -> Self {
		Self { re: self.re - rhs.re, im: self.im - rhs.im }
	}
}

impl Mul for Complex {
	type Output = Self;
	fn mul(self, rhs: Self) -> Self {
		Self {
			re: self.re * rhs.re - self.im * rhs.im,
			im: self.re * rhs.im + self.im * rhs.re,
		}
	}
}

impl Div for Complex {
	type Output = Self;
	fn div(self, rhs: Self) -> Self {
		// (a+bi)/(c+di) = (a+bi)(c-di)/(c²+d²)
		let denom = rhs.re * rhs.re + rhs.im * rhs.im;
		debug_assert!(denom != 0.0, "division by zero complex number");

		let re = (self.re * rhs.re + self.im * rhs.im) / denom;
		let im = (self.im * rhs.re - self.re * rhs.im) / denom;
		Self { re, im }
	}
}

impl Neg for Complex {
	type Output = Self;
	fn neg(self) -> Self {
		Self { re: -self.re, im: -self.im }
	}
}

#[cfg(test)]
mod tests {
	use super::Complex;
	use crate::math::scalar::{Abs2, Conj, One, Zero};

	#[test]
	fn complex_basics() {
		let a = Complex::new(1.0, 2.0);
		let b = Complex::new(3.0, 4.0);

		assert_eq!(a + b, Complex::new(4.0, 6.0));
		assert_eq!(a - b, Complex::new(-2.0, -2.0));

		// (1 + 2i)(3 + 4i) = -5 + 10i
		assert_eq!(a * b, Complex::new(-5.0, 10.0));

		assert_eq!(a.conj(), Complex::new(1.0, -2.0));
		assert_eq!(a.abs2(), 5.0);

		assert_eq!(Complex::zero(), Complex::new(0.0, 0.0));
		assert_eq!(Complex::one(), Complex::new(1.0, 0.0));
	}

	#[test]
	fn complex_division() {
		let a = Complex::new(1.0, 2.0);
		let b = Complex::new(3.0, 4.0);

		// a / b = (1+2i)/(3+4i) = (11/25) + (2/25)i = 0.44 + 0.08i
		let q = a / b;
		assert!((q.re - 0.44).abs() < 1e-6);
		assert!((q.im - 0.08).abs() < 1e-6);
	}
}