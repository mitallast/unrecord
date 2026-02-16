mod audio;
mod record;
mod ui;
mod wav_file;
mod waveform;

use crate::audio::CoreAudioDriver;
use crate::record::RecordTask;
use crate::ui::{
    align_right_center, app_button, app_button_primary, app_card, app_combo, app_style,
    app_weak_label, waveform_view,
};
use crate::wav_file::read_file;
use crate::waveform::WaveformShape;
use anyhow::Result;
use coreaudio_sys::AudioObjectID;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::thread::spawn;

fn main() -> Result<()> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };

    let app = UnrecordApp::new()?;
    eframe::run_native("Unrecord", options, Box::new(|_| Ok(Box::new(app))))
        .map_err(|err| anyhow::anyhow!("{:?}", err))
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum RecordingStatus {
    IDLE,
    STOPPING,
    RUNNING,
    FINISHED,
}

struct UnrecordApp {
    devices: Vec<(Option<AudioObjectID>, String)>,
    selected_device: Option<AudioObjectID>,
    source_file: Option<PathBuf>,
    source_waveform: Option<(WaveformShape, WaveformShape)>,
    output_dir: Option<PathBuf>,
    recording_round: Arc<AtomicUsize>,
    recording_status: Arc<RwLock<RecordingStatus>>,
    tasks: Arc<RwLock<Vec<RecordTask>>>,
}

impl UnrecordApp {
    fn new() -> Result<Self> {
        let driver = CoreAudioDriver;
        let devices: Vec<(Option<AudioObjectID>, String)> = driver
            .list_devices()?
            .iter()
            .map(|device| (Some(device.get_id()), device.get_name().unwrap_or_default()))
            .collect();

        Ok(Self {
            devices,
            selected_device: None,
            source_file: None,
            source_waveform: None,
            output_dir: None,
            recording_round: Arc::new(AtomicUsize::new(0)),
            recording_status: Arc::new(RwLock::new(RecordingStatus::IDLE)),
            tasks: Arc::new(RwLock::new(Vec::new())),
        })
    }
}

impl eframe::App for UnrecordApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        app_style(ctx);

        let mut side_margin = ctx.style().spacing.window_margin;
        side_margin.left = 0;
        egui::SidePanel::right("side_panel")
            .resizable(false)
            .show_separator_line(false)
            .exact_width(340.0)
            .frame(
                egui::Frame::new()
                    .inner_margin(side_margin)
                    .fill(ctx.style().visuals.panel_fill),
            )
            .show(ctx, |ui| {
                settings_card(ui, self);
            });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .inner_margin(ctx.style().spacing.window_margin)
                    .fill(ctx.style().visuals.panel_fill),
            )
            .show(ctx, |ui| {
                records_card(ui, self);
            });
    }
}

fn settings_card(ui: &mut egui::Ui, app: &mut UnrecordApp) {
    app_card(ui, "Settings", Some("Target".into()), |ui| {
        ui.add_space(4.0);

        egui::Grid::new("settings_grid")
            .num_columns(2)
            .spacing([12.0, 10.0])
            .min_row_height(31.0)
            .show(ui, |ui| {
                app_weak_label(ui, "Audio Device");
                app_combo(
                    ui,
                    "selected_audio_device",
                    &mut app.selected_device,
                    app.devices.as_slice(),
                );
                ui.end_row();

                app_weak_label(ui, "Source wav file");
                if app_button(ui, "Open file").clicked()
                    && let Some(path) = rfd::FileDialog::new()
                        .add_filter("WAV", &["wav"])
                        .pick_file()
                {
                    let (sample_rate, samples) = read_file(&path).unwrap();
                    let samples_l: Vec<f32> = samples.iter().step_by(2).copied().collect();
                    let samples_r: Vec<f32> = samples.iter().skip(1).step_by(2).copied().collect();
                    let waveform_l =
                        WaveformShape::generate(&samples_l, sample_rate as usize, 2, 2);
                    let waveform_r =
                        WaveformShape::generate(&samples_r, sample_rate as usize, 2, 2);
                    app.source_waveform = Some((waveform_l, waveform_r));
                    app.source_file = Some(path);
                }
                ui.end_row();

                app_weak_label(ui, "Output directory");
                if app_button(ui, "Change...").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        app.output_dir = Some(path);
                    }
                }
                ui.end_row();
            });

        if let Some(source_file) = &app.source_file {
            ui.add(egui::Label::new(
                egui::RichText::new(source_file.to_string_lossy().to_string())
                    .monospace()
                    .color(ui.visuals().weak_text_color()),
            ));
        }

        if let Some(output_dir) = &app.output_dir {
            ui.add(egui::Label::new(
                egui::RichText::new(output_dir.to_string_lossy().to_string())
                    .monospace()
                    .color(ui.visuals().weak_text_color()),
            ));
        }

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            align_right_center(ui, |ui| {
                let recording_round = app.recording_round.clone();
                let recording_state = app.recording_status.clone();
                let tasks = app.tasks.clone();

                let curr_state = *recording_state.read().unwrap();
                let curr_round = recording_round.load(Ordering::Relaxed);
                if curr_state == RecordingStatus::IDLE || curr_state == RecordingStatus::FINISHED {
                    if app_button_primary(ui, "Start recording").clicked()
                        && let Some(device_id) = app.selected_device.clone()
                        && let Some(source_file) = app.source_file.clone()
                        && let Some(output_dir) = app.output_dir.clone()
                    {
                        *recording_state.write().unwrap() = RecordingStatus::RUNNING;
                        spawn(move || {
                            tasks.write().unwrap().clear();

                            for round in 0..10 {
                                if *recording_state.read().unwrap() != RecordingStatus::RUNNING {
                                    break;
                                }
                                recording_round.store(round, Ordering::Relaxed);
                                let from_path = if round == 0 {
                                    source_file.clone()
                                } else {
                                    let prev = round - 1;
                                    let from_filename = format!("output_file_{prev}.wav");
                                    output_dir.join(&from_filename)
                                };
                                let to_filename = format!("output_file_{round}.wav");
                                let to_path = output_dir.join(&to_filename);

                                let mut task =
                                    RecordTask::new(device_id, from_path, to_path).unwrap();
                                task.record().ok();
                                tasks.write().unwrap().push(task);
                            }
                            *recording_state.write().unwrap() = RecordingStatus::FINISHED;
                        });
                    }
                } else if curr_state == RecordingStatus::RUNNING {
                    if app_button_primary(ui, format!("Recording ({curr_round})")).clicked() {
                        *recording_state.write().unwrap() = RecordingStatus::STOPPING;
                    }
                } else if curr_state == RecordingStatus::STOPPING {
                    app_button_primary(ui, format!("Stop recording ({curr_round})"));
                }
            });
        });
    });
}

fn records_card(ui: &mut egui::Ui, app: &mut UnrecordApp) {
    app_card(
        ui,
        "Records",
        Some(format!("{} items", app.tasks.read().unwrap().len())),
        |ui| {
            ui.add_space(4.0);

            let available_h = ui.available_height();

            egui::ScrollArea::vertical()
                .min_scrolled_height(available_h)
                .show(ui, |ui| {
                    if let Some((waveform_l, waveform_r)) = &app.source_waveform
                        && let Some(path) = app.source_file.clone()
                    {
                        waveform_view(
                            ui,
                            path.as_os_str().to_string_lossy().to_string(),
                            waveform_l,
                            waveform_r,
                        );
                    }

                    for record in app.tasks.read().unwrap().iter() {
                        if let Some((waveform_l, waveform_r)) = record.output_waveform() {
                            waveform_view(ui, record.destination_path(), waveform_l, waveform_r);

                            // ui.style_mut().visuals.extreme_bg_color =
                            //     Color32::from_rgb(66, 122, 162);
                            // ui.style_mut().visuals.widgets.noninteractive.fg_stroke =
                            //     Stroke::new(1.0, Color32::WHITE);
                            //
                            // let mut cursor = TimeCursor::default();
                            // Waveform::default()
                            //     .entry(WaveformItem::new(waveform))
                            //     .cursor(&mut cursor)
                            //     .show(ui);
                        }
                    }
                });

            // // TableBuilder: липкий заголовок + скролл в области таблицы
            // TableBuilder::new(ui)
            //     .striped(true)
            //     .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            //     .column(Column::remainder()) // File
            //     .column(Column::initial(260.0)) // Spec
            //     .column(Column::initial(96.0)) // Action
            //     .column(Column::remainder()) // Result
            //     .min_scrolled_height(available_h)
            //     .header(row_h, |mut header| {
            //         header.col(|ui| app_weak_label(ui, "Source"));
            //         header.col(|ui| app_weak_label(ui, "Destination"));
            //     })
            //     .body(|mut body| {
            //         for record in app.tasks.read().unwrap().iter() {
            //             body.row(row_h, |mut row| {
            //                 row.col(|ui| {
            //                     ui.label(&record.source_path());
            //                 });
            //                 row.col(|ui| {
            //                     ui.label(&record.destination_path());
            //                 });
            //             });
            //         }
            //     });
        },
    );
}
