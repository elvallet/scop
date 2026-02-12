mod app;
mod renderer;

use winit::event_loop::{ControlFlow, EventLoop};
use app::App;

fn main() {
	println!("=== SCOP - Starting ===");

	let event_loop = EventLoop::new().expect("Failed to create event loop");

	event_loop.set_control_flow(ControlFlow::Poll);

	let mut app = App::default();

	let _ = event_loop.run_app(&mut app);
}