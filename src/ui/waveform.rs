use crate::waveform::{TimeCursor, Waveform, WaveformItem, WaveformShape};
use eframe::epaint::Color32;
use egui::CornerRadius;

pub fn waveform_view(
    ui: &mut egui::Ui,
    title: impl Into<String>,
    waveform_left: &WaveformShape,
    waveform_right: &WaveformShape,
) {
    let bg_color = Color32::from_rgb(66, 122, 162);
    let line_stroke = egui::Stroke::new(1.0, Color32::WHITE);
    let corner_radius = CornerRadius::same(6);
    let height = 100.0;
    let gain = 0.9;

    egui::Frame::new()
        .inner_margin(egui::Margin::same(0))
        .corner_radius(corner_radius)
        .fill(bg_color)
        .stroke(line_stroke)
        .show(ui, |ui| {
            ui.style_mut().visuals.widgets.noninteractive.fg_stroke = line_stroke;

            ui.vertical_centered(|ui| {
                ui.label(title.into());
            });

            let mut cursor = TimeCursor::default();
            Waveform::default()
                .entry(WaveformItem::new(waveform_left).with_gain(gain))
                .cursor(&mut cursor)
                .height(height)
                .corner_radius(corner_radius)
                .show(ui);

            Waveform::default()
                .entry(WaveformItem::new(waveform_right).with_gain(gain))
                .cursor(&mut cursor)
                .height(height)
                .corner_radius(corner_radius)
                .show(ui);
        });
}
