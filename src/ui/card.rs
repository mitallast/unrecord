use eframe::egui;

pub fn app_card(
    ui: &mut egui::Ui,
    title: &str,
    pill: Option<String>,
    add: impl FnOnce(&mut egui::Ui),
) {
    egui::Frame::new()
        .fill(egui::Color32::from_rgba_unmultiplied(255, 255, 255, 10))
        .stroke(egui::Stroke::new(
            1.0,
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 24),
        ))
        .corner_radius(egui::CornerRadius::same(14))
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(title).strong());
                if let Some(p) = pill {
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new(p).color(ui.visuals().weak_text_color()));
                }
            });
            ui.add_space(8.0);

            ui.set_min_size(ui.available_size());
            add(ui);
        });
}
