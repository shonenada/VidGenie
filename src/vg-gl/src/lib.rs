extern crate nalgebra_glm as glm;

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

#[cfg(target_os = "linux")]
mod egl_headless {
    use khronos_egl as egl;
    use std::sync::Arc;

    const EGL_PLATFORM_SURFACELESS_MESA: egl::Enum = 0x31DD;

    pub struct EglContext {
        _egl: Arc<egl::DynamicInstance<egl::EGL1_5>>,
        _display: egl::Display,
        _context: egl::Context,
    }

    impl EglContext {
        pub fn new() -> Result<Self, String> {
            let egl = Arc::new(
                unsafe { egl::DynamicInstance::<egl::EGL1_5>::load_required() }
                    .map_err(|e| format!("Failed to load EGL: {}", e))?,
            );

            let display = unsafe {
                egl.get_platform_display(
                    EGL_PLATFORM_SURFACELESS_MESA,
                    egl::DEFAULT_DISPLAY,
                    &[egl::ATTRIB_NONE],
                )
            }
            .map_err(|e| format!("Failed to get EGL display: {}", e))?;

            egl.initialize(display)
                .map_err(|e| format!("Failed to initialize EGL: {}", e))?;

            let config_attribs = [
                egl::RENDERABLE_TYPE,
                egl::OPENGL_BIT,
                egl::SURFACE_TYPE,
                0,
                egl::NONE,
            ];

            let config = egl
                .choose_first_config(display, &config_attribs)
                .map_err(|e| format!("Failed to choose EGL config: {}", e))?
                .ok_or("No suitable EGL config found")?;

            egl.bind_api(egl::OPENGL_API)
                .map_err(|e| format!("Failed to bind OpenGL API: {}", e))?;

            let context_attribs = [
                egl::CONTEXT_MAJOR_VERSION,
                3,
                egl::CONTEXT_MINOR_VERSION,
                3,
                egl::CONTEXT_OPENGL_PROFILE_MASK,
                egl::CONTEXT_OPENGL_CORE_PROFILE_BIT,
                egl::NONE,
            ];

            let context = egl
                .create_context(display, config, None, &context_attribs)
                .map_err(|e| format!("Failed to create EGL context: {}", e))?;

            egl.make_current(display, None, None, Some(context))
                .map_err(|e| format!("Failed to make EGL context current: {}", e))?;

            gl::load_with(|s| {
                egl.get_proc_address(s)
                    .map(|p| p as *const std::ffi::c_void)
                    .unwrap_or(std::ptr::null())
            });

            Ok(Self {
                _egl: egl,
                _display: display,
                _context: context,
            })
        }
    }
}

#[allow(dead_code)]
pub enum GLContext {
    #[cfg(not(target_os = "linux"))]
    Windowed {
        event_loop: glutin::event_loop::EventLoop<()>,
        context: glutin::ContextWrapper<glutin::PossiblyCurrent, glutin::window::Window>,
    },
    #[cfg(not(target_os = "linux"))]
    Headless {
        event_loop: glutin::event_loop::EventLoop<()>,
        context: glutin::Context<glutin::PossiblyCurrent>,
    },
    #[cfg(target_os = "linux")]
    Windowed {
        _context: egl_headless::EglContext,
    },
    #[cfg(target_os = "linux")]
    EglHeadless {
        _context: egl_headless::EglContext,
    },
}

pub fn init_gl(width: u32, height: u32) -> GLContext {
    if should_use_headless() {
        init_gl_headless(width, height)
    } else {
        init_gl_windowed(width, height)
    }
}

#[cfg(not(target_os = "linux"))]
fn init_gl_windowed(width: u32, height: u32) -> GLContext {
    use glutin::dpi::LogicalSize;
    use glutin::event_loop::EventLoop;
    use glutin::window::WindowBuilder;
    use glutin::ContextBuilder;

    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title("VidGenie")
        .with_inner_size(LogicalSize::new(width, height))
        .with_max_inner_size(LogicalSize::new(width, height));
    let cb = ContextBuilder::new().with_vsync(false);
    let context = cb.build_windowed(wb, &event_loop).unwrap();
    let context = unsafe { context.make_current().unwrap() };
    gl::load_with(|s| context.get_proc_address(s) as *const _);

    GLContext::Windowed {
        context,
        event_loop,
    }
}

#[cfg(target_os = "linux")]
fn init_gl_windowed(_width: u32, _height: u32) -> GLContext {
    let context = egl_headless::EglContext::new().expect("Failed to create EGL context");
    GLContext::Windowed { _context: context }
}

#[cfg(target_os = "linux")]
fn init_gl_headless(_width: u32, _height: u32) -> GLContext {
    let context = egl_headless::EglContext::new().expect("Failed to create EGL headless context");
    GLContext::EglHeadless { _context: context }
}

#[cfg(not(target_os = "linux"))]
fn init_gl_headless(width: u32, height: u32) -> GLContext {
    use glutin::dpi::PhysicalSize;
    use glutin::event_loop::EventLoop;
    use glutin::ContextBuilder;

    let event_loop = EventLoop::new();
    let cb = ContextBuilder::new().with_vsync(false);
    let size = PhysicalSize::new(width, height);
    let context = cb.build_headless(&event_loop, size).unwrap();
    let context = unsafe { context.make_current().unwrap() };
    gl::load_with(|s| context.get_proc_address(s) as *const _);

    GLContext::Headless {
        context,
        event_loop,
    }
}

fn should_use_headless() -> bool {
    if !cfg!(target_os = "linux") {
        return false;
    }

    if let Ok(val) = std::env::var("VIDGENIE_HEADLESS") {
        let val = val.to_lowercase();
        return val == "1" || val == "true" || val == "yes";
    }

    std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err()
}
