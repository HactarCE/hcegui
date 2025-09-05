//! Demo crate.

mod dnd;
mod util;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
enum Panel {
    #[default]
    Dnd,
    Util,
}

fn main() -> eframe::Result {
    let mut current_panel = Panel::default();

    let mut dnd_demo = dnd::DndDemo::default();
    let mut util_demo = util::UtilDemo::default();

    eframe::run_simple_native(
        "egui_reorder demo",
        eframe::NativeOptions::default(),
        move |ctx, _frame| {
            egui::CentralPanel::default().show(ctx, |ui| {
                egui::Sides::new().show(
                    ui,
                    |ui| {
                        ui.selectable_value(&mut current_panel, Panel::Dnd, "dnd");
                        ui.selectable_value(&mut current_panel, Panel::Util, "util");
                    },
                    |ui| egui::global_theme_preference_buttons(ui),
                );

                ui.separator();

                match current_panel {
                    Panel::Dnd => dnd_demo.show(ui),
                    Panel::Util => util_demo.show(ui),
                }
            });
        },
    )
}
