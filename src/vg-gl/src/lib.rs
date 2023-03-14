pub use crate::buffer::GLBuffer;
pub use crate::program::Program;
pub use crate::shader::Shader;
pub use crate::texture::Texture;
pub use crate::vertex_array::VertexArray;

type Pos = [f32; 2];
type TextureCoords = [f32; 2];

#[repr(C, packed)]
pub struct Vertex(pub Pos, pub TextureCoords);

mod buffer;
mod error;
mod macros;
mod program;
mod shader;
mod texture;
mod vertex_array;
