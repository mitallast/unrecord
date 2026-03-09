mod audio;
mod components;
mod time;
mod ui;

use crate::components::grid::GridState;
use crate::time::SampleRate;
use crate::ui::{ClipInfoState, GridProjectView, SessionPanel, SessionState, TrackInfoPanel};
use anyhow::Result;
use gpui::{
    App, AppContext, Application, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, WindowOptions,
    div, px, rgb,
};
use gpui_component::{Root, gray_400};
use gpui_component_assets::Assets;

fn main() -> Result<()> {
    env_logger::init();

    Application::new().with_assets(Assets).run(|cx: &mut App| {
        gpui_component::init(cx);
        {
            let theme = gpui_component::Theme::global_mut(cx);
            theme.font_size = px(16.0);
            theme.radius = px(8.0);

            // window background
            theme.colors.background = rgb(0x333333).into();
            theme.colors.muted = rgb(0x393939).into();
            theme.colors.foreground = rgb(0xF5F5F7).into();
            theme.colors.sidebar = rgb(0x2A2C31).into();
            theme.colors.accent = rgb(0x3492ff).into();

            theme.colors.secondary_hover = rgb(0x3b3f47).into();
            theme.colors.secondary_active = rgb(0x454a54).into();

            theme.colors.primary = rgb(0x3492ff).into();
            theme.colors.primary_hover = rgb(0x4aa0ff).into();
            theme.colors.primary_active = rgb(0x2b8fff).into();
            theme.colors.primary_foreground = theme.colors.foreground;

            theme.colors.border = rgb(0x3A3D44).into();
            theme.colors.input = rgb(0x393939).into();
            theme.colors.slider_bar = gray_400();
            theme.colors.slider_thumb = rgb(0x1f2126).into();
            theme.colors.table_head = rgb(0x24272d).into();
            theme.colors.table = rgb(0x24272d).into();
        }

        cx.spawn(async move |cx| {
            let options = WindowOptions::default();
            cx.open_window(options, |window, cx| {
                window.set_rem_size(px(16.0));
                let view = cx.new(|cx| UnrecordApp::new(window, cx).unwrap());
                // This first level on the window, should be a Root.
                cx.new(|cx| Root::new(view, window, cx))
            })?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
        cx.activate(true);
    });

    Ok(())
}

struct UnrecordApp {
    session: Entity<SessionState>,
    info: Entity<ClipInfoState>,
    grid: Entity<GridState>,
}

impl UnrecordApp {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Result<Self> {
        let info_state = cx.new(ClipInfoState::new);
        let grid_state = cx.new(|cx| GridState::new(SampleRate::Hz44100, cx));
        let session_state = cx.new(|cx| SessionState::new(window, cx, &grid_state, &info_state).unwrap());

        Ok(Self {
            session: session_state,
            info: info_state,
            grid: grid_state,
        })
    }
}

impl Render for UnrecordApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .gap(px(2.0))
            .bg(rgb(0x141414))
            .flex()
            .flex_row()
            .child(
                div()
                    .w(px(350.0))
                    .h_full()
                    .flex()
                    .flex_col()
                    .flex_nowrap()
                    .gap(px(2.0))
                    .child(SessionPanel::new(&self.session))
                    .child(TrackInfoPanel::new(&self.info)),
            )
            .child(GridProjectView::new(&self.grid))
            .children(Root::render_dialog_layer(window, cx))
            .children(Root::render_sheet_layer(window, cx))
            .children(Root::render_notification_layer(window, cx))
    }
}
