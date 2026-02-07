use super::matrix::Matrix;
use super::vector::Vector;

#[derive(Debug)]
pub struct Transform {

}
impl Transform {
	pub fn translation(tx: f32, ty: f32, tz: f32) -> Matrix<f32> {
		let v : Vec<f32> = vec![
			1.0, 0.0, 0.0, 0.0,
			0.0, 1.0, 0.0, 0.0,
			0.0, 0.0, 1.0, 0.0,
			tx, ty, tz, 1.0
		];

		Matrix::new(v, 4, 4)
	}

	pub fn scale(sx: f32, sy: f32, sz:f32) -> Matrix<f32> {
		let v : Vec<f32> = vec![
			sx, 0.0, 0.0, 0.0,
			0.0, sy, 0.0, 0.0,
			0.0, 0.0, sz, 0.0,
			0.0, 0.0, 0.0, 1.0,
		];

		Matrix::new(v, 4, 4)
	}

	pub fn rotation_x(angle: f32) -> Matrix<f32> {
		let v : Vec<f32> = vec![
			1.0, 0.0,             0.0,              0.0,
			0.0, f32::cos(angle), f32::sin(angle),  0.0,
			0.0, -f32::sin(angle), f32::cos(angle),  0.0,
			0.0, 0.0,             0.0,              1.0,
		];

		Matrix::new(v, 4, 4)
	}

	pub fn rotation_y(angle: f32) -> Matrix<f32> {
		let v : Vec<f32> = vec![
			f32::cos(angle),  0.0, -f32::sin(angle), 0.0,
			0.0,              1.0, 0.0,             0.0,
			f32::sin(angle), 0.0, f32::cos(angle), 0.0,
			0.0,              0.0, 0.0,             1.0,
		];

		Matrix::new(v, 4, 4)
	}

	pub fn rotation_z(angle: f32) -> Matrix<f32> {
		let v : Vec<f32> = vec![
			f32::cos(angle), f32::sin(angle), 0.0, 0.0,
			-f32::sin(angle), f32::cos(angle),  0.0, 0.0,
			0.0,             0.0,              1.0, 0.0,
			0.0,             0.0,              0.0, 1.0,
		];

		Matrix::new(v, 4, 4)
	}

pub fn look_at(eye: &Vector<f32>, target: &Vector<f32>, up: &Vector<f32>) -> Matrix<f32> {
    let forward = target.sub_vec(eye).normalize();
    let right = forward.cross(up).normalize();
    let camera_up = right.cross(&forward);
    
    let f = forward.as_slice();
    let r = right.as_slice();
    let u = camera_up.as_slice();
    
    Matrix::new(
        vec![
            r[0], r[1], r[2], -right.dot(eye),
            u[0], u[1], u[2], -camera_up.dot(eye),
            -f[0], -f[1], -f[2], forward.dot(eye),
            0.0, 0.0, 0.0, 1.0
        ],
        4,
        4,
    )
}
}

#[cfg(test)]
mod tests {
	use crate::math::{Vector, transform::Transform};

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
	fn translation_works() {
		let t = Transform::translation(10.0, 20.0, 30.0);
		let p = Vector::new(vec![1.0, 2.0, 3.0, 1.0]);
		let result = t.mul_vec(&p);
		
		assert_f32_approx_eq(result.as_slice()[0], 11.0, 1e-5);
		assert_f32_approx_eq(result.as_slice()[1], 22.0, 1e-5);
		assert_f32_approx_eq(result.as_slice()[2], 33.0, 1e-5);
	}

	#[test]
	fn rotation_z_90deg() {
		let r = Transform::rotation_z(std::f32::consts::FRAC_PI_2);
		let p = Vector::new(vec![1.0, 0.0, 0.0, 1.0]);
		let result = r.mul_vec(&p);
		
		// X devient Y après 90° autour de Z
		assert_f32_approx_eq(result.as_slice()[0], 0.0, 1e-5);
		assert_f32_approx_eq(result.as_slice()[1], 1.0, 1e-5);
	}
}