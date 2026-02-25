use crate::audio::read_file;

use anyhow::Result;
use gpui::SharedString;
use std::path::Path;
use std::sync::Arc;

pub struct SessionTrack {
    pub(in crate::ui) path: SharedString,
    pub(in crate::ui) filename: SharedString,
    pub(in crate::ui) sample_rate: f64,
    pub(in crate::ui) samples: Arc<Vec<f32>>,
}

impl SessionTrack {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let (sample_rate, samples) = read_file(&path)?;

        let filename = path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("No filename"))?
            .to_string_lossy()
            .to_string();

        let path = path.to_string_lossy().to_string();

        Ok(Self {
            path: SharedString::from(path),
            filename: SharedString::from(filename),
            sample_rate,
            samples: Arc::new(samples),
        })
    }
}
