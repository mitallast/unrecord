use crate::components::region::TrackRegion;
use crate::components::waveform::WaveClip;
use gpui::SharedString;
use std::cmp::Ordering;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Track {
    title: SharedString,
    regions: Arc<Mutex<Vec<TrackRegion>>>,
}

impl Track {
    pub fn new<T: Into<SharedString>>(title: T) -> Self {
        Self {
            title: title.into(),
            regions: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn title(&self) -> SharedString {
        self.title.clone()
    }

    pub fn regions(&self) -> Vec<TrackRegion> {
        self.regions.lock().unwrap().clone()
    }

    pub fn add_clip(&self, clip: &WaveClip, track_offset: usize) {
        let region = TrackRegion::new(clip, 0, clip.frame_count(), track_offset);
        self.add_region(region);
    }

    pub fn add_region(&self, region: TrackRegion) {
        self.regions.lock().unwrap().push(region);
        self.clean_regions();
    }

    fn clean_regions(&self) {
        let mut regions = self.regions.lock().unwrap();
        regions.retain(|region| region.frames() > 0);
        regions.sort_by(|a, b| {
            let mut ordering = a.track_start_frame().cmp(&b.track_start_frame());
            if ordering == Ordering::Equal {
                ordering = a.track_end_frame().cmp(&b.track_end_frame());
            }
            ordering
        });
        let mut i = 0;
        while i + 1 < regions.len() {
            let curr_end_frame = regions[i].track_end_frame();
            let next_start_frame = regions[i + 1].track_start_frame();
            if curr_end_frame > next_start_frame {
                regions[i].set_track_end_frame(next_start_frame);
            }
            i += 1;
        }
        regions.retain(|region| region.frames() > 0);
    }

    pub fn frames(&self) -> usize {
        let regions = self.regions.lock().unwrap();
        regions.iter().fold(0, |acc, region| {
            let end_frame = region.track_start_frame() + region.frames();
            acc.max(end_frame)
        })
    }
}
