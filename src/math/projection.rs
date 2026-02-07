//! Perspective projection matrix (4x4).
//!
//! # Conventions
//! - Output matrix is **column-major** (Fortran/BLAS style), consistent with the rest of this crate.
//! - NDC ranges are:
//!  - X, Y in [-1, 1]
//!  - Z in [0, 1]
//!
//! This matches the conventions expected by the provided display software provided for the bonus exercice.

use crate::math::matrix::Matrix;

/// Builds a 4x4 perspective projection matrix.
///
/// # Parameters
/// - `fov`: Vertical field of view angle in radians.
/// - `ratio`: Aspect ratio `w / h`.
/// - `near`: Distance to the near plane (> 0).
/// - `far`: Distance to the far place (> near).
///
/// # Returns
/// A projection matrix in column-major order.
///
/// # Panics (debug)
/// Panics in debug builds if parameters are invalid.
///
/// # Notes
/// This projection maps depth to **Z in [0, 1]** in NDC (not [-1, 1]).
pub fn projection(fov: f32, ratio: f32, near: f32, far: f32) -> Matrix {
	debug_assert!(fov > 0.0, "fov must be > 0");
	debug_assert!(ratio > 0.0, "ratio must be > 0");
	debug_assert!(near > 0.0, "near must be > 0");
	debug_assert!(far > near, "far must be > near");

	let f = 1.0 / (fov * 0.5).tan();
	let inv = 1.0 / (far - near);

	// NDC: Z in [0, 1]
	//
	// [	f/ratio,	0,	0,					0							]
	// [	0,			f,	0,					0							]
	// [	0,			0,	far/(far - near),	- far * near / (far - near)	]
	// [	0,			0,	1,					0							]
	//
	// Column-major data:
	// col0: [f/ratio, 0, 0, 0]
	// col1: [0, f, 0, 0]
	// col2: [0, 0, far/(far-near), 1]
	// col3: [0, 0, -far*near/(far-near), 0]
	let a = f / ratio;
	let b = f;
	let c = far * inv;
	let d = -(far * near) * inv;

	Matrix::new(
		vec![
			a, 0.0, 0.0, 0.0,
			0.0, b, 0.0, 0.0,
			0.0, 0.0, c, 1.0,
			0.0, 0.0, d, 0.0,
		],
		4,
		4,
	)
}

#[cfg(test)]
mod tests {
	use super::projection;

	fn assert_f32_approx_eq(a: f32, b: f32, eps: f32) {
		assert!(
			(a - b).abs() <= eps,
			"expected approx equal: a={} b={} (diff={})",
			a,
			b,
			(a - b).abs()
		);
	}

	#[test]
	fn projection_has_expected_shape_and_key_entries() {
		// 90 degrees vertical FOV
		let fov = core::f32::consts::FRAC_PI_2;
		let ratio = 16.0 / 9.0;
		let near = 0.1;
		let far = 100.0;

		let p = projection(fov, ratio, near, far);

		assert_eq!(p.rows(), 4);
		assert_eq!(p.cols(), 4);

		// f = 1 / tan(fov/2) = 1 / tan(pi/4) = 1
		// so p(0, 0) = 1/ratio and p(1, 1) = 1
		assert_f32_approx_eq(p.get(0, 0), 1.0 / ratio, 1e-6);
		assert_f32_approx_eq(p.get(1, 1), 1.0, 1e-6);

		// The "perspective" term (row 3 col 2) must be 1 for the convention
		// expected by the display software
		assert_f32_approx_eq(p.get(3, 2), 1.0, 1e-6);
	}

	#[test]
	fn projection_depth_coefficients_match_formula() {
		let fov = 1.0;
		let ratio = 1.0;
		let near = 0.5;
		let far = 10.0;

		let p = projection(fov, ratio, near, far);

		let inv = 1.0 / (far - near);
		let c = far * inv;
		let d = -(far * near) * inv;

		assert_f32_approx_eq(p.get(2, 2), c, 1e-6);
		assert_f32_approx_eq(p.get(2, 3), d, 1e-6);
		assert_f32_approx_eq(p.get(3, 2), 1.0, 1e-6);
		assert_f32_approx_eq(p.get(3, 3), 0.0, 1e-6);
	}
}