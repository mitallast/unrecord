use crate::time::TimeCode;

use gpui::{Pixels, SharedString};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum GridTickType {
    PRIMARY = 0,
    SECONDARY = 1,
}

pub struct GridTick {
    pub tick_type: GridTickType,
    pub time: TimeCode,
    pub offset_x: Pixels,
    pub label: Option<SharedString>,
}
