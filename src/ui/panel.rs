use gpui::{App, Div, Styled, div, px, rems};
use gpui_component::{ActiveTheme, StyledExt, gray_400};

pub fn app_panel(cx: &App) -> Div {
    let theme = cx.theme();
    div().rounded_xl().bg(theme.sidebar).border(px(1.0))
}

pub fn app_panel_title() -> Div {
    div()
        .text_size(rems(1.0))
        .line_height(rems(1.5))
        .font_semibold()
}

pub fn app_panel_text() -> Div {
    div()
        .text_size(rems(0.75))
        .line_height(rems(1.0))
        .text_color(gray_400())
}
