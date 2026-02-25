use crate::audio::{CoreAudioDevice, CoreAudioDriver, RecordSession};
use crate::ui::session::track::SessionTrack;
use crate::ui::{SessionStatus, TrackInfoState};
use anyhow::{Result, anyhow};
use async_std::channel::unbounded;
use async_std::prelude::FutureExt;
use gpui::{
    AppContext, ClickEvent, Context, Entity, PathPromptOptions, SharedString, Subscription, Window,
};
use gpui_component::IndexPath;
use gpui_component::input::{InputEvent, InputState, NumberInputEvent, StepAction};
use gpui_component::select::{SelectEvent, SelectItem, SelectState};
use log::{error, info};
use objc2_core_audio::AudioObjectID;
use std::path::PathBuf;

pub struct SessionState {
    pub(super) select_device_state: Entity<SelectState<Vec<DeviceSelectItem>>>,
    pub(super) iteration_count_state: Entity<InputState>,
    pub(super) source_path_state: Entity<InputState>,
    pub(super) destination_path_state: Entity<InputState>,

    current_device_id: Option<AudioObjectID>,
    pub(super) current_source_path: Option<PathBuf>,
    pub(super) current_destination_path: Option<PathBuf>,
    current_iteration_count: Option<u32>,

    pub(super) session_status: SessionStatus,
    pub session_tracks: Vec<SessionTrack>,
    info_state: Entity<TrackInfoState>,

    _subscriptions: Vec<Subscription>,
}

impl SessionState {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        info_state: &Entity<TrackInfoState>,
    ) -> Result<Self> {
        let driver = CoreAudioDriver;

        let devices: Vec<DeviceSelectItem> = driver
            .list_devices()?
            .iter()
            .map(|device| DeviceSelectItem::from(device))
            .collect();

        let default = devices.iter().enumerate().find_map(|(index, device)| {
            if device.is_stereo() {
                Some((IndexPath::new(index), device.device_id))
            } else {
                None
            }
        });

        let select_device_state =
            cx.new(|cx| SelectState::new(devices, default.map(|f| f.0), window, cx));

        let select_device_sub =
            cx.subscribe(&select_device_state, |this, _, event, cx| match event {
                SelectEvent::Confirm(value) => {
                    info!("device selected: {:?}", value);
                    this.current_device_id = value.clone();
                    cx.notify();
                }
            });

        let iteration_count_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Enter number of iterations")
                .validate(|v, _| v.parse::<u32>().is_ok())
                .default_value("100")
        });

        let iteration_count_input_sub = cx.subscribe_in(
            &iteration_count_state,
            window,
            |this, state, event, _, cx| match event {
                InputEvent::Change => {
                    let text = state.read(cx).value();
                    this.current_iteration_count = text.parse::<u32>().ok();
                    cx.notify();
                }
                _ => {}
            },
        );
        let iteration_count_inc_sub = cx.subscribe_in(
            &iteration_count_state,
            window,
            |this, state, event, window, cx| match event {
                NumberInputEvent::Step(step_action) => match step_action {
                    StepAction::Decrement => {
                        if let Some(value) = this.current_iteration_count {
                            let value = value.saturating_sub(1);
                            state.update(cx, |input, cx| {
                                input.set_value(format!("{value}"), window, cx)
                            });
                            this.current_iteration_count = Some(value);
                            cx.notify();
                        }
                    }
                    StepAction::Increment => {
                        if let Some(value) = this.current_iteration_count {
                            let value = value.saturating_add(1);
                            state.update(cx, |input, cx| {
                                input.set_value(format!("{value}"), window, cx)
                            });
                            this.current_iteration_count = Some(value);
                            cx.notify();
                        }
                    }
                },
            },
        );

        let source_file_state = cx.new(|cx| {
            InputState::new(window, cx) //
                .placeholder("Select a source file")
        });

        let destination_dir_state = cx.new(|cx| {
            InputState::new(window, cx) //
                .placeholder("Select a destination dir")
        });

        Ok(Self {
            select_device_state,
            iteration_count_state,
            info_state: info_state.clone(),
            source_path_state: source_file_state,
            destination_path_state: destination_dir_state,
            current_device_id: default.map(|p| p.1),
            current_source_path: None,
            current_destination_path: None,
            current_iteration_count: Some(100),
            session_status: SessionStatus::IDLE,
            session_tracks: vec![],
            _subscriptions: vec![
                select_device_sub,
                iteration_count_input_sub,
                iteration_count_inc_sub,
            ],
        })
    }

    pub fn select_source_file(
        &mut self,
        _: &ClickEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let options = PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: None,
        };
        let select_file_prompt = cx.prompt_for_paths(options);

        cx.spawn_in(window, async move |state, window| {
            let paths = select_file_prompt.await??.ok_or(anyhow!("no files"))?;
            let source_file = paths.first().ok_or(anyhow!("no file"))?;
            state.update_in(window, |state, window, cx| {
                state.current_source_path = Some(source_file.clone());
                cx.notify();
                cx.update_entity(&state.source_path_state, |view, cx| {
                    view.set_value(source_file.to_string_lossy().to_string(), window, cx);
                });
                cx.update_entity(&state.info_state, |view, _| {
                    match SessionTrack::new(source_file.clone()) {
                        Ok(track) => view.set_track(track),
                        Err(error) => error!("failed to open source track: {}", error),
                    }
                });
            })
        })
        .detach();
    }

    pub fn select_destination_dir(
        &mut self,
        _: &ClickEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let options = PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: None,
        };
        let select_dir_prompt = cx.prompt_for_paths(options);

        cx.spawn_in(window, async move |view, window| {
            let paths = select_dir_prompt.await??.ok_or(anyhow!("no dirs"))?;
            let destination_dir = paths.first().ok_or(anyhow!("no dir"))?;
            view.update_in(window, |view, window, cx| {
                view.current_destination_path = Some(destination_dir.clone());
                cx.notify();
                cx.update_entity(&view.destination_path_state, |view, cx| {
                    view.set_value(destination_dir.to_string_lossy().to_string(), window, cx);
                });
            })
        })
        .detach();
    }

    pub fn record(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        match &self.session_status {
            SessionStatus::RUNNING(sender) => {
                sender.try_send(()).ok();
                return;
            }
            _ => {}
        };

        if let Some(device_id) = self.current_device_id
            && let Some(source_path) = self.current_source_path.clone()
            && let Some(destination_path) = self.current_destination_path.clone()
            && let Some(iteration_count) = self.current_iteration_count.clone()
            && self.session_status.is_stopped()
        {
            let (sender, receiver) = unbounded();
            self.session_status = SessionStatus::RUNNING(sender);
            cx.notify();

            cx.spawn_in(window, async move |entity, cx| -> Result<()> {
                let mut iteration = 0;
                while iteration < iteration_count {
                    let (source_path, destination_path) = if iteration == 0 {
                        let source_path = source_path.clone();
                        let destination_path =
                            destination_path.join(format!("output_{}.wav", iteration));
                        (source_path, destination_path)
                    } else {
                        let source_path =
                            destination_path.join(format!("output_{}.wav", iteration - 1));
                        let destination_path =
                            destination_path.join(format!("output_{}.wav", iteration));
                        (source_path, destination_path)
                    };
                    iteration += 1;

                    info!("new session");
                    let mut session =
                        RecordSession::new(device_id, source_path, destination_path.clone())
                            .await?;

                    info!("start recording");
                    session.start().await?;

                    info!("wait recording");
                    let cancel = receiver.recv();
                    let timeout = async {
                        session.wait().await;
                        Ok(())
                    };
                    cancel.race(timeout).await?;

                    info!("stop recording");
                    match session.stop().await {
                        Ok(_) => {
                            info!("successfully finished recording");
                            let track = SessionTrack::new(destination_path)?;
                            entity.update(cx, |state, cx| {
                                state.session_tracks.push(track.clone());
                                state.info_state.update(cx, |state, cx| {
                                    state.set_track(track);
                                });
                            })?;
                        }
                        Err(error) => {
                            error!("failed to finish recording: {error}");
                            entity.update(cx, |state, _| {
                                state.session_status =
                                    SessionStatus::FAILED(SharedString::new(error.to_string()));
                            })?;
                        }
                    };
                }
                entity.update(cx, |state, _| {
                    state.session_status = SessionStatus::FINISHED;
                })?;
                Ok(())
            })
            .detach();
        }
    }
}

#[derive(Clone)]
pub struct DeviceSelectItem {
    device_id: AudioObjectID,
    input_channels: u32,
    output_channels: u32,
    title: SharedString,
}

impl DeviceSelectItem {
    pub fn from(device: &CoreAudioDevice) -> Self {
        let device_id = device.get_id();
        let title = device.get_name().unwrap();
        let input_channels = device.get_input_channels().unwrap();
        let output_channels = device.get_output_channels().unwrap();

        let title = SharedString::from(title);
        Self {
            device_id,
            input_channels,
            output_channels,
            title,
        }
    }

    pub fn is_stereo(&self) -> bool {
        self.input_channels >= 2 && self.output_channels >= 2
    }
}

impl SelectItem for DeviceSelectItem {
    type Value = AudioObjectID;

    fn title(&self) -> SharedString {
        self.title.clone()
    }

    fn value(&self) -> &Self::Value {
        &self.device_id
    }
}
