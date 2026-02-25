use crate::ui::listener::StateEventListener;
use crate::ui::{
    SessionState, app_input_label, app_input_text, app_panel, app_panel_text, app_panel_title,
};
use gpui::{
    App, Entity, IntoElement, ParentElement, RenderOnce, StyleRefinement, Styled, Window, div,
};
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::input::Input;
use gpui_component::select::Select;

#[derive(IntoElement)]
pub struct SessionPanel {
    state: Entity<SessionState>,
    style: StyleRefinement,
}

impl SessionPanel {
    pub fn new(state: &Entity<SessionState>) -> Self {
        Self {
            state: state.clone(),
            style: StyleRefinement::default(),
        }
    }
}

impl Styled for SessionPanel {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for SessionPanel {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let current_state = self.state.read(cx);

        app_panel(cx)
            .p_4()
            .flex()
            .flex_col()
            .child(app_panel_title().child("Session Setup"))
            .child(
                app_panel_text()
                    .mt_1()
                    .child("Configure interface, input file and output folder."),
            )
            .child(app_input_label().child("Audio Device").mt_4())
            .child(Select::new(&current_state.select_device_state))
            .child(app_input_label().child("Input WAV File").mt_3())
            .child(
                div()
                    .relative()
                    .grid()
                    .grid_cols(12)
                    .gap_2()
                    .child(
                        Input::new(&current_state.source_path_state)
                            .disabled(true)
                            .col_span(9),
                    )
                    .child(
                        Button::new("select_source_file")
                            .label("Select")
                            .outline()
                            .col_span(3)
                            .on_click(
                                window.event_listener_for(
                                    &self.state,
                                    SessionState::select_source_file,
                                ),
                            ),
                    ),
            )
            .child(app_input_text().child("Formats: WAV PCM 16/24/32-bit, mono/stereo."))
            .child(app_input_label().child("Output Directory").mt_3())
            .child(
                div()
                    .relative()
                    .grid()
                    .grid_cols(12)
                    .gap_2()
                    .child(
                        Input::new(&current_state.destination_path_state)
                            .disabled(true)
                            .col_span(9),
                    )
                    .child(
                        Button::new("select_destination_dir")
                            .label("Browse")
                            .outline()
                            .col_span(3)
                            .on_click(window.event_listener_for(
                                &self.state,
                                SessionState::select_destination_dir,
                            )),
                    ),
            )
            .child(app_input_label().block().child("Iterations").mt_3())
            .child(
                div()
                    .relative()
                    .grid()
                    .grid_cols(12)
                    .gap_2()
                    .child(Input::new(&current_state.iteration_count_state).col_span(9))
                    .child(
                        Button::new("start_render")
                            .label(current_state.session_status.title())
                            .primary()
                            .col_span(3)
                            .on_click(window.event_listener_for(&self.state, SessionState::record)),
                    ),
            )
    }
}
