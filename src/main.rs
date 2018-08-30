extern crate csv;
extern crate gl;
extern crate glfw;
extern crate image;
extern crate rand;
extern crate rusttype;

use gl::types::*;
use glfw::{Action, Context, Key};
use image::{Rgba, RgbaImage};
use rand::{thread_rng, Rng};
use rusttype::{point, Font, Scale};
use std::env;
use std::io::Read;

struct Header {
    number_of_hints: i32,
    questions: Vec<String>,
}

struct Slide {
    image: Option<RgbaImage>,
    hints: Vec<String>,
    answers: Vec<String>,
}

//All function parameters are expected to be in pixels.
fn add_textured_box(
    window_position: (f32, f32),
    layer: f32,
    texture_offset: (f32, f32),
    color: (f32, f32, f32),
    text_size: (f32, f32),
    window_size: (f32, f32),
    texture_size: (f32, f32),
    verts: &mut Vec<GLfloat>,
) {
    let pos = (
        (window_position.0 * 2.0) / window_size.0 - 1.0,
        (window_position.1 * -2.0) / window_size.1 + 1.0,
    );
    let offset = (
        texture_offset.0 / texture_size.0,
        texture_offset.1 / texture_size.1,
    );
    let box_size = (
        (text_size.0 * 2.0) / window_size.0,
        (text_size.1 * -2.0) / window_size.1,
    );
    let text_size = (text_size.0 / texture_size.0, text_size.1 / texture_size.1);

    //triangle 1
    //pos
    verts.push(pos.0);
    verts.push(pos.1);
    verts.push(layer);
    //color
    verts.push(color.0);
    verts.push(color.1);
    verts.push(color.2);
    //tex coord
    verts.push(offset.0);
    verts.push(offset.1);
    //enable texture
    verts.push(1.0);

    //pos
    verts.push(pos.0 + box_size.0);
    verts.push(pos.1);
    verts.push(layer);
    //color
    verts.push(color.0);
    verts.push(color.1);
    verts.push(color.2);
    //tex coord
    verts.push(offset.0 + text_size.0);
    verts.push(offset.1);
    //enable texture
    verts.push(1.0);

    //pos
    verts.push(pos.0 + box_size.0);
    verts.push(pos.1 + box_size.1);
    verts.push(layer);
    //color
    verts.push(color.0);
    verts.push(color.1);
    verts.push(color.2);
    //tex coord
    verts.push(offset.0 + text_size.0);
    verts.push(offset.1 + text_size.1);
    //enable texture
    verts.push(1.0);

    //triangle 2
    //pos
    verts.push(pos.0);
    verts.push(pos.1);
    verts.push(layer);
    //color
    verts.push(color.0);
    verts.push(color.1);
    verts.push(color.2);
    //tex coord
    verts.push(offset.0);
    verts.push(offset.1);
    //enable texture
    verts.push(1.0);

    //pos
    verts.push(pos.0);
    verts.push(pos.1 + box_size.1);
    verts.push(layer);
    //color
    verts.push(color.0);
    verts.push(color.1);
    verts.push(color.2);
    //tex coord
    verts.push(offset.0);
    verts.push(offset.1 + text_size.1);
    //enable texture
    verts.push(1.0);

    //pos
    verts.push(pos.0 + box_size.0);
    verts.push(pos.1 + box_size.1);
    verts.push(layer);
    //color
    verts.push(color.0);
    verts.push(color.1);
    verts.push(color.2);
    //tex coord
    verts.push(offset.0 + text_size.0);
    verts.push(offset.1 + text_size.1);
    //enable texture
    verts.push(1.0);
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
    gl::DepthFunc(gl::LESS);

    gl::Enable(gl::BLEND);
    gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
}

unsafe fn vertex_buffer_setup(vert_buffer: &mut GLuint) {
    gl::GenBuffers(1, vert_buffer as *mut GLuint);
    gl::BindBuffer(gl::ARRAY_BUFFER, *vert_buffer);

    gl::EnableVertexAttribArray(0);
    gl::EnableVertexAttribArray(1);
    gl::EnableVertexAttribArray(2);
    gl::EnableVertexAttribArray(3);

    gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 9 * 4, std::ptr::null());
    gl::VertexAttribPointer(
        1,
        3,
        gl::FLOAT,
        gl::FALSE,
        9 * 4,
        std::mem::transmute::<u64, *const std::os::raw::c_void>(3 * 4),
    );
    gl::VertexAttribPointer(
        2,
        2,
        gl::FLOAT,
        gl::FALSE,
        9 * 4,
        std::mem::transmute::<u64, *const std::os::raw::c_void>(6 * 4),
    );
    gl::VertexAttribPointer(
        3,
        1,
        gl::FLOAT,
        gl::FALSE,
        9 * 4,
        std::mem::transmute::<u64, *const std::os::raw::c_void>(8 * 4),
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

fn copy_image_into_image(
    source_image: &RgbaImage,
    offset: (u32, u32),
    destination_image: &mut RgbaImage,
) {
    for (x, y, pixel) in source_image.enumerate_pixels() {
        destination_image.put_pixel(offset.0 + x, offset.1 + y, *pixel);
    }
}

fn read_slides(csv_reader: &mut csv::Reader<std::fs::File>, header: &Header) -> Vec<Slide> {
    let mut slides: Vec<Slide> = Vec::new();

    for record in csv_reader.records() {
        let record = record.unwrap();

        let mut slide = Slide {
            image: None,
            hints: Vec::new(),
            answers: Vec::new(),
        };

        let mut i = 0;
        for value in record.iter() {
            let value = value.to_string();
            match i {
                0 => {
                    if !value.is_empty() {
                        slide.image = Some(
                            image::open(value)
                                .expect("Unable to open slide image!")
                                .to_rgba(),
                        );
                    }
                }
                _ if i <= header.number_of_hints => {
                    slide.hints.push(value);
                }
                _ => {
                    slide.answers.push(value);
                }
            }
            i += 1;
        }
        slides.push(slide);
    }

    slides
}

fn read_header(csv_reader: &mut csv::Reader<std::fs::File>) -> Header {
    let mut header = Header {
        number_of_hints: 0,
        questions: Vec::new(),
    };

    //It expects the first item to be "image file name." Hints are expected to only be placed immedently after "image file name." Everything after hints is expected to be a question.
    for header_item in csv_reader.headers().unwrap().iter() {
        let header_item = header_item.to_string();
        match header_item.as_str() {
            "hint" => header.number_of_hints += 1,
            "image file name" => (),
            _ => {
                header.questions.push(header_item);
            }
        }
    }

    header
}

fn generate_question(
    category: usize,
    header: &Header,
    slide: &Slide,
    all_slides: &Vec<Slide>,
) -> usize {
    let mut rng = thread_rng();

    let mut answers: Vec<String> = Vec::new();
    let right_answer_index = rng.gen_range(0, 3);

    while answers.len() < 4 {
        if answers.len() == right_answer_index {
            answers.push(slide.answers[category].clone());
        } else {
            let answer = &rng
                .choose(all_slides)
                .expect("all_slides empty! Please fix!")
                .answers[category];
            if *answer != slide.answers[category] {
                answers.push(answer.clone());
            }
        }
    }

    println!("{}?", header.questions[category]);
    for i in 0..4 {
        println!("[{}] {}", i + 1, answers[i]);
    }

    right_answer_index + 1
}

fn generate_slide_texture(
    slide: &Slide,
    hint_image_sizes: &mut Vec<(f32, f32)>,
    hint_offsets: &mut Vec<(f32, f32)>,
    font: &Font,
) -> RgbaImage {
    let mut size: (f32, f32) = (0.0, 0.0);
    let mut hint_images: Vec<RgbaImage> = Vec::new();

    for h in slide.hints.iter() {
        let image = render_text(&font, 20.0, &h);
        size.0 += image.width() as f32;
        size.1 = size.1.max(image.height() as f32);
        hint_image_sizes.push((image.width() as f32, image.height() as f32));
        hint_images.push(image);
    }

    let mut offset = (0, 0);

    if let Some(ref image) = slide.image {
        size.0 += image.width() as f32;
        size.1 = size.1.max(image.height() as f32);
        offset.0 = image.width();
    }
    let mut texture = RgbaImage::new(size.0 as u32, size.1 as u32);

    if let Some(ref image) = slide.image {
        copy_image_into_image(image, (0, 0), &mut texture);
    }
    for i in hint_images.iter() {
        copy_image_into_image(&i, offset, &mut texture);
        hint_offsets.push((offset.0 as f32, offset.1 as f32));
        offset.1 += i.height();
    }
    texture
}

fn main() {
    let font_data = std::fs::read("./resources/Ubuntu-R.ttf").expect("Unable to open font file!");
    let font = Font::from_bytes(font_data.as_slice()).expect("Error constructing Font");

    let mut csv_reader = {
        let args: Vec<String> = env::args().collect();
        csv::Reader::from_path(
            &args
                .get(1)
                .expect("No argument provided for quiz filename!"),
        ).expect("Quiz file processing error!")
    };

    let header = read_header(&mut csv_reader);

    let slides = {
        let mut slides = read_slides(&mut csv_reader, &header);

        thread_rng().shuffle(&mut slides);

        slides
    };

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
        .create_window(1000, 500, "Rusty Quiz", glfw::WindowMode::Windowed)
        .unwrap()
        .0;

    gl::load_with(|s| window.get_proc_address(s) as *const std::os::raw::c_void);

    let mut vert_buffer: GLuint = 0;
    let mut texture_buffer: GLuint = 0;

    unsafe {
        opengl_setup();
        setup_shaders(&vert_src, &frag_src);
        vertex_buffer_setup(&mut vert_buffer);
        texture_setup(&mut texture_buffer);
    }

    print_gl_error();

    let mut texture: RgbaImage = RgbaImage::new(0, 0);
    let mut hint_offsets: Vec<(f32, f32)> = Vec::new();
    let mut hint_image_sizes: Vec<(f32, f32)> = Vec::new();

    let mut slides_iter = slides.iter();
    let mut current_slide = slides_iter.next();
    let mut current_category = 0;
    let mut right_answer = 0;
    let mut next_question = false;

    let mut already_guessed = false;
    let mut number_right = 0;
    let mut number_wrong = 0;

    let mut old_key_state: [bool; 4] = [false; 4];
    let mut key_activated: [bool; 4] = [false; 4];

    if let Some(slide) = current_slide {
        right_answer = generate_question(current_category, &header, &slide, &slides);
        texture = generate_slide_texture(&slide, &mut hint_image_sizes, &mut hint_offsets, &font);
        unsafe {
            set_texture_data(&texture);
        }
    }

    while !window.should_close() && current_slide.is_some() {
        if let Some(slide) = current_slide {
            let window_size = window.get_size();
            let window_size = (window_size.0 as f32, window_size.1 as f32);
            for i in 0..4 {
                if key_activated[i] {
                    if i + 1 == right_answer {
                        println!("Answer #{} is correct", right_answer);
                        if !already_guessed {
                            number_right += 1;
                        }
                        next_question = true;
                    } else {
                        println!("Try again!");
                        if !already_guessed {
                            number_wrong += 1;
                            already_guessed = true;
                        }
                    }
                }
            }

            let mut verts: Vec<GLfloat> = Vec::new();

            if let Some(ref image) = slide.image {
                add_textured_box(
                    (0.0, 0.0),
                    0.0,
                    (0.0, 0.0),
                    (1.0, 1.0, 1.0),
                    (image.width() as f32, image.height() as f32),
                    window_size,
                    (texture.width() as f32, texture.height() as f32),
                    &mut verts,
                );
            }

            for i in 0..hint_offsets.len() {
                add_textured_box(
                    hint_offsets[i],
                    0.0,
                    hint_offsets[i],
                    (0.1, 0.1, 0.1),
                    hint_image_sizes[i],
                    window_size,
                    (texture.width() as f32, texture.height() as f32),
                    &mut verts,
                );
            }

            unsafe {
                set_vertex_data(&verts);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                let mut number_of_tris = hint_offsets.len() * 6;
                if slide.image.is_some() {
                    number_of_tris += 6;
                }
                gl::DrawArrays(gl::TRIANGLES, 0, number_of_tris as i32);
            }
            window.swap_buffers();

            if next_question {
                next_question = false;
                already_guessed = false;
                current_category += 1;
                if current_category < header.questions.len() {
                    right_answer = generate_question(current_category, &header, &slide, &slides);
                } else {
                    current_slide = slides_iter.next();
                    current_category = 0;
                    if let Some(slide) = current_slide {
                        right_answer =
                            generate_question(current_category, &header, &slide, &slides);
                        texture = generate_slide_texture(
                            &slide,
                            &mut hint_image_sizes,
                            &mut hint_offsets,
                            &font,
                        );
                        unsafe {
                            set_texture_data(&texture);
                        }
                    }
                }
            }
        }
        glfw.poll_events();

        for x in key_activated.iter_mut() {
            *x = false;
        }

        if window.get_key(Key::Num1) == Action::Release {
            if old_key_state[0] {
                key_activated[0] = true;
            }
            old_key_state[0] = false;
        } else {
            old_key_state[0] = true;
        }
        if window.get_key(Key::Num2) == Action::Release {
            if old_key_state[1] {
                key_activated[1] = true;
            }
            old_key_state[1] = false;
        } else {
            old_key_state[1] = true;
        }
        if window.get_key(Key::Num3) == Action::Release {
            if old_key_state[2] {
                key_activated[2] = true;
            }
            old_key_state[2] = false;
        } else {
            old_key_state[2] = true;
        }
        if window.get_key(Key::Num4) == Action::Release {
            if old_key_state[3] {
                key_activated[3] = true;
            }
            old_key_state[3] = false;
        } else {
            old_key_state[3] = true;
        }
    }
    println!("You're done!");
    println!("Number right: {}", number_right);
    println!("Number wrong: {}", number_wrong);
    println!(
        "Percent correct: {}%",
        (number_right as f32 / (number_right + number_wrong) as f32) * 100.0
    );
}
