#![allow(missing_docs)]

use egui::Widget;
use hcegui::dnd;

pub struct DndDemo {
    keyboard_layouts: Vec<(&'static str, &'static str)>,
    poem: Vec<&'static str>,
    list_of_lists: Vec<Vec<&'static str>>,
}

impl Default for DndDemo {
    fn default() -> Self {
        Self {
            keyboard_layouts: vec![
                ("QWERTY", "QWERTYUIOP\nASDFGHJKL;\nZXCVBNM,./"),
                ("Colemak", "QWFPGJLUY;\nARSTDHNEIO\nZXCVBKM,./"),
                ("Dvorak", "',.PYFGCRL\nAOEUIDHTNS\n;QJKXBMWVZ"),
                ("Workman", "QDRWBJFUP;\nASHTGYNEOI\nZXMCVKL,./"),
            ],

            poem: vec![
                "Pointless machines",
                "Resurrections",
                "Scattered and lost",
                "Eye of the storm",
                "Heavy and frail",
                "Quiet and falling",
                "Pink sunrise",
                "Heart of the mountain",
                "Sever the skyline",
                "Black moonrise",
                "Good karma",
                "Golden feather",
                "Mirror magic",
                "Center of the earth",
                "No more running",
                "Say goodbye",
            ],

            list_of_lists: vec![
                vec!["akesi", "soweli", "kala", "waso"],
                vec!["reptile", "dog", "fish", "bird"],
                vec!["snek", "doggo", "fishy", "birb"],
                vec![],
                vec!["The quick, brown fox jumps over the lazy dog."],
                vec!["The horse is a noble animal."],
                vec![],
            ],
        }
    }
}

impl DndDemo {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.style_mut().interaction.selectable_labels = false;
        ui.style_mut().spacing.scroll = egui::style::ScrollStyle::solid();
        ui.visuals_mut().collapsing_header_frame = true;

        ui.columns(3, |uis| {
            let ui = &mut uis[0];

            // Reordering with handles
            ui.heading("Reorder with handles");
            let mut dnd = dnd::Dnd::new(ui.ctx(), "poem");
            for (i, &poem_line) in self.poem.iter().enumerate() {
                dnd.reorderable_with_handle(ui, i, |ui, _| ui.label(poem_line));
            }
            if let Some(r) = dnd.finish(ui).if_done_dragging() {
                r.reorder(&mut self.poem);
            }

            let ui = &mut uis[1];

            // Reordering with no handles
            ui.heading("Reorder with no handles");
            let mut dnd = dnd::Dnd::new(ui.ctx(), "keyboard_layouts");
            let is_dragging = dnd.is_dragging();
            for (i, &(name, details)) in self.keyboard_layouts.iter().enumerate() {
                dnd.reorderable(ui, i, |ui, _| {
                    let r = egui::CollapsingHeader::new(name)
                        .open(is_dragging.then_some(false))
                        .show(ui, |ui| ui.code(details));
                    (r.header_response, ())
                });
            }
            if let Some(r) = dnd.finish(ui).if_done_dragging() {
                r.reorder(&mut self.keyboard_layouts);
            }

            let ui = &mut uis[2];

            // Nesting + custom reordering logic
            ui.heading("Nested");
            show_list_of_lists_demo(ui, &mut self.list_of_lists);
        });
    }
}

fn show_list_of_lists_demo(ui: &mut egui::Ui, lists: &mut Vec<Vec<&'static str>>) {
    let mut row_dnd = dnd::Dnd::new(ui.ctx(), "rows");
    let mut item_dnd = dnd::Dnd::new(ui.ctx(), "items");
    let mut index_to_delete = None;

    // Display items
    for (i, list) in lists.iter_mut().enumerate() {
        row_dnd.reorderable_with_handle(ui, i, |ui, _| {
            let r = egui::ScrollArea::horizontal()
                .auto_shrink(false)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        for (j, &item) in list.iter().enumerate() {
                            let r = item_dnd.draggable(ui, (i, j), |ui, _| {
                                (egui::Label::new(item).sense(egui::Sense::drag()).ui(ui), ())
                            });
                            item_dnd.reorder_drop_zone_before_after(ui, &r.response, (i, Some(j)));
                        }

                        // Delete button
                        if list.is_empty()
                            && !item_dnd.is_dragging()
                            && ui.button(egui::RichText::new("🗑").small()).clicked()
                        {
                            index_to_delete = Some(i);
                        }
                    });
                });
            let r = ui.interact(r.inner_rect, r.id.with(1), egui::Sense::empty());
            if list.is_empty() {
                item_dnd.drop_zone(ui, &r, ((i, None), dnd::BeforeOrAfter::Before));
            }
        });
    }
    if ui.button("Add list").clicked() {
        lists.push(vec![]);
    }

    // Reorder individual items
    if let Some(r) = item_dnd.finish(ui).if_done_dragging() {
        let (i1, j1) = r.payload;
        let ((i2, j2), placement) = r.target;
        if i1 == i2
            && let Some(j2) = j2
        {
            dnd::DndMove::new(j1, (j2, placement)).reorder(&mut lists[i1]);
        } else {
            let elem = lists[i1].remove(j1);
            if let Some(j2) = j2 {
                let j2 = match placement {
                    dnd::BeforeOrAfter::Before => j2,
                    dnd::BeforeOrAfter::After => j2 + 1,
                };
                lists[i2].insert(j2, elem);
            } else {
                lists[i2].push(elem);
            }
        }
    }

    // Reorder whole lists
    if let Some(r) = row_dnd.finish(ui).if_done_dragging() {
        r.reorder(lists);
    }

    // Delete empty list
    if let Some(i) = index_to_delete {
        lists.remove(i);
    }
}
