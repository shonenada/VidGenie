use crate::{GLBuffer, Program, Quad, set_attribute, Shader, VertexArray};

type Pos = [f32; 2];
type TextureCoords = [f32; 2];

#[derive(Clone, Copy, Debug, Default)]
#[repr(C, packed)]
pub struct Vertex(pub Pos, pub TextureCoords);

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

    pub fn set_index_buffer_data(&self, data: &[i32; 6]) -> anyhow::Result<()> {
        self.index_buffer.set_data(data, gl::STATIC_DRAW);
        Ok(())
    }

    pub fn set_attrs(&self) -> anyhow::Result<()> {
        let pos_attrib = self.program.get_attrib_location("position")?;
        let color_attrib = self.program.get_attrib_location("verTexCoord")?;
        let va = &self.vertex_array;
        unsafe {
            set_attribute!(va, pos_attrib, Vertex::0);
            set_attribute!(va, color_attrib, Vertex::1);
        }

        Ok(())
    }
}
