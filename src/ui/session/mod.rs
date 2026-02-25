mod panel;
mod state;
mod track;

use async_std::channel::Sender;
use gpui::SharedString;
pub use panel::*;
pub use state::*;

#[derive(Clone, Debug)]
pub enum SessionStatus {
    IDLE,
    RUNNING(Sender<()>),
    FINISHED,
    FAILED(SharedString),
}

const IDLE_TITLE: SharedString = SharedString::new_static("Start");
const RUNNING_TITLE: SharedString = SharedString::new_static("Stop");
const STOPPING_TITLE: SharedString = SharedString::new_static("Stoping");
const FINISHED_TITLE: SharedString = SharedString::new_static("Finished");
const FAILED_TITLE: SharedString = SharedString::new_static("Failed");

impl SessionStatus {
    pub fn is_stopped(&self) -> bool {
        match self {
            SessionStatus::IDLE => true,
            SessionStatus::RUNNING(_) => false,
            SessionStatus::FINISHED => true,
            SessionStatus::FAILED(_) => true,
        }
    }

    pub fn is_running(&self) -> bool {
        match self {
            SessionStatus::IDLE => false,
            SessionStatus::RUNNING(_) => true,
            SessionStatus::FINISHED => false,
            SessionStatus::FAILED(_) => false,
        }
    }

    pub fn title(&self) -> SharedString {
        match self {
            SessionStatus::IDLE => IDLE_TITLE,
            SessionStatus::RUNNING(_) => RUNNING_TITLE,
            SessionStatus::FINISHED => FINISHED_TITLE,
            SessionStatus::FAILED(_) => FINISHED_TITLE,
        }
    }
}
