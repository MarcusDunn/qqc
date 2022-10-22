use egui::Button;
use std::collections::{BTreeMap, BTreeSet};
use std::convert::Infallible;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};

use tracing::error;

use crate::app::interview::InterviewSwiper;
use crate::app::section::{primary_section, secondary_section};

mod file_upload;
mod interview;
mod parse_interview;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct QualityQualitativeCoding {
    settings: Settings,
    /// the interview itself
    interview: Option<InterviewSwiper>,
    /// the codes to choose from
    codes: BTreeMap<u64, Code>,
    /// a code the user has not added yet,
    code_builder: Code,
    /// receive files asynchronously
    #[serde(skip)]
    interview_channel: (Sender<Vec<u8>>, Receiver<Vec<u8>>),
    #[serde(skip)]
    codes_channel: (Sender<Vec<u8>>, Receiver<Vec<u8>>),
}
#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct Settings {
    code_columns: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self { code_columns: 5 }
    }
}

impl Default for QualityQualitativeCoding {
    fn default() -> Self {
        Self {
            settings: Default::default(),
            interview: None,
            codes: BTreeMap::default(),
            code_builder: Code {
                name: "".to_string(),
                description: "".to_string(),
            },
            interview_channel: channel(),
            codes_channel: channel(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
struct Code {
    name: String,
    description: String,
}

#[derive(serde::Deserialize, serde::Serialize, Default, Debug)]
pub struct Interview {
    /// speaker_id and names
    speakers: BTreeMap<u64, String>,
    /// the sections of speach
    sections: Vec<Section>,
}

#[derive(serde::Deserialize, serde::Serialize, Default, Debug)]
pub struct Section {
    speaker_id: u64,
    text: String,
    /// references the key of a code
    codes: BTreeSet<u64>,
}

impl QualityQualitativeCoding {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load previous app state (if any).
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            QualityQualitativeCoding::default()
        }
    }

    fn try_update_codes(_: &mut BTreeMap<u64, Code>, codes_recv: &mut Receiver<Vec<u8>>) {
        match codes_recv.try_recv() {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(_) => {
                    todo!()
                }
                Err(err) => {
                    error!(error = ?err, "failed to parse string")
                }
            },
            Err(TryRecvError::Empty) => { /* no file has been uploaded yet - no problem! */ }
            Err(TryRecvError::Disconnected) => {
                panic!("impossible to upload files. sender has been dropped.")
            }
        }
    }

    fn try_update_interview(
        interview: &mut Option<InterviewSwiper>,
        receiver: &mut Receiver<Vec<u8>>,
    ) {
        match receiver.try_recv() {
            Ok(bytes) => {
                match parse_interview::from_json_slice(&*bytes) {
                    Ok(parsed_interview) => {
                        tracing::trace!("parsed interview");
                        *interview = Some(InterviewSwiper::new(parsed_interview))
                    }
                    Err(err) => {
                        tracing::trace!(error = ?err, "failed to parse json");
                    }
                };
            }
            Err(TryRecvError::Empty) => { /* no file has been uploaded yet - no problem! */ }
            Err(TryRecvError::Disconnected) => {
                panic!("impossible to upload files. sender has been dropped.")
            }
        }
    }

    fn add_new_code(codes: &mut BTreeMap<u64, Code>, code_builder: &mut Code) {
        let name = std::mem::take(&mut code_builder.name);
        let description = std::mem::take(&mut code_builder.description);
        codes.insert(
            codes.keys().max().copied().unwrap_or(0) + 1,
            Code { name, description },
        );
    }

    fn open_tsv_upload_dialog(codes_tx: &mut Sender<Vec<u8>>) {
        file_upload::open_upload_dialog(codes_tx.clone(), ("tsv", &["tsv"]))
    }

    fn open_interview_upload_dialog(interview_tx: &mut Sender<Vec<u8>>) {
        file_upload::open_upload_dialog(interview_tx.clone(), ("json", &["json"]))
    }
}

impl eframe::App for QualityQualitativeCoding {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        let Self {
            interview,
            codes,
            code_builder,
            interview_channel: (interview_tx, interview_rx),
            codes_channel: (codes_tx, codes_rx),
            settings,
        } = self;

        Self::try_update_interview(interview, interview_rx);
        Self::try_update_codes(codes, codes_rx);

        egui::SidePanel::right("codes edit and create").show(ctx, |ui| {
            ui.heading("Codes");
            if codes.is_empty() {
                ui.label("no codes at the moment, try adding one!");
            }
            for (_, Code { name, description }) in codes.iter_mut() {
                ui.group(|ui| {
                    ui.text_edit_singleline(name);
                    ui.text_edit_singleline(description);
                });
            }
            ui.heading("New Code");
            ui.label("name");
            ui.text_edit_singleline(&mut code_builder.name);
            ui.label("description");
            ui.text_edit_singleline(&mut code_builder.description);
            if ui.button("add new code").clicked() {
                Self::add_new_code(codes, code_builder);
            }
            if ui.button("upload").clicked() {
                Self::open_tsv_upload_dialog(codes_tx);
            }
        });

        if let Some(interview) = interview {
            egui::SidePanel::left("speaker and next").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("next").clicked() {
                        interview.try_next();
                    }
                    if ui.button("prev").clicked() {
                        interview.try_prev();
                    }
                })
            });

            egui::TopBottomPanel::bottom("codes select").show(ctx, |ui| {
                let current = interview.current_mut();
                egui::Grid::new("code grid").show(ui, |ui| {
                    for (idx, (id, Code { name, description })) in codes.iter().enumerate() {
                        if idx != 0 && idx % settings.code_columns == 0 {
                            ui.end_row()
                        }
                        let checked = &mut current.codes.contains(id);
                        let checkbox = ui.checkbox(checked, name).on_hover_text(description);
                        if checkbox.changed() {
                            if *checked {
                                current.codes.insert(*id);
                            } else {
                                current.codes.remove(id);
                            }
                        }
                    }
                });
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| match interview {
            None => {
                if ui.button("Upload interview").clicked() {
                    Self::open_interview_upload_dialog(interview_tx);
                }
            }
            Some(interview) => {
                ui.heading("coding interview");
                let (before, curr, after) = interview.window(1, 1);
                for section in before {
                    ui.add(secondary_section(section));
                }
                ui.add(primary_section(curr));
                for section in after {
                    ui.add(secondary_section(section));
                }
            }
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}

mod section;
