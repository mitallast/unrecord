use eframe::egui;

pub fn app_combo<T: PartialEq + Copy>(
    ui: &mut egui::Ui,
    id: &str,
    val: &mut T,
    items: &[(T, String)],
) {
    let current = items
        .iter()
        .find(|(v, _)| v == val)
        .map(|(_, t)| t.clone())
        .unwrap_or("—".to_string());

    egui::ComboBox::from_id_salt(id)
        .selected_text(egui::RichText::new(current))
        .show_ui(ui, |ui| {
            for (v, t) in items {
                ui.selectable_value(val, *v, t.clone());
            }
        });
}
