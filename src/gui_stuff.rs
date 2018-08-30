    let img_right = image::open("./resources/icons8-checked-50.png").unwrap();
    let img_wrong = image::open("./resources/icons8-cancel-50.png").unwrap();
    let img_continue = image::open("./resources/icons8-circled-right-50.png").unwrap();












struct CurrentSlideInfo<'a> {
    slide: &'a Slide,
    render_info: CurrentSlideRenderInfo,
    questions: Vec<Question>,
}








#[derive(PartialEq)]
enum QuestionStatus {
    right,
    wrong,
    untouched,
}
struct Question {
    question: RgbaImage,
    correct_answer: TexturedButton,
    wrong_answers: Vec<TexturedButton>,
    status: QuestionStatus,
}














trait Renderable {
    fn render(&self, verts: &mut Vec<GLfloat>, window_size: (f32, f32));
    fn width(&self) -> f32;
    fn height(&self) -> f32;

    fn size(&self) -> (f32, f32) {
        (self.width(), self.height())
    }
}

trait TextureUser {
    fn required_width(&self) -> f32;
    fn required_height(&self) -> f32;

    fn required_size(&self) -> (f32, f32) {
        (self.required_width(), self.required_height())
    }

    fn set_offset(&mut self, offset: (f32, f32));
    fn get_offset(&self) -> (f32, f32);

    fn update_texture(&self, texture: &mut RgbaImage);
}







trait Clickable {
    fn update_state(&mut self, mouse_pos: (f32, f32), button_pressed: bool);
    fn is_clicked(&self) -> bool;
}



struct CurrentSlideRenderInfo {
    image_offset: (f32, f32),
    right_offset: (f32, f32),
    wrong_offset: (f32, f32),
    continue_offset: (f32, f32),
}








impl<'a> CurrentSlideInfo<'a> {
    fn new(
        slide: &Slide,
        all_slides: &Vec<Slide>,
        slide_offset: (f32, f32),
        right_offset: (f32, f32),
        wrong_offset: (f32, f32),
        continue_offset: (f32, f32),
    ) -> CurrentSlideInfo<'a> {
        let render_info = CurrentSlideRenderInfo {
            image_offset: slide_offset,
            right_offset: right_offset,
            wrong_offset: wrong_offset,
            continue_offset: continue_offset,
        };
        let mut wrong_answers: Vec<Vec<String>> = Vec::new();
        let mut rng = rand::thread_rng();
        for category in 0..slide.answers.len() {
            let mut random_answers = Vec::new();
            while random_answers.len() < 3 {
                let random_answer = rng
                    .choose(all_slides)
                    .expect("all_slides empty! Please fix!")
                    .answers[category];
                if random_answer != slide.answers[category] {
                    random_answers.push(random_answer);
                }
            }
            wrong_answers.push(random_answers);
        }
        CurrentSlideInfo {
            slide: slide,
            render_info: render_info,
            questions: questions,
        }
    }

    fn is_solved(&self) -> bool {
        let mut solved = true;
        for q in self.questions.iter() {
            if q.status != QuestionStatus::right {
                solved = false;
            }
        }
        solved
    }
}

fn add_colored_box(
    window_position: (f32, f32),
    layer: f32,
    color: (f32, f32, f32),
    box_size: (f32, f32),
    window_size: (f32, f32),
    verts: &mut Vec<GLfloat>,
) {
    let pos = (
        (window_position.0 * 2.0) / window_size.0 - 1.0,
        (window_position.1 * -2.0) / window_size.1 + 1.0,
    );
    let box_size = (
        (box_size.0 * 2.0) / window_size.0,
        (box_size.1 * -2.0) / window_size.1,
    );

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
    verts.push(0.0);
    verts.push(0.0);
    //enable texture
    verts.push(0.0);

    //pos
    verts.push(pos.0 + box_size.0);
    verts.push(pos.1);
    verts.push(layer);
    //color
    verts.push(color.0);
    verts.push(color.1);
    verts.push(color.2);
    //tex coord
    verts.push(0.0);
    verts.push(0.0);
    //enable texture
    verts.push(0.0);

    //pos
    verts.push(pos.0 + box_size.0);
    verts.push(pos.1 + box_size.1);
    verts.push(layer);
    //color
    verts.push(color.0);
    verts.push(color.1);
    verts.push(color.2);
    //tex coord
    verts.push(0.0);
    verts.push(0.0);
    //enable texture
    verts.push(0.0);

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
    verts.push(0.0);
    verts.push(0.0);
    //enable texture
    verts.push(0.0);

    //pos
    verts.push(pos.0);
    verts.push(pos.1 + box_size.1);
    verts.push(layer);
    //color
    verts.push(color.0);
    verts.push(color.1);
    verts.push(color.2);
    //tex coord
    verts.push(0.0);
    verts.push(0.0);
    //enable texture
    verts.push(0.0);

    //pos
    verts.push(pos.0 + box_size.0);
    verts.push(pos.1 + box_size.1);
    verts.push(layer);
    //color
    verts.push(color.0);
    verts.push(color.1);
    verts.push(color.2);
    //tex coord
    verts.push(0.0);
    verts.push(0.0);
    //enable texture
    verts.push(0.0);
}











//Textured button stuff
struct TexturedButton {
    position: (f32, f32),
    texture: RgbaImage,
    texture_offset: (f32, f32),
    mouse_hover: bool,
    mouse_down: bool,
    clicked: bool,
}

impl TexturedButton{
    fn new(position: (f32,f32),texture: RgbaImage,texture_offset:(f32,f32)) -> TexturedButton {
        TexturedButton {
            position: position,
            texture: texture,
            texture_offset: texture_offset,
            mouse_down: false,
            clicked: false,
        }
    }
}

impl Renderable for TexturedButton {
    fn render(&self, verts: &mut Vec<GLfloat>, window_size: (f32, f32)) {
        if self.mouse_hover {
            if self.mouse_down {
            add_colored_box((self.position.0 + BUTTON_BORDER_THICKNESS,self.position.1 + BUTTON_BORDER_THICKNESS),0.0,BUTTON_BORDER_COLOR_HOVER,(texture.width()-BUTTON_BORDER_THICKNESS,texture.height()-BUTTON_BORDER_THICKNESS),window_size,verts);
        } else {

        }
        }
    }
    fn width(&self) -> f32 {
        self.texture.width()
    }
    fn height(&self) -> f32{
        self.texture.height()
    }
}

impl TextureUser for TexturedButton {
    fn required_width(&self) -> f32 {
        self.texture.width() as f32
    }

    fn required_height(&self) -> f32 {
        self.texture.height() as f32
    }

    fn set_offset(&mut self, offset: (f32, f32)) {
        self.texture_offset = offset;
    }

    fn get_offset(&self) -> (f32, f32) {
        self.texture_offset
    }

    fn update_texture(&self, texture: &mut RgbaImage) {
        for (x, y, pixel) in self.texture.enumerate_pixels() {
            texture.put_pixel(
                self.texture_offset.0 as u32 + x,
                self.texture_offset.1 as u32 + y,
                *pixel,
            );
        }
    }
}

impl Clickable for TexturedButton {
    fn update_state(&mut self, mouse_pos: (f32, f32), button_pressed: bool) {
        let lower_left_corner = (
            self.position.0 + self.texture.width() as f32,
            self.position.1 + self.texture.height() as f32,
        );
        self.mouse_hover = mouse_pos.0 > self.position.0
            && mouse_pos.1 > self.position.1
            && mouse_pos.0 < lower_left_corner.0
            && mouse_pos.1 < lower_left_corner.1;
        if self.mouse_hover {
            if button_pressed {
                self.mouse_down = true;
            } else {
                self.clicked = self.mouse_down;
                self.mouse_down = false;
            }
        } else {
            self.clicked = false;
            self.mouse_down = false;
        }
    }

    fn is_clicked(&self) -> bool {
        self.clicked
    }
}









//Question stuff
struct Question {
    question: RgbaImage,
    correct_answer: TexturedButton,
    wrong_answers: Vec<TexturedButton>,
    status: QuestionStatus,
}

impl Renderable for Question {
    fn render(&self, verts: &mut Vec<GLfloat>, window_size: (f32, f32)) {
        add_textured_box()
    }
    fn width(&self) -> f32 {
        let mut width = 0;
        width = width.max(question.width());
        width = width.max(correct_answer.width());
        for b in wrong_answers.iter() {
            width = width.max(b.width());
        }
        width * ((self.wrong_answers.size()) as f32 / 2.0) + 10.0
    }
    fn height(&self) -> f32 {
        let mut height = 0;
        height = width.max(question.height());
        height = width.max(correct_answer.height());
        for b in wrong_answers.iter() {
            width = width.max(b.height());
        }
        height * 2.0 + 20.0
    }

    fn size(&self) -> (f32, f32) {
        (self.width(), self.height())
    }
}

impl Question {
    fn new(question: String, correct_answer: String, incorrect_answers: Vec<String>, texture_offset: (f32,f32), font: &Font) -> Question{
        let question_image = render_text(font,15.0,question.as_str());
        let correct_image = render_text(font,15.0,correct_answer.as_str());
        let mut wrong_images: Vec<RgbaImage> = Vec::new();
        for s in incorrect_answers {
            wrong_images.push( render_text(font,15.0,s.as_str()));
        }
        let mut question =  Question {
            question: question_image,
            correct_answer: correct_image,
            wrong_answers: Vec<TexturedButton>,
            status: QuestionStatus,
        };
    }
}

impl TextureUser for Question {
    fn required_width(&self) -> f32 {
        let mut width = self.correct_answer.required_width();
        for b in self.wrong_answers.iter() {
            width += b.required_width();
        }
        width
    }
    fn required_height(&self) -> f32 {
        let mut height = self.correct_answer.required_height();
        for b in self.wrong_answers.iter() {
            height += height.max(b.required_height());
        }
        height
    }

    fn required_size(&self) -> (f32, f32) {
        (self.required_width(), self.required_height())
    }

    fn set_offset(&mut self, offset: (f32, f32)) {
        let mut x_offset = offset.0;
        self.correct_answer.set_offset((x_offset,offset.1));
        x_offset += self.correct_answer.required_width();
        for b in wrong_answers.iter() {
            self.correct_answer.set_offset((x_offset,offset.1));
            x_offset += self.correct_answer.required_width();
        }
    }
    fn get_offset(&self) -> (f32, f32) {
        self.correct_answer.get_offset()
    }

    fn update_texture(&self, texture: &mut RgbaImage) {
        correct_answer.update_texture(texture);
        for b in wrong_answers.iter() {
            b.update_texture(texture);
        }
    }
}

impl Clickable for Question {
    fn update_state(&mut self, mouse_pos: (f32, f32), button_pressed: bool) {
        self.correct_answer.update_state(mouse_pos, button_pressed);
        for b in self.wrong_answers.iter() {
            b.update_state(mouse_pos, button_pressed);
            if b.is_clicked() {
                self.status = QuestionStatus::wrong;
            }
        }
        if self.correct_answer.is_clicked() {
            self.status = QuestionStatus::right;
        }
    }

    fn is_clicked(&self) -> bool {
        false
    }
}
