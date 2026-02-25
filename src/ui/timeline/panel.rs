use crate::ui::{
    SessionState, TimelineGrid, TimelineScale, TimelineState, app_panel, app_panel_title,
};
use gpui::{
    App, Entity, IntoElement, ParentElement, RenderOnce, StyleRefinement, Styled, Window, div, px,
    rems,
};
use gpui_component::StyledExt;

#[derive(IntoElement)]
pub struct TimelinePanel {
    session_state: Entity<SessionState>,
    timeline_state: Entity<TimelineState>,
    style: StyleRefinement,
}

impl TimelinePanel {
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
}

impl Styled for TimelinePanel {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for TimelinePanel {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        app_panel(cx)
            .flex()
            .flex_col()
            .refine_style(&self.style)
            .child(
                div()
                    .px_4()
                    .py_3()
                    .border_b_1()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .child(app_panel_title().child("Timeline"))
                    .child(TimelineScale::new(&self.timeline_state).w(rems(20.0))),
            )
            .child(TimelineGrid::new(&self.session_state, &self.timeline_state).flex_1())
            .child(div().h(px(40.0)))
    }
}
