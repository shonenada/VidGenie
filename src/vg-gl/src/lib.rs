use glutin::dpi::LogicalSize;
use glutin::event_loop::EventLoop;
use glutin::window::{Window, WindowBuilder};
use glutin::{ContextBuilder, ContextWrapper, PossiblyCurrent};

pub use crate::buffer::GLBuffer;
pub use crate::program::Program;
pub use crate::renderer::{Renderer, Vertex};
pub use crate::shader::Shader;
pub use crate::texture::Texture;
pub use crate::vertex_array::VertexArray;

mod buffer;
mod error;
mod macros;
mod program;
mod renderer;
mod shader;
mod texture;
mod vertex_array;

#[allow(dead_code)]
pub struct GLContext {
    event_loop: EventLoop<()>,
    context: ContextWrapper<PossiblyCurrent, Window>,
}

pub fn init_gl(width: u32, height: u32) -> GLContext {
    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title("VidGenie")
        .with_inner_size(LogicalSize::new(width, height))
        .with_max_inner_size(LogicalSize::new(width, height));
    let cb = ContextBuilder::new().with_vsync(false);
    let context = cb.build_windowed(wb, &event_loop).unwrap();
    let context = unsafe { context.make_current().unwrap() };
    gl::load_with(|s| context.get_proc_address(s) as *const _);

    GLContext {
        context,
        event_loop,
    }
}
