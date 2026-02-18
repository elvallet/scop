use std::collections::HashMap;

use crate::mesh::{Mesh, Vertex};

#[derive(Debug, Clone)]
pub struct ObjData {
	positions: Vec<[f32; 3]>,
	tex_coords: Vec<[f32; 2]>,
	normals: Vec<[f32; 3]>,

	faces: Vec<Face>,
}

#[derive(Debug, Clone)]
pub struct Face {
	vertices: Vec<FaceVertex>,
}

#[derive(Debug, Clone)]
pub struct FaceVertex {
	position_idx: usize,
	tex_coord_idx: Option<usize>,
	normal_idx: Option<usize>,
}

#[derive(Hash, Eq, PartialEq)]
struct VertexKey {
	position: [i32; 3],
	tex_coords: [i32; 2],
	normal: [i32; 3],
}

impl VertexKey {
	fn quantize(f: f32) -> i32 {
		(f * 10000.0).round() as i32
	}

	fn from_vertex(v: &Vertex) -> Self {
		Self {
			position: [
				Self::quantize(v.position[0]),
				Self::quantize(v.position[1]),
				Self::quantize(v.position[2]),
			],
			tex_coords: [
				Self::quantize(v.tex_coords[0]),
				Self::quantize(v.tex_coords[1]),
			],
			normal: [
				Self::quantize(v.normal[0]),
				Self::quantize(v.normal[1]),
				Self::quantize(v.normal[2]),
			]
		}
	}
}

pub fn load_obj(file_path: &str) -> Result<Mesh, String> {
	let obj_data = parse_obj(file_path)?;
	Ok(obj_to_mesh(obj_data))
}

pub fn parse_obj(file_path: &str) -> Result<ObjData, String> {
	let content = std::fs::read_to_string(file_path)
		.map_err(|e| format!("Failed to read file: {}", e))?;

	parse_obj_from_string(&content)
}

pub fn parse_obj_from_string(content: &str) -> Result<ObjData, String> {
	let mut positions = Vec::new();
	let mut tex_coords = Vec::new();
	let mut normals = Vec::new();
	let mut faces = Vec::new();

	for (line_num, line) in content.lines().enumerate() {
		let line = line.trim();

		if line.is_empty() || line.starts_with('#') {
			continue;
		}

		let tokens: Vec<&str> = line.split_whitespace().collect();
		if tokens.is_empty() {
			continue;
		}


		let result = match tokens[0] {
			"v" => parse_vertex(&tokens, &mut positions),
			"vt" => parse_tex_coord(&tokens, &mut tex_coords),
			"vn" => parse_normal(&tokens, &mut normals),
			"f" => parse_face(&tokens, &mut faces),
			"mtllib" | "usemtl" | "o" | "g" | "s" => Ok(()),
			_ => {
				eprintln!("Warning: Unknown directive '{}' at line {}", tokens[0], line_num + 1);
				Ok(())
			}
		};

		if let Err(e) = result {
			return Err(format!("Error at line {}: {}", line_num + 1, e));
		}
	}

	Ok(ObjData { positions, tex_coords, normals, faces })
}

fn parse_vertex(tokens: &[&str], positions: &mut Vec<[f32; 3]>) -> Result<(), String> {
	if tokens.len() < 4 {
		return Err("Invalid vertex format: expected 'v x y z'".to_string());
	}

	let x = tokens[1].parse::<f32>()
		.map_err(|_| format!("Invalid x coordinate: '{}'", tokens[1]))?;
	let y = tokens[2].parse::<f32>()
		.map_err(|_| format!("Invalid y coordinate: '{}'", tokens[2]))?;
	let z = tokens[3].parse::<f32>()
		.map_err(|_| format!("Invalid z coordinate: '{}'", tokens[3]))?;

	positions.push([x, y, z]);
	Ok(())
}

fn parse_tex_coord(tokens: &[&str], tex_coords: &mut Vec<[f32; 2]>) -> Result<(), String> {
	if tokens.len() < 3 {
		return Err("Invalid texture coordinate format: expected 'vt u v'".to_string());
	}

	let u = tokens[1].parse::<f32>()
		.map_err(|_| format!("Invalid u coordinate: '{}'", tokens[1]))?;
	let v = tokens[2].parse::<f32>()
		.map_err(|_| format!("Invalid v coordinate: '{}'", tokens[2]))?;

	tex_coords.push([u, v]);
	Ok(())
}

fn parse_normal(tokens: &[&str], normals: &mut Vec<[f32; 3]>) -> Result<(), String> {
	if tokens.len() < 4 {
		return Err("Invalid normal format: expected 'vn x y z".to_string());
	}

	let x = tokens[1].parse::<f32>()
		.map_err(|_| format!("Invalid normal x: '{}'", tokens[1]))?;
	let y = tokens[2].parse::<f32>()
		.map_err(|_| format!("Invalid normal y: '{}'", tokens[2]))?;
	let z = tokens[3].parse::<f32>()
		.map_err(|_| format!("Invalid normal z: '{}'", tokens[3]))?;

	let length = (x*x + y*y + z*z).sqrt();

	if length > 0.0001 {
		normals.push([x / length, y / length, z / length]);
	} else {
		normals.push([0.0, 1.0, 0.0])
	}

	Ok(())
}

fn parse_face(tokens: &[&str], faces: &mut Vec<Face>) -> Result<(), String> {
	if tokens.len() < 4 {
		return Err("Face must have at least 3 vertices".to_string());
	}

	let mut face_vertices = Vec::new();

	for i in 1..tokens.len() {
		let vertex = parse_face_vertex(tokens[i])?;
		face_vertices.push(vertex);
	}

	let triangulated = triangulate_face(&face_vertices);
	faces.extend(triangulated);

	Ok(())
}

fn parse_face_vertex(token: &str) -> Result<FaceVertex, String> {
	let parts: Vec<&str> = token.split('/').collect();

	let position_idx = parts[0].parse::<isize>()
		.map_err(|_| format!("Invalid vertex index: '{}'", parts[0]))?;

	let tex_coord_idx = if parts.len() > 1 && !parts[1].is_empty() {
		Some(parts[1].parse::<isize>()
			.map_err(|_| format!("Invalid texture index: '{}'", parts[1]))?)
	} else {
		None
	};

	let normal_idx = if parts.len() > 2 {
		Some(parts[2].parse::<isize>()
			.map_err(|_| format!("Invalid normal index: '{}'", parts[2]))?)
	} else {
		None
	};

	Ok(FaceVertex {
		position_idx: handle_obj_index(position_idx)?,
		tex_coord_idx: tex_coord_idx.map(|i| handle_obj_index(i)).transpose()?,
		normal_idx: normal_idx.map(|i| handle_obj_index(i)).transpose()?,
	})
}


fn handle_obj_index(idx: isize) -> Result<usize, String> {
	if idx > 0 {
		Ok((idx - 1) as usize)
	} else if idx < 0 {
		Err("Negative indices not supported yet".to_string())
	} else {
		Err("Index cannot be 0".to_string())
	}
}

fn triangulate_face(vertices: &[FaceVertex]) -> Vec<Face> {
	if vertices.len() == 3 {
		return vec![Face { vertices: vertices.to_vec() }];
	}

	let mut triangles = Vec::new();

	for i in 1..(vertices.len() - 1) {
		triangles.push(Face {
			vertices: vec![
				vertices[0].clone(),
				vertices[i].clone(),
				vertices[i + 1].clone(),
			],
		});
	}

	triangles
}

pub fn obj_to_mesh(obj: ObjData) -> Mesh {
	let mut vertices = Vec::new();
	let mut indices = Vec::new();
	let mut vertex_cache: HashMap<VertexKey, u32> = HashMap::new();

	for (face_idx, face) in obj.faces.iter().enumerate() {
		for face_vertex in &face.vertices {
			let position = obj.positions[face_vertex.position_idx];

			let tex_coords = face_vertex.tex_coord_idx
				.map(|i| obj.tex_coords[i])
				.unwrap_or([0.0, 0.0]);

			let normal = face_vertex.normal_idx
				.map(|i| obj.normals[i])
				.unwrap_or_else(|| compute_face_normal(&obj, face));

			let color = generate_face_color(face_idx);
			let vertex = Vertex {
				position,
				tex_coords,
				normal,
				color,
			};

			let key = VertexKey::from_vertex(&vertex);

			let index = *vertex_cache.entry(key).or_insert_with(|| {
				let idx = vertices.len() as u32;
				vertices.push(vertex);
				idx
			});

			indices.push(index);
		}
	}

	Mesh { vertices, indices }
}

fn generate_face_color(face_index: usize) -> [f32; 3] {
    let shades = [
        [0.3, 0.3, 0.3],
        [0.4, 0.4, 0.4],
        [0.5, 0.5, 0.5],
        [0.6, 0.6, 0.6],
        [0.7, 0.7, 0.7],
        [0.8, 0.8, 0.8],
    ];

	shades[face_index % shades.len()]
}

fn compute_face_normal(obj: &ObjData, face: &Face) -> [f32; 3] {
	if face.vertices.len() < 3 {
		return [0.0, 1.0, 0.0];
	}

	let v0 = obj.positions[face.vertices[0].position_idx];
	let v1 = obj.positions[face.vertices[1].position_idx];
	let v2 = obj.positions[face.vertices[2].position_idx];

	let edge1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
	let edge2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

	let normal = [
		edge1[1] * edge2[2] - edge1[2] * edge2[1],
		edge1[2] * edge2[0] - edge1[0] * edge2[2],
		edge1[0] * edge2[1] - edge1[1] * edge2[0],
	];

	let len = (normal[0]*normal[0] + normal[1]*normal[1] + normal[2]*normal[2]).sqrt();

	if len > 0.0001 {
		[normal[0]/len, normal[1]/len, normal[2]/len]
	} else {
		[0.0, 1.0, 0.0]
	}
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_triangle() {
        let content = "
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.0 1.0 0.0
f 1 2 3
";
        let mesh = parse_obj_from_string(content)
            .map(obj_to_mesh)
            .unwrap();
        
        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.indices.len(), 3);
    }

    #[test]
    fn quad_becomes_two_triangles() {
        let content = "
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 1.0 1.0 0.0
v 0.0 1.0 0.0
f 1 2 3 4
";
        let obj = parse_obj_from_string(content).unwrap();
        assert_eq!(obj.faces.len(), 2);  // Triangulé
    }
}
