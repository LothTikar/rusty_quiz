extern crate csv;
extern crate gl;
extern crate glfw;

use gl::types::*;
use glfw::Context;
use std::env;
use std::io::Read;

fn print_gl_error() {
    println!(
        "{}",
        match unsafe { gl::GetError() } {
            gl::NO_ERROR => "GL_NO_ERROR",
            gl::INVALID_ENUM => "GL_INVALID_ENUM",
            gl::INVALID_VALUE => "GL_INVALID_VALUE",
            gl::INVALID_OPERATION => "GL_INVALID_OPERATION",
            gl::INVALID_FRAMEBUFFER_OPERATION => "GL_INVALID_FRAMEBUFFER_OPERATION",
            gl::OUT_OF_MEMORY => "GL_OUT_OF_MEMORY",
            _ => panic!("gl::GetError() giving unknown value!"),
        }
    );
}

fn print_shader_log(shader: GLuint) {
    let mut program_log: Vec<u8> = Vec::new();
    program_log.resize(10000, 0);
    let mut log_size: GLsizei = 0;
    unsafe {
        gl::GetShaderInfoLog(
            shader,
            program_log.len() as i32,
            &mut log_size,
            program_log.as_mut_ptr() as *mut i8,
        );
    }
    program_log.resize(log_size as usize, 0);
    println!("log_size:{}", log_size);
    println!("program_log:");
    println!("{}", std::string::String::from_utf8(program_log).unwrap())
}

unsafe fn setup_shaders(vert_src: &String, frag_src: &String) {
    let vert_shader = gl::CreateShader(gl::VERTEX_SHADER);
    let frag_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
    {
        let s = std::mem::transmute::<&u8, *const GLchar>(&vert_src.as_bytes()[0]);
        gl::ShaderSource(
            vert_shader,
            1,
            &s as *const *const GLchar,
            std::mem::transmute::<&usize, *const GLint>(&vert_src.len()),
        );
    }
    {
        let s = std::mem::transmute::<&u8, *const GLchar>(&frag_src.as_bytes()[0]);
        gl::ShaderSource(
            frag_shader,
            1,
            &s as *const *const GLchar,
            std::mem::transmute::<&usize, *const GLint>(&frag_src.len()),
        );
    }
    gl::CompileShader(vert_shader);
    gl::CompileShader(frag_shader);

    print_shader_log(vert_shader);
    print_shader_log(frag_shader);

    let shader = gl::CreateProgram();

    gl::AttachShader(shader, vert_shader);
    gl::AttachShader(shader, frag_shader);

    gl::LinkProgram(shader);
    gl::UseProgram(shader);
}

fn main() {
    //debug code
    let path = env::current_dir().unwrap();
    println!("The current directory is {}", path.display());

    let mut csv_reader = {
        let args: Vec<String> = env::args().collect();
        csv::Reader::from_path(&args[1]).unwrap()
    };

    let mut headers: Vec<String> = Vec::new();

    for header in csv_reader.headers().unwrap().iter() {
        headers.push(header.to_string());
    }

    let mut content: Vec<Vec<String>> = Vec::new();

    for result in csv_reader.records() {
        content.push({
            let mut record: Vec<String> = Vec::new();
            for header in result.unwrap().iter() {
                record.push(header.to_string());
            }
            record
        });
    }

    //debug code
    println!("header");
    println!("{:?}", headers);
    println!("content");
    println!("{:?}", content);

    let vert_src = {
        let mut file = std::fs::File::open("./resources/vert.glsl").unwrap();
        let mut src = String::new();
        file.read_to_string(&mut src).unwrap();
        src
    };

    let frag_src = {
        let mut file = std::fs::File::open("./resources/frag.glsl").unwrap();
        let mut src = String::new();
        file.read_to_string(&mut src).unwrap();
        src
    };

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

    let mut vert_buffer: GLuint = 0;
    // let mut color_buffer: GLuint = 0;

    let mut verts: Vec<GLfloat> = Vec::with_capacity(3 * 3);

    verts.push(-0.5);
    verts.push(-0.5);

    verts.push(1.0);
    verts.push(0.0);
    verts.push(0.0);

    verts.push(0.5);
    verts.push(-0.5);

    verts.push(0.0);
    verts.push(1.0);
    verts.push(0.0);

    verts.push(-0.5);
    verts.push(0.5);

    verts.push(0.0);
    verts.push(0.0);
    verts.push(1.0);

    println!("number of tris:{}", verts.len());

    unsafe {
        let mut vao: GLuint = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        gl::ClearColor(0.95, 0.95, 0.95, 0.0);
        gl::Enable(gl::DEPTH_TEST);

        setup_shaders(&vert_src, &frag_src);

        gl::GenBuffers(1, &mut vert_buffer as *mut GLuint);
        gl::BindBuffer(gl::ARRAY_BUFFER, vert_buffer);

        println!("vert_buffer:{}", vert_buffer);

        gl::EnableVertexAttribArray(0);
        gl::EnableVertexAttribArray(1);

        print_gl_error();
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 4 * 5, std::ptr::null());
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            4 * 5,
            std::mem::transmute::<u64, *const std::os::raw::c_void>(2 * 4),
        );
        print_gl_error();

        gl::BufferData(
            gl::ARRAY_BUFFER,
            4 * verts.len() as isize,
            verts.as_ptr() as *const std::os::raw::c_void,
            gl::STATIC_DRAW,
        );
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
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }
        window.swap_buffers();
        glfw.poll_events();
    }
}
