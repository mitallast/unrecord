use crate::components::region::region::TrackRegion;
use crate::components::waveform::WaveClipView;
use gpui::{
    AnyElement, App, AvailableSpace, BorderStyle, Bounds, ContentMask, Corners, Edges, Element, ElementId,
    GlobalElementId, InspectorElementId, LayoutId, PaintQuad, Pixels, Refineable, Style, Window, point, px, size,
};
use gpui::{IntoElement, StyleRefinement, Styled};
use gpui_component::ActiveTheme;
use std::panic::Location;

pub struct TrackRegionView {
    region: TrackRegion,
    start_frame: usize,
    end_frame: usize,
    frames_per_px: f64,
    style: StyleRefinement,
}

impl TrackRegionView {
    pub fn new(region: &TrackRegion, start_frame: usize, end_frame: usize, frames_per_px: f64) -> Self {
        Self {
            region: region.clone(),
            start_frame,
            end_frame,
            frames_per_px,
            style: StyleRefinement::default(),
        }
    }
}

impl Styled for TrackRegionView {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl IntoElement for TrackRegionView {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

pub struct TrackRegionPrepaintState {
    style: Style,
    region_quad: PaintQuad,
    clip: AnyElement,
}

struct TrackRegionLayoutResponse {
    region_quad: PaintQuad,
    clip: AnyElement,
    clip_bounds: Bounds<Pixels>,
}

impl Element for TrackRegionView {
    type RequestLayoutState = ();
    type PrepaintState = TrackRegionPrepaintState;

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
        let layout = self.prepaint_items(bounds, &style, window, cx).unwrap();

        TrackRegionPrepaintState {
            style,
            region_quad: layout.region_quad,
            clip: layout.clip,
        }
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
                    window.paint_quad(prepaint.region_quad.clone());
                    prepaint.clip.paint(window, cx);
                })
            })
        });
    }
}

impl TrackRegionView {
    fn prepaint_items(
        &mut self,
        bounds: Bounds<Pixels>,
        style: &Style,
        window: &mut Window,
        cx: &mut App,
    ) -> Result<TrackRegionLayoutResponse, ()> {
        window.transact(|window| {
            let mut layout = self.layout_items(bounds, &style, window, cx);

            window.with_content_mask(Some(ContentMask { bounds }), |window| {
                layout.clip.prepaint_at(layout.clip_bounds.origin, window, cx);
            });

            Ok(layout)
        })
    }

    fn layout_items(
        &self,
        bounds: Bounds<Pixels>,
        style: &Style,
        window: &mut Window,
        cx: &mut App,
    ) -> TrackRegionLayoutResponse {
        let available_item_space = size(
            AvailableSpace::Definite(bounds.size.width),
            AvailableSpace::Definite(bounds.size.height),
        );
        let padding = style.padding.to_pixels(bounds.size.into(), window.rem_size());

        let left = bounds.left() + padding.left;
        let right = bounds.right() - padding.right;
        let top = bounds.top() + padding.top;
        let bottom = bounds.bottom() - padding.bottom;

        let clip_bounds = Bounds::from_corners(point(left, top), point(right, bottom));

        let track_offset = self.region.track_start_frame() + self.region.clip_start_frame();
        let clip_start_frame = self.start_frame.saturating_sub(track_offset);
        let clip_end_frame = self.end_frame.saturating_sub(track_offset);

        let mut clip = WaveClipView::new(
            &self.region.clip(),
            clip_start_frame,
            clip_end_frame,
            self.frames_per_px,
        )
        .h(clip_bounds.size.height)
        .w(clip_bounds.size.width)
        .gap(px(8.0))
        .into_any_element();

        clip.layout_as_root(available_item_space, window, cx);

        let theme = cx.theme();
        let mut region_quad = PaintQuad {
            bounds,
            corner_radii: Corners::all(px(8.0)),
            background: theme.primary.alpha(0.5).into(),
            border_widths: Edges::all(px(1.0)),
            border_color: theme.primary,
            border_style: BorderStyle::Solid,
        };

        // cut left
        if clip_start_frame > self.region.clip_start_frame() {
            region_quad.border_widths.left = px(0.0);
            region_quad.corner_radii.top_left = px(0.0);
            region_quad.corner_radii.bottom_left = px(0.0);
        }
        // cut right
        if clip_end_frame < self.region.clip_end_frame() {
            region_quad.border_widths.right = px(0.0);
            region_quad.corner_radii.top_right = px(0.0);
            region_quad.corner_radii.bottom_right = px(0.0);
        }

        TrackRegionLayoutResponse {
            region_quad,
            clip,
            clip_bounds,
        }
    }
}
