use crate::UnrecordApp;
use crate::ui::{app_panel, app_panel_title};
use gpui::{
    App, Context, Div, Element, IntoElement, ListAlignment, ListState, ParentElement, RenderOnce,
    StyleRefinement, Styled, Window, div, list, px, rems,
};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{StyledExt, gray_400};

pub struct TrackInfoItem {
    key: String,
    value: String,
}

impl TrackInfoItem {
    pub fn new(key: &str, value: &str) -> Self {
        Self {
            key: key.to_string(),
            value: value.to_string(),
        }
    }
}

#[derive(IntoElement)]
pub struct TrackInfoPanel {
    items: Vec<TrackInfoItem>,
    style: StyleRefinement,
}

impl TrackInfoPanel {
    pub fn new() -> Self {
        Self {
            items: Self::info(),
            style: StyleRefinement::default(),
        }
    }

    fn info() -> Vec<TrackInfoItem> {
        vec![
            TrackInfoItem::new("Duration", "00:05.312"),
            TrackInfoItem::new("Duration", "00:05.312"),
            TrackInfoItem::new("Sample Rate", "48,000 Hz"),
            TrackInfoItem::new("Bit Depth", "24-bit PCM"),
            TrackInfoItem::new("Channels", "Stereo 2.0"),
            TrackInfoItem::new("Bitrate", "2,304 kbps"),
            TrackInfoItem::new("File Size", "1.48 MB"),
            TrackInfoItem::new("Peak", "-0.9 dBFS"),
            TrackInfoItem::new("True Peak", "-0.6 dBTP"),
            TrackInfoItem::new("RMS", "-13.3 dBFS"),
            TrackInfoItem::new("LUFS Integrated", "-16.1 LUFS"),
            TrackInfoItem::new("Crest Factor", "12.4 dB"),
            TrackInfoItem::new("Noise Floor", "-81.7 dBFS"),
            TrackInfoItem::new("DC Offset", "0.03 %"),
        ]
    }
}

impl Styled for TrackInfoPanel {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for TrackInfoPanel {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let list_state = window
            .use_keyed_state("track-info-state", cx, |_, _| {
                ListState::new(0, ListAlignment::Top, px(10.))
            })
            .read(cx)
            .clone();

        if list_state.item_count() != self.items.len() {
            list_state.reset(self.items.len());
        }

        div()
            .size_full()
            .relative()
            .refine_style(&self.style) //
            .child(
                list(list_state.clone(), move |i, _, _| {
                    let item = &self.items[i];
                    div()
                        .w_full()
                        .flex()
                        .flex_row()
                        .justify_between()
                        .text_size(rems(0.75))
                        .line_height(rems(1.0))
                        .child(div().text_color(gray_400()).child(item.key.clone()))
                        .child(div().text_right().child(item.value.clone()))
                        .into_any()
                })
                .size_full(),
            )
            .vertical_scrollbar(&list_state)
    }
}

pub fn info_panel(cx: &mut Context<UnrecordApp>) -> Div {
    app_panel(cx)
        .p_4()
        .w_full()
        .flex()
        .flex_col()
        .child(app_panel_title().child("Track info"))
        .child(TrackInfoPanel::new().relative().size_full())
}
