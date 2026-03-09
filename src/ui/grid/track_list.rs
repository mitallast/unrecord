use crate::components::grid::{GridViewport, GridViewportHandle};
use crate::components::tick::GridTickType;
use crate::components::track::{Track, TrackView};
use gpui::{
    AnyElement, App, AvailableSpace, Bounds, ContentMask, DispatchPhase, Element, ElementId, GlobalElementId, Hitbox,
    HitboxBehavior, InspectorElementId, IntoElement, LayoutId, Pixels, Point, Refineable, ScrollDelta,
    ScrollWheelEvent, Style, StyleRefinement, Styled, Window, fill, point, px, rgb, size,
};
use std::collections::VecDeque;
use std::panic::Location;

pub struct GridTrackList {
    tracks: Vec<Track>,
    viewport: GridViewport,
    style: StyleRefinement,
}

impl GridTrackList {
    pub fn new(tracks: Vec<Track>, viewport: &GridViewport) -> Self {
        Self {
            tracks,
            viewport: viewport.clone(),
            style: StyleRefinement::default(),
        }
    }

    fn layout_items(&self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) -> GridTrackListLayoutResponse {
        let available_item_space = size(AvailableSpace::Definite(bounds.size.width), AvailableSpace::MinContent);

        let mut layouts = VecDeque::new();

        self.viewport.set_viewport_width(bounds.size.width);
        self.viewport.set_viewport_height(bounds.size.height);

        let element_width = self.viewport.viewport_size().width;
        let header_size = self.viewport.header_size();
        let scroll_offset_y = self.viewport.scroll_offset().y;

        let mut origin = bounds.origin;
        origin.y += scroll_offset_y;

        // todo optimize skip to start
        // todo optimize break on end

        for track in &self.tracks {
            let intersects = (origin.y + header_size.height >= bounds.origin.y)
                && (origin.y <= bounds.origin.y + bounds.size.height);

            if intersects {
                let mut element = TrackView::new(&self.viewport, &track)
                    .w(element_width)
                    .h(header_size.height)
                    .py(px(8.0))
                    .gap(px(8.0))
                    .into_any_element();

                element.layout_as_root(available_item_space, window, cx);
                layouts.push_back(GridTrackLayout { element, origin });
            }

            origin.y += header_size.height;
        }

        GridTrackListLayoutResponse { layouts }
    }

    fn prepaint_items(
        &self,
        bounds: Bounds<Pixels>,
        window: &mut Window,
        cx: &mut App,
    ) -> Result<GridTrackListLayoutResponse, ()> {
        window.transact(|window| {
            let mut layout = self.layout_items(bounds, window, cx);

            for item in &mut layout.layouts {
                window.with_content_mask(Some(ContentMask { bounds }), |window| {
                    item.element.prepaint_at(item.origin, window, cx);
                });
            }

            Ok(layout)
        })
    }
}

impl IntoElement for GridTrackList {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Styled for GridTrackList {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

struct GridTrackListLayoutResponse {
    layouts: VecDeque<GridTrackLayout>,
}

pub struct GridTrackListPrepaintState {
    hitbox: Hitbox,
    layouts: VecDeque<GridTrackLayout>,
}

struct GridTrackLayout {
    element: AnyElement,
    origin: Point<Pixels>,
}

impl Element for GridTrackList {
    type RequestLayoutState = ();
    type PrepaintState = GridTrackListPrepaintState;

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
        let hitbox = window.insert_hitbox(bounds, HitboxBehavior::Normal);

        let layouts = self.prepaint_items(bounds, window, cx).unwrap().layouts;

        GridTrackListPrepaintState { hitbox, layouts }
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
        let current_view = window.current_view();

        let mut style = Style::default();
        style.refine(&self.style);
        style.paint(bounds, window, cx, |window, cx| {
            window.with_text_style(style.text_style().cloned(), |window| {
                window.with_content_mask(Some(ContentMask { bounds }), |window| {
                    // background grid
                    for tick in self.viewport.ticks() {
                        let grid_bounds = Bounds::new(
                            point(bounds.origin.x + tick.offset_x, bounds.origin.y),
                            size(px(1.0), bounds.size.height),
                        );
                        let color = match tick.tick_type {
                            GridTickType::PRIMARY => rgb(0x414141),
                            GridTickType::SECONDARY => rgb(0x393939),
                        };
                        window.paint_quad(fill(grid_bounds, color));
                    }

                    for item in &mut prepaint.layouts {
                        item.element.paint(window, cx);
                    }
                })
            })
        });

        let viewport = self.viewport.clone();
        let hitbox_id = prepaint.hitbox.id;
        let mut accumulated_scroll_delta = ScrollDelta::default();

        window.on_mouse_event(move |event: &ScrollWheelEvent, phase, window, cx| {
            if phase == DispatchPhase::Bubble && hitbox_id.should_handle_scroll(window) {
                accumulated_scroll_delta = accumulated_scroll_delta.coalesce(event.delta);
                let pixel_delta = accumulated_scroll_delta.pixel_delta(px(20.));
                viewport.on_scroll(pixel_delta);
                cx.notify(current_view);
            }
        });
    }
}
