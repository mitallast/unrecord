use gpui::{Div, Styled, div, rems};
use gpui_component::{StyledExt, gray_400};

pub fn app_panel_title() -> Div {
    div().text_size(rems(1.0)).line_height(rems(1.5)).font_semibold()
}

pub fn app_panel_text() -> Div {
    div()
        .text_size(rems(0.75))
        .line_height(rems(1.0))
        .text_color(gray_400())
}
