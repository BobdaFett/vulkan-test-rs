mod app;
mod common;
pub mod gpu;
// pub mod math;

use crate::app::MainApplication;
use std::error::Error;
use vulkano::swapchain::Surface;
use winit::event_loop::{ControlFlow, EventLoop};

// Compute shader compilation. Happens at runtime, but file contents are included during compilation.
mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/shaders/comp.glsl",
    }
}

mod mandelbrot_shader {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/shaders/mandelbrot.glsl"
    }
}

mod triangle {
    vulkano_shaders::shader! {
        shaders: {
            vertex: {
                ty: "vertex",
                path: "src/shaders/triangle_vert.glsl"
            },
            fragment: {
                ty: "fragment",
                path: "src/shaders/triangle_frag.glsl"
            },
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Beginning initialization of vulkano");

    // Create the window that we'll use for graphics rendering with a swapchain.
    let event_loop = EventLoop::new()?;
    let required_extensions = Surface::required_extensions(&event_loop)?;

    // Create and start the window.
    let mut app = MainApplication::new(required_extensions)?;
    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop.run_app(&mut app)?;

    println!("All operations successful, exiting.");

    Ok(())
}
