use crate::app::Section;

pub fn primary_section(section: &Section) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| primary_section_ui(ui, section)
}

pub fn secondary_section(section: &Section) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| secondary_section_ui(ui, section)
}

fn primary_section_ui(ui: &mut egui::Ui, Section { text, .. }: &Section) -> egui::Response {
    ui.label(text)
}

fn secondary_section_ui(ui: &mut egui::Ui, Section { text, .. }: &Section) -> egui::Response {
    ui.weak(text)
}
