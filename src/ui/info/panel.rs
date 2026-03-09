use crate::ui::ClipInfoState;
use gpui::{
    App, Element, Entity, IntoElement, ListAlignment, ListState, ParentElement, RenderOnce, StyleRefinement, Styled,
    Window, div, list, px, rems, rgb, white,
};
use gpui_component::StyledExt;
use gpui_component::scroll::ScrollableElement;

#[derive(IntoElement)]
pub struct TrackInfoPanel {
    state: Entity<ClipInfoState>,
    style: StyleRefinement,
}

impl TrackInfoPanel {
    pub fn new(state: &Entity<ClipInfoState>) -> Self {
        Self {
            state: state.clone(),
            style: StyleRefinement::default(),
        }
    }
}

impl Styled for TrackInfoPanel {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for TrackInfoPanel {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let items = self.state.read(cx).info();
        let list_state = window
            .use_keyed_state("track-info-state", cx, |_, _| {
                ListState::new(0, ListAlignment::Top, px(10.))
            })
            .read(cx)
            .clone();

        if list_state.item_count() != items.len() {
            list_state.reset(items.len());
        }

        div()
            .bg(rgb(0x474747))
            .size_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .bg(rgb(0x474747))
                    .line_height(rems(3.0))
                    .border_b_1()
                    .border_color(rgb(0x303030))
                    .px_4()
                    .child("Region info"),
            )
            .child(
                div()
                    .size_full()
                    .relative()
                    .refine_style(&self.style) //
                    .child(
                        list(list_state.clone(), move |i, _, _| {
                            let item = &items[i];
                            let bg_color = if i & 1 == 0 { rgb(0x525252) } else { rgb(0x4E4E4E) };
                            div()
                                .w_full()
                                .grid()
                                .grid_cols(2)
                                .gap(rems(1.0))
                                .text_size(rems(0.75))
                                .line_height(rems(1.5))
                                .bg(bg_color)
                                .child(div().text_right().text_color(white().alpha(0.7)).child(item.key()))
                                .child(div().text_left().child(item.value()))
                                .into_any()
                        })
                        .size_full(),
                    )
                    .vertical_scrollbar(&list_state),
            )
    }
}
