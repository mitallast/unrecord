use crate::ui::timeline::state::TimelineState;
use gpui::{
    Entity, InteractiveElement, IntoElement, ParentElement, RenderOnce, StyleRefinement, Styled,
    Window, div,
};
use gpui_component::slider::Slider;
use gpui_component::{ActiveTheme, StyledExt};

#[derive(IntoElement)]
pub struct TimelineScale {
    state: Entity<TimelineState>,
    style: StyleRefinement,
}

impl TimelineScale {
    pub fn new(state: &Entity<TimelineState>) -> Self {
        Self {
            state: state.clone(),
            style: StyleRefinement::default(),
        }
    }
}

impl Styled for TimelineScale {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for TimelineScale {
    fn render(self, _: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let theme = cx.theme();
        let state = self.state.read(cx);
        div()
            .id(("ts", self.state.entity_id()))
            .rounded_xl()
            .border_1()
            .border_color(theme.border)
            .gap_4()
            .px_4()
            .flex()
            .flex_row()
            .flex_nowrap()
            .refine_style(&self.style)
            .child(Slider::new(&state.height_slider).col_span(1))
            .child(Slider::new(&state.timescale_slider).col_span(1))
    }
}
