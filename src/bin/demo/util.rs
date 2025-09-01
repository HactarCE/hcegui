pub struct UtilDemo;

impl UtilDemo {
    pub fn show(ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                egui::Resize::default()
                    .default_width(ui.available_width())
                    .max_height(0.0)
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            #[allow(unused_must_use)]
                            hcegui::util::show_on_one_line(ui, |ui| {
                                ui.button("This was a triumph");
                                ui.button("I'm making a note here; \"Huge success\"");
                                ui.button("It's hard to overstate");
                                ui.button("My satisfaction");
                                ui.button("Aperture Science:");
                                ui.button("We do what we must");
                                ui.button("Because we can");
                                ui.button("For the good of all of us");
                                ui.button("Except the ones who are dead");
                                ui.button("But there's no sense crying");
                                ui.button("Over every mistake");
                                ui.button("You just keep on trying");
                                ui.button("Till you run out of cake");
                                ui.button("And the science gets done");
                                ui.button("And you make a neat gun");
                                ui.button("For the people who are");
                                ui.button("Still alive");
                                ui.button("I'm not even angry");
                                ui.button("I'm being so sincere right now");
                                ui.button("Even though you broke my heart,");
                                ui.button("And killed me");
                                ui.button("And tore me to pieces");
                                ui.button("And threw every piece into a fire");
                                ui.button("As they burned it hurt because");
                                ui.button("I was so happy for you");
                                ui.button("Now, these points of data");
                                ui.button("Make a beautiful line");
                                ui.button("And we're out of beta");
                                ui.button("We're releasing on time");
                                ui.button("So I'm GLaD I got burned");
                                ui.button("Think of all the things we learned-");
                                ui.button("For the people who are");
                                ui.button("Still alive");
                                ui.button("Go ahead and leave me");
                                ui.button("I think I'd prefer to stay inside");
                                ui.button("Maybe you'll find someone else");
                                ui.button("To help you?");
                                ui.button("Maybe Black Mesa?");
                                ui.button("That was a joke *Haha - Fat Chance*");
                                ui.button("Anyway this cake is great");
                                ui.button("It's so delicious and moist");
                                ui.button("Look at me: still talking");
                                ui.button("When there's science to do");
                                ui.button("When I look out there,");
                                ui.button("It makes me GLaD I'm not you");
                                ui.button("I've experiments to run");
                                ui.button("There is research to be done");
                                ui.button("On the people who are");
                                ui.button("Still alive");
                                ui.button("And believe me I am");
                                ui.button("Still alive");
                                ui.button("I'm doing science and I'm");
                                ui.button("Still alive");
                                ui.button("I feel fantastic and I'm");
                                ui.button("Still alive");
                                ui.button("While you're dying I'll be");
                                ui.button("Still alive");
                                ui.button("And when you're dead I will be");
                                ui.button("Still alive");
                                ui.button("Still alive");
                            });
                        });
                    });
            });
    }
}
