use crate::components::tick::GridTick;
use gpui::{
    App, Bounds, Hsla, IntoElement, ParentElement, RenderOnce, StyleRefinement, Styled, Window, canvas, div, fill,
    point, px, rgb, size,
};
use gpui_component::StyledExt;

#[derive(IntoElement)]
pub struct GridTickView {
    ticks: Vec<GridTick>,
    style: StyleRefinement,
    tick_color: Hsla,
}

impl Styled for GridTickView {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl GridTickView {
    pub fn new(ticks: Vec<GridTick>) -> Self {
        Self {
            ticks,
            style: StyleRefinement::default(),
            tick_color: rgb(0x696869).into(),
        }
    }
}

impl RenderOnce for GridTickView {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let ticks = canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                for tick in &self.ticks {
                    let grid_bounds = Bounds::new(
                        point(bounds.origin.x + tick.offset_x, bounds.origin.y),
                        size(px(1.0), bounds.size.height),
                    );
                    window.paint_quad(fill(grid_bounds, self.tick_color));
                }
            },
        )
        .size_full();

        div()
            .h(px(18.0))
            .bg(rgb(0x333333))
            .border_b(px(1.0))
            .border_color(rgb(0x1F1F1F))
            .w_full()
            .overflow_hidden()
            .refine_style(&self.style)
            .child(ticks)
    }
}
