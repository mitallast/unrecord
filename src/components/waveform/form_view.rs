use crate::components::waveform::WaveForm;
use gpui::{
    App, Bounds, ContentMask, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement, LayoutId, Path,
    Pixels, Point, Refineable, Style, StyleRefinement, Styled, Window, point, px,
};
use gpui_component::{ActiveTheme, PixelsExt};
use std::panic::Location;

pub struct WaveFormView {
    waveform: WaveForm,
    start_frame: usize,
    end_frame: usize,
    frames_per_px: f32,
    stroke_width_half: Pixels,
    style: StyleRefinement,
}

impl WaveFormView {
    pub fn new(waveform: &WaveForm, start_frame: usize, end_frame: usize, frames_per_px: f64) -> Self {
        Self {
            waveform: waveform.clone(),
            start_frame,
            end_frame,
            frames_per_px: frames_per_px as f32,
            stroke_width_half: px(0.5),
            style: StyleRefinement::default(),
        }
    }
}

impl Styled for WaveFormView {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl IntoElement for WaveFormView {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

pub struct WaveFormPrepaintState {
    path: Option<Path<Pixels>>,
}

impl Element for WaveFormView {
    type RequestLayoutState = ();
    type PrepaintState = WaveFormPrepaintState;

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
        _window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
        let min_frames_per_px = self.waveform.first_frames_per_bucket() as f32;
        let path = if self.frames_per_px >= min_frames_per_px {
            self.prepaint_mipmap(bounds)
        } else if self.frames_per_px >= 0.5 {
            self.prepaint_samples_lines(bounds)
        } else {
            self.prepaint_samples_square(bounds)
        };

        WaveFormPrepaintState { path }
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
        let mut style = Style::default();
        style.refine(&self.style);
        style.paint(bounds, window, cx, |window, cx| {
            let theme = cx.theme();
            window.with_text_style(style.text_style().cloned(), |window| {
                window.with_content_mask(Some(ContentMask { bounds }), |window| {
                    let color = theme.primary_foreground;
                    if let Some(path) = prepaint.path.take() {
                        window.paint_path(path, color);
                    }
                })
            })
        });
    }
}

impl WaveFormView {
    const STP: Point<f32> = point(0., 1.);
    const ST: (Point<f32>, Point<f32>, Point<f32>) = (Self::STP, Self::STP, Self::STP);

    fn prepaint_samples_square(&self, bounds: Bounds<Pixels>) -> Option<Path<Pixels>> {
        let mid_y = bounds.center().y;
        let amp = bounds.size.height * 0.5;
        let frame_width = px(1.0 / self.frames_per_px);

        let start_frame = self.start_frame.min(self.waveform.samples().len().saturating_sub(1));
        let end_frame = self.end_frame.max(start_frame).min(self.waveform.samples().len());

        let range = start_frame..end_frame;
        let samples = self.waveform.samples()[range].iter().enumerate();

        let st = (point(0., 1.), point(0., 1.), point(0., 1.));

        // tesselation requires too much time, so manually
        let mut path = Path::new(bounds.origin);
        let mut prev: Option<Point<Pixels>> = None;

        for (frame, sample) in samples {
            let y = mid_y - amp * sample.clamp(-1.0, 1.0);
            let x = bounds.left() + (frame_width * frame);

            if let Some(prev) = prev {
                // draw vertical line
                let l = x - self.stroke_width_half;
                let r = x + self.stroke_width_half;
                let t = prev.y - self.stroke_width_half;
                let b = y + self.stroke_width_half;
                path.push_triangle((point(l, b), point(l, t), point(r, t)), st);
                path.push_triangle((point(l, b), point(r, t), point(r, b)), st);
            }

            // draw horizontal line
            let l = x;
            let r = x + frame_width;
            let t = y + self.stroke_width_half;
            let b = y - self.stroke_width_half;
            path.push_triangle((point(l, b), point(l, t), point(r, t)), st);
            path.push_triangle((point(l, b), point(r, t), point(r, b)), st);

            prev = Some(point(r, y));
        }
        Some(path)
    }

    fn prepaint_samples_lines(&self, bounds: Bounds<Pixels>) -> Option<Path<Pixels>> {
        let mid_y = bounds.center().y;
        let amp = bounds.size.height * 0.5;
        let frame_width = px(1.0 / self.frames_per_px);

        let start_frame = self.start_frame.min(self.waveform.samples().len().saturating_sub(1));
        let end_frame = self.end_frame.max(start_frame).min(self.waveform.samples().len());
        let range = start_frame..end_frame;
        let samples = self.waveform.samples()[range].iter().enumerate();

        // tesselation requires too much time, so manually
        let mut path = Path::new(bounds.origin);
        let mut prev: Option<Point<Pixels>> = None;

        for (frame, sample) in samples {
            let y = mid_y - amp * sample.clamp(-1.0, 1.0);
            let x = bounds.left() + (frame_width * frame);
            let curr = point(x, y);

            if let Some(prev) = prev {
                self.push_line_as_triangles(&mut path, prev, curr);
            }

            prev = Some(point(x, y));
        }
        Some(path)
    }

    fn prepaint_mipmap(&self, bounds: Bounds<Pixels>) -> Option<Path<Pixels>> {
        let mid_y = bounds.center().y;
        let amp = bounds.size.height * 0.5;
        let width = bounds.size.width.as_f64().round().max(1.0) as usize;

        let frame_offset = self.start_frame;

        // tesselation requires too much time, so manually
        let mut path = Path::new(bounds.origin);
        let mut prev: Option<(f32, f32)> = None;
        for x in 0..width {
            let start_frame = frame_offset + (x as f32 * self.frames_per_px).round() as usize;
            let end_frame = frame_offset + ((x + 1) as f32 * self.frames_per_px).round() as usize;

            if let Some((min, max)) = self
                .waveform
                .min_max_for_frames(start_frame, end_frame, self.frames_per_px)
            {
                if let Some((prev_min, prev_max)) = prev {
                    let left = bounds.left() + px(x as f32 - 1.0) - self.stroke_width_half;
                    let right = bounds.left() + px(x as f32) + self.stroke_width_half;

                    let left_max = mid_y - amp * prev_max.clamp(-1.0, 1.0) - self.stroke_width_half;
                    let left_min = mid_y - amp * prev_min.clamp(-1.0, 1.0) - self.stroke_width_half;

                    let right_max = mid_y - amp * max.clamp(-1.0, 1.0) + self.stroke_width_half;
                    let right_min = mid_y - amp * min.clamp(-1.0, 1.0) + self.stroke_width_half;

                    // top triangle
                    path.push_triangle(
                        (point(left, left_min), point(left, left_max), point(right, right_max)),
                        Self::ST,
                    );
                    // bottom triangle
                    path.push_triangle(
                        (point(left, left_min), point(right, right_max), point(right, right_min)),
                        Self::ST,
                    );
                }

                prev = Some((min, max));
            }
        }
        Some(path)
    }

    fn push_line_as_triangles(&self, path: &mut Path<Pixels>, a: Point<Pixels>, b: Point<Pixels>) {
        let dx = b.x.as_f32() - a.x.as_f32();
        let dy = b.y.as_f32() - a.y.as_f32();

        let len = (dx * dx + dy * dy).sqrt();
        if len == 0.0 {
            return;
        }

        let half = self.stroke_width_half.as_f32();

        // Нормаль к линии
        let nx = px(-dy / len * half);
        let ny = px(dx / len * half);

        let p0 = point(a.x + nx, a.y + ny);
        let p1 = point(a.x - nx, a.y - ny);
        let p2 = point(b.x + nx, b.y + ny);
        let p3 = point(b.x - nx, b.y - ny);

        path.push_triangle((p0, p1, p2), Self::ST);
        path.push_triangle((p2, p1, p3), Self::ST);
    }
}
