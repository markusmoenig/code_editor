mod scanner;
mod draw2d;
pub mod theme;
pub mod settings;
pub mod codeeditor;

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum WidgetKey {
    Escape,
    Return,
    Delete,
    Up,
    Right,
    Down,
    Left,
    Space,
    Tab
}

pub mod prelude {
    pub use crate::scanner::*;
    pub use crate::settings::*;
    pub use crate::theme::*;
    pub use crate::WidgetKey;
    pub use crate::draw2d::*;
    pub use crate::codeeditor::*;
}
