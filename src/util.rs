//! Miscellaneous helper functions.

/// Displays UI in a wrapping layout, pushing this widget onto the next line if
/// it cannot be displayed on the current line without wrapping.
pub fn show_on_one_line<R>(
    ui: &mut egui::Ui,
    mut add_contents: impl FnMut(&mut egui::Ui) -> R,
) -> R {
    if ui.layout().main_wrap
        && ui.layout().is_horizontal()
        && ui.cursor().left() > ui.max_rect().left()
        && non_wrapping_size_of_ui(ui, &mut add_contents).x >= ui.available_size_before_wrap().x
    {
        force_horizontal_wrap(ui);
    }

    add_contents(ui)
}

/// Returns the size used `add_contents()` in a non-wrapping, non-justified
/// layout.
pub fn non_wrapping_size_of_ui<R>(
    ui: &mut egui::Ui,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::Vec2 {
    let ui_builder = egui::UiBuilder::new()
        .layout(egui::Layout {
            main_wrap: false,
            main_justify: false,
            ..*ui.layout()
        })
        .invisible()
        .sizing_pass();
    let r = ui.new_child(ui_builder).scope(add_contents);
    r.response.rect.size()
}

/// Wraps to the next line in a horizontal wrapping layout.
fn force_horizontal_wrap(ui: &mut egui::Ui) {
    // This is really hacky but I don't know anything else that works.
    let old_x_spacing = std::mem::take(&mut ui.spacing_mut().item_spacing.x);
    ui.add_space(ui.available_size_before_wrap().x);
    ui.allocate_exact_size(egui::vec2(1.0, 1.0), egui::Sense::hover());
    ui.add_space(-1.0);
    ui.spacing_mut().item_spacing.x = old_x_spacing;
}
