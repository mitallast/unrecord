mod audio;
mod ui;

use crate::ui::{
    SessionPanel, SessionState, TimelinePanel, TimelineState, TrackInfoPanel, TrackInfoState,
};

use anyhow::Result;
use gpui::{
    App, AppContext, Application, Context, Entity, IntoElement, ParentElement, Render, Styled,
    Window, WindowOptions, div, px, rgb,
};
use gpui_component::{ActiveTheme, Root, gray_400};
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
            theme.colors.tiles = rgb(0x1B1C1F).into();

            theme.colors.background = rgb(0x1F2126).into();
            theme.colors.muted = rgb(0x1F2126).into();
            theme.colors.foreground = rgb(0xF5F5F7).into();
            theme.colors.sidebar = rgb(0x2A2C31).into();

            theme.colors.secondary_hover = rgb(0x3b3f47).into();
            theme.colors.secondary_active = rgb(0x454a54).into();

            theme.colors.primary = rgb(0x3492ff).into();
            theme.colors.primary_hover = rgb(0x4aa0ff).into();
            theme.colors.primary_active = rgb(0x2b8fff).into();
            theme.colors.primary_foreground = theme.colors.foreground;

            theme.colors.border = rgb(0x3A3D44).into();
            theme.colors.input = rgb(0x434853).into();
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
    session_state: Entity<SessionState>,
    timeline_state: Entity<TimelineState>,
    info_state: Entity<TrackInfoState>,
}

impl UnrecordApp {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Result<Self> {
        let info_state = cx.new(TrackInfoState::new);
        let session_state = cx.new(|cx| SessionState::new(window, cx, &info_state).unwrap());
        let timeline_state = cx.new(TimelineState::new);

        Ok(Self {
            session_state,
            timeline_state,
            info_state,
        })
    }
}

impl Render for UnrecordApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .size_full()
            .p_4()
            .gap_4()
            .bg(theme.tiles)
            .flex()
            .flex_row()
            .child(
                div()
                    .w(px(350.0))
                    .h_full()
                    .flex()
                    .flex_col()
                    .flex_nowrap()
                    .gap_4()
                    .child(SessionPanel::new(&self.session_state))
                    .child(TrackInfoPanel::new(&self.info_state)),
            )
            .child(
                TimelinePanel::new(&self.session_state, &self.timeline_state)
                    .flex_1()
                    .h_full(),
            )
            .children(Root::render_dialog_layer(window, cx))
            .children(Root::render_sheet_layer(window, cx))
            .children(Root::render_notification_layer(window, cx))
    }
}
