use gpui::{Pixels, SharedString};

mod generator;
mod label_view;
mod view;

#[allow(unused_imports)]
pub use generator::*;
#[allow(unused_imports)]
pub use label_view::*;
#[allow(unused_imports)]
pub use view::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum GridTickType {
    PRIMARY = 0,
    SECONDARY = 1,
}

#[derive(Clone)]
pub struct GridTick {
    pub tick_type: GridTickType,
    pub offset_x: Pixels,
    pub label: Option<SharedString>,
}
