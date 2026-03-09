use crate::components::grid::{GridViewport, GridViewportHandle};
use crate::components::region::TrackRegionView;
use crate::components::track::Track;
use gpui::{
    AnyElement, App, AvailableSpace, Bounds, ContentMask, Element, ElementId, GlobalElementId, InspectorElementId,
    IntoElement, LayoutId, Pixels, Refineable, Style, StyleRefinement, Styled, Window, point, px, size,
};
use std::collections::VecDeque;
use std::panic::Location;

pub struct TrackView {
    viewport: GridViewport,
    track: Track,
    style: StyleRefinement,
}

impl TrackView {
    pub fn new(viewport: &GridViewport, track: &Track) -> Self {
        Self {
            viewport: viewport.clone(),
            track: track.clone(),
            style: StyleRefinement::default(),
        }
    }
}

impl IntoElement for TrackView {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Styled for TrackView {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

struct TrackLayoutResponse {
    layouts: VecDeque<TrackRegionLayout>,
}

struct TrackRegionLayout {
    element: AnyElement,
    bounds: Bounds<Pixels>,
}

pub struct TrackPrepaintState {
    style: Style,
    layouts: VecDeque<TrackRegionLayout>,
}

impl Element for TrackView {
    type RequestLayoutState = ();
    type PrepaintState = TrackPrepaintState;

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
            window.request_layout(style.clone(), None, cx)
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
        let layouts = self.prepaint_items(bounds, window, cx).unwrap().layouts;
        TrackPrepaintState { style, layouts }
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

impl TrackView {
    fn layout_items(&self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) -> TrackLayoutResponse {
        let available_item_space = size(
            AvailableSpace::Definite(bounds.size.width),
            AvailableSpace::Definite(bounds.size.height),
        );

        let top_px = px(4.0);
        let height_px = bounds.size.height - (top_px * 2);

        let mut layouts = VecDeque::new();
        for region in self.track.regions() {
            let left_px = self.viewport.frame_to_scroll_offset(region.track_start_frame());
            let width_px = self.viewport.frame_to_track_offset(region.frames());

            let region_window_bounds = Bounds {
                origin: bounds.origin + point(left_px, top_px), // window coords
                size: size(width_px, height_px),
            };
            if !region_window_bounds.intersects(&bounds) {
                continue;
            }

            let mut region_bounds = region_window_bounds.intersect(&bounds);

            let mut region_scroll_bounds = region_bounds;
            region_scroll_bounds.origin -= bounds.origin;

            // track offset
            let start_frame = self.viewport.scroll_offset_to_frame(region_scroll_bounds.left());
            let end_frame = self.viewport.scroll_offset_to_frame(region_scroll_bounds.right());

            // scroll coords
            let left_px = self.viewport.frame_to_scroll_offset(start_frame);
            let right_px = region_scroll_bounds.right();

            // window coords
            region_bounds.origin.x = bounds.origin.x + left_px;
            region_bounds.size.width = right_px - left_px;

            let mut element = TrackRegionView::new(&region, start_frame, end_frame + 1, self.viewport.frames_per_px())
                .w(region_bounds.size.width)
                .h(region_bounds.size.height)
                .py(px(8.0))
                .into_any_element();

            element.layout_as_root(available_item_space, window, cx);
            layouts.push_back(TrackRegionLayout {
                element,
                bounds: region_bounds,
            });
        }

        TrackLayoutResponse { layouts }
    }

    fn prepaint_items(
        &mut self,
        bounds: Bounds<Pixels>,
        window: &mut Window,
        cx: &mut App,
    ) -> Result<TrackLayoutResponse, ()> {
        window.transact(|window| {
            let mut layout = self.layout_items(bounds, window, cx);
            for item in &mut layout.layouts {
                window.with_content_mask(Some(ContentMask { bounds }), |window| {
                    item.element.prepaint_at(item.bounds.origin, window, cx);
                });
            }
            Ok(layout)
        })
    }
}
