use crate::components::grid::GridViewport;
use crate::components::track::Track;
use crate::components::waveform::WaveClip;
use crate::time::SampleRate;
use gpui::{AppContext, Context, Entity, Subscription, px};
use gpui_component::slider::{SliderEvent, SliderScale, SliderState};
use log::info;
use std::sync::{Arc, Mutex};

pub struct GridState {
    tracks: Arc<Mutex<Vec<Track>>>,
    pub viewport: GridViewport,
    pub x_slider: Entity<SliderState>,
    pub y_slider: Entity<SliderState>,
    _subscriptions: Vec<Subscription>,
}

impl GridState {
    pub fn new(sample_rate: SampleRate, cx: &mut Context<Self>) -> Self {
        let x_slider = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(100.0)
                .step(0.01)
                .default_value(100.0)
                .scale(SliderScale::Linear)
        });
        let x_sub = cx.subscribe(&x_slider, |this, _, event, cx| match event {
            SliderEvent::Change(_) => {
                let scale = this.x_slider.read(cx).value().start() as f64;
                this.viewport.set_scale_log(scale);
                cx.notify();
            }
        });

        let y_slider = cx.new(|_| {
            SliderState::new()
                .min(80.0)
                .max(800.0)
                .step(1.0)
                .default_value(160.0)
                .scale(SliderScale::Linear)
        });
        let y_sub = cx.subscribe(&y_slider, |this, _, event, cx| match event {
            SliderEvent::Change(_) => {
                let value = this.y_slider.read(cx).value().start();
                this.viewport.set_header_height(px(value));
                cx.notify();
            }
        });

        let state = Self {
            tracks: Arc::new(Mutex::new(random_tracks())),
            viewport: GridViewport::new(sample_rate),
            x_slider,
            y_slider,
            _subscriptions: vec![x_sub, y_sub],
        };
        state.update_viewport();
        state
    }

    pub fn tracks(&self) -> Vec<Track> {
        self.tracks.lock().unwrap().clone()
    }

    pub fn update_tracks(&self, updater: impl Fn(&mut Vec<Track>)) {
        info!("update_tracks get lock");
        let mut tracks = self.tracks.lock().unwrap();
        info!("update_tracks call function");
        updater(&mut tracks);
        drop(tracks);
        info!("update_tracks update viewport");
        self.update_viewport();
        info!("update_tracks complete");
    }

    fn tracks_frames(&self) -> usize {
        self.tracks
            .lock()
            .unwrap()
            .iter()
            .fold(0, |acc, track| acc.max(track.frames()))
    }

    fn update_viewport(&self) {
        let tracks_count = self.tracks.lock().unwrap().len();
        self.viewport.set_tracks_count(tracks_count);
        self.viewport.set_total_frames(self.tracks_frames());
    }
}

fn random_tracks() -> Vec<Track> {
    let clip = WaveClip::open("test_signal.wav").unwrap();

    (0..100)
        .map(|t| {
            let title = format!("test {t}");
            let track = Track::new(title);
            track.add_clip(&clip, 0);
            track.add_clip(&clip, clip.frame_count());
            track
        })
        .collect()
}
