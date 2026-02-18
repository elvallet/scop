#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
	pub position: [f32; 3],
	pub tex_coords: [f32; 2],
	pub normal: [f32; 3],
	pub color: [f32; 3],
}

#[derive(Debug, Clone, Copy)]
pub enum DominantAxis { X, Y, Z }

impl Vertex {
	pub fn default() -> Self {
		Self {
			position: [0.0, 0.0, 0.0],
			tex_coords: [0.0, 0.0],
			normal: [0.0, 1.0, 0.0],
			color: [0.0, 0.0, 0.0],
		}
	}
    pub fn check_size() {
        println!("Vertex size: {} bytes", std::mem::size_of::<Self>());
        println!("  position offset: {}", 0);
        println!("  tex_coords offset: {}", std::mem::offset_of!(Vertex, tex_coords));
        println!("  normal offset: {}", std::mem::offset_of!(Vertex, normal));
        println!("  color offset: {}", std::mem::offset_of!(Vertex, color));
    }
}

#[derive(Debug)]
pub struct Mesh {
	pub vertices: Vec<Vertex>,
	pub indices: Vec<u32>,
}

impl Mesh {
	pub fn compute_bounding_box(&self) -> ([f32; 3], [f32; 3]) {
		if self.vertices.is_empty() {
			return ([0.0; 3], [0.0; 3]);
		}

		let mut min = self.vertices[0].position;
		let mut max = self.vertices[0].position;

		for vertex in &self.vertices {
			for i in 0..3 {
				min[i] = min[i].min(vertex.position[i]);
				max[i] = max[i].max(vertex.position[i]);
			}
		}

		(min, max)
	}

pub fn normalize(&mut self) {
    let (min, max) = self.compute_bounding_box();

    let center = [
        (min[0] + max[0]) / 2.0,
        (min[1] + max[1]) / 2.0,
        (min[2] + max[2]) / 2.0,
    ];

    let max_half_extent = (0..3)
        .map(|i| (max[i] - min[i]) / 2.0)
        .fold(0.0_f32, f32::max);

    for vertex in &mut self.vertices {
        for i in 0..3 {
            vertex.position[i] = (vertex.position[i] - center[i]) / max_half_extent;
        }
    }
}

	pub fn compute_dominant_axis(&self) -> DominantAxis {
		let bounding_box = self.compute_bounding_box();
		let mut min_value = 2.0;
		let mut min_indice: usize = 0;

		for i in 0..3 {
			let min = bounding_box.0[i];
			let max = bounding_box.1[i];
			let dim = (max - min).abs();
			if dim < min_value {
				min_value = dim;
				min_indice = i;
			}
		}

		match min_indice {
			0 => DominantAxis::X,
			1 => DominantAxis::Y,
			2 => DominantAxis::Z,
			_ => DominantAxis::X,
		}
	}
}
