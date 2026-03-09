use crate::components::track::Track;
use gpui::{App, IntoElement, ParentElement, RenderOnce, StyleRefinement, Styled, Window, div, px, rems, rgb};
use gpui_component::{StyledExt, gray_400};

#[derive(IntoElement)]
pub struct TrackHeaderView {
    track: Track,
    style: StyleRefinement,
}

impl TrackHeaderView {
    pub fn new(track: &Track) -> Self {
        Self {
            track: track.clone(),
            style: StyleRefinement::default(),
        }
    }
}

impl Styled for TrackHeaderView {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for TrackHeaderView {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let title = self.track.title();

        div()
            .border_r(px(1.0))
            .border_b(px(1.0))
            .border_color(rgb(0x454545))
            .px_3()
            .py_1()
            .bg(rgb(0x575757))
            .refine_style(&self.style)
            .child(div().text_size(rems(0.875)).line_height(rems(1.25)).child(title))
            .child(
                div()
                    .text_size(rems(0.75))
                    .line_height(rems(1.0))
                    .text_color(gray_400())
                    .child("0 LUFS / 0 dBP"),
            )
    }
}
