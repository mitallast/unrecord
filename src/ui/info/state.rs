use crate::ui::SessionTrack;
use gpui::{Context, SharedString};
use std::sync::Arc;

pub struct TrackInfoState {
    track: Option<SessionTrack>,
    info: Vec<TrackInfoItem>,
}

impl TrackInfoState {
    pub fn new(_: &mut Context<Self>) -> Self {
        Self {
            track: None,
            info: vec![],
        }
    }

    pub fn set_track(&mut self, track: SessionTrack) {
        self.info = track.info();
        self.track = Some(track);
    }

    pub fn info(&self) -> Vec<TrackInfoItem> {
        self.info.clone()
    }
}

#[derive(Clone, Debug)]
pub struct TrackInfoItem {
    key: SharedString,
    value: SharedString,
}

impl TrackInfoItem {
    pub fn new(key: impl Into<Arc<str>>, value: impl Into<Arc<str>>) -> Self {
        Self {
            key: SharedString::new(key),
            value: SharedString::new(value),
        }
    }

    pub fn key(&self) -> SharedString {
        self.key.clone()
    }

    pub fn value(&self) -> SharedString {
        self.value.clone()
    }
}
