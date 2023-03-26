use crate::{GLBuffer, INDICES_PER_QUAD, Program, Quad, set_attribute, Shader, VertexArray};

type Pos = [f32; 2];
// x, y
type TextureCoords = [f32; 2];
type TexIdx = f32;

#[derive(Clone, Copy, Debug, Default)]
#[repr(C, packed)]
pub struct Vertex(pub Pos, pub TextureCoords, pub TexIdx);

#[derive(Clone, Copy, Debug, Default)]
#[repr(C, packed)]
pub struct Indices(pub [i32; INDICES_PER_QUAD]);

const VERTEX_SRC: &str = include_str!("shader/vertex.glsl");
const FRAGMENT_SRC: &str = include_str!("shader/fragment.glsl");

pub struct Renderer {
    pub program: Program,
    pub vertex_array: VertexArray,

    #[allow(dead_code)]
    index_buffer: GLBuffer,
    #[allow(dead_code)]
    vertex_buffer: GLBuffer,
}

impl Renderer {
    pub fn new() -> anyhow::Result<Self> {
        let vertex_shader = Shader::new_vertex(VERTEX_SRC)?;
        let fragment_shader = Shader::new_fragment(FRAGMENT_SRC)?;
        let program = Program::new(&[vertex_shader, fragment_shader])?;

        let vertex_array = VertexArray::new();
        vertex_array.bind();

        let vertex_buffer = GLBuffer::new_array_buffer();
        let index_buffer = GLBuffer::new_element_array_buffer();

        Ok(Self {
            program,
            vertex_array,
            index_buffer,
            vertex_buffer,
        })
    }

    pub fn set_vertex_buffer_data(&self, data: &[Quad]) -> anyhow::Result<()> {
        self.vertex_buffer.set_data(data, gl::STATIC_DRAW);
        Ok(())
    }

    pub fn set_index_buffer_data(&self, data: &[Indices]) -> anyhow::Result<()> {
        self.index_buffer.set_data(data, gl::STATIC_DRAW);
        Ok(())
    }

    pub fn set_attrs(&self) -> anyhow::Result<()> {
        let pos_attrib = self.program.get_attrib_location("position")?;
        let color_attrib = self.program.get_attrib_location("verTexCoord")?;
        let tex_idx_attrib = self.program.get_attrib_location("inTexIdx")?;
        let va = &self.vertex_array;
        unsafe {
            set_attribute!(va, pos_attrib, Vertex::0);
            set_attribute!(va, color_attrib, Vertex::1);
            set_attribute!(va, tex_idx_attrib, Vertex::2);
        }

        Ok(())
    }
}
