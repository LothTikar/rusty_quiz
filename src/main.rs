extern crate csv;
extern crate rand;
extern crate gtk;

use gtk::prelude::*;
use gtk::{ButtonsType, DialogFlags, MessageType, MessageDialog, Window, WindowType};
use rand::{thread_rng, Rng};
use std::env;

struct Header {
    number_of_hints: i32,
    questions: Vec<String>,
}

struct Slide {
    image_filename: String,
    hints: Vec<String>,
    answers: Vec<String>,
}

fn read_slides(csv_reader: &mut csv::Reader<std::fs::File>, header: &Header) -> Vec<Slide> {
    let mut slides: Vec<Slide> = Vec::new();

    for record in csv_reader.records() {
        let record = record.unwrap();

        let mut slide = Slide {
    image_filename: String::new(),
            hints: Vec::new(),
            answers: Vec::new(),
        };

        let mut i = 0;
        for value in record.iter() {
            let value = value.to_string();
            match i {
                0 => {
                    if !value.is_empty() {
                        slide.image_filename = value;
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

fn main() {
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
        gtk::init().unwrap();
    // Create the main window.
    let window = Window::new(WindowType::Toplevel);
    // UI initialization.
    // ...
    // Don't forget to make all widgets visible.
    window.show_all();
    // Handle closing of the window.
    window.connect_delete_event(|_, _| {
        // Stop the main loop.
        gtk::main_quit();
        // Let the default handler destroy the window.
        Inhibit(false)
    });
    // Run the main loop.
    gtk::main();
}
