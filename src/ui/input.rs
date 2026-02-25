use gpui::{Div, Styled, div, rems};
use gpui_component::{StyledExt, gray_300, gray_500};

pub fn app_input_label() -> Div {
    div()
        .mb_1()
        .block()
        .text_size(rems(0.75))
        .line_height(rems(1.0))
        .font_medium()
        .text_color(gray_300())
}

pub fn app_input_text() -> Div {
    div()
        .mt_1()
        .text_size(rems(0.75))
        .line_height(rems(1.0))
        .text_color(gray_500())
}
