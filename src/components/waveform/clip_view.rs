use crate::components::waveform::{WaveClip, WaveFormView};
use gpui::{
    AnyElement, App, AvailableSpace, Bounds, ContentMask, Element, ElementId, GlobalElementId, InspectorElementId,
    IntoElement, LayoutId, Pixels, Refineable, Style, StyleRefinement, Styled, Window, point, px, size,
};
use gpui_component::PixelsExt;
use std::collections::VecDeque;
use std::panic::Location;

pub struct WaveClipView {
    clip: WaveClip,
    start_frame: usize,
    end_frame: usize,
    frames_per_px: f64,
    style: StyleRefinement,
}

impl WaveClipView {
    pub fn new(clip: &WaveClip, start_frame: usize, end_frame: usize, frames_per_px: f64) -> Self {
        Self {
            clip: clip.clone(),
            start_frame,
            end_frame,
            frames_per_px,
            style: StyleRefinement::default(),
        }
    }
}

impl Styled for WaveClipView {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl IntoElement for WaveClipView {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

pub struct WaveClipPrepaintState {
    style: Style,
    layouts: VecDeque<WaveFormLayout>,
}

struct WaveClipLayoutResponse {
    layouts: VecDeque<WaveFormLayout>,
}

struct WaveFormLayout {
    element: AnyElement,
    bounds: Bounds<Pixels>,
}

impl Element for WaveClipView {
    type RequestLayoutState = ();
    type PrepaintState = WaveClipPrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.refine(&self.style);
        let layout_id = window.with_text_style(style.text_style().cloned(), |window| {
            window.request_layout(style, None, cx)
        });
        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let mut style = Style::default();
        style.refine(&self.style);

        let layouts = self.prepaint_items(bounds, &style, window, cx).unwrap().layouts;

        WaveClipPrepaintState { style, layouts }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        prepaint.style.paint(bounds, window, cx, |window, cx| {
            window.with_text_style(prepaint.style.text_style().cloned(), |window| {
                window.with_content_mask(Some(ContentMask { bounds }), |window| {
                    for item in &mut prepaint.layouts {
                        item.element.paint(window, cx);
                    }
                })
            })
        });
    }
}

impl WaveClipView {
    fn prepaint_items(
        &mut self,
        bounds: Bounds<Pixels>,
        style: &Style,
        window: &mut Window,
        cx: &mut App,
    ) -> Result<WaveClipLayoutResponse, ()> {
        window.transact(|window| {
            let mut layout = self.layout_items(bounds, &style, window, cx);

            for item in &mut layout.layouts {
                window.with_content_mask(Some(ContentMask { bounds }), |window| {
                    item.element.prepaint_at(item.bounds.origin, window, cx);
                });
            }

            Ok(layout)
        })
    }

    fn layout_items(
        &mut self,
        bounds: Bounds<Pixels>,
        style: &Style,
        window: &mut Window,
        cx: &mut App,
    ) -> WaveClipLayoutResponse {
        let available_item_space = size(
            AvailableSpace::Definite(bounds.size.width),
            AvailableSpace::Definite(bounds.size.height),
        );
        let channel_bounds = self.split_region_channels(&style, bounds, window);
        let channels_iter = self.clip.channels().iter().zip(channel_bounds.iter());

        let mut layouts = VecDeque::with_capacity(self.clip.channels().len());
        for (waveform, channel_bounds) in channels_iter {
            let mut element = WaveFormView::new(waveform, self.start_frame, self.end_frame, self.frames_per_px)
                .h(channel_bounds.size.height)
                .w(channel_bounds.size.width)
                .into_any_element();

            element.layout_as_root(available_item_space, window, cx);
            layouts.push_back(WaveFormLayout {
                element,
                bounds: channel_bounds.clone(),
            });
        }

        WaveClipLayoutResponse { layouts }
    }

    fn split_region_channels(&self, style: &Style, bounds: Bounds<Pixels>, window: &Window) -> Vec<Bounds<Pixels>> {
        let channels = self.clip.channel_count();
        if channels == 0 {
            return Vec::new();
        }

        let padding = style.padding.to_pixels(bounds.size.into(), window.rem_size());

        let gap = style.gap.height.to_pixels(bounds.size.height.into(), window.rem_size());

        let left = bounds.left() + padding.left;
        let right = bounds.right() - padding.right;
        let top = bounds.top() + padding.top;
        let bottom = bounds.bottom() - padding.bottom;
        if right <= left || bottom <= top {
            return Vec::new();
        }

        let total_gap = gap * (channels.saturating_sub(1) as f32);
        let available_height = (bottom - top - total_gap).max(px(0.0));
        let channel_height_value = available_height.as_f64() / channels as f64;
        let channel_height = px(channel_height_value as f32);
        let mut result = Vec::with_capacity(channels);

        for index in 0..channels {
            let channel_top = top + px((index as f64 * (channel_height.as_f64() + gap.as_f64())) as f32);
            let channel_bottom = if index + 1 == channels {
                bottom
            } else {
                channel_top + channel_height
            };

            if channel_bottom <= channel_top {
                break;
            }

            result.push(Bounds::from_corners(
                point(left, channel_top),
                point(right, channel_bottom),
            ));
        }

        result
    }
}
