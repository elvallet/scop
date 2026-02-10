pub mod math;

pub mod parser;
pub use parser::load_obj;

pub mod mesh;
pub use mesh::{Mesh, Vertex};