use gl::types::{GLuint, GLenum, GLsizeiptr};

pub struct GLBuffer {
    pub id: GLuint,
    target: GLenum,
}

impl GLBuffer {
    pub fn new(target: GLenum) -> Self {
        let mut id :GLuint = 0;
        unsafe {
            gl::GenBuffers(1, &mut id);
        }
        Self { id, target }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(self.target, self.id);
        }
    }

    /// Usage should be one of:
    /// - `gl::STREAM_DRAW`: the vertex data is set once and drawn once
    /// - `gl::STATIC_DRAW`: the vertex data is set once and drawn many times (as in our case with triangle)
    /// - `gl::DYNAMIC_DRAW`: the vertex data is changed a lot and drawn many times
    pub fn set_data<D>(&self, data: &[D], usage: GLenum) {
        self.bind();
        unsafe {
            let (_, data_bytes, _) = data.align_to::<u8>();
            gl::BufferData(
                self.target,
                data_bytes.len() as GLsizeiptr,
                data_bytes.as_ptr() as *const _,
                usage,
            )
        }
    }

    // shortcut for gl::ARRAY_BUFFER
    pub fn new_array_buffer() -> Self {
        Self::new(gl::ARRAY_BUFFER)
    }

    // shortcut for gl::ELEMENT_ARRAY_BUFFER
    pub fn new_element_array_buffer() -> Self {
        Self::new(gl::ELEMENT_ARRAY_BUFFER)
    }
}

impl Drop for GLBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, [self.id].as_ptr());
        }
    }
}