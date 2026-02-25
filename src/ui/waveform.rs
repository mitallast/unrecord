use gpui::{
    AbsoluteLength, App, Background, BorderStyle, Bounds, Corners, Edges, Hsla, IntoElement,
    ParentElement, PathBuilder, Pixels, RenderOnce, Styled, Window, canvas, div, point, px, quad,
    size,
};
use gpui_component::{ActiveTheme, PixelsExt};
use std::sync::Arc;

#[derive(IntoElement)]
pub struct WaveRegion {
    // interleaved stereo: L,R,L,R...
    samples: Arc<Vec<f32>>,
    frames: usize,

    // viewport (как region + zoom)
    start_frame: usize,
    visible_frames: usize,

    // peaks cache for current width/zoom
    cache_key: (u32, usize, usize), // (width_px, start_frame, visible_frames)
    l: Vec<(f32, f32)>,             // per x: (min,max)
    r: Vec<(f32, f32)>,

    // styles
    paddings: Option<Edges<AbsoluteLength>>,
    corner_radius: Option<Corners<Pixels>>,
    background: Option<Background>,
    border_widths: Edges<Pixels>,
    border_color: Option<Hsla>,
    border_style: BorderStyle,
    gap: Pixels,
    stroke_width: Pixels,
    stroke_color: Option<Hsla>,
}

impl WaveRegion {
    pub fn new(samples: Arc<Vec<f32>>) -> Self {
        let frames = samples.len() / 2;
        Self {
            samples,
            frames,
            start_frame: 0,
            visible_frames: frames.min(48_000), // дефолт
            cache_key: (0, 0, 0),
            l: vec![],
            r: vec![],
            paddings: None,
            corner_radius: None,
            background: None,
            border_widths: Edges::default(),
            border_color: None,
            border_style: BorderStyle::default(),
            gap: px(4.0),
            stroke_width: px(1.0),
            stroke_color: None,
        }
    }

    pub fn paddings<L>(self, paddings: impl Into<Edges<L>>) -> Self
    where
        L: Into<AbsoluteLength> + Clone + Default + std::fmt::Debug + PartialEq,
    {
        let paddings = paddings.into();
        let paddings = paddings.map(|p| p.clone().into());
        Self {
            paddings: Some(paddings),
            ..self
        }
    }

    pub fn corner_radius(self, corner_radius: impl Into<Corners<Pixels>>) -> Self {
        Self {
            corner_radius: Some(corner_radius.into()),
            ..self
        }
    }

    /// Sets the border widths of the quad.
    pub fn border_widths(self, border_widths: impl Into<Edges<Pixels>>) -> Self {
        let border_widths = border_widths.into();
        Self {
            border_widths,
            ..self
        }
    }

    /// Sets the border color of the quad.
    pub fn border_color(self, border_color: impl Into<Hsla>) -> Self {
        Self {
            border_color: Some(border_color.into()),
            ..self
        }
    }

    /// Sets the background color of the quad.
    pub fn background(self, background: impl Into<Background>) -> Self {
        Self {
            background: Some(background.into()),
            ..self
        }
    }

    fn rebuild_cache_if_needed(&mut self, width_px: u32) {
        let key = (width_px.max(1), self.start_frame, self.visible_frames);
        if self.cache_key == key {
            return;
        }
        self.cache_key = key;

        let w = key.0 as usize;
        self.l.clear();
        self.r.clear();
        self.l.reserve(w);
        self.r.reserve(w);

        let start = self.start_frame.min(self.frames);
        let vis = self.visible_frames.min(self.frames.saturating_sub(start));
        let spp = (vis as f32 / w as f32).max(1.0); // frames per pixel

        for x in 0..w {
            let a = start as f32 + x as f32 * spp;
            let b = start as f32 + (x as f32 + 1.0) * spp;
            let ia = a.floor() as usize;
            let ib = b.ceil() as usize;

            let ia = ia.min(start + vis);
            let ib = ib.min(start + vis);

            let mut lmin = 0.0f32;
            let mut lmax = 0.0f32;
            let mut rmin = 0.0f32;
            let mut rmax = 0.0f32;

            if ib > ia {
                lmin = 1.0;
                lmax = -1.0;
                rmin = 1.0;
                rmax = -1.0;
                for f in ia..ib {
                    let i = f * 2;
                    if i + 1 >= self.samples.len() {
                        break;
                    }
                    let lv = self.samples[i];
                    let rv = self.samples[i + 1];
                    lmin = lmin.min(lv);
                    lmax = lmax.max(lv);
                    rmin = rmin.min(rv);
                    rmax = rmax.max(rv);
                }
            }

            self.l.push((lmin, lmax));
            self.r.push((rmin, rmax));
        }
    }

    fn apply_padding(&self, bounds: Bounds<Pixels>, paddings: Edges<Pixels>) -> Bounds<Pixels> {
        let w = paddings.left + paddings.right;
        let h = paddings.top + paddings.bottom;
        let p_size = size(w, h);

        let origin = bounds.origin + point(paddings.left, paddings.top);
        let size = bounds.size - p_size;

        Bounds::new(origin, size)
    }

    fn split_bounds(&self, bounds: Bounds<Pixels>) -> (Bounds<Pixels>, Bounds<Pixels>) {
        let half_height = (bounds.size.height - self.gap) * 0.5;
        let left = bounds.left();
        let right = bounds.right();
        let top = bounds.top();

        let upper = Bounds::from_corners(point(left, top), point(right, top + half_height));
        let lower_top = upper.bottom() + self.gap;
        let lower = Bounds::from_corners(point(left, lower_top), point(right, bounds.bottom()));
        (upper, lower)
    }

    fn paint_channel(
        &self,
        peaks: &[(f32, f32)],
        bounds: Bounds<Pixels>,
        color: Hsla,
        window: &mut Window,
    ) {
        let mid_y = bounds.center().y;
        let amp = bounds.size.height * 0.5;
        let x_start = bounds.left();

        let clamp = |v: f32| v.clamp(-1.0, 1.0);
        let mut builder = PathBuilder::stroke(self.stroke_width);

        for (x, &(mn, mx)) in peaks.iter().enumerate() {
            let xx = x_start + px(x as f32 + 0.5);
            let y1 = mid_y - amp * clamp(mx);
            let y2 = mid_y - amp * clamp(mn);
            builder.move_to(point(xx, y1));
            builder.line_to(point(xx, y2));
        }

        if let Ok(path) = builder.build() {
            window.paint_path(path, color);
        }
    }
}

impl RenderOnce for WaveRegion {
    fn render(mut self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div().gap_2().size_full().child(
            canvas(
                move |_, _, _| {}, // input layer (не обязателен)
                move |bounds, _, window, cx| {
                    let theme = cx.theme();

                    let rem_size = window.rem_size();
                    let inner_bounds = self.apply_padding(
                        bounds,
                        self.paddings
                            .unwrap_or_else(|| Edges {
                                top: theme.radius.into(),
                                bottom: theme.radius.into(),
                                left: px(1.0).into(),
                                right: px(1.0).into(),
                            })
                            .to_pixels(rem_size),
                    );
                    let inner_bounds = self.apply_padding(inner_bounds, self.border_widths);

                    let width_px = inner_bounds.size.width.as_f32().round().max(1.0);

                    self.rebuild_cache_if_needed(width_px as u32);

                    // фон “региона”
                    window.paint_quad(quad(
                        bounds,
                        self.corner_radius.unwrap_or(theme.radius.into()),
                        self.background.unwrap_or(theme.primary.into()),
                        self.border_widths,
                        self.border_color.unwrap_or(theme.border),
                        self.border_style,
                    ));
                    // window.paint_quad(fill(bounds, rgb(0xFF0000)));

                    let (left_bounds, right_bounds) = self.split_bounds(inner_bounds);
                    let stroke_color = self.stroke_color.unwrap_or(theme.primary_foreground);
                    self.paint_channel(&self.l, left_bounds, stroke_color, window);
                    self.paint_channel(&self.r, right_bounds, stroke_color, window);
                },
            )
            .size_full(),
        )
    }
}
