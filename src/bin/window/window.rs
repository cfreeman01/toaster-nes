use gl::types::*;
use glfw::{Action, Context, Glfw, GlfwReceiver, Key, PWindow, WindowEvent};

pub struct Window {
    glfw: Glfw,
    window: PWindow,
    events: GlfwReceiver<(f64, WindowEvent)>,
}

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

        unsafe {
            gl::Viewport(
                0,
                0,
                (native_width * scale) as i32,
                (native_height * scale) as i32,
            );
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        }

        Self {
            glfw,
            window,
            events,
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
        self.window.swap_buffers();

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }
}
