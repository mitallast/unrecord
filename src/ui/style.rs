use eframe::egui;

pub fn app_style(ctx: &egui::Context) {
    ctx.style_mut(|style| {
        style.spacing.item_spacing = egui::vec2(10.0, 8.0);
        style.spacing.button_padding = egui::vec2(20.0, 8.0);
        style.spacing.window_margin = egui::Margin::same(10);
        style.spacing.combo_width = 140.0;
        style.spacing.combo_height = 400.0;
        style.spacing.interact_size = egui::vec2(0.0, 31.0);

        // style.visuals.window_rounding = egui::Rounding::same(14.0);

        style.visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(10);
        style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(10);
        style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(10);
        style.visuals.widgets.active.corner_radius = egui::CornerRadius::same(10);

        // чуть “стекляннее”
        style.visuals.panel_fill = egui::Color32::from_rgba_unmultiplied(18, 20, 26, 200);
    });
}
