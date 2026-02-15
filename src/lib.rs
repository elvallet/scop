pub mod math;

pub mod parser;
pub use parser::load_obj;

pub mod mesh;
pub use mesh::{Mesh, Vertex};

mod renderer;
pub use renderer::instance::VulkanInstance;

pub mod app;