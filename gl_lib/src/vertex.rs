use std::{ffi::c_void, marker::PhantomData};

use gl::types::*;

pub struct VertexBufferObject<T> {
    id: u32,
    attribute_struct: PhantomData<T>,
}

#[derive(Debug, Copy, Clone)]
pub enum DrawHint {
    StaticDraw,
    DynamicDraw,
    StreamDraw,
}

impl DrawHint {
    fn to_opengl(self) -> GLenum {
        match self {
            Self::StaticDraw => gl::STATIC_DRAW,
            Self::DynamicDraw => gl::DYNAMIC_DRAW,
            Self::StreamDraw => gl::STREAM_DRAW,
        }
    }
}

impl<T> VertexBufferObject<T> {
    pub fn new(data: &[T], draw_hint: Option<DrawHint>) -> Result<VertexBufferObject<T>, String> {
        let id = unsafe {
            // Reset any error beforehand
            gl::GetError();

            let mut id = 0;
            gl::GenBuffers(1, &mut id);
            gl::BindBuffer(gl::ARRAY_BUFFER, id);

            // Set draw hint if provided
            let draw_hint = if let Some(hint) = draw_hint {
                hint.to_opengl()
            } else {
                gl::DYNAMIC_DRAW
            };

            // Fill buffer
            gl::BufferData(
                gl::ARRAY_BUFFER,
                size_of_val(data) as isize,
                data.as_ptr() as *const c_void,
                draw_hint,
            );

            let error = gl::GetError();
            if error == gl::OUT_OF_MEMORY {
                return Err(String::from("Ran out of memory"));
            } else {
                assert_eq!(gl::NO_ERROR, error);
            }

            // Unbind
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);

            id
        };

        Ok(VertexBufferObject {
            id,
            attribute_struct: PhantomData,
        })
    }

    /// # Safety
    /// Make id is a valid Vertex Buffer Object class with correct data layout.
    pub unsafe fn from_id(id: u32) -> VertexBufferObject<T> {
        VertexBufferObject {
            id,
            attribute_struct: PhantomData,
        }
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }
}

impl<T> Drop for VertexBufferObject<T> {
    fn drop(&mut self) {
        unsafe { gl::DeleteBuffers(1, &self.id) }
    }
}
