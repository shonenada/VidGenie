extern crate nalgebra_glm as glm;

use glutin::{ContextBuilder, ContextWrapper, PossiblyCurrent};
use glutin::dpi::LogicalSize;
use glutin::event_loop::EventLoop;
use glutin::window::{Window, WindowBuilder};

pub use crate::buffer::GLBuffer;
pub use crate::framebuffer::FrameBuffer;
pub use crate::program::Program;
pub use crate::quad::Quad;
pub use crate::renderbuffer::RenderBuffer;
pub use crate::renderer::{Indices, Renderer, Vertex};
pub use crate::shader::Shader;
pub use crate::texture::Texture;
pub use crate::transformer::Transformer;
pub use crate::vertex_array::VertexArray;

mod buffer;
mod error;
mod framebuffer;
mod macros;
mod program;
mod quad;
mod renderbuffer;
mod renderer;
mod shader;
mod texture;
mod transformer;
mod vertex_array;

pub const VERTEX_PER_QUAD: usize = 4;
pub const INDICES_PER_QUAD: usize = 6;

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
