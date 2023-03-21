use crate::Vertex;

#[derive(Clone, Copy, Debug, Default)]
#[repr(C, packed)]
pub struct Quad(pub [Vertex; 4]);