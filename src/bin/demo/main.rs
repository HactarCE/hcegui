//! Demo crate.

mod reorder;
mod util;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
enum Panel {
    #[default]
    Reorder,
    Util,
}

fn main() -> eframe::Result {
    let mut current_panel = Panel::default();

    let mut reorder_demo = reorder::ReorderDemo::default();

    eframe::run_simple_native(
        "egui_reorder demo",
        eframe::NativeOptions::default(),
        move |ctx, _frame| {
            egui::CentralPanel::default().show(ctx, |ui| {
                egui::Sides::new().show(
                    ui,
                    |ui| {
                        for panel in [Panel::Reorder, Panel::Util] {
                            ui.selectable_value(&mut current_panel, panel, format!("{panel:?}"));
                        }
                    },
                    |ui| egui::global_theme_preference_buttons(ui),
                );
                ui.separator();
                match current_panel {
                    Panel::Reorder => reorder_demo.show(ui),
                    Panel::Util => util::UtilDemo::show(ui),
                }
            });
        },
    )
}
