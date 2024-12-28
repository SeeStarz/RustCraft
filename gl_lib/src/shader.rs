use gl::types::*;
use std::ffi::CStr;
use std::fmt;
use std::ops::Drop;
use std::ptr;

pub trait Shader: private::Sealed {
    fn from_cstr(source: &CStr) -> Result<Self, String>
    where
        Self: Sized;

    /// # Safety
    /// Make sure id is a valid OpenGL shader of the correct type
    /// Shader struct represents a valid compiled OpenGL shader
    unsafe fn from_id(id: u32) -> Self
    where
        Self: Sized;

    /// # Safety
    /// Do not delete the shader, it will automatically get deleted when it's dropped
    unsafe fn get_id(&self) -> u32;
}

pub struct VertexShader {
    id: u32,
}
pub struct TessControlShader {
    id: u32,
}
pub struct TessEvaluationShader {
    id: u32,
}
pub struct GeometryShader {
    id: u32,
}
pub struct FragmentShader {
    id: u32,
}

impl private::Sealed for VertexShader {}
impl private::Sealed for TessControlShader {}
impl private::Sealed for TessEvaluationShader {}
impl private::Sealed for GeometryShader {}
impl private::Sealed for FragmentShader {}

impl Shader for VertexShader {
    fn from_cstr(source: &CStr) -> Result<Self, String> {
        let result = create_shader(source, ShaderType::Vertex);
        match result {
            Ok(id) => Ok(VertexShader { id }),
            Err(string) => Err(string),
        }
    }
    unsafe fn from_id(id: u32) -> Self {
        VertexShader { id }
    }
    unsafe fn get_id(&self) -> u32 {
        self.id
    }
}

impl Shader for TessControlShader {
    fn from_cstr(source: &CStr) -> Result<Self, String> {
        let result = create_shader(source, ShaderType::TessControl);
        match result {
            Ok(id) => Ok(TessControlShader { id }),
            Err(string) => Err(string),
        }
    }
    unsafe fn from_id(id: u32) -> Self {
        TessControlShader { id }
    }
    unsafe fn get_id(&self) -> u32 {
        self.id
    }
}

impl Shader for TessEvaluationShader {
    fn from_cstr(source: &CStr) -> Result<Self, String> {
        let result = create_shader(source, ShaderType::TessEvaluation);
        match result {
            Ok(id) => Ok(TessEvaluationShader { id }),
            Err(string) => Err(string),
        }
    }
    unsafe fn from_id(id: u32) -> Self {
        TessEvaluationShader { id }
    }
    unsafe fn get_id(&self) -> u32 {
        self.id
    }
}

impl Shader for GeometryShader {
    fn from_cstr(source: &CStr) -> Result<Self, String> {
        let result = create_shader(source, ShaderType::Geometry);
        match result {
            Ok(id) => Ok(GeometryShader { id }),
            Err(string) => Err(string),
        }
    }
    unsafe fn from_id(id: u32) -> Self {
        GeometryShader { id }
    }
    unsafe fn get_id(&self) -> u32 {
        self.id
    }
}

impl Shader for FragmentShader {
    fn from_cstr(source: &CStr) -> Result<Self, String> {
        let result = create_shader(source, ShaderType::Fragment);
        match result {
            Ok(id) => Ok(FragmentShader { id }),
            Err(string) => Err(string),
        }
    }
    unsafe fn from_id(id: u32) -> Self {
        FragmentShader { id }
    }
    unsafe fn get_id(&self) -> u32 {
        self.id
    }
}

impl Drop for VertexShader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

impl Drop for TessControlShader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

impl Drop for TessEvaluationShader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

impl Drop for GeometryShader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

impl Drop for FragmentShader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

mod private {
    pub trait Sealed {}
}

fn create_shader(source: &CStr, shader_type: ShaderType) -> Result<u32, String> {
    unsafe {
        // Reset any error beforehand
        gl::GetError();

        // Create shader object on the GPU
        let id = gl::CreateShader(shader_type.to_opengl());
        if id == 0 {
            return Err(String::from("OpenGL failed to create shader object"));
        }

        // Send data to the GPU
        gl::ShaderSource(
            id as GLuint,
            1 as GLsizei,
            &source.as_ptr() as *const *const GLchar,
            &(source.to_bytes().len() as GLint) as *const GLint,
        );

        gl::CompileShader(id as GLuint);

        // Check if compilation succeed
        let mut status = 0;
        gl::GetShaderiv(id as GLuint, gl::COMPILE_STATUS, &mut status as *mut GLint);

        // No error
        if status as GLboolean == gl::TRUE {
            assert_eq!(gl::NO_ERROR, gl::GetError());
            return Ok(id);
        }

        let mut info_length = 0;
        gl::GetShaderiv(
            id as GLuint,
            gl::INFO_LOG_LENGTH,
            &mut info_length as *mut GLint,
        );

        if info_length == 0 {
            gl::DeleteShader(id);
            assert_eq!(gl::NO_ERROR, gl::GetError());
            return Err(format!(
                "Failed to compile {shader_type}, no info log available."
            ));
        }

        let mut info_log: Vec<u8> = Vec::with_capacity((info_length - 1) as usize);
        gl::GetShaderInfoLog(
            id as GLuint,
            (info_length - 1) as GLsizei,
            ptr::null_mut() as *mut GLsizei,
            info_log.as_mut_ptr() as *mut GLchar,
        );

        gl::DeleteShader(id);
        assert_eq!(gl::NO_ERROR, gl::GetError());
        if let Ok(string) = String::from_utf8(info_log) {
            Err(format!("Failed to compile {shader_type}: {string}"))
        } else {
            Err(format!(
                "Failed to compile {shader_type}, info log cannot be parsed to UTF-8."
            ))
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum ShaderType {
    // ComputeShader, // Only for OpenGL 4.3+
    Vertex,
    TessControl,
    TessEvaluation,
    Geometry,
    Fragment,
}

impl ShaderType {
    fn to_opengl(self) -> GLenum {
        match self {
            // Self::ComputeShader => gl::COMPUTE_SHADER,
            Self::Vertex => gl::VERTEX_SHADER,
            Self::TessControl => gl::TESS_CONTROL_SHADER,
            Self::TessEvaluation => gl::TESS_EVALUATION_SHADER,
            Self::Geometry => gl::GEOMETRY_SHADER,
            Self::Fragment => gl::FRAGMENT_SHADER,
        }
    }
}

impl fmt::Display for ShaderType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match self {
            // Self::ComputeShader => String::from("COMPUTE_SHADER"),
            Self::Vertex => String::from("VERTEX_SHADER"),
            Self::TessControl => String::from("TESS_CONTROL_SHADER"),
            Self::TessEvaluation => String::from("TESS_EVALUATION_SHADER"),
            Self::Geometry => String::from("GEOMETRY_SHADER"),
            Self::Fragment => String::from("FRAGMENT_SHADER"),
        };
        write!(f, "{}", string)
    }
}
