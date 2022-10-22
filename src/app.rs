use std::collections::{BTreeMap, BTreeSet};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::time::Duration;

use egui::Direction;
use egui_toast::Toasts;

mod file_upload;
mod parse_interview;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct QualityQualitativeCoding {
    /// the interview itself
    interview: Option<Interview>,
    /// the codes to choose from
    codes: BTreeSet<Code>,
    /// receive files asynchronously
    #[serde(skip)]
    file_channel: (Sender<Vec<u8>>, Receiver<Vec<u8>>),
}

impl Default for QualityQualitativeCoding {
    fn default() -> Self {
        Self {
            interview: None,
            codes: BTreeSet::default(),
            file_channel: channel(),
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
    codes: BTreeSet<String>,
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
}

impl eframe::App for QualityQualitativeCoding {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        let mut toasts = Toasts::new()
            .anchor(ctx.available_rect().right_bottom())
            .direction(Direction::BottomUp);

        let Self {
            interview,
            codes,
            file_channel: (sender, receiver),
        } = self;

        match receiver.try_recv() {
            Ok(bytes) => {
                match parse_interview::from_json_slice(&*bytes) {
                    Ok(parsed_interview) => {
                        tracing::trace!("parsed interview");
                        *interview = Some(parsed_interview)
                    }
                    Err(err) => {
                        tracing::trace!(error = ?err, "failed to parse json");
                        toasts.error(
                            format!("Could not parse JSON {}", err),
                            Duration::from_secs(3),
                        );
                    }
                };
            }
            Err(TryRecvError::Empty) => { /* no file has been uploaded yet - no problem! */ }
            Err(TryRecvError::Disconnected) => {
                panic!("impossible to upload files. sender has been dropped.")
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| match interview {
            None => {
                if ui.button("Upload interview").clicked() {
                    file_upload::open_upload_dialog(sender.clone())
                }
            }
            Some(_) => {
                ui.heading("coding interview");
            }
        });

        toasts.show(ctx);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
