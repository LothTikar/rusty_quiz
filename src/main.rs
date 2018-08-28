extern crate csv;
extern crate gl;
extern crate glfw;
extern crate image;
extern crate rusttype;

use gl::types::*;
use glfw::Context;
use image::{Rgba, RgbaImage};
use rusttype::{point, Font, Scale};
use std::env;
use std::io::Read;

struct Header {
    number_of_hints: i32,
    questions: Vec<String>,
    rendered_questions: Vec<RgbaImage>,
}

struct Slide {
    image: Option<RgbaImage>,
    hints: Vec<String>,
    answers: Vec<String>,
    rendered_hints: Vec<RgbaImage>,
    rendered_answers: Vec<RgbaImage>,
}

//All function parameters are expected to be in pixels.
fn add_text_vertices(
    window_position: (f32, f32),
    texture_offset: (f32, f32),
    text_size: (f32, f32),
    window_size: (f32, f32),
    texture_size: (f32, f32),
    verts: &mut Vec<GLfloat>,
) {
    let pos = (
        (window_position.0 * 2.0) / window_size.0 - 1.0,
        (window_position.1 * -2.0) / window_size.1 - 1.0,
    );
    let offset = (
        texture_offset.0 / texture_size.0,
        texture_offset.1 / texture_size.1,
    );
    let box_size = (
        (text_size.0 * 2.0) / window_size.0 - 1.0,
        (text_size.1 * -2.0) / window_size.1 - 1.0,
    );
    let text_size = (text_size.0 / texture_size.0, text_size.1 / texture_size.1);
	println!("{:?}",pos);
	println!("{:?}",offset);
	println!("{:?}",box_size);
	println!("{:?}",text_size);
	
    //triangle 1
    //pos
    verts.push(pos.0);
    verts.push(pos.1);
    //color
    verts.push(1.0);
    verts.push(1.0);
    verts.push(1.0);
    //tex coord
    verts.push(offset.0);
    verts.push(offset.1);

    //pos
    verts.push(pos.0 + box_size.0);
    verts.push(pos.1);
    //color
    verts.push(1.0);
    verts.push(1.0);
    verts.push(1.0);
    //tex coord
    verts.push(offset.0 + text_size.0);
    verts.push(offset.1);

    //pos
    verts.push(pos.0 + box_size.0);
    verts.push(pos.1 + box_size.1);
    //color
    verts.push(1.0);
    verts.push(1.0);
    verts.push(1.0);
    //tex coord
    verts.push(offset.0 + text_size.0);
    verts.push(offset.1 + text_size.1);

    //triangle 2
    //pos
    verts.push(pos.0);
    verts.push(pos.1);
    //color
    verts.push(1.0);
    verts.push(1.0);
    verts.push(1.0);
    //tex coord
    verts.push(offset.0);
    verts.push(offset.1);

    //pos
    verts.push(pos.0);
    verts.push(pos.1 + box_size.1);
    //color
    verts.push(1.0);
    verts.push(1.0);
    verts.push(1.0);
    //tex coord
    verts.push(offset.0);
    verts.push(offset.1 + text_size.1);

    //pos
    verts.push(pos.0 + box_size.0);
    verts.push(pos.1 + box_size.1);
    //color
    verts.push(1.0);
    verts.push(1.0);
    verts.push(1.0);
    //tex coord
    verts.push(offset.0 + text_size.0);
    verts.push(offset.1 + text_size.1);
}

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

unsafe fn set_texture_data(image: &RgbaImage) {
    gl::TexImage2D(
        gl::TEXTURE_2D,
        0,
        gl::RGBA as i32,
        image.width() as i32,
        image.height() as i32,
        0,
        gl::RGBA,
        gl::UNSIGNED_BYTE,
        image.as_ptr() as *const std::os::raw::c_void,
    );
}

unsafe fn set_vertex_data(verts: &Vec<GLfloat>) {
    gl::BufferData(
        gl::ARRAY_BUFFER,
        4 * verts.len() as isize,
        verts.as_ptr() as *const std::os::raw::c_void,
        gl::STATIC_DRAW,
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

unsafe fn vertex_buffer_setup(vert_buffer: &mut GLuint) {
    gl::GenBuffers(1, vert_buffer as *mut GLuint);
    gl::BindBuffer(gl::ARRAY_BUFFER, *vert_buffer);

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
}

fn render_text(font: &Font, scale: f32, text: &str) -> RgbaImage {
    let scale = Scale::uniform(scale);

    let v_metrics = font.v_metrics(scale);

    let glyphs: Vec<_> = font
        .layout(text, scale, point(10.0, 10.0 + v_metrics.ascent))
        .collect();

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

    let mut image = RgbaImage::new(glyphs_width + 20, glyphs_height + 20);

    for glyph in glyphs {
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            glyph.draw(|x, y, v| {
                image.put_pixel(
                    x + bounding_box.min.x as u32,
                    y + bounding_box.min.y as u32,
                    Rgba {
                        data: [255, 255, 255, (v * 255.0) as u8],
                    },
                )
            });
        }
    }

    image
}

fn main() {
    let font_data = std::fs::read("./resources/Ubuntu-R.ttf").unwrap();
    let font = Font::from_bytes(font_data.as_slice()).expect("Error constructing Font");

    let mut csv_reader = {
        let args: Vec<String> = env::args().collect();
        csv::Reader::from_path(&args[1]).unwrap()
    };

    let mut header = Header {
        number_of_hints: 0,
        questions: Vec::new(),
        rendered_questions: Vec::new(),
    };

    //It expects the first item to be "image file name." Hints are expected to only be placed immedently after "image file name." Everything after hints is expected to be a question.
    for header_item in csv_reader.headers().unwrap().iter() {
        let header_item = header_item.to_string();
        match header_item.as_str() {
            "hint" => header.number_of_hints += 1,
            "image file name" => (),
            _ => {
                header
                    .rendered_questions
                    .push(render_text(&font, 15.0, header_item.as_str()));
                header.questions.push(header_item);
            }
        }
    }

    let header = header;

    let mut slides: Vec<Slide> = Vec::new();

    for record in csv_reader.records() {
        let record = record.unwrap();

        let mut slide = Slide {
            image: None,
            hints: Vec::new(),
            answers: Vec::new(),
            rendered_hints: Vec::new(),
            rendered_answers: Vec::new(),
        };

        let mut i = 0;
        for value in record.iter() {
            let value = value.to_string();
            match i {
                0 => {
                    if !value.is_empty() {
                        slide.image = Some(image::open(value).unwrap().to_rgba());
                    }
                }
                _ if i <= header.number_of_hints => {
                    slide
                        .rendered_hints
                        .push(render_text(&font, 15.0, value.as_str()));
                    slide.hints.push(value);
                }
                _ => {
                    slide
                        .rendered_answers
                        .push(render_text(&font, 15.0, value.as_str()));
                    slide.answers.push(value);
                }
            }
            i += 1;
        }
        slides.push(slide);
    }

    let slides = slides;

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

    let mut test_offset = (0.0, 0.0);
    let mut test_size = (0.0, 0.0);

    let texture = {
        let mut texture_width = 0;
        let mut texture_height = 0;
        let mut offsets: Vec<(u32, u32)> = Vec::new();
        for slide in slides.iter() {
            let mut my_width = 0;
            let mut my_height = 0;
            for image in slide.rendered_answers.iter() {
                offsets.push((my_width, texture_height));
                my_width += image.width();
                my_height = my_height.max(image.height());
            }
            texture_width = texture_width.max(my_width);
            texture_height += my_height;
            test_size = (my_width as f32, my_height as f32);
        }
        println!("{}, {}", texture_width, texture_height);
        let mut image = RgbaImage::new(1000, 500);
        let mut offset_iter = offsets.iter();
        for slide in slides.iter() {
            for rendered_text in slide.rendered_answers.iter() {
                let offset = offset_iter.next().unwrap();
                test_offset = (offset.0 as f32, offset.1 as f32);
                for (x, y, pixel) in rendered_text.enumerate_pixels() {
                    image.put_pixel(offset.0 + x, offset.1 + y, *pixel);
                }
            }
        }
        image
    };

    unsafe {
        opengl_setup();
        setup_shaders(&vert_src, &frag_src);
        vertex_buffer_setup(&mut vert_buffer);
        texture_setup(&mut texture_buffer);
        set_texture_data(&texture);
    }

    print_gl_error();

    while !window.should_close() {
        let window_size = window.get_size();
        let window_size = (window_size.0 as f32, window_size.1 as f32);
        let cursor_pos = {
            let mut pos = window.get_cursor_pos();

            (
                pos.0.max(0.0).min(500.0) as f32,
                pos.1.max(0.0).min(500.0) as f32,
            )
        };

        let mut verts: Vec<GLfloat> = Vec::new();
        /*
        add_text_vertices(
            cursor_pos,
            test_offset,
            test_size,
            window_size,
            (texture.width() as f32, texture.height() as f32),
            &mut verts,
        );
*/

        add_text_vertices(
            (50.0, 50.0),
            (0.0, 0.0),
            (100.0, 40.0),
            window_size,
            (texture.width() as f32, texture.height() as f32),
            &mut verts,
        );

        unsafe {
            set_vertex_data(&verts);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }
        window.swap_buffers();
        glfw.poll_events();
    }
}
