use crate::components::grid::{GridState, GridViewportHandle};
use crate::components::tick::{GridTickLabelView, GridTickView};
use crate::ui::app_panel_title;
use crate::ui::grid::header_list::GridHeaderList;
use crate::ui::grid::track_list::GridTrackList;
use gpui::{App, Div, Entity, Hsla, IntoElement, ParentElement, RenderOnce, Styled, Window, div, px, rems, rgb};
use gpui_component::scroll::{Scrollbar, ScrollbarShow};
use gpui_component::slider::Slider;
use gpui_component::{ActiveTheme, gray_400};

#[derive(IntoElement)]
pub struct GridProjectView {
    head_bg: Hsla,
    project: Entity<GridState>,
}

impl GridProjectView {
    pub fn new(project: &Entity<GridState>) -> Self {
        Self {
            head_bg: rgb(0x474747).into(),
            project: project.clone(),
        }
    }

    fn grid_title(&self, _: &mut Window, cx: &mut App) -> Div {
        let project = self.project.read(cx);

        let theme = cx.theme();
        div()
            .px_4()
            .py_3()
            .border_b_1()
            .flex()
            .flex_row()
            .justify_between()
            .bg(self.head_bg)
            .child(app_panel_title().child("Timeline"))
            .child(
                div()
                    .rounded_xl()
                    .border_1()
                    .border_color(theme.border)
                    .gap_4()
                    .px_4()
                    .flex()
                    .flex_row()
                    .flex_nowrap()
                    .child(Slider::new(&project.x_slider))
                    .child(Slider::new(&project.y_slider))
                    .w(rems(20.0)),
            )
    }

    fn grid_header(&self, cx: &mut App) -> Div {
        let project = self.project.read(cx);
        let header_width = project.viewport.header_size().width;

        div()
            .flex()
            .flex_row()
            .child(self.tracks_head(cx).w(header_width))
            .child(self.grid_ruler(cx).flex_1())
    }

    fn tracks_head(&self, cx: &mut App) -> Div {
        let theme = cx.theme();
        div()
            .px_3()
            .py_2()
            .bg(self.head_bg)
            .text_color(gray_400())
            .text_size(rems(0.75))
            .line_height(rems(1.0))
            .border_r(px(1.0))
            .border_b(px(1.0))
            .border_t(px(1.0))
            .border_color(theme.border)
            .child("Track / Pass")
    }

    fn grid_ruler(&self, cx: &mut App) -> Div {
        let project = self.project.read(cx);

        div()
            .flex()
            .flex_col()
            .child(GridTickLabelView::new(project.viewport.ticks()))
            .child(GridTickView::new(project.viewport.ticks()))
    }

    fn tracks(&self, _: &mut Window, cx: &mut App) -> Div {
        let project = self.project.read(cx);

        div().flex_1().flex().flex_col().child(
            div()
                .flex_1()
                .flex()
                .flex_row()
                .child(
                    GridHeaderList::new(project.tracks(), &project.viewport)
                        .h_full()
                        .w(project.viewport.header_size().width),
                )
                .child(
                    div()
                        .flex_1()
                        .h_full()
                        .relative()
                        .bg(rgb(0x2E2E2E))
                        .child(
                            GridTrackList::new(project.tracks(), &project.viewport)
                                .absolute()
                                .size_full(),
                        )
                        .child(
                            Scrollbar::new(&project.viewport) //
                                .id("grid-scrollbar-both2")
                                .scrollbar_show(ScrollbarShow::Always),
                        ),
                ),
        )
    }
}

impl RenderOnce for GridProjectView {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(self.grid_title(window, cx))
            .child(self.grid_header(cx))
            .child(self.tracks(window, cx))
    }
}
