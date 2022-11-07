use egui::{Response, Ui};


pub fn number_changer(number: &mut usize) -> impl egui::Widget + '_ {
    move |ui: &mut Ui| number_changer_ui(number, ui)
}

fn number_changer_ui(number: &mut usize, ui: &mut Ui) -> Response {
    ui.label(number.to_string());
    ui.horizontal(|ui| {
        if ui.button("+").clicked() {
            *number = number.saturating_add(1);
        }
        if ui.button("-").clicked() {
            *number = number.saturating_sub(1);
        }
    })
    .response
}
