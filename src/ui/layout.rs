use eframe::egui;

pub fn align_right_center<R>(
    ui: &mut egui::Ui,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    ui.with_layout(
        egui::Layout::right_to_left(egui::Align::Center),
        add_contents,
    )
    .inner
}
