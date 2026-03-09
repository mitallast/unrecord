use crate::components::waveform::WaveClip;

#[derive(Clone)]
pub struct TrackRegion {
    clip: WaveClip,
    clip_start_frame: usize,
    clip_end_frame: usize,
    track_start_frame: usize,
}

impl TrackRegion {
    pub fn new(clip: &WaveClip, clip_start_frame: usize, clip_end_frame: usize, track_offset: usize) -> Self {
        let mut region = Self {
            clip: clip.clone(),
            clip_start_frame: 0,
            clip_end_frame: 0,
            track_start_frame: track_offset,
        };
        region.set_clip_start_frame(clip_start_frame);
        region.set_clip_end_frame(clip_end_frame);
        region
    }

    pub fn clip(&self) -> WaveClip {
        self.clip.clone()
    }

    pub fn clip_start_frame(&self) -> usize {
        self.clip_start_frame
    }

    pub fn clip_end_frame(&self) -> usize {
        self.clip_end_frame
    }

    pub fn track_start_frame(&self) -> usize {
        self.track_start_frame
    }

    pub fn track_end_frame(&self) -> usize {
        self.track_start_frame + self.frames()
    }

    pub fn set_clip_start_frame(&mut self, frame: usize) {
        self.clip_start_frame = frame.min(self.clip.frame_count().saturating_sub(1));
        self.clip_end_frame = self.clip_end_frame.max(self.clip_start_frame);
    }

    pub fn set_clip_end_frame(&mut self, frame: usize) {
        self.clip_end_frame = frame.max(self.clip_start_frame);
    }

    #[allow(dead_code)]
    pub fn set_track_start_frame(&mut self, frame: usize) {
        self.track_start_frame = frame;
    }

    pub fn set_track_end_frame(&mut self, frame: usize) {
        let len = self.track_start_frame.saturating_sub(frame);
        let end_frame = self.clip_start_frame + len;
        self.set_clip_end_frame(end_frame);
    }

    pub fn frames(&self) -> usize {
        self.clip_end_frame.saturating_sub(self.clip_start_frame)
    }
}
