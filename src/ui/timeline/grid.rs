use super::TimelineState;
use crate::ui::{SessionState, WaveRegion};
use gpui::{
    App, Div, Element, Entity, IntoElement, ListAlignment, ListState, ParentElement, RenderOnce,
    StyleRefinement, Styled, Window, div, list, px, rems,
};

use gpui_component::scroll::ScrollableElement;
use gpui_component::{ActiveTheme, StyledExt, gray_400};

#[derive(IntoElement)]
pub struct TimelineGrid {
    session_state: Entity<SessionState>,
    timeline_state: Entity<TimelineState>,
    style: StyleRefinement,
}

impl TimelineGrid {
    pub fn new(
        session_state: &Entity<SessionState>,
        timeline_state: &Entity<TimelineState>,
    ) -> Self {
        Self {
            session_state: session_state.clone(),
            timeline_state: timeline_state.clone(),
            style: StyleRefinement::default(),
        }
    }

    fn tracks_head(&self, cx: &mut App) -> Div {
        let theme = cx.theme();
        div()
            .px_3()
            .py_2()
            .bg(theme.table_head)
            .text_color(gray_400())
            .text_size(rems(0.75))
            .line_height(rems(1.0))
            .border_r(px(1.0))
            .border_b(px(1.0))
            .border_color(theme.border)
            .child("Track / Pass")
    }

    fn regions_head(&self, cx: &mut App) -> Div {
        let theme = cx.theme();
        div()
            .px_3()
            .py_2()
            .bg(theme.table_head)
            .text_color(gray_400())
            .text_size(rems(0.75))
            .line_height(rems(1.0))
            .border_b(px(1.0))
            .border_color(theme.border)
            .flex()
            .justify_between()
            .children(vec![
                div().child("00:00"),
                div().child("01:00"),
                div().child("02:00"),
            ])
    }

    fn track_head(&self, track_index: usize, cx: &mut App) -> Div {
        let track = &self.session_state.read(cx).session_tracks[track_index];
        let theme = cx.theme();
        div()
            .border_r(px(1.0))
            .border_b(px(1.0))
            .border_color(theme.border)
            .px_3()
            .py_1()
            .bg(theme.table_head)
            .child(
                div()
                    .text_size(rems(0.875))
                    .line_height(rems(1.25))
                    .child(track.filename.clone()),
            )
            .child(
                div()
                    .text_size(rems(0.75))
                    .line_height(rems(1.0))
                    .text_color(gray_400())
                    .child("-18.1 LUFS / -2.2 dBTP"),
            )
    }

    fn track_region(&self, track_index: usize, cx: &mut App) -> Div {
        let track = &self.session_state.read(cx).session_tracks[track_index];
        let theme = cx.theme();
        div()
            .border_r(px(1.0))
            .border_b(px(1.0))
            .border_color(theme.border)
            .child(WaveRegion::new(track.samples.clone()))
    }
}

impl Styled for TimelineGrid {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for TimelineGrid {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let session_state = self.session_state.read(cx);
        let timeline_state = self.timeline_state.read(cx);
        let track_count = session_state.session_tracks.len();
        let track_height = px(timeline_state.current_height);
        let header_width = px(200.0);

        let list_state = window
            .use_keyed_state("timeline-track-list-state", cx, |_, _| {
                ListState::new(track_count, ListAlignment::Top, px(10.))
            })
            .read(cx)
            .clone();

        if list_state.item_count() != track_count {
            list_state.reset(track_count);
        }

        div()
            .flex()
            .flex_col()
            .refine_style(&self.style)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .child(self.tracks_head(cx).w(header_width))
                    .child(self.regions_head(cx).flex_1()),
            )
            .child(
                div()
                    .flex_1()
                    .child(
                        list(list_state.clone(), move |i, _, cx| {
                            div()
                                .w_full()
                                .h(track_height)
                                .flex()
                                .flex_row()
                                .child(self.track_head(i, cx).w(header_width))
                                .child(self.track_region(i, cx).flex_1())
                                .into_any()
                        })
                        .size_full(),
                    )
                    .vertical_scrollbar(&list_state),
            )
    }
}
