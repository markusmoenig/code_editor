use std::collections::HashMap;

use crate::prelude::*;

use fontdue::{ Font, Metrics };

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum CodeEditorMode {
    Rhai,
    Text,
    Settings,
}

pub struct CodeEditor {

    font                    : Option<Font>,
    draw2d                  : Draw2D,

    rect                    : (usize, usize, usize, usize),
    text                    : String,

    pub font_size           : f32,

    cursor_offset           : usize,
    pub cursor_pos          : (usize, usize),
    pub cursor_rect         : (usize, usize, usize, usize),

    needs_update            : bool,
    pub mode                : CodeEditorMode,

    line_numbers_buffer     : Vec<u8>,
    line_numbers_size       : (usize, usize),

    text_buffer             : Vec<u8>,
    text_buffer_size        : (usize, usize),

    metrics                 : HashMap<char, (Metrics, Vec<u8>)>,
    advance_width           : usize,
    advance_height          : usize,

    shift                   : bool,
    ctrl                    : bool,
    alt                     : bool,
    logo                    : bool,

    pub theme               : Theme,
    pub settings            : Settings,

    error                   : Option<(String, Option<usize>)>,

    mouse_wheel_delta       : (isize, isize),
    offset                  : (isize, isize),
    max_offset              : (usize, usize),

    range_buffer            : (usize, usize),
    range_start             : Option<(usize, usize)>,
    range_end               : Option<(usize, usize)>,

    last_pos                : (usize, usize),
    last_click              : u128,
    click_stage             : i32,

    pub drag_pos            : Option<(usize, usize)>,
}

impl CodeEditor {

    pub fn new() -> Self where Self: Sized {

        Self {
            font                        : None,
            draw2d                      : Draw2D {},

            rect                        : (0, 0, 0, 0),
            text                        : "".to_string(),

            font_size                   : 17.0,

            cursor_offset               : 0,
            cursor_pos                  : (0, 0),
            cursor_rect                 : (0, 0, 2, 0),

            needs_update                : true,
            mode                        : CodeEditorMode::Rhai,

            line_numbers_buffer         : vec![0;1],
            line_numbers_size           : (0, 0),

            text_buffer                 : vec![0;1],
            text_buffer_size            : (0, 0),

            metrics                     : HashMap::new(),
            advance_width               : 10,
            advance_height              : 22,

            shift                       : false,
            ctrl                        : false,
            alt                         : false,
            logo                        : false,

            theme                       : Theme::new(),
            settings                    : Settings::new(),

            error                       : None,

            mouse_wheel_delta           : (0, 0),
            offset                      : (0, 0),
            max_offset                  : (0, 0),

            range_buffer                : (0, 0),
            range_start                 : None,
            range_end                   : None,

            last_pos                    : (0, 0),
            last_click                  : 0,
            click_stage                 : 0,

            drag_pos                    : None,
        }
    }

    /// Sets the path to the font file
    pub fn set_font(&mut self, path: &str) {

        if let Some(font_bytes) = std::fs::read(path).ok() {
            if let Some(font) = Font::from_bytes(font_bytes, fontdue::FontSettings::default()).ok() {
                self.font = Some(font);
            }
        }
    }

    pub fn set_font_size(&mut self, font_size: f32) {

        if let Some(font) = &self.font {
            let m = font.rasterize('w', font_size);
            self.advance_width = m.0.advance_width as usize;
            self.advance_height = (font_size + 4.0) as usize;
            self.font_size = font_size;
        }
    }

    /// Set the text / code to be edited
    pub fn set_text(&mut self, text: String) {
        self.text = text;
        self.needs_update = true;
    }

    /// Returns the edited text
    pub fn get_text(&mut self) -> String {
        self.text.clone()
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    pub fn set_error(&mut self, error: Option<(String, Option<usize>)>) {
        self.error = error;
        self.needs_update = true;
    }

    /// Sets the mode of the editor
    pub fn set_mode(&mut self, mode: CodeEditorMode) {
        self.offset = (0, 0);
        self.mode = mode;
    }

    pub fn draw(&mut self, frame: &mut [u8], rect: (usize, usize, usize, usize), stride: usize) {

        if self.needs_update {
            self.process_text();
        }

        if let Some(drag_pos) = self.drag_pos {
            if drag_pos.1 >= rect.1 + rect.3 - 50 {
                if (self.offset.1 as usize) < self.max_offset.1 {
                    self.offset.1 += 1;
                    self.mouse_dragged(drag_pos);
                }
            } else
            if drag_pos.1 <= rect.1 + 50 {
                if self.offset.1 > 0 {
                    self.offset.1 -= 1;
                    self.mouse_dragged(drag_pos);
                }
            }
        }

        self.rect = rect.clone();

        self.draw2d.draw_rect(frame, &rect, stride, &self.theme.background);
        self.draw2d.draw_rect(frame, &(rect.0, rect.1, 95, rect.3), stride, &self.theme.line_numbers_bg);

        let x = self.line_numbers_size.0 as isize + rect.0 as isize - self.offset.0 * self.advance_width as isize;
        let y = rect.1 as isize - self.offset.1 * self.advance_height as isize;

        // Line Numbers
        self.draw2d.blend_slice_safe(frame, &mut &self.line_numbers_buffer[..], &(0, y, self.line_numbers_size.0, self.line_numbers_size.1), stride, &rect);

        // Code
        let code_safe_rect = (rect.0 + self.line_numbers_size.0, rect.1, rect.2 - self.line_numbers_size.0, rect.3);
        self.draw2d.blend_slice_safe(frame, &mut self.text_buffer[..], &(x, y, self.text_buffer_size.0, self.text_buffer_size.1), stride, &code_safe_rect);

        // Cursor
        self.draw2d.draw_rect_safe(frame, &((rect.0 + self.line_numbers_size.0 + self.cursor_rect.0) as isize - self.offset.0 * self.advance_width as isize, (rect.1 + self.cursor_rect.1) as isize - self.offset.1 * self.advance_height as isize, self.cursor_rect.2, self.cursor_rect.3), stride, &self.theme.cursor, &code_safe_rect);
    }

    // Inside the selection range ?
    fn inside_selection(&self, x: usize, y: usize) -> bool {
        let mut inside = false;

        if let Some(range_start) = self.range_start {
            if let Some(range_end) = self.range_end {
                if y > range_start.1 && y < range_end.1 || (y == range_start.1 && x >= range_start.0 && y != range_end.1) || (y == range_end.1 && x <= range_end.0 && y != range_start.1) || (y == range_start.1 && y == range_end.1 && x >= range_start.0 && x <= range_end.0) {
                    inside = true;
                }
            }
        }

        inside
    }

    /// Takes the current text and renders it to the text_buffer bitmap
    fn process_text(&mut self) {

        if let Some(font) = &self.font {

            let mut lines = self.text.lines();

            let mut screen_width = 0_usize;
            let mut screen_height = 0_usize;

            let mut y = 0;

            while let Some(line) = lines.next() {
                let mut x = 0;

                let mut chars = line.chars();
                let mut line_width = 0;
                while let Some(c) = chars.next() {
                    if self.metrics.contains_key(&c) == false {
                        let m= font.rasterize(c, self.font_size);
                        self.metrics.insert(c, m);
                    }

                    if let Some((metrics, _bitmap)) = self.metrics.get(&c) {
                        line_width += metrics.advance_width.ceil() as usize;
                        x += 1;
                    }
                }

                if line_width > screen_width {
                    screen_width = line_width;
                }

                screen_height += self.advance_height;
                self.last_pos = (x, y);
                y += 1;
            }

            //println!("{} x {}", screen_width, screen_height);

            self.max_offset.0 = screen_width / self.advance_width;

            let left_size = self.settings.line_number_width;
            screen_height += left_size;
            self.needs_update = false;

            self.line_numbers_buffer = vec![0; left_size * screen_height * 4];
            self.line_numbers_size = (left_size, screen_height);

            self.text_buffer = vec![0; screen_width * screen_height * 4];
            self.text_buffer_size = (screen_width, screen_height);

            // Draw it

            let mut scanner = Scanner::new(self.text.as_str());

            let mut x = 0;
            let mut y = 0;

            let stride = screen_width;

            let mut line_number = 1;

            let mut finished = false;
            let mut color : [u8;4] = self.theme.text;
            let mut number_printed_for_line = 0_usize;

            let selection_color = [45, 133, 200, 255];//self.theme.keywords;

            while finished == false {

                let token = scanner.scan_token();
                let mut printit = false;

                match token.kind {

                    TokenType::LineFeed => {

                        let mut text_color = &self.theme.line_numbers;
                        if let Some(error) = &self.error {
                            if let Some(line) = error.1 {
                                if line == line_number {
                                    text_color = &self.theme.error;
                                }
                            }
                        }
                        self.draw2d.draw_text_rect(&mut self.line_numbers_buffer[..], &(0, y, left_size - 20, self.advance_height), left_size, font, self.font_size, format!("{}", line_number).as_str(), &text_color, &self.theme.background, crate::draw2d::TextAlignment::Right);
                        number_printed_for_line = line_number;

                        if x == 0 {
                            // Draw empty selection marker ?
                            if self.inside_selection(0,  y / self.advance_height) {
                                let bcolor = [45, 133, 200, 255];//self.theme.keywords;
                                self.draw2d.blend_rect(&mut self.text_buffer[..], &(x, y, self.advance_width / 2, self.advance_height), stride, &bcolor);
                            }
                        }
                        x = 0;
                        y += self.advance_height;
                        line_number += 1;
                    },
                    TokenType::Space => {

                        // Inside the selection range ?
                        if self.inside_selection(x / self.advance_width,  y / self.advance_height) {
                            let bcolor = [45, 133, 200, 255];//self.theme.keywords;
                            self.draw2d.blend_rect(&mut self.text_buffer[..], &(x, y, self.advance_width, self.advance_height), stride, &bcolor);
                        }

                        x += self.advance_width
                    },
                    TokenType::Eof => {

                        if number_printed_for_line != line_number {
                            let mut text_color = &self.theme.line_numbers;
                            if let Some(error) = &self.error {
                                if let Some(line) = error.1 {
                                    if line == line_number {
                                        text_color = &self.theme.error;
                                    }
                                }
                            }
                            self.draw2d.draw_text_rect(&mut self.line_numbers_buffer[..], &(0, y, left_size - 20, self.advance_height), left_size, font, self.font_size, format!("{}", line_number).as_str(), &text_color, &self.theme.background, crate::draw2d::TextAlignment::Right);
                        }

                        finished = true },

                    TokenType::Identifier if self.mode == CodeEditorMode::Rhai || self.mode == CodeEditorMode::Settings => { color = self.theme.identifier; printit = true; },
                    TokenType::SingeLineComment if self.mode == CodeEditorMode::Rhai || self.mode == CodeEditorMode::Settings => { color = self.theme.comments; printit = true; },
                    TokenType::HexColor if self.mode == CodeEditorMode::Settings => { color = self.theme.string; printit = true; },
                    TokenType::Number if self.mode == CodeEditorMode::Rhai || self.mode == CodeEditorMode::Settings => { color = self.theme.number; printit = true; },
                    TokenType::String  if self.mode == CodeEditorMode::Rhai || self.mode == CodeEditorMode::Settings => { color = self.theme.string; printit = true; },
                    TokenType::While if self.mode == CodeEditorMode::Rhai => { color = self.theme.keywords; printit = true; },
                    TokenType::For if self.mode == CodeEditorMode::Rhai => { color = self.theme.keywords; printit = true; },
                    TokenType::If if self.mode == CodeEditorMode::Rhai => { color = self.theme.keywords; printit = true; },
                    TokenType::Else if self.mode == CodeEditorMode::Rhai => { color = self.theme.keywords; printit = true; },
                    TokenType::Let if self.mode == CodeEditorMode::Rhai => { color = self.theme.keywords; printit = true; },
                    TokenType::Fun if self.mode == CodeEditorMode::Rhai => { color = self.theme.keywords; printit = true; },
                    TokenType::Print if self.mode == CodeEditorMode::Rhai => { color = self.theme.keywords; printit = true; },
                    TokenType::Quotation if self.mode == CodeEditorMode::Rhai => { color = self.theme.string; printit = true; },

                    TokenType::LeftBrace | TokenType::RightBrace | TokenType::LeftParen | TokenType::RightParen | TokenType::Dollar => { color = self.theme.brackets; printit = true; },

                    _ => {
                        color = self.theme.text;
                        printit = true;
                    }
                }

                // Print the current lexeme
                if printit {

                    let mut chars = token.lexeme.chars();
                    while let Some(c) = chars.next() {

                        if let Some((metrics, bitmap)) = self.metrics.get(&c) {

                            let mut bcolor = self.theme.background;

                            // Inside the selection range ?
                            if self.inside_selection(x / self.advance_width,  y / self.advance_height) {
                                bcolor = selection_color;
                                self.draw2d.blend_rect( &mut self.text_buffer[..], &(x, y, self.advance_width, self.advance_height), stride, &bcolor);
                            }

                            let text_buffer_frame = &mut self.text_buffer[..];
                            for cy in 0..metrics.height {
                                for cx in 0..metrics.width {

                                    let fy = (self.font_size as isize - metrics.height as isize - metrics.ymin as isize) as usize;

                                    let i = (x + cx + metrics.xmin as usize) * 4 + (y + cy + fy) * stride * 4;
                                    let m = bitmap[cx + cy * metrics.width];

                                    text_buffer_frame[i..i + 4].copy_from_slice(&self.draw2d.mix_color(&bcolor, &color, m as f64 / 255.0));
                                }
                            }
                            x += self.advance_width;
                        }
                    }
                }
            }

            self.max_offset.1 = line_number;
        }
    }

    /// Sets the cursor offset based on the given screen position
    fn set_cursor_offset_from_pos(&mut self, pos: (usize, usize)) -> bool {

        let mut lines = self.text.lines();

        let px = pos.0;
        let py = pos.1;

        let left_size = 0;
        let line_height = self.advance_height;

        self.cursor_offset = 0;

        let mut curr_line_index = 0_usize;

        let mut y = 0;

        let mut found = false;

        if self.text.is_empty() {
            self.cursor_pos.0 = 0;
            self.cursor_pos.1 = 0;
            self.cursor_rect.0 = 0;
            self.cursor_rect.1 = 0;
            self.cursor_rect.3 = self.advance_height;
            return true;
        }

        while let Some(line) = lines.next() {

            if py >= y && py <= y + self.advance_height {

                self.cursor_pos.0 = 0;
                self.cursor_pos.1 = curr_line_index;
                self.cursor_rect.0 = 0;
                self.cursor_rect.1 = y;
                self.cursor_rect.3 = line_height;

                if px > left_size {
                    self.cursor_pos.0 = (px - left_size) / self.advance_width + 1;
                    if (px - left_size) % self.advance_width < self.advance_width /2 && self.cursor_pos.0 > 0 && self.cursor_pos.0 <= line.len() {
                        self.cursor_pos.0 -= 1;
                    }
                    self.cursor_pos.0 = std::cmp::min(self.cursor_pos.0, line.len());
                    if self.cursor_pos.0 > 0 {
                        self.cursor_rect.0 += self.cursor_pos.0 * self.advance_width - 2;
                    }
                }

                self.cursor_offset += self.cursor_pos.0;
                found = true;

                break;
            } else {
                self.cursor_offset += line.len();
            }

            curr_line_index += 1;
            y += line_height;
            self.cursor_offset += 1;
        }

        // Check if there is an line feed at the end as this is cut off by lines()
        if found == false {
            if self.text.ends_with("\n") {
                self.cursor_pos.0 = 0;
                self.cursor_pos.1 = curr_line_index;
                self.cursor_rect.0 = 0;
                self.cursor_rect.1 = y;
                self.cursor_rect.3 = line_height;
            } else {
                self.cursor_offset -= 1;
            }
        }

        true
    }

    /// Sets the cursor to the given position
    fn set_cursor(&mut self, pos: (usize, usize)) {
        self.cursor_pos = pos;
        self.cursor_rect.0 = pos.0 * self.advance_width;
        self.cursor_rect.1 = (pos.1+1) * self.advance_height;
        self.set_cursor_offset_from_pos((self.cursor_rect.0, self.cursor_rect.1));
    }

    /// Copies the given range and returns it
    fn copy_range(&self, start: Option<(usize, usize)>, end: Option<(usize, usize)>) -> String {
        let mut s = "".to_string();

        let mut x = 0;
        let mut y = 0;

        let mut inside = false;

        let mut chars = self.text.chars();

        while let Some(c) = chars.next() {

            if inside == false {
                if let Some(start) = start {
                    if y > start.1 || (y == start.1 && x >= start.0)  {
                        inside = true;
                    }
                } else {
                    inside = true;
                }
            }

            if inside {
                if let Some(end) = end {
                    if y > end.1 || (y == end.1 && x >= end.0) {
                        break;
                    }
                }
            }

            if inside {
                s.push(c);
            }

            if c == '\n' {
                x = 0;
                y += 1;
            } else {
                x += 1;
            }
        }
        s
    }

    /// Copies the given range and returns it
    fn copy_range_incl(&self, start: Option<(usize, usize)>, end: Option<(usize, usize)>) -> String {
        let mut s = "".to_string();

        let mut x = 0;
        let mut y = 0;

        let mut inside = false;

        let mut chars = self.text.chars();

        while let Some(c) = chars.next() {

            if inside == false {
                if let Some(start) = start {
                    if y > start.1 || (y == start.1 && x >= start.0)  {
                        inside = true;
                    }
                } else {
                    inside = true;
                }
            }

            if inside {
                if let Some(end) = end {
                    if y > end.1 || (y == end.1 && x > end.0) {
                        break;
                    }
                }
            }

            if inside {
                s.push(c);
            }

            if c == '\n' {
                x = 0;
                y += 1;
            } else {
                x += 1;
            }
        }
        s
    }

    pub fn key_down(&mut self, char: Option<char>, key: Option<WidgetKey>) -> bool {
        if self.logo || self.ctrl {
            use copypasta::{ClipboardContext, ClipboardProvider};

            // Copy
            if char == Some('c') || char == Some('C') {
                let clip = self.copy_range_incl(self.range_start, self.range_end);
                //println!("{}", clip);

                let mut ctx = ClipboardContext::new().unwrap();
                _ = ctx.set_contents(clip.to_owned());

                return true;
            }

            // Cut
            if char == Some('x') || char == Some('X') {
                let clip = self.copy_range_incl(self.range_start, self.range_end);

                let mut ctx = ClipboardContext::new().unwrap();
                _ = ctx.set_contents(clip.to_owned());

                if let Some(start) = self.range_start {
                    if let Some(end) = self.range_end {
                        let first_half = self.copy_range(None, Some((std::cmp::max(start.0, 0), start.1)));
                        let second_half = self.copy_range(Some((end.0 + 1, end.1)), None);
                        let text = first_half + second_half.as_str();
                        self.text = text;

                        self.range_start = None;
                        self.range_end = None;
                        self.process_text();

                        self.set_cursor((start.0, start.1));
                    }
                }

                return true;
            }

            // Paste
            if char == Some('v') || char == Some('V') {
                let mut ctx = ClipboardContext::new().unwrap();
                if let Some(text) = ctx.get_contents().ok() {

                    let half = self.cursor_pos.clone();

                    let first_half = self.copy_range(None, Some(half));
                    let second_half = self.copy_range(Some(self.cursor_pos), None);

                    let new_text = first_half + text.as_str();

                    self.text = new_text.clone();
                    self.process_text();
                    self.set_cursor(self.last_pos);

                    self.text = new_text + second_half.as_str();
                    self.needs_update = true;
                }
                return true;
            }
        }

        if let Some(key) = key {
            match key {
                WidgetKey::Delete => {

                    let mut handled = false;
                    if let Some(start) = self.range_start {
                        if let Some(end) = self.range_end {
                            let first_half = self.copy_range(None, Some((std::cmp::max(start.0, 0), start.1)));
                            let second_half = self.copy_range(Some((end.0 + 1, end.1)), None);
                            let text = first_half + second_half.as_str();
                            self.text = text;
                            self.range_start = None;
                            self.range_end = None;
                            self.process_text();
                            handled = true;

                            self.set_cursor(start);
                        }
                    }
                    if handled == false && self.text.is_empty() == false && self.cursor_offset >= 1 {
                        let index  = self.cursor_offset - 1;

                        let mut number_of_chars_on_prev_line = 0_usize;
                        let delete_line;
                        if self.cursor_pos.0 == 0 {
                            delete_line = true;
                            if let Some(prev_line) = self.text.lines().nth(self.cursor_pos.1 - 1) {
                                number_of_chars_on_prev_line = prev_line.len();
                            }
                        } else {
                            delete_line = false;
                        }

                        self.text.drain(index..index+1).next();
                        self.process_text();

                        if delete_line == false {
                            let x = if self.cursor_rect.0 > self.advance_width { self.cursor_rect.0 - self.advance_width } else { 0 };
                            self.set_cursor_offset_from_pos((x, self.cursor_rect.1 + 10));
                        } else {
                            self.set_cursor_offset_from_pos((number_of_chars_on_prev_line * self.advance_width - 2, self.cursor_rect.1 - 5));
                        }
                    }
                    return  true;
                },

                WidgetKey::Tab => {
                    self.text.insert(self.cursor_offset, ' ');
                    self.text.insert(self.cursor_offset + 1, ' ');
                    self.process_text();
                    self.set_cursor_offset_from_pos((self.cursor_rect.0 + self.advance_width * 2, self.cursor_rect.1 + 10));
                    return  true;
                },

                WidgetKey::Return => {
                    self.text.insert(self.cursor_offset, '\n');
                    self.process_text();
                    self.set_cursor_offset_from_pos((0, self.cursor_rect.1 + 30));
                    return  true;
                },

                WidgetKey::Up => {
                    if self.cursor_rect.1 >= 5 {
                        self.set_cursor_offset_from_pos((self.cursor_rect.0, self.cursor_rect.1 - 5));
                    }
                    return  true;
                },

                WidgetKey::Down => {
                    self.set_cursor_offset_from_pos((self.cursor_rect.0, self.cursor_rect.1 + 30));
                    return  true;
                },

                WidgetKey::Left => {

                    if self.logo || self.ctrl {
                        self.set_cursor_offset_from_pos((0, self.cursor_rect.1 + 10));
                    } else {
                        if self.cursor_pos.0 > 0 {//&& self.cursor_rect.0 >= 5 {
                            // Go one left
                            if self.cursor_rect.0 > self.advance_width {
                                self.set_cursor_offset_from_pos((self.cursor_rect.0 - self.advance_width, self.cursor_rect.1 + 10));
                            } else {
                                self.set_cursor_offset_from_pos((0, self.cursor_rect.1 + 10));
                            }
                        } else {
                            // Go one up
                            if self.cursor_rect.1 >= 5 {
                                self.set_cursor_offset_from_pos((100000, self.cursor_rect.1 - 5));
                            }
                        }
                    }
                    return  true;
                },

                WidgetKey::Right => {
                    if self.logo || self.ctrl {
                        self.set_cursor_offset_from_pos((100000, self.cursor_rect.1 + 10));
                    } else {
                        if let Some(c) = self.text.chars().nth(self.cursor_offset) {
                            if c == '\n' {
                                // Go down
                                self.set_cursor_offset_from_pos((0, self.cursor_rect.1 + 30));
                            } else {
                                // Go Right
                                self.set_cursor_offset_from_pos((self.cursor_rect.0 + self.advance_width, self.cursor_rect.1 + 10));
                            }
                        }
                    }
                    return  true;
                },
                _ => {}
            }
        }

        if let Some(c) = char {
            if c.is_ascii() && c.is_control() == false {

                let mut handled = false;
                if let Some(start) = self.range_start {
                    if let Some(end) = self.range_end {
                        let first_half = self.copy_range(None, Some((std::cmp::max(start.0 - 1, 0), start.1)));
                        let second_half = self.copy_range(Some((end.0 + 1, end.1)), None);
                        let text = first_half + c.to_string().as_str() + second_half.as_str();
                        self.text = text;
                        self.process_text();
                        handled = true;

                        self.set_cursor((start.0 + 1, start.1));

                        self.range_start = None;
                        self.range_end = None;
                    }
                }

                if handled == false {
                    if self.text.is_empty() {
                        self.text.push(c);
                    } else {
                        self.text.insert(self.cursor_offset, c);
                    }
                    self.process_text();
                    self.set_cursor_offset_from_pos((self.cursor_rect.0 + self.advance_width, self.cursor_rect.1 + 10));
                }
                return true;
            }
        }
        false
    }

    pub fn mouse_down(&mut self, p: (usize, usize)) -> bool {

        let mut pos = p.clone();
        pos.0 = pos.0.max(self.settings.line_number_width);

        let time = self.get_time();

        if time - self.last_click > 500 {
            let consumed = self.set_cursor_offset_from_pos((pos.0 - self.settings.line_number_width + self.offset.0 as usize * self.advance_width as usize, pos.1 + self.offset.1 as usize * self.advance_height as usize));
            self.range_buffer = self.cursor_pos.clone();
            self.range_start = Some(self.cursor_pos.clone());
            self.range_end = None;
            self.needs_update = true;
            self.last_click = time;
            self.click_stage = 0;
            return consumed;
        } else {

            if self.click_stage == 0 {

                let line = self.cursor_pos.1;

                let mut left =  self.cursor_pos.0;
                let mut right = self.cursor_pos.0;

                let mut range_start = Some((left, line));
                let mut range_end = Some((right, line));

                let t = self.copy_range_incl(range_start, range_end);
                let c = t.chars().next();

                if let Some(c) = c {
                    if c.is_alphanumeric() {

                        // Go left

                        while left > 0 {
                            left -= 1;
                            let r_start = Some((left, line));
                            let t = self.copy_range_incl(r_start, range_end);
                            let c = t.chars().next();

                            if let Some(c) = c {
                                if c.is_alphanumeric() == false {
                                    left += 1;
                                    range_start = Some((left, line));
                                    break;
                                } else {
                                    range_start = Some((left, line));
                                }
                            }
                        }

                        // Go Right

                        loop {
                            right += 1;
                            let r_end = Some((right, line));
                            let t = self.copy_range_incl(range_start, r_end);
                            let c = t.chars().last();

                            if let Some(c) = c {
                                if c.is_alphanumeric() == false {
                                    right -= 1;
                                    range_end = Some((right, line));
                                    break;
                                }
                            }
                        }

                        self.range_start = range_start;
                        self.range_end = range_end;
                        self.needs_update = true;

                        self.last_click = time;
                        self.click_stage = 1;

                        return true;
                    }
                }
            } else {
                let line = self.cursor_pos.1;
                self.range_start = Some((0, line));
                self.range_end = Some((100000, line));
                self.needs_update = true;
                self.click_stage = 2;
                return  true;
            }
        }
        false
    }

    pub fn mouse_up(&mut self, _pos: (usize, usize)) -> bool {
        if self.range_start.is_none() || self.range_end.is_none() {
            self.range_start = None;
            self.range_end = None;
            self.needs_update = true;
        }
        self.drag_pos = None;
        false
    }

    pub fn mouse_dragged(&mut self, mut pos: (usize, usize)) -> bool {
        if pos.0 < self.settings.line_number_width {
            pos.0 = self.settings.line_number_width;
        }

        let consumed = self.set_cursor_offset_from_pos((pos.0 - self.settings.line_number_width + self.offset.0 as usize * self.advance_width as usize, pos.1 + self.offset.1 as usize * self.advance_height as usize));

        if (self.cursor_pos.1 == self.range_buffer.1 && self.cursor_pos.0 <= self.range_buffer.0) || self.cursor_pos.1 < self.range_buffer.1 {
            self.range_start = Some(self.cursor_pos.clone());
            let mut end = self.range_buffer.clone();
            if end.0 > 0 { end.0 -= 1; }
            self.range_end = Some(end);
        } else {
            if self.range_end.is_some() {
                self.range_start = Some(self.range_buffer);
                let mut end = self.cursor_pos.clone();
                if end.0 > 0 { end.0 -= 1; }
                self.range_end = Some(end);
            } else {
                let mut end = self.cursor_pos.clone();
                if end.0 > 0 { end.0 -= 1; }
                self.range_end = Some(end);
            }
        }

        self.drag_pos = Some(pos);

        self.needs_update = true;
        consumed
    }

    pub fn mouse_hover(&mut self, _pos: (usize, usize)) -> bool {
        false
    }

    pub fn mouse_wheel(&mut self, delta: (isize, isize)) -> bool {
        self.mouse_wheel_delta.0 += delta.0;
        self.mouse_wheel_delta.1 += delta.1;
        self.offset.0 += self.mouse_wheel_delta.0 / (self.advance_width as isize * 6);
        self.offset.1 -= self.mouse_wheel_delta.1 / (self.advance_height as isize * 1);
        self.offset.0 = self.offset.0.clamp(0, self.max_offset.0 as isize);
        self.offset.1 = self.offset.1.clamp(0, self.max_offset.1 as isize);
        self.mouse_wheel_delta.0 -= (self.mouse_wheel_delta.0 / (self.advance_width as isize * 6)) * self.advance_width as isize;
        self.mouse_wheel_delta.1 -= (self.mouse_wheel_delta.1 / (self.advance_height as isize * 1)) * self.advance_height as isize;
        true
    }

    pub fn modifier_changed(&mut self, shift: bool, ctrl: bool, alt: bool, logo: bool) -> bool {
        self.shift = shift;
        self.ctrl = ctrl;
        self.alt = alt;
        self.logo = logo;
        false
    }

    /// Gets the current time in milliseconds
    fn get_time(&self) -> u128 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let stop = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
            stop.as_millis()
    }

}