pub struct Settings {

    pub line_number_width           : usize,

}

impl Settings {

    pub fn new() -> Self {
        Self {
            line_number_width       : 100,
        }
    }
}