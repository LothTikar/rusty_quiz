extern crate csv;
extern crate gl;
extern crate glfw;
extern crate image;
extern crate rusttype;

use gl::types::*;
use glfw::Context;
use image::{DynamicImage, GenericImage, Rgba};
use std::env;
use std::io::Read;
use rusttype::{point, Font, Scale};

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

unsafe fn texture_setup(texture: &mut GLuint) {
    gl::GenTextures(1, texture);
    gl::BindTexture(gl::TEXTURE_2D, *texture);

    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
}

unsafe fn set_texture_data(image: &image::DynamicImage) {
    gl::TexImage2D(
        gl::TEXTURE_2D,
        0,
        gl::RGBA as i32,
        image.width() as i32,
        image.height() as i32,
        0,
        gl::RGBA,
        gl::UNSIGNED_BYTE,
        image.raw_pixels().as_ptr() as *const std::os::raw::c_void,
    );
}

unsafe fn opengl_setup() {
    let mut vao: GLuint = 0;
    gl::GenVertexArrays(1, &mut vao);
    gl::BindVertexArray(vao);

    gl::ClearColor(0.95, 0.95, 0.95, 0.0);
    gl::Enable(gl::DEPTH_TEST);

    gl::Enable(gl::BLEND);
    gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
}

unsafe fn vertex_buffer_setup(vert_buffer: &mut GLuint, verts: &Vec<GLfloat>) {
    gl::GenBuffers(1, vert_buffer as *mut GLuint);
    gl::BindBuffer(gl::ARRAY_BUFFER, *vert_buffer);

    println!("vert_buffer:{}", vert_buffer);

    gl::EnableVertexAttribArray(0);
    gl::EnableVertexAttribArray(1);
    gl::EnableVertexAttribArray(2);

    gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 7 * 4, std::ptr::null());
    gl::VertexAttribPointer(
        1,
        3,
        gl::FLOAT,
        gl::FALSE,
        7 * 4,
        std::mem::transmute::<u64, *const std::os::raw::c_void>(2 * 4),
    );
    gl::VertexAttribPointer(
        2,
        2,
        gl::FLOAT,
        gl::FALSE,
        7 * 4,
        std::mem::transmute::<u64, *const std::os::raw::c_void>(5 * 4),
    );

    gl::BufferData(
        gl::ARRAY_BUFFER,
        4 * verts.len() as isize,
        verts.as_ptr() as *const std::os::raw::c_void,
        gl::STATIC_DRAW,
    );
}

fn main() {
    // Load the font
    let font_data = std::fs::read("./resources/Ubuntu-R.ttf").unwrap();
    // This only succeeds if collection consists of one font
    let font = Font::from_bytes(font_data.as_slice()).expect("Error constructing Font");

    // The font size to use
    let scale = Scale::uniform(32.0);

    // The text to render
    let text = "This is RustType rendered into a png!";

    let v_metrics = font.v_metrics(scale);

    // layout the glyphs in a line with 20 pixels padding
    let glyphs: Vec<_> = font
        .layout(text, scale, point(20.0, 20.0 + v_metrics.ascent))
        .collect();

    // work out the layout size
    let glyphs_height = (v_metrics.ascent - v_metrics.descent).ceil() as u32;
    let glyphs_width = {
        let min_x = glyphs
            .first()
            .map(|g| g.pixel_bounding_box().unwrap().min.x)
            .unwrap();
        let max_x = glyphs
            .last()
            .map(|g| g.pixel_bounding_box().unwrap().max.x)
            .unwrap();
        (max_x - min_x) as u32
    };

    // Create a new rgba image with some padding
    let mut image = DynamicImage::new_rgba8(glyphs_width + 40, glyphs_height + 40);

    // Loop through the glyphs in the text, positing each one on a line
    for glyph in glyphs {
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            // Draw the glyph into the image per-pixel by using the draw closure
            glyph.draw(|x, y, v| {
                image.put_pixel(
                    // Offset the position by the glyph bounding box
                    x + bounding_box.min.x as u32,
                    y + bounding_box.min.y as u32,
                    // Turn the coverage into an alpha value
                    Rgba {
                        data: [255, 255, 255, (v * 255.0) as u8],
                    },
                )
            });
        }
    }




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

    let img_right = image::open("./resources/icons8-checked-50.png").unwrap();
	let img_wrong = image::open("./resources/icons8-cancel-50.png").unwrap();
	let img_continue = image::open("./resources/icons8-circled-right-50.png").unwrap();

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
    let mut texture_buffer: GLuint = 0;

    let mut verts: Vec<GLfloat> = Vec::new();

    //pos
    verts.push(-0.5);
    verts.push(-0.5);
    //color
    verts.push(1.0);
    verts.push(0.0);
    verts.push(0.0);
    //tex coord
    verts.push(0.0);
    verts.push(1.0);

    //pos
    verts.push(0.5);
    verts.push(-0.5);
    //color
    verts.push(0.0);
    verts.push(1.0);
    verts.push(0.0);
    //tex coord
    verts.push(1.0);
    verts.push(1.0);

    //pos
    verts.push(-0.5);
    verts.push(0.5);
    //color
    verts.push(0.0);
    verts.push(0.0);
    verts.push(1.0);
    //tex coord
    verts.push(0.0);
    verts.push(0.0);

    //pos
    verts.push(-0.5);
    verts.push(0.5);
    //color
    verts.push(0.0);
    verts.push(0.0);
    verts.push(1.0);
    //tex coord
    verts.push(0.0);
    verts.push(0.0);

    //pos
    verts.push(0.5);
    verts.push(0.5);
    //color
    verts.push(1.0);
    verts.push(1.0);
    verts.push(1.0);
    //tex coord
    verts.push(1.0);
    verts.push(0.0);

    //pos
    verts.push(0.5);
    verts.push(-0.5);
    //color
    verts.push(0.0);
    verts.push(1.0);
    verts.push(0.0);
    //tex coord
    verts.push(1.0);
    verts.push(1.0);

    println!("number of verts:{}", verts.len());

    unsafe {
		opengl_setup();
        setup_shaders(&vert_src, &frag_src);
        vertex_buffer_setup(&mut vert_buffer, &verts);
        texture_setup(&mut texture_buffer);
        set_texture_data(&image);
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
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }
        window.swap_buffers();
        glfw.poll_events();
    }
}
