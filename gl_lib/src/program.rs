use std::{ffi::CStr, ptr};

use cgmath::prelude::*;
use cgmath::Matrix4;
use gl::types::*;

use crate::{FragmentShader, GeometryShader, Shader, VertexShader};

pub struct Program {
    id: u32,
}

impl Program {
    pub fn new(
        vertex_shader: &VertexShader,
        fragment_shader: &FragmentShader,
        geometry_shader: Option<&GeometryShader>,
    ) -> Result<Program, String> {
        unsafe {
            // Reset any error beforehand
            gl::GetError();

            // Create program object on the GPU
            let id = gl::CreateProgram();
            if id == 0 {
                return Err(String::from("OpenGL failed to create program object."));
            }

            gl::AttachShader(id as GLuint, vertex_shader.get_id() as GLuint);
            gl::AttachShader(id as GLuint, fragment_shader.get_id() as GLuint);
            if let Some(geometry_shader) = geometry_shader {
                gl::AttachShader(id as GLuint, geometry_shader.get_id() as GLuint);
            }

            gl::LinkProgram(id as GLuint);

            // Detach shader so it may be used by other program
            gl::DetachShader(id as GLuint, vertex_shader.get_id() as GLuint);
            gl::DetachShader(id as GLuint, fragment_shader.get_id() as GLuint);
            if let Some(geometry_shader) = geometry_shader {
                gl::DetachShader(id as GLuint, geometry_shader.get_id() as GLuint);
            }

            let mut status = 0;
            gl::GetProgramiv(id as GLuint, gl::LINK_STATUS, &mut status as *mut GLint);

            if status as GLboolean == gl::TRUE {
                assert_eq!(gl::NO_ERROR, gl::GetError());
                return Ok(Program { id });
            }

            let mut info_length = 0;
            gl::GetProgramiv(
                id as GLuint,
                gl::INFO_LOG_LENGTH,
                &mut info_length as *mut GLint,
            );

            if info_length == 0 {
                gl::DeleteProgram(id as GLuint);
                assert_eq!(gl::NO_ERROR, gl::GetError());
                return Err(String::from(
                    "Failed to link program, no info log available.",
                ));
            }

            let mut info_log: Vec<u8> = Vec::with_capacity((info_length - 1) as usize);
            gl::GetProgramInfoLog(
                id as GLuint,
                (info_length - 1) as GLsizei,
                ptr::null_mut() as *mut GLsizei,
                info_log.as_mut_ptr() as *mut GLchar,
            );

            gl::DeleteProgram(id as GLuint);
            assert_eq!(gl::NO_ERROR, gl::GetError());
            if let Ok(string) = String::from_utf8(info_log) {
                Err(format!("Failed to link program: {string}"))
            } else {
                Err(String::from(
                    "Failed to link program, info log cannot be parsed to UTF-8.",
                ))
            }
        }
    }

    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.id as GLuint);
        }
    }

    pub fn unset_program() {
        unsafe {
            gl::UseProgram(0 as GLuint);
        }
    }

    pub fn set_1f(&self, name: &CStr, value: f32) -> Result<(), String> {
        let location = self.get_location(name)?;

        unsafe {
            gl::GetError();
            gl::Uniform1f(location as GLint, value as GLfloat);
        }

        if let gl::NO_ERROR = unsafe { gl::GetError() } {
            Ok(())
        } else {
            Err(format!("Invalid uniform value: {value:?}"))
        }
    }

    pub fn set_1i(&self, name: &CStr, value: i32) -> Result<(), String> {
        let location = self.get_location(name)?;

        unsafe {
            gl::GetError();
            gl::Uniform1i(location as GLint, value as GLint);
        }

        if let gl::NO_ERROR = unsafe { gl::GetError() } {
            Ok(())
        } else {
            Err(format!("Invalid uniform value: {value:?}"))
        }
    }

    pub fn set_2f(&self, name: &CStr, value: &[f32; 2]) -> Result<(), String> {
        let location = self.get_location(name)?;

        unsafe {
            gl::GetError();
            gl::Uniform2f(location as GLint, value[0] as GLfloat, value[1] as GLfloat);
        }

        if let gl::NO_ERROR = unsafe { gl::GetError() } {
            Ok(())
        } else {
            Err(format!("Invalid uniform value: {value:?}"))
        }
    }

    pub fn set_3f(&self, name: &CStr, value: &[f32; 3]) -> Result<(), String> {
        let location = self.get_location(name)?;

        unsafe {
            gl::GetError();
            gl::Uniform3f(
                location as GLint,
                value[0] as GLfloat,
                value[1] as GLfloat,
                value[2] as GLfloat,
            );
        }

        if let gl::NO_ERROR = unsafe { gl::GetError() } {
            Ok(())
        } else {
            Err(format!("Invalid uniform value: {value:?}"))
        }
    }

    pub fn set_4f(&self, name: &CStr, value: &[f32; 4]) -> Result<(), String> {
        let location = self.get_location(name)?;

        unsafe {
            gl::GetError();
            gl::Uniform4f(
                location as GLint,
                value[0] as GLfloat,
                value[1] as GLfloat,
                value[2] as GLfloat,
                value[3] as GLfloat,
            );
        }

        if let gl::NO_ERROR = unsafe { gl::GetError() } {
            Ok(())
        } else {
            Err(format!("Invalid uniform value: {value:?}"))
        }
    }

    pub fn set_matrix4f(&self, name: &CStr, value: &Matrix4<f32>) -> Result<(), String> {
        let location = self.get_location(name)?;

        unsafe {
            gl::GetError();
            gl::UniformMatrix4fv(location, 1, gl::FALSE, value.as_ptr())
        }

        if let gl::NO_ERROR = unsafe { gl::GetError() } {
            Ok(())
        } else {
            Err(format!("Invalid uniform value: {value:?}"))
        }
    }

    /// # Safety
    /// Ensure the program remains valid (linked successfully)
    /// If attaching any shader, make sure to detach afterwards
    pub unsafe fn get_id(&self) -> u32 {
        self.id
    }

    fn get_location(&self, name: &CStr) -> Result<i32, String> {
        let location =
            unsafe { gl::GetUniformLocation(self.id as GLuint, name.as_ptr() as *const GLchar) };

        if location == -1 {
            Err(format!("Invalid uniform name: {name:?}"))
        } else {
            Ok(location)
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}
