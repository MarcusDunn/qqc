use crate::app::Section;

pub fn primary_section<'a>(section: &'a Section, speaker: &'a str) -> impl egui::Widget + 'a {
    move |ui: &mut egui::Ui| primary_section_ui(ui, section, speaker)
}

pub fn secondary_section<'a>(section: &'a Section, speaker: &'a str) -> impl egui::Widget + 'a {
    move |ui: &mut egui::Ui| secondary_section_ui(ui, section, speaker)
}

fn primary_section_ui(
    ui: &mut egui::Ui,
    Section { text, .. }: &Section,
    speaker: &str,
) -> egui::Response {
    ui.vertical(|ui| {
        ui.label(speaker);
        ui.label(text);
    }).response
}

fn secondary_section_ui(
    ui: &mut egui::Ui,
    Section { text, .. }: &Section,
    speaker: &str,
) -> egui::Response {
    ui.vertical(|ui| {
        ui.weak(speaker);
        ui.weak(text);
    }).response
}
