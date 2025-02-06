use gl::{types::*, VERTEX_SHADER};
use glfw::{Action, Context, Glfw, GlfwReceiver, Key, PWindow, WindowEvent};
use core::str;
use std::{
    ffi::{c_void, CString},
    ptr,
};

pub struct Window {
    glfw: Glfw,
    window: PWindow,
    events: GlfwReceiver<(f64, WindowEvent)>,
    native_width: u32,
    native_height: u32,
    vao: u32,
    shader: u32,
    texture: u32,
}

const VERTEX_SHADER_SRC: &str = &"#version 330 core

out vec2 texCoords;

vec2 vertexCoordsArr[6] = vec2[](
      vec2(-1.0f, 1.0f),
      vec2(1.0f, -1.0f),
      vec2(-1.0f, -1.0f),
      vec2(-1.0f, 1.0f),
      vec2(1.0f, 1.0f),
      vec2(1.0f, -1.0f)
);
   
vec2 texCoordsArr[6] = vec2[](
      vec2(0.0f, 0.0f),
      vec2(1.0f, 1.0f),
      vec2(0.0f, 1.0f),
      vec2(0.0f, 0.0f),
      vec2(1.0f, 0.0f),
      vec2(1.0f, 1.0f)
);

void main() {
    gl_Position = vec4(vertexCoordsArr[gl_VertexID], 0.0, 1.0);
    texCoords = texCoordsArr[gl_VertexID];
}";

const FRAG_SHADER_SRC: &str = &"#version 330 core

uniform sampler2D texSampler;

in vec2 texCoords;

out vec4 outColor;

void main() {
    outColor = texture(texSampler, texCoords);
}";

impl Window {
    pub fn init(title: &str, native_width: u32, native_height: u32, scale: u32) -> Self {
        let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();

        let (mut window, events) = glfw
            .create_window(
                native_width * scale,
                native_height * scale,
                title,
                glfw::WindowMode::Windowed,
            )
            .expect("Failed to create window.");

        window.set_key_polling(true);
        window.make_current();

        gl::load_with(|ptr| window.get_proc_address(ptr) as *const _);

        let mut vao_ptr: *mut u32 = std::ptr::null_mut();
        let mut vao;
        let shader = create_shader();
        let texture = create_texture();

        unsafe {
            gl::GenVertexArrays(1, vao_ptr);
            vao = *vao_ptr;

            gl::Viewport(
                0,
                0,
                (native_width * scale) as i32,
                (native_height * scale) as i32,
            );

            gl::ClearColor(0.0, 0.0, 0.0, 1.0);

            gl::BindVertexArray(vao);
        }

        Self {
            glfw,
            window,
            events,
            native_width,
            native_height,
            vao,
            shader,
            texture,
        }
    }

    pub fn poll_events(&mut self) {
        self.glfw.poll_events();
        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    self.window.set_should_close(true)
                }
                _ => (),
            }
        }
    }

    pub fn closed(&self) -> bool {
        self.window.should_close()
    }

    pub fn render(&mut self, frame: &[u8]) {
        unsafe {
            let frame = frame.as_ptr() as *mut c_void;

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB as i32,
                self.native_width as i32,
                self.native_height as i32,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                frame,
            );

            gl::GenerateMipmap(gl::TEXTURE_2D);

            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }

        self.window.swap_buffers();

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }
}

fn create_shader() -> u32 {
    let vertex_shader_src = CString::new(VERTEX_SHADER_SRC.as_bytes()).unwrap();
    let frag_shader_src = CString::new(FRAG_SHADER_SRC.as_bytes()).unwrap();

    unsafe {
        let shader = gl::CreateProgram();
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        let frag_shader = gl::CreateShader(gl::FRAGMENT_SHADER);

        gl::ShaderSource(vertex_shader, 1, &vertex_shader_src.as_ptr(), ptr::null());
        gl::CompileShader(vertex_shader);

        gl::ShaderSource(frag_shader, 1, &frag_shader_src.as_ptr(), ptr::null());
        gl::CompileShader(frag_shader);

        gl::AttachShader(shader, vertex_shader);
        gl::AttachShader(shader, frag_shader);
        gl::LinkProgram(shader);

        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(frag_shader);

        gl::UseProgram(shader);

        shader
    }
}

fn create_texture() -> u32 {
    let mut texture: *mut u32 = std::ptr::null_mut();

    unsafe {
        gl::GenTextures(1, texture);

        gl::ActiveTexture(gl::TEXTURE0);

        gl::BindTexture(gl::TEXTURE_2D, *texture);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

        *texture
    }
}
