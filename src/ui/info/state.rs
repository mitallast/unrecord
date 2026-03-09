use crate::components::waveform::{WaveClip, WaveClipParameter};
use gpui::Context;

pub struct ClipInfoState {
    track: Option<WaveClip>,
    info: Vec<WaveClipParameter>,
}

impl ClipInfoState {
    pub fn new(_: &mut Context<Self>) -> Self {
        Self {
            track: None,
            info: vec![],
        }
    }

    pub fn set_clip(&mut self, track: &WaveClip) {
        self.info = track.metadata().info();
        self.track = Some(track.clone());
    }

    pub fn info(&self) -> Vec<WaveClipParameter> {
        self.info.clone()
    }
}
