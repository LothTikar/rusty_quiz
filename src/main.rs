extern crate gl;
extern crate glfw;
extern crate csv;

use glfw::Context;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    if args.len() < 2 {
        println!("Error: No filename for quiz given!");
        return;
    }

    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    let mut window = glfw
        .create_window(1000, 500, "Rusty Glider", glfw::WindowMode::Windowed)
        .unwrap()
        .0;

    gl::load_with(|s| window.get_proc_address(s) as *const std::os::raw::c_void);

    unsafe {
        gl::ClearColor(0.95, 0.95, 0.95, 0.0);
    }

    while !window.should_close() {
        let window_size = window.get_size();
        let window_size = (window_size.0 as f64, window_size.1 as f64);
        let cursor_pos = {
            let mut pos = window.get_cursor_pos();

            pos.0 = pos.0.max(0.0).min(500.0);
            pos.1 = pos.1.max(0.0).min(500.0);

            pos
        };
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        window.swap_buffers();
        glfw.poll_events();
    }
}
