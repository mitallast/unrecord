use crate::components::grid::{GridViewport, GridViewportHandle};
use crate::components::track::{Track, TrackHeaderView};
use gpui::{
    AnyElement, App, AvailableSpace, Bounds, ContentMask, DispatchPhase, Element, ElementId, GlobalElementId, Hitbox,
    HitboxBehavior, InspectorElementId, IntoElement, LayoutId, Pixels, Point, Refineable, ScrollDelta,
    ScrollWheelEvent, Style, StyleRefinement, Styled, Window, px, rgb, size,
};
use std::collections::VecDeque;
use std::panic::Location;

pub struct GridHeaderList {
    tracks: Vec<Track>,
    viewport: GridViewport,
    style: StyleRefinement,
}

impl GridHeaderList {
    pub fn new(tracks: Vec<Track>, viewport: &GridViewport) -> Self {
        Self {
            tracks,
            viewport: viewport.clone(),
            style: StyleRefinement::default(),
        }
    }

    fn layout_items(&self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) -> GridHeaderListLayoutResponse {
        self.viewport.set_viewport_height(bounds.size.height);

        let available_item_space = size(
            AvailableSpace::Definite(self.viewport.viewport_size().width),
            AvailableSpace::MinContent,
        );

        let mut header_layouts = VecDeque::new();
        let element_size = self.viewport.header_size();

        let mut origin = bounds.origin;
        origin.y += self.viewport.scroll_offset().y;

        // todo optimize skip to start
        // todo optimize break on end
        for track in &self.tracks {
            let intersects = (origin.y + element_size.height >= bounds.origin.y)
                && (origin.y <= bounds.origin.y + bounds.size.height);

            if intersects {
                let mut element = TrackHeaderView::new(&track)
                    .w(element_size.width)
                    .h(element_size.height)
                    .into_any_element();

                element.layout_as_root(available_item_space, window, cx);
                header_layouts.push_back(GridHeaderLayout { element, origin });
            }

            origin.y += element_size.height;
        }

        GridHeaderListLayoutResponse { header_layouts }
    }

    fn prepaint_items(
        &self,
        bounds: Bounds<Pixels>,
        window: &mut Window,
        cx: &mut App,
    ) -> Result<GridHeaderListLayoutResponse, ()> {
        window.transact(|window| {
            let mut layout = self.layout_items(bounds, window, cx);

            for item in &mut layout.header_layouts {
                window.with_content_mask(Some(ContentMask { bounds }), |window| {
                    item.element.prepaint_at(item.origin, window, cx);
                });
            }

            Ok(layout)
        })
    }
}

impl IntoElement for GridHeaderList {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Styled for GridHeaderList {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

struct GridHeaderListLayoutResponse {
    header_layouts: VecDeque<GridHeaderLayout>,
}

pub struct GridHeaderListPrepaintState {
    hitbox: Hitbox,
    header_layouts: VecDeque<GridHeaderLayout>,
}

struct GridHeaderLayout {
    element: AnyElement,
    origin: Point<Pixels>,
}

impl Element for GridHeaderList {
    type RequestLayoutState = ();
    type PrepaintState = GridHeaderListPrepaintState;

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

        let header_layouts = self.prepaint_items(bounds, window, cx).unwrap().header_layouts;

        GridHeaderListPrepaintState { hitbox, header_layouts }
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
        style.background = Some(rgb(0x575757).into());
        style.border_widths.right = px(1.0).into();
        style.border_color = Some(rgb(0x454545).into());

        style.refine(&self.style);
        style.paint(bounds, window, cx, |window, cx| {
            window.with_text_style(style.text_style().cloned(), |window| {
                window.with_content_mask(Some(ContentMask { bounds }), |window| {
                    for item in &mut prepaint.header_layouts {
                        item.element.paint(window, cx);
                    }
                })
            })
        });

        let hitbox_id = prepaint.hitbox.id;
        let mut accumulated_scroll_delta = ScrollDelta::default();

        let viewport = self.viewport.clone();
        window.on_mouse_event(move |event: &ScrollWheelEvent, phase, window, cx| {
            if phase == DispatchPhase::Bubble && hitbox_id.should_handle_scroll(window) {
                accumulated_scroll_delta = accumulated_scroll_delta.coalesce(event.delta);
                let pixel_delta = accumulated_scroll_delta.pixel_delta(px(20.));
                viewport.on_scroll_y(pixel_delta.y);
                cx.notify(current_view)
            }
        });
    }
}
