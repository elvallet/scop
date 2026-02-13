use std::process::Command;
use std::path::Path;

fn main() {
	println!("cargo:rerun-if-changed=shaders/");

	compile_shaders("shaders/shader.vert");
	compile_shaders("shaders/shader.frag");
}

fn compile_shaders(shader_path: &str) {
	let input = Path::new(shader_path);
	let output = format!("{}.spv", shader_path);

	let status = Command::new("glslc")
		.arg(input)
		.arg("-o")
		.arg(&output)
		.status();

	match status {
		Ok(status) if status.success() => {
			println!("✓ Compiled {}", shader_path);
		}
		Ok(status) => {
			panic!("Failed to compile {}: {:?}", shader_path, status);
		}
		Err(e) => {
			panic!("Failed to run glsl (is Vulkan SDK installed?): {}", e);
		}
	}
}