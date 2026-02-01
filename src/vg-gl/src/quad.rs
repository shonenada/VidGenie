use crate::{Vertex, VERTEX_PER_QUAD};

#[derive(Clone, Copy, Debug, Default)]
#[repr(C, packed)]
pub struct Quad(pub [Vertex; VERTEX_PER_QUAD]);
