use crate::components::tick::{GridTick, GridTickGenerator};
use crate::time::{Duration, SampleRate, TimeCode};
use gpui::{Negate, Pixels, Point, Size, point, px, size};
use gpui_component::PixelsExt;
use gpui_component::scroll::ScrollbarHandle;
use std::cell::RefCell;
use std::ops::Sub;
use std::rc::Rc;

#[derive(Clone)]
pub struct GridViewport {
    inner: Rc<RefCell<GridViewportInner>>,
}

impl GridViewport {
    pub fn new(sample_rate: SampleRate) -> Self {
        Self {
            inner: Rc::new(RefCell::new(GridViewportInner::new(sample_rate))),
        }
    }

    #[allow(dead_code)]
    pub fn set_header_width(&self, width: Pixels) {
        let mut state = self.inner.borrow_mut();
        state.header_size.width = width;
    }

    pub fn set_header_height(&self, height: Pixels) {
        let mut state = self.inner.borrow_mut();
        state.header_size.height = height;
        state.update_track_height();
        state.update_scroll_offset_y();
    }

    #[allow(dead_code)]
    pub fn set_viewport_padding(&self, padding: Pixels) {
        let mut state = self.inner.borrow_mut();
        if state.viewport_padding != padding {
            state.viewport_padding = padding;
            state.update_frames_per_px();
            state.update_track_width();
            state.update_scroll_offset_x();
            state.update_ticks();
        }
    }

    pub fn set_viewport_width(&self, width: Pixels) {
        let mut state = self.inner.borrow_mut();
        if state.viewport_size.width != width {
            state.viewport_size.width = width;
            state.update_frames_per_px();
            state.update_track_width();
            state.update_scroll_offset_x();
            state.update_ticks();
        }
    }

    pub fn set_viewport_height(&self, height: Pixels) {
        let mut state = self.inner.borrow_mut();
        state.viewport_size.height = height;
        state.update_scroll_offset_y();
    }

    // (slider) 0.0 ..= 100.0
    pub fn set_scale_log(&self, scale: f64) {
        let t = (scale / 100.0).clamp(0.0, 1.0);
        // если макс зум в начале (x=0), больше точности:
        let gamma = 3.0f64; // 1.5..3.0
        let t_shaped = t.powf(gamma);

        // лог-ощущение
        let k = 9.0_f64; // 3..30
        let scale_log_x = ((1.0 + k * t_shaped).ln() / (1.0 + k).ln()).clamp(0.0, 1.0);
        self.set_scale(scale_log_x);
    }

    pub fn set_scale(&self, scale: f64) {
        let mut state = self.inner.borrow_mut();
        let prev_frames_per_px = state.frames_per_px;

        state.scale = scale;
        state.update_frames_per_px();
        state.update_track_width();
        state.update_scroll_offset_x_on_scale(prev_frames_per_px);
        state.update_scroll_offset_x();
        state.update_ticks();
    }

    pub fn set_tracks_count(&self, count: usize) {
        let mut state = self.inner.borrow_mut();
        state.tracks_count = count;
        state.update_track_height();
    }

    pub fn set_total_frames(&self, count: usize) {
        let mut state = self.inner.borrow_mut();
        state.total_frames = count;
        state.update_frames_per_px();
        state.update_track_width();
        state.update_scroll_offset_x();
        state.update_ticks();
    }

    pub fn on_scroll(&self, delta: Point<Pixels>) {
        self.on_scroll_x(delta.x);
        self.on_scroll_y(delta.y);
    }

    pub fn on_scroll_x(&self, delta: Pixels) {
        let mut state = self.inner.borrow_mut();

        let scroll_max = state
            .scroll_size
            .width
            .sub(state.viewport_size.width)
            .max(px(0.0))
            .negate();

        state.scroll_offset.x = (state.scroll_offset.x + delta).min(px(0.0)).max(scroll_max);
        state.update_ticks();
    }

    pub fn on_scroll_y(&self, delta: Pixels) {
        let mut state = self.inner.borrow_mut();

        let scroll_max = state
            .scroll_size
            .height
            .sub(state.viewport_size.height)
            .max(px(0.0))
            .negate();

        state.scroll_offset.y = (state.scroll_offset.y + delta).min(px(0.0)).max(scroll_max);
    }

    pub fn ticks(&self) -> Vec<GridTick> {
        self.inner.borrow().ticks.clone()
    }
}

// scroll_width = padding + track_width + padding
// track_width = total_frames * frames_per_px
struct GridViewportInner {
    sample_rate: SampleRate,
    header_size: Size<Pixels>, // configured (slider)

    viewport_size: Size<Pixels>, // provided on render
    viewport_padding: Pixels,    // configured

    track_size: Size<Pixels>,   // derived, without header
    scroll_size: Size<Pixels>,    // track_size + viewport_padding
    scroll_offset: Point<Pixels>, // state (scroll) negative

    scale: f64, // configured, 0.0 ..= 1.0

    min_frames_per_px: f64, // configured

    tracks_count: usize, // provided (project)
    total_frames: usize, // provided (project)
    frames_per_px: f64,  // derived
    seconds_per_px: f64, // derived

    generator: GridTickGenerator,
    ticks: Vec<GridTick>,
}

impl GridViewportInner {
    fn new(sample_rate: SampleRate) -> Self {
        Self {
            sample_rate,
            header_size: size(px(200.0), px(160.0)),
            viewport_size: size(px(100.0), px(100.0)),
            viewport_padding: px(8.0),
            track_size: size(px(0.0), px(0.0)),
            scroll_size: size(px(0.0), px(0.0)),
            scroll_offset: point(px(0.0), px(0.0)),
            scale: 1.0,
            min_frames_per_px: 0.125,
            tracks_count: 0,
            total_frames: 0,
            frames_per_px: 0.0,
            seconds_per_px: 0.0,
            generator: GridTickGenerator::new(px(60.0)),
            ticks: vec![],
        }
    }

    fn update_track_width(&mut self) {
        self.track_size.width = px((self.total_frames as f64 / self.frames_per_px) as f32);
        self.scroll_size.width =
            (self.track_size.width + self.viewport_padding + self.viewport_padding).max(self.viewport_padding);
    }

    fn update_track_height(&mut self) {
        let base = self.header_size.height * self.tracks_count;
        self.track_size.height = base.max(self.viewport_size.height);
        self.scroll_size.height = self.track_size.height;
    }

    fn update_frames_per_px(&mut self) {
        let sample_rate: f64 = self.sample_rate.into();

        // compute frames_per_px on scale 100%
        let base_width = self.viewport_size.width - self.viewport_padding - self.viewport_padding;
        let base_frames_per_px = self.total_frames as f64 / base_width.as_f64();
        let frames_per_px = base_frames_per_px * self.scale;

        self.frames_per_px = frames_per_px.max(self.min_frames_per_px);
        self.seconds_per_px = self.frames_per_px / sample_rate;
    }

    fn update_scroll_offset_x_on_scale(&mut self, prev_frames_per_px: f64) {
        if self.scroll_offset.x.abs() > self.viewport_padding {
            self.scroll_offset.x = px(
                ((self.scroll_offset.x + self.viewport_padding).as_f64() * prev_frames_per_px / self.frames_per_px)
                    as f32,
            ) - self.viewport_padding;
        }
    }

    fn update_scroll_offset_x(&mut self) {
        let scroll_max = self
            .scroll_size
            .width
            .sub(self.viewport_size.width)
            .max(px(0.0))
            .negate();

        self.scroll_offset.x = self.scroll_offset.x.min(px(0.0)).max(scroll_max);
    }

    fn update_scroll_offset_y(&mut self) {
        let scroll_max = self
            .scroll_size
            .height
            .sub(self.viewport_size.height)
            .max(px(0.0))
            .negate();

        self.scroll_offset.y = self.scroll_offset.y.min(px(0.0)).max(scroll_max);
    }

    fn update_ticks(&mut self) {
        self.ticks = self.generator.generate(self);
    }
}

impl ScrollbarHandle for GridViewport {
    fn offset(&self) -> Point<Pixels> {
        self.inner.borrow().scroll_offset
    }

    fn set_offset(&self, offset: Point<Pixels>) {
        let mut state = self.inner.borrow_mut();
        state.scroll_offset = offset;
        state.update_ticks();
    }

    fn content_size(&self) -> Size<Pixels> {
        self.inner.borrow().scroll_size
    }
}

#[allow(dead_code)]
pub trait GridViewportHandle {
    fn header_size(&self) -> Size<Pixels>;
    fn viewport_size(&self) -> Size<Pixels>;
    fn viewport_padding(&self) -> Pixels;
    fn track_size(&self) -> Size<Pixels>;
    fn scroll_size(&self) -> Size<Pixels>;
    fn scroll_offset(&self) -> Point<Pixels>;
    fn frames_per_px(&self) -> f64;

    fn frame_to_track_offset(&self, frame: usize) -> Pixels;
    fn frame_to_scroll_offset(&self, frame: usize) -> Pixels;
    fn duration_to_track_offset(&self, duration: Duration) -> Pixels;
    fn duration_to_scroll_offset(&self, duration: Duration) -> Pixels;
    fn time_to_track_offset(&self, time: TimeCode) -> Pixels;
    fn time_to_scroll_offset(&self, time: TimeCode) -> Pixels;
    fn track_offset_to_time(&self, track_offset: Pixels) -> TimeCode;
    fn scroll_offset_to_time(&self, scroll_offset: Pixels) -> TimeCode;
    fn track_offset_to_frame(&self, track_offset: Pixels) -> usize;
    fn scroll_offset_to_frame(&self, scroll_offset: Pixels) -> usize;
}

impl GridViewportHandle for GridViewportInner {
    fn header_size(&self) -> Size<Pixels> {
        self.header_size
    }

    fn viewport_size(&self) -> Size<Pixels> {
        self.viewport_size
    }

    fn viewport_padding(&self) -> Pixels {
        self.viewport_padding
    }

    fn track_size(&self) -> Size<Pixels> {
        self.track_size
    }

    fn scroll_size(&self) -> Size<Pixels> {
        self.scroll_size
    }

    fn scroll_offset(&self) -> Point<Pixels> {
        self.scroll_offset
    }

    fn frames_per_px(&self) -> f64 {
        self.frames_per_px
    }

    fn frame_to_track_offset(&self, frame: usize) -> Pixels {
        let offset = frame as f64 / self.frames_per_px;
        px(offset as f32)
    }

    fn frame_to_scroll_offset(&self, frame: usize) -> Pixels {
        let track_offset = frame as f64 / self.frames_per_px;
        self.scroll_offset.x + self.viewport_padding + px(track_offset as f32)
    }

    fn duration_to_track_offset(&self, duration: Duration) -> Pixels {
        let seconds = duration.to_millis() as f64 / 1000f64;
        px((seconds / self.seconds_per_px) as f32)
    }

    fn duration_to_scroll_offset(&self, duration: Duration) -> Pixels {
        let seconds = duration.to_millis() as f64 / 1000f64;
        let track_offset = seconds / self.seconds_per_px;
        self.scroll_offset.x + self.viewport_padding + px(track_offset as f32)
    }

    fn time_to_track_offset(&self, time: TimeCode) -> Pixels {
        let seconds = time.to_millis() as f64 / 1000f64;
        px((seconds / self.seconds_per_px) as f32)
    }

    fn time_to_scroll_offset(&self, time: TimeCode) -> Pixels {
        let seconds = time.to_millis() as f64 / 1000f64;
        let track_offset = seconds / self.seconds_per_px;
        self.scroll_offset.x + self.viewport_padding + px(track_offset as f32)
    }

    fn track_offset_to_time(&self, track_offset: Pixels) -> TimeCode {
        let track_offset = track_offset.to_f64();
        let millis = track_offset * self.seconds_per_px * 1000f64;
        TimeCode::from_millis(millis as u64)
    }

    fn scroll_offset_to_time(&self, scroll_offset: Pixels) -> TimeCode {
        let track_offset = scroll_offset - self.scroll_offset.x - self.viewport_padding;
        let track_offset = track_offset.to_f64();
        let millis = track_offset * self.seconds_per_px * 1000f64;
        TimeCode::from_millis(millis as u64)
    }

    fn track_offset_to_frame(&self, track_offset: Pixels) -> usize {
        let track_offset = track_offset.to_f64();
        let frame = track_offset * self.frames_per_px;
        frame.floor() as usize
    }

    fn scroll_offset_to_frame(&self, scroll_offset: Pixels) -> usize {
        let track_offset = scroll_offset - self.scroll_offset.x - self.viewport_padding;
        let track_offset = track_offset.to_f64();
        let frame = track_offset * self.frames_per_px;
        frame.floor() as usize
    }
}

impl GridViewportHandle for GridViewport {
    fn header_size(&self) -> Size<Pixels> {
        self.inner.borrow().header_size
    }

    fn viewport_size(&self) -> Size<Pixels> {
        self.inner.borrow().viewport_size
    }

    fn viewport_padding(&self) -> Pixels {
        self.inner.borrow().viewport_padding
    }

    fn track_size(&self) -> Size<Pixels> {
        self.inner.borrow().track_size
    }

    fn scroll_size(&self) -> Size<Pixels> {
        self.inner.borrow().scroll_size
    }

    fn scroll_offset(&self) -> Point<Pixels> {
        self.inner.borrow().scroll_offset
    }

    fn frames_per_px(&self) -> f64 {
        self.inner.borrow().frames_per_px
    }

    fn frame_to_track_offset(&self, frame: usize) -> Pixels {
        self.inner.borrow().frame_to_track_offset(frame)
    }

    fn frame_to_scroll_offset(&self, frame: usize) -> Pixels {
        self.inner.borrow().frame_to_scroll_offset(frame)
    }

    fn duration_to_track_offset(&self, duration: Duration) -> Pixels {
        self.inner.borrow().duration_to_track_offset(duration)
    }

    fn duration_to_scroll_offset(&self, duration: Duration) -> Pixels {
        self.inner.borrow().duration_to_scroll_offset(duration)
    }

    fn time_to_track_offset(&self, time: TimeCode) -> Pixels {
        self.inner.borrow().time_to_track_offset(time)
    }

    fn time_to_scroll_offset(&self, time: TimeCode) -> Pixels {
        self.inner.borrow().time_to_scroll_offset(time)
    }

    fn track_offset_to_time(&self, track_offset: Pixels) -> TimeCode {
        self.inner.borrow().track_offset_to_time(track_offset)
    }

    fn scroll_offset_to_time(&self, scroll_offset: Pixels) -> TimeCode {
        self.inner.borrow().scroll_offset_to_time(scroll_offset)
    }

    fn track_offset_to_frame(&self, track_offset: Pixels) -> usize {
        self.inner.borrow().track_offset_to_frame(track_offset)
    }

    fn scroll_offset_to_frame(&self, scroll_offset: Pixels) -> usize {
        self.inner.borrow().scroll_offset_to_frame(scroll_offset)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const SR: SampleRate = SampleRate::Hz44100;

    #[test]
    fn test_base() {
        let view = GridViewport::new(SR);
        view.set_viewport_padding(px(0.0));
        view.set_viewport_width(px(100.0));
        view.set_total_frames(SR * 10);

        // assert_eq!(view.get_start_frame(), 0);
        assert_eq!(view.track_size().width, px(100.0));
        assert_eq!(view.scroll_size().width, px(100.0));

        assert_eq!(view.frame_to_track_offset(0), px(0.0));
        assert_eq!(view.frame_to_track_offset(SR * 1), px(10.0));
        assert_eq!(view.frame_to_track_offset(SR * 10), px(100.0));

        assert_eq!(view.frame_to_scroll_offset(0), px(0.0));
        assert_eq!(view.frame_to_scroll_offset(SR * 1), px(10.0));
        assert_eq!(view.frame_to_scroll_offset(SR * 10), px(100.0));

        assert_eq!(view.duration_to_track_offset(Duration::Seconds(1)), px(10.0));
        assert_eq!(view.duration_to_track_offset(Duration::Seconds(10)), px(100.0));
        assert_eq!(view.duration_to_scroll_offset(Duration::Seconds(1)), px(10.0));
        assert_eq!(view.duration_to_scroll_offset(Duration::Seconds(10)), px(100.0));

        assert_eq!(view.time_to_track_offset(TimeCode::from_seconds(1)), px(10.0));
        assert_eq!(view.time_to_track_offset(TimeCode::from_seconds(10)), px(100.0));
        assert_eq!(view.time_to_scroll_offset(TimeCode::from_seconds(1)), px(10.0));
        assert_eq!(view.time_to_scroll_offset(TimeCode::from_seconds(10)), px(100.0));

        assert_eq!(view.track_offset_to_time(px(0.0)), TimeCode::from_seconds(0));
        assert_eq!(view.track_offset_to_time(px(10.0)), TimeCode::from_seconds(1));
        assert_eq!(view.track_offset_to_time(px(50.0)), TimeCode::from_seconds(5));
        assert_eq!(view.track_offset_to_time(px(100.0)), TimeCode::from_seconds(10));
        assert_eq!(view.scroll_offset_to_time(px(0.0)), TimeCode::from_seconds(0));
        assert_eq!(view.scroll_offset_to_time(px(10.0)), TimeCode::from_seconds(1));
        assert_eq!(view.scroll_offset_to_time(px(50.0)), TimeCode::from_seconds(5));
        assert_eq!(view.scroll_offset_to_time(px(100.0)), TimeCode::from_seconds(10));

        assert_eq!(view.track_offset_to_frame(px(0.0)), SR * 0);
        assert_eq!(view.track_offset_to_frame(px(10.0)), SR * 1);
        assert_eq!(view.track_offset_to_frame(px(50.0)), SR * 5);
        assert_eq!(view.track_offset_to_frame(px(100.0)), SR * 10);
        assert_eq!(view.scroll_offset_to_frame(px(0.0)), SR * 0);
        assert_eq!(view.scroll_offset_to_frame(px(10.0)), SR * 1);
        assert_eq!(view.scroll_offset_to_frame(px(50.0)), SR * 5);
        assert_eq!(view.scroll_offset_to_frame(px(100.0)), SR * 10);
    }

    #[test]
    fn test_padding() {
        let view = GridViewport::new(SR);
        view.set_viewport_padding(px(10.0));
        view.set_viewport_width(px(120.0));
        view.set_total_frames(SR * 10);

        // assert_eq!(view.get_start_frame(), 0);
        assert_eq!(view.track_size().width, px(100.0));
        assert_eq!(view.scroll_size().width, px(120.0));

        assert_eq!(view.frame_to_track_offset(0), px(0.0));
        assert_eq!(view.frame_to_track_offset(SR * 1), px(10.0));
        assert_eq!(view.frame_to_track_offset(SR * 10), px(100.0));

        assert_eq!(view.frame_to_scroll_offset(0), px(10.0));
        assert_eq!(view.frame_to_scroll_offset(SR * 1), px(20.0));
        assert_eq!(view.frame_to_scroll_offset(SR * 10), px(110.0));

        assert_eq!(view.duration_to_track_offset(Duration::Seconds(1)), px(10.0));
        assert_eq!(view.duration_to_track_offset(Duration::Seconds(10)), px(100.0));
        assert_eq!(view.duration_to_scroll_offset(Duration::Seconds(1)), px(20.0));
        assert_eq!(view.duration_to_scroll_offset(Duration::Seconds(10)), px(110.0));

        assert_eq!(view.time_to_track_offset(TimeCode::from_seconds(1)), px(10.0));
        assert_eq!(view.time_to_track_offset(TimeCode::from_seconds(10)), px(100.0));
        assert_eq!(view.time_to_scroll_offset(TimeCode::from_seconds(1)), px(20.0));
        assert_eq!(view.time_to_scroll_offset(TimeCode::from_seconds(10)), px(110.0));

        assert_eq!(view.track_offset_to_time(px(0.0)), TimeCode::from_seconds(0));
        assert_eq!(view.track_offset_to_time(px(10.0)), TimeCode::from_seconds(1));
        assert_eq!(view.track_offset_to_time(px(50.0)), TimeCode::from_seconds(5));
        assert_eq!(view.track_offset_to_time(px(100.0)), TimeCode::from_seconds(10));

        assert_eq!(view.scroll_offset_to_time(px(0.0)), TimeCode::from_seconds(0));
        assert_eq!(view.scroll_offset_to_time(px(10.0)), TimeCode::from_seconds(0));
        assert_eq!(view.scroll_offset_to_time(px(20.0)), TimeCode::from_seconds(1));
        assert_eq!(view.scroll_offset_to_time(px(60.0)), TimeCode::from_seconds(5));
        assert_eq!(view.scroll_offset_to_time(px(110.0)), TimeCode::from_seconds(10));

        assert_eq!(view.track_offset_to_frame(px(0.0)), SR * 0);
        assert_eq!(view.track_offset_to_frame(px(10.0)), SR * 1);
        assert_eq!(view.track_offset_to_frame(px(50.0)), SR * 5);
        assert_eq!(view.track_offset_to_frame(px(100.0)), SR * 10);
        assert_eq!(view.scroll_offset_to_frame(px(10.0)), SR * 0);
        assert_eq!(view.scroll_offset_to_frame(px(20.0)), SR * 1);
        assert_eq!(view.scroll_offset_to_frame(px(60.0)), SR * 5);
        assert_eq!(view.scroll_offset_to_frame(px(110.0)), SR * 10);
    }

    #[test]
    fn test_scale() {
        let view = GridViewport::new(SR);
        view.set_viewport_padding(px(0.0));
        view.set_viewport_width(px(100.0));
        view.set_total_frames(SR * 10);
        view.set_scale(0.1); // 1 SR

        assert_eq!(view.track_size().width, px(1000.0));
        assert_eq!(view.scroll_size().width, px(1000.0));
        assert_eq!(view.frames_per_px(), SR * 0.01);

        assert_eq!(view.frame_to_track_offset(0), px(0.0));
        assert_eq!(view.frame_to_track_offset(SR * 1), px(100.0));
        assert_eq!(view.frame_to_track_offset(SR * 10), px(1000.0));

        assert_eq!(view.frame_to_scroll_offset(0), px(0.0));
        assert_eq!(view.frame_to_scroll_offset(SR * 1), px(100.0));
        assert_eq!(view.frame_to_scroll_offset(SR * 10), px(1000.0));

        assert_eq!(view.duration_to_track_offset(Duration::Seconds(1)), px(100.0));
        assert_eq!(view.duration_to_track_offset(Duration::Seconds(10)), px(1000.0));
        assert_eq!(view.duration_to_scroll_offset(Duration::Seconds(1)), px(100.0));
        assert_eq!(view.duration_to_scroll_offset(Duration::Seconds(10)), px(1000.0));

        assert_eq!(view.time_to_track_offset(TimeCode::from_seconds(1)), px(100.0));
        assert_eq!(view.time_to_track_offset(TimeCode::from_seconds(10)), px(1000.0));
        assert_eq!(view.time_to_scroll_offset(TimeCode::from_seconds(1)), px(100.0));
        assert_eq!(view.time_to_scroll_offset(TimeCode::from_seconds(10)), px(1000.0));

        assert_eq!(view.track_offset_to_time(px(0.0)), TimeCode::from_seconds(0));
        assert_eq!(view.track_offset_to_time(px(100.0)), TimeCode::from_seconds(1));
        assert_eq!(view.track_offset_to_time(px(500.0)), TimeCode::from_seconds(5));
        assert_eq!(view.track_offset_to_time(px(1000.0)), TimeCode::from_seconds(10));
        assert_eq!(view.scroll_offset_to_time(px(0.0)), TimeCode::from_seconds(0));
        assert_eq!(view.scroll_offset_to_time(px(100.0)), TimeCode::from_seconds(1));
        assert_eq!(view.scroll_offset_to_time(px(500.0)), TimeCode::from_seconds(5));
        assert_eq!(view.scroll_offset_to_time(px(1000.0)), TimeCode::from_seconds(10));

        assert_eq!(view.track_offset_to_frame(px(0.0)), SR * 0);
        assert_eq!(view.track_offset_to_frame(px(10.0)), (SR * 0.1) as usize);
        assert_eq!(view.track_offset_to_frame(px(50.0)), (SR * 0.5) as usize);
        assert_eq!(view.track_offset_to_frame(px(100.0)), SR * 1);
        assert_eq!(view.scroll_offset_to_frame(px(0.0)), SR * 0);
        assert_eq!(view.scroll_offset_to_frame(px(10.0)), (SR * 0.1) as usize);
        assert_eq!(view.scroll_offset_to_frame(px(50.0)), (SR * 0.5) as usize);
        assert_eq!(view.scroll_offset_to_frame(px(100.0)), SR * 1);
    }

    #[test]
    fn test_scale_and_padding() {
        let view = GridViewport::new(SR);
        view.set_viewport_padding(px(10.0));
        view.set_viewport_width(px(120.0));
        view.set_total_frames(SR * 10);
        view.set_scale(0.1); // 1 SR

        assert_eq!(view.track_size().width, px(1000.0));
        assert_eq!(view.scroll_size().width, px(1020.0));
        assert_eq!(view.frames_per_px(), SR * 0.01);

        assert_eq!(view.frame_to_track_offset(0), px(0.0));
        assert_eq!(view.frame_to_track_offset(SR * 1), px(100.0));
        assert_eq!(view.frame_to_track_offset(SR * 10), px(1000.0));

        assert_eq!(view.frame_to_scroll_offset(0), px(10.0));
        assert_eq!(view.frame_to_scroll_offset(SR * 1), px(110.0));
        assert_eq!(view.frame_to_scroll_offset(SR * 10), px(1010.0));

        assert_eq!(view.duration_to_track_offset(Duration::Seconds(1)), px(100.0));
        assert_eq!(view.duration_to_track_offset(Duration::Seconds(10)), px(1000.0));
        assert_eq!(view.duration_to_scroll_offset(Duration::Seconds(1)), px(110.0));
        assert_eq!(view.duration_to_scroll_offset(Duration::Seconds(10)), px(1010.0));

        assert_eq!(view.time_to_track_offset(TimeCode::from_seconds(1)), px(100.0));
        assert_eq!(view.time_to_track_offset(TimeCode::from_seconds(10)), px(1000.0));
        assert_eq!(view.time_to_scroll_offset(TimeCode::from_seconds(1)), px(110.0));
        assert_eq!(view.time_to_scroll_offset(TimeCode::from_seconds(10)), px(1010.0));

        assert_eq!(view.track_offset_to_time(px(0.0)), TimeCode::from_seconds(0));
        assert_eq!(view.track_offset_to_time(px(100.0)), TimeCode::from_seconds(1));
        assert_eq!(view.track_offset_to_time(px(500.0)), TimeCode::from_seconds(5));
        assert_eq!(view.track_offset_to_time(px(1000.0)), TimeCode::from_seconds(10));
        assert_eq!(view.scroll_offset_to_time(px(0.0)), TimeCode::from_seconds(0));
        assert_eq!(view.scroll_offset_to_time(px(10.0)), TimeCode::from_seconds(0));
        assert_eq!(view.scroll_offset_to_time(px(110.0)), TimeCode::from_seconds(1));
        assert_eq!(view.scroll_offset_to_time(px(510.0)), TimeCode::from_seconds(5));
        assert_eq!(view.scroll_offset_to_time(px(1010.0)), TimeCode::from_seconds(10));

        assert_eq!(view.track_offset_to_frame(px(0.0)), SR * 0);
        assert_eq!(view.track_offset_to_frame(px(10.0)), (SR * 0.1) as usize);
        assert_eq!(view.track_offset_to_frame(px(50.0)), (SR * 0.5) as usize);
        assert_eq!(view.track_offset_to_frame(px(100.0)), SR * 1);
        assert_eq!(view.scroll_offset_to_frame(px(10.0)), SR * 0);
        assert_eq!(view.scroll_offset_to_frame(px(20.0)), (SR * 0.1) as usize);
        assert_eq!(view.scroll_offset_to_frame(px(60.0)), (SR * 0.5) as usize);
        assert_eq!(view.scroll_offset_to_frame(px(110.0)), SR * 1);
    }

    #[test]
    fn test_scale_and_scroll_to_middle() {
        let view = GridViewport::new(SR);
        view.set_viewport_padding(px(0.0));
        view.set_viewport_width(px(100.0));
        view.set_total_frames(SR * 10);
        view.set_scale(0.1); // 1 SR
        view.set_offset(point(px(-500.0), px(0.0))); // middle of track

        assert_eq!(view.track_size().width, px(1000.0));
        assert_eq!(view.scroll_size().width, px(1000.0));
        assert_eq!(view.frames_per_px(), SR * 0.01);

        assert_eq!(view.frame_to_track_offset(0), px(0.0));
        assert_eq!(view.frame_to_track_offset(SR * 1), px(100.0));
        assert_eq!(view.frame_to_track_offset(SR * 10), px(1000.0));

        assert_eq!(view.frame_to_scroll_offset(0), px(-500.0));
        assert_eq!(view.frame_to_scroll_offset(SR * 1), px(-400.0));
        assert_eq!(view.frame_to_scroll_offset(SR * 5), px(0.0));
        assert_eq!(view.frame_to_scroll_offset(SR * 10), px(500.0));

        assert_eq!(view.duration_to_track_offset(Duration::Seconds(1)), px(100.0));
        assert_eq!(view.duration_to_track_offset(Duration::Seconds(10)), px(1000.0));
        assert_eq!(view.duration_to_scroll_offset(Duration::Seconds(1)), px(-400.0));
        assert_eq!(view.duration_to_scroll_offset(Duration::Seconds(10)), px(500.0));

        assert_eq!(view.time_to_track_offset(TimeCode::from_seconds(1)), px(100.0));
        assert_eq!(view.time_to_track_offset(TimeCode::from_seconds(10)), px(1000.0));
        assert_eq!(view.time_to_scroll_offset(TimeCode::from_seconds(1)), px(-400.0));
        assert_eq!(view.time_to_scroll_offset(TimeCode::from_seconds(10)), px(500.0));

        assert_eq!(view.track_offset_to_time(px(0.0)), TimeCode::from_seconds(0));
        assert_eq!(view.track_offset_to_time(px(100.0)), TimeCode::from_seconds(1));
        assert_eq!(view.track_offset_to_time(px(500.0)), TimeCode::from_seconds(5));
        assert_eq!(view.track_offset_to_time(px(1000.0)), TimeCode::from_seconds(10));
        assert_eq!(view.scroll_offset_to_time(px(-500.0)), TimeCode::from_seconds(0));
        assert_eq!(view.scroll_offset_to_time(px(-400.0)), TimeCode::from_seconds(1));
        assert_eq!(view.scroll_offset_to_time(px(0.0)), TimeCode::from_seconds(5));
        assert_eq!(view.scroll_offset_to_time(px(500.0)), TimeCode::from_seconds(10));

        assert_eq!(view.track_offset_to_frame(px(0.0)), SR * 0);
        assert_eq!(view.track_offset_to_frame(px(100.0)), SR * 1);
        assert_eq!(view.track_offset_to_frame(px(500.0)), SR * 5);
        assert_eq!(view.track_offset_to_frame(px(1000.0)), SR * 10);
        assert_eq!(view.scroll_offset_to_frame(px(-500.0)), SR * 0);
        assert_eq!(view.scroll_offset_to_frame(px(-400.0)), SR * 1);
        assert_eq!(view.scroll_offset_to_frame(px(0.0)), SR * 5);
        assert_eq!(view.scroll_offset_to_frame(px(500.0)), SR * 10);
    }

    #[test]
    fn test_scale_and_scroll_to_padding() {
        let view = GridViewport::new(SR);
        view.set_viewport_padding(px(10.0));
        view.set_viewport_width(px(120.0));
        view.set_total_frames(SR * 10);
        view.set_scale(0.1); // 1 SR
        view.set_offset(point(px(-10.0), px(0.0))); // padding

        assert_eq!(view.track_size().width, px(1000.0));
        assert_eq!(view.scroll_size().width, px(1020.0));
        assert_eq!(view.frames_per_px(), SR * 0.01);

        assert_eq!(view.frame_to_track_offset(0), px(0.0));
        assert_eq!(view.frame_to_track_offset(SR * 1), px(100.0));
        assert_eq!(view.frame_to_track_offset(SR * 10), px(1000.0));

        assert_eq!(view.frame_to_scroll_offset(0), px(0.0));
        assert_eq!(view.frame_to_scroll_offset(SR * 1), px(100.0));
        assert_eq!(view.frame_to_scroll_offset(SR * 5), px(500.0));
        assert_eq!(view.frame_to_scroll_offset(SR * 10), px(1000.0));

        assert_eq!(view.duration_to_track_offset(Duration::Seconds(1)), px(100.0));
        assert_eq!(view.duration_to_track_offset(Duration::Seconds(10)), px(1000.0));
        assert_eq!(view.duration_to_scroll_offset(Duration::Seconds(1)), px(100.0));
        assert_eq!(view.duration_to_scroll_offset(Duration::Seconds(10)), px(1000.0));

        assert_eq!(view.time_to_track_offset(TimeCode::from_seconds(1)), px(100.0));
        assert_eq!(view.time_to_track_offset(TimeCode::from_seconds(10)), px(1000.0));
        assert_eq!(view.time_to_scroll_offset(TimeCode::from_seconds(1)), px(100.0));
        assert_eq!(view.time_to_scroll_offset(TimeCode::from_seconds(10)), px(1000.0));

        assert_eq!(view.track_offset_to_time(px(0.0)), TimeCode::from_seconds(0));
        assert_eq!(view.track_offset_to_time(px(100.0)), TimeCode::from_seconds(1));
        assert_eq!(view.track_offset_to_time(px(500.0)), TimeCode::from_seconds(5));
        assert_eq!(view.track_offset_to_time(px(1000.0)), TimeCode::from_seconds(10));
        assert_eq!(view.scroll_offset_to_time(px(0.0)), TimeCode::from_seconds(0));
        assert_eq!(view.scroll_offset_to_time(px(100.0)), TimeCode::from_seconds(1));
        assert_eq!(view.scroll_offset_to_time(px(500.0)), TimeCode::from_seconds(5));
        assert_eq!(view.scroll_offset_to_time(px(1000.0)), TimeCode::from_seconds(10));

        assert_eq!(view.track_offset_to_frame(px(0.0)), SR * 0);
        assert_eq!(view.track_offset_to_frame(px(100.0)), SR * 1);
        assert_eq!(view.track_offset_to_frame(px(500.0)), SR * 5);
        assert_eq!(view.track_offset_to_frame(px(1000.0)), SR * 10);
        assert_eq!(view.scroll_offset_to_frame(px(0.0)), SR * 0);
        assert_eq!(view.scroll_offset_to_frame(px(100.0)), SR * 1);
        assert_eq!(view.scroll_offset_to_frame(px(500.0)), SR * 5);
        assert_eq!(view.scroll_offset_to_frame(px(1000.0)), SR * 10);
    }

    #[test]
    fn test_scale_and_padding_and_scroll_to_middle() {
        let view = GridViewport::new(SR);
        view.set_viewport_padding(px(10.0));
        view.set_viewport_width(px(120.0));
        view.set_total_frames(SR * 10);
        view.set_scale(0.1); // 1 SR
        view.set_offset(point(px(-510.0), px(0.0))); // padding + track middle

        assert_eq!(view.track_size().width, px(1000.0));
        assert_eq!(view.scroll_size().width, px(1020.0));
        assert_eq!(view.frames_per_px(), SR * 0.01);

        assert_eq!(view.frame_to_track_offset(0), px(0.0));
        assert_eq!(view.frame_to_track_offset(SR * 1), px(100.0));
        assert_eq!(view.frame_to_track_offset(SR * 10), px(1000.0));

        assert_eq!(view.frame_to_scroll_offset(0), px(-500.0)); // == scroll_offset + padding
        assert_eq!(view.frame_to_scroll_offset(SR * 1), px(-400.0));
        assert_eq!(view.frame_to_scroll_offset(SR * 5), px(0.0));
        assert_eq!(view.frame_to_scroll_offset(SR * 10), px(500.0));

        assert_eq!(view.duration_to_track_offset(Duration::Seconds(1)), px(100.0));
        assert_eq!(view.duration_to_track_offset(Duration::Seconds(10)), px(1000.0));
        assert_eq!(view.duration_to_scroll_offset(Duration::Seconds(1)), px(-400.0));
        assert_eq!(view.duration_to_scroll_offset(Duration::Seconds(10)), px(500.0));

        assert_eq!(view.time_to_track_offset(TimeCode::from_seconds(1)), px(100.0));
        assert_eq!(view.time_to_track_offset(TimeCode::from_seconds(10)), px(1000.0));
        assert_eq!(view.time_to_scroll_offset(TimeCode::from_seconds(1)), px(-400.0));
        assert_eq!(view.time_to_scroll_offset(TimeCode::from_seconds(10)), px(500.0));

        assert_eq!(view.track_offset_to_time(px(0.0)), TimeCode::from_seconds(0));
        assert_eq!(view.track_offset_to_time(px(100.0)), TimeCode::from_seconds(1));
        assert_eq!(view.track_offset_to_time(px(500.0)), TimeCode::from_seconds(5));
        assert_eq!(view.track_offset_to_time(px(1000.0)), TimeCode::from_seconds(10));
        assert_eq!(view.scroll_offset_to_time(px(-500.0)), TimeCode::from_seconds(0));
        assert_eq!(view.scroll_offset_to_time(px(-400.0)), TimeCode::from_seconds(1));
        assert_eq!(view.scroll_offset_to_time(px(0.0)), TimeCode::from_seconds(5));
        assert_eq!(view.scroll_offset_to_time(px(500.0)), TimeCode::from_seconds(10));

        assert_eq!(view.track_offset_to_frame(px(0.0)), SR * 0);
        assert_eq!(view.track_offset_to_frame(px(100.0)), SR * 1);
        assert_eq!(view.track_offset_to_frame(px(500.0)), SR * 5);
        assert_eq!(view.track_offset_to_frame(px(1000.0)), SR * 10);
        assert_eq!(view.scroll_offset_to_frame(px(-500.0)), SR * 0);
        assert_eq!(view.scroll_offset_to_frame(px(-400.0)), SR * 1);
        assert_eq!(view.scroll_offset_to_frame(px(0.0)), SR * 5);
        assert_eq!(view.scroll_offset_to_frame(px(500.0)), SR * 10);
    }

    #[test]
    fn test_scale_after_scroll_to_middle() {
        let view = GridViewport::new(SR);
        view.set_viewport_padding(px(0.0));
        view.set_viewport_width(px(100.0));
        view.set_total_frames(SR * 10);
        view.set_scale(0.01); // 0.1 SR
        view.set_offset(point(px(-5000.0), px(0.0))); // track middle
        view.set_scale(0.1); // 1 SR

        assert_eq!(view.track_size().width, px(1000.0));
        assert_eq!(view.scroll_size().width, px(1000.0));
        assert_eq!(view.frames_per_px(), SR * 0.01);

        assert_eq!(view.frame_to_track_offset(0), px(0.0));
        assert_eq!(view.frame_to_track_offset(SR * 1), px(100.0));
        assert_eq!(view.frame_to_track_offset(SR * 10), px(1000.0));

        assert_eq!(view.frame_to_scroll_offset(0), px(-500.0)); // == scroll_offset + padding
        assert_eq!(view.frame_to_scroll_offset(SR * 1), px(-400.0));
        assert_eq!(view.frame_to_scroll_offset(SR * 5), px(0.0));
        assert_eq!(view.frame_to_scroll_offset(SR * 10), px(500.0));

        assert_eq!(view.duration_to_track_offset(Duration::Seconds(1)), px(100.0));
        assert_eq!(view.duration_to_track_offset(Duration::Seconds(10)), px(1000.0));
        assert_eq!(view.duration_to_scroll_offset(Duration::Seconds(1)), px(-400.0));
        assert_eq!(view.duration_to_scroll_offset(Duration::Seconds(10)), px(500.0));

        assert_eq!(view.time_to_track_offset(TimeCode::from_seconds(1)), px(100.0));
        assert_eq!(view.time_to_track_offset(TimeCode::from_seconds(10)), px(1000.0));
        assert_eq!(view.time_to_scroll_offset(TimeCode::from_seconds(1)), px(-400.0));
        assert_eq!(view.time_to_scroll_offset(TimeCode::from_seconds(10)), px(500.0));

        assert_eq!(view.track_offset_to_time(px(0.0)), TimeCode::from_seconds(0));
        assert_eq!(view.track_offset_to_time(px(100.0)), TimeCode::from_seconds(1));
        assert_eq!(view.track_offset_to_time(px(500.0)), TimeCode::from_seconds(5));
        assert_eq!(view.track_offset_to_time(px(1000.0)), TimeCode::from_seconds(10));
        assert_eq!(view.scroll_offset_to_time(px(-500.0)), TimeCode::from_seconds(0));
        assert_eq!(view.scroll_offset_to_time(px(-400.0)), TimeCode::from_seconds(1));
        assert_eq!(view.scroll_offset_to_time(px(0.0)), TimeCode::from_seconds(5));
        assert_eq!(view.scroll_offset_to_time(px(500.0)), TimeCode::from_seconds(10));

        assert_eq!(view.track_offset_to_frame(px(0.0)), SR * 0);
        assert_eq!(view.track_offset_to_frame(px(100.0)), SR * 1);
        assert_eq!(view.track_offset_to_frame(px(500.0)), SR * 5);
        assert_eq!(view.track_offset_to_frame(px(1000.0)), SR * 10);
        assert_eq!(view.scroll_offset_to_frame(px(-500.0)), SR * 0);
        assert_eq!(view.scroll_offset_to_frame(px(-400.0)), SR * 1);
        assert_eq!(view.scroll_offset_to_frame(px(0.0)), SR * 5);
        assert_eq!(view.scroll_offset_to_frame(px(500.0)), SR * 10);
    }

    #[test]
    fn test_scale_after_scroll_to_middle_and_padding() {
        let view = GridViewport::new(SR);
        view.set_viewport_padding(px(10.0));
        view.set_viewport_width(px(120.0));
        view.set_total_frames(SR * 10);
        view.set_scale(0.01); // 0.1 SR
        view.set_offset(point(px(-5010.0), px(0.0))); // padding + track middle
        view.set_scale(0.1); // 1 SR

        assert_eq!(view.track_size().width, px(1000.0));
        assert_eq!(view.scroll_size().width, px(1020.0));
        assert_eq!(view.frames_per_px(), SR * 0.01);

        assert_eq!(view.frame_to_track_offset(0), px(0.0));
        assert_eq!(view.frame_to_track_offset(SR * 1), px(100.0));
        assert_eq!(view.frame_to_track_offset(SR * 10), px(1000.0));

        assert_eq!(view.frame_to_scroll_offset(0), px(-500.0)); // == scroll_offset + padding
        assert_eq!(view.frame_to_scroll_offset(SR * 1), px(-400.0));
        assert_eq!(view.frame_to_scroll_offset(SR * 5), px(0.0));
        assert_eq!(view.frame_to_scroll_offset(SR * 10), px(500.0));

        assert_eq!(view.duration_to_track_offset(Duration::Seconds(1)), px(100.0));
        assert_eq!(view.duration_to_track_offset(Duration::Seconds(10)), px(1000.0));
        assert_eq!(view.duration_to_scroll_offset(Duration::Seconds(1)), px(-400.0));
        assert_eq!(view.duration_to_scroll_offset(Duration::Seconds(10)), px(500.0));

        assert_eq!(view.time_to_track_offset(TimeCode::from_seconds(1)), px(100.0));
        assert_eq!(view.time_to_track_offset(TimeCode::from_seconds(10)), px(1000.0));
        assert_eq!(view.time_to_scroll_offset(TimeCode::from_seconds(1)), px(-400.0));
        assert_eq!(view.time_to_scroll_offset(TimeCode::from_seconds(10)), px(500.0));

        assert_eq!(view.track_offset_to_time(px(0.0)), TimeCode::from_seconds(0));
        assert_eq!(view.track_offset_to_time(px(100.0)), TimeCode::from_seconds(1));
        assert_eq!(view.track_offset_to_time(px(500.0)), TimeCode::from_seconds(5));
        assert_eq!(view.track_offset_to_time(px(1000.0)), TimeCode::from_seconds(10));
        assert_eq!(view.scroll_offset_to_time(px(-500.0)), TimeCode::from_seconds(0));
        assert_eq!(view.scroll_offset_to_time(px(-400.0)), TimeCode::from_seconds(1));
        assert_eq!(view.scroll_offset_to_time(px(0.0)), TimeCode::from_seconds(5));
        assert_eq!(view.scroll_offset_to_time(px(500.0)), TimeCode::from_seconds(10));

        assert_eq!(view.track_offset_to_frame(px(0.0)), SR * 0);
        assert_eq!(view.track_offset_to_frame(px(100.0)), SR * 1);
        assert_eq!(view.track_offset_to_frame(px(500.0)), SR * 5);
        assert_eq!(view.track_offset_to_frame(px(1000.0)), SR * 10);
        assert_eq!(view.scroll_offset_to_frame(px(-500.0)), SR * 0);
        assert_eq!(view.scroll_offset_to_frame(px(-400.0)), SR * 1);
        assert_eq!(view.scroll_offset_to_frame(px(0.0)), SR * 5);
        assert_eq!(view.scroll_offset_to_frame(px(500.0)), SR * 10);
    }
}
