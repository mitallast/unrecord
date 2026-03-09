use crate::components::tick::{GridTick, GridTickType};
use gpui::{
    App, Div, IntoElement, ParentElement, RenderOnce, SharedString, StyleRefinement, Styled, Window, div, px, rems, rgb,
};
use gpui_component::gray_400;

#[derive(IntoElement)]
pub struct GridTickLabelView {
    ticks: Vec<GridTick>,
    style: StyleRefinement,
}

impl GridTickLabelView {
    pub fn new(ticks: Vec<GridTick>) -> Self {
        Self {
            ticks,
            style: StyleRefinement::default(),
        }
    }
}

impl Styled for GridTickLabelView {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

const EMPTY_LABEL: SharedString = SharedString::new_static("");

impl RenderOnce for GridTickLabelView {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let top = px(1.0);
        let tick_color = rgb(0x696969);
        
        let labels: Vec<Div> = self
            .ticks
            .iter()
            .filter(|tick| tick.tick_type == GridTickType::PRIMARY)
            .map(|tick| {
                div()
                    .absolute()
                    .left(tick.offset_x)
                    .top(top)
                    .text_color(gray_400())
                    .text_size(rems(0.75))
                    .line_height(rems(1.0))
                    .border_l_1()
                    .border_color(tick_color)
                    .pl_1()
                    .child(tick.label.clone().unwrap_or_else(|| EMPTY_LABEL))
            })
            .collect();

        div()
            .bg(rgb(0x333333))
            .border_b(px(1.0))
            .border_color(rgb(0x1F1F1F))
            .w_full()
            .h(px(18.0))
            .overflow_hidden()
            .child(div().size_full().relative().children(labels))
    }
}
