use crate::{GLBuffer, Program, set_attribute, Shader, VertexArray};

type Pos = [f32; 2];
type TextureCoords = [f32; 2];

#[repr(C, packed)]
pub struct Vertex(pub Pos, pub TextureCoords);

#[rustfmt::skip]
const VERTICES: [Vertex; 4] = [
    Vertex([-0.5, -0.5], [0.0, 1.0]),
    Vertex([0.5, -0.5], [1.0, 1.0]),
    Vertex([0.5, 0.5], [1.0, 0.0]),
    Vertex([-0.5, 0.5], [0.0, 0.0]),
];

#[rustfmt::skip]
const INDICES: [i32; 6] = [
    0, 1, 2,
    2, 3, 0
];

pub struct Renderer {
    pub program: Program,
    pub vertex_array: VertexArray,

    #[allow(dead_code)]
    index_buffer: GLBuffer,
    #[allow(dead_code)]
    vertex_buffer: GLBuffer,
}

impl Renderer {
    pub fn new(vertex_source: &str, fragment_source: &str) -> anyhow::Result<Self> {
        let vertex_shader = Shader::new_vertex(vertex_source)?;
        let fragment_shader = Shader::new_fragment(fragment_source)?;
        let program = Program::new(&[vertex_shader, fragment_shader])?;

        let vertex_array = VertexArray::new();
        vertex_array.bind();

        let vertex_buffer = GLBuffer::new_array_buffer();
        vertex_buffer.set_data(&VERTICES, gl::STATIC_DRAW);

        let index_buffer = GLBuffer::new_element_array_buffer();
        index_buffer.set_data(&INDICES, gl::STATIC_DRAW);

        let pos_attrib = program.get_attrib_location("position")?;
        let color_attrib = program.get_attrib_location("verTexCoord")?;
        unsafe {
            set_attribute!(vertex_array, pos_attrib, Vertex::0);
            set_attribute!(vertex_array, color_attrib, Vertex::1);
        }

        Ok(Self {
            program,
            vertex_array,
            index_buffer,
            vertex_buffer,
        })
    }
}
