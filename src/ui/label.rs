use eframe::egui;

pub fn app_weak_label(ui: &mut egui::Ui, s: &str) {
    ui.label(
        egui::RichText::new(s)
            .strong()
            .color(ui.visuals().weak_text_color()),
    );
}
