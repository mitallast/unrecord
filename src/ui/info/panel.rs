use crate::ui::{TrackInfoState, app_panel, app_panel_title};
use gpui::{
    App, Element, Entity, IntoElement, ListAlignment, ListState, ParentElement, RenderOnce,
    StyleRefinement, Styled, Window, div, list, px, rems,
};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{StyledExt, gray_400};

#[derive(IntoElement)]
pub struct TrackInfoPanel {
    state: Entity<TrackInfoState>,
    style: StyleRefinement,
}

impl TrackInfoPanel {
    pub fn new(state: &Entity<TrackInfoState>) -> Self {
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

        app_panel(cx)
            .p_4()
            .size_full()
            .flex()
            .flex_col()
            .child(app_panel_title().child("Track info"))
            .child(
                div()
                    .size_full()
                    .relative()
                    .refine_style(&self.style) //
                    .child(
                        list(list_state.clone(), move |i, _, _| {
                            let item = &items[i];
                            div()
                                .w_full()
                                .flex()
                                .flex_row()
                                .justify_between()
                                .text_size(rems(0.75))
                                .line_height(rems(1.0))
                                .child(div().text_color(gray_400()).child(item.key()))
                                .child(div().text_right().child(item.value()))
                                .into_any()
                        })
                        .size_full(),
                    )
                    .vertical_scrollbar(&list_state),
            )
    }
}
