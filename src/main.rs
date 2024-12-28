use gl::types::*;
use gl_lib::Shader;
use std::{
    ffi::{c_void, CStr},
    path::Path,
    ptr,
};

use glfw::{Action, Context, Key};

// settings
const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

// shader code
const VERTEX_SHADER: &CStr = cr#"
#version 330 core
layout (location = 0) in vec3 inPos;
layout (location = 1) in vec2 inTexCoord;
out vec2 texCoord;

void main() {
    gl_Position = vec4(inPos.xyz, 1.0f);
    texCoord = inTexCoord;
}
"#;

const FRAGMENT_SHADER: &CStr = cr#"
#version 330 core
out vec4 FragColor;
in vec2 texCoord;

uniform sampler2D boxTexture;

void main() {
    FragColor = texture(boxTexture, texCoord);
}
"#;

fn main() {
    // glfw initialization
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    // glfw window creation
    let (mut window, events) = glfw
        .create_window(
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            "Rust Craft",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window");

    window.make_current();
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);

    // load all OpenGL function pointers
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    #[rustfmt::skip]
    let vertices = [
        // Position       // Texture
        -0.5, -0.5,  0.0, 0.0, 0.0,
        -0.5,  0.5,  0.0, 0.0, 1.0,
         0.5, -0.5,  0.0, 1.0, 0.0,
         0.5,  0.5,  0.0, 1.0, 1.0f32,
    ];

    #[rustfmt::skip]
    let indices = [
        0, 1, 2,
        1, 2, 3u32,
    ];

    let (vao, program, texture) = {
        let vao = unsafe {
            let mut vao = 0;
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            let mut vbo = 0;
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                size_of_val(&vertices) as GLsizeiptr,
                vertices.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            gl::VertexAttribPointer(
                0,
                3 as GLint,
                gl::FLOAT,
                gl::FALSE,
                5 * size_of_val(&vertices[0]) as GLint,
                ptr::null(),
            );
            gl::EnableVertexAttribArray(0);

            gl::VertexAttribPointer(
                1,
                2 as GLint,
                gl::FLOAT,
                gl::FALSE,
                5 * size_of_val(&vertices[0]) as GLint,
                (3 * size_of_val(&vertices[0])) as *const c_void,
            );
            gl::EnableVertexAttribArray(1);

            let mut ebo = 0;
            gl::GenBuffers(1, &mut ebo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                size_of_val(&indices) as GLsizeiptr,
                indices.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            gl::BindVertexArray(0);

            vao
        };

        let vertex_shader = gl_lib::VertexShader::from_cstr(VERTEX_SHADER).unwrap();
        let fragment_shader = gl_lib::FragmentShader::from_cstr(FRAGMENT_SHADER).unwrap();

        let program = gl_lib::Program::new(&vertex_shader, &fragment_shader, None).unwrap();

        let texture_image =
            image::open(Path::new("asset/box.jpg")).expect("Cannot open box texture");
        let texture_data = texture_image
            .as_flat_samples_u8()
            .expect("Cannot flatten texture image")
            .samples;

        let texture = unsafe {
            let mut texture = 0;
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB as GLint,
                texture_image.width() as GLint,
                texture_image.height() as GLint,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                texture_data.as_ptr() as *const c_void,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
            gl::BindTexture(gl::TEXTURE_2D, 0);

            texture
        };

        (vao, program, texture)
    };

    // render loop
    while !window.should_close() {
        process_events(&mut window, &events);

        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::BindVertexArray(vao);
            program.use_program();
            gl::BindTexture(gl::TEXTURE_2D, texture);

            gl::DrawElements(
                gl::TRIANGLES,
                indices.len() as GLsizei,
                gl::UNSIGNED_INT,
                ptr::null(),
            );

            gl::BindTexture(gl::TEXTURE_2D, 0);
            gl_lib::Program::unset_program();
            gl::BindVertexArray(0);
        }

        window.swap_buffers();
        glfw.poll_events();
    }
}

fn process_events(
    window: &mut glfw::Window,
    events: &glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::FramebufferSize(width, height) => {
                // make sure the viewport matches the new window dimensions; note that width and
                // height will be significantly larger than specified on retina displays.
                unsafe { gl::Viewport(0, 0, width, height) }
            }
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                window.set_should_close(true)
            }
            _ => {}
        }
    }
}
