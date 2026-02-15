use eframe::egui;

pub fn app_button(ui: &mut egui::Ui, text: impl Into<String>) -> egui::Response {
    let button = egui::Button::new(
        egui::RichText::new(text)
            .strong()
            .color(ui.visuals().strong_text_color()),
    )
    .corner_radius(egui::CornerRadius::same(10));

    ui.add(button)
}

pub fn app_button_primary(ui: &mut egui::Ui, text: impl Into<String>) -> egui::Response {
    ui.scope(|ui| {
        let color = egui::Color32::from_rgb(10, 132, 255); // macOS blue-ish
        ui.style_mut().visuals.widgets.inactive.weak_bg_fill = color.gamma_multiply(0.85);
        ui.style_mut().visuals.widgets.hovered.weak_bg_fill = color;
        ui.style_mut().visuals.widgets.active.weak_bg_fill = color.gamma_multiply(0.75);

        app_button(ui, text)
    })
    .inner
}

pub fn app_button_danger(ui: &mut egui::Ui, text: impl Into<String>) -> egui::Response {
    ui.scope(|ui| {
        let color = egui::Color32::from_rgb(255, 69, 58);
        ui.style_mut().visuals.widgets.inactive.weak_bg_fill = color.gamma_multiply(0.85);
        ui.style_mut().visuals.widgets.hovered.weak_bg_fill = color;
        ui.style_mut().visuals.widgets.active.weak_bg_fill = color.gamma_multiply(0.75);

        app_button(ui, text)
    })
    .inner
}
