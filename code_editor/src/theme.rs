
pub struct Theme {

    pub background          : [u8;4],
    pub line_numbers        : [u8;4],
    pub line_numbers_bg     : [u8;4],

    pub text                : [u8;4],
    pub cursor              : [u8;4],

    pub identifier          : [u8;4],
    pub number              : [u8;4],
    pub keywords            : [u8;4],
    pub brackets            : [u8;4],
    pub comments            : [u8;4],
    pub string              : [u8;4],

    pub error               : [u8;4],
}

impl Theme {

    pub fn new() -> Self {
        Self {
            background      : [34, 34, 36, 255],
            line_numbers    : [160, 160, 160, 255],
            line_numbers_bg : [30, 30, 32, 255],

            text            : [255, 255, 255, 255],
            cursor          : [170, 170, 170, 255],

            identifier      : [120, 214, 255, 255],
            number          : [159, 197, 146, 255],
            keywords        : [45, 133, 200, 255],
            brackets        : [226, 73, 146, 212],
            comments        : [69, 128, 56, 212],
            string          : [197, 117, 92, 212],

            error           : [237, 55, 54, 255],
        }
    }
}