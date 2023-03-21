use gl::types::{GLint, GLuint};

pub struct VertexArray {
    pub id: GLuint,
}

impl VertexArray {
    pub fn new() -> Self {
        let mut id: GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut id);
        }
        Self { id }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
        }
    }

    pub fn set_attribute<V: Sized>(&self, attrib_pos: GLuint, size: GLint, offset: GLuint) {
        self.bind();
        unsafe {
            gl::VertexAttribPointer(
                attrib_pos,
                size,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<V>() as GLint,
                offset as *const _,
            );
            gl::EnableVertexAttribArray(attrib_pos);
        }
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, [self.id].as_ptr());
        }
    }
}
