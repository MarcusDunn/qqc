use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, BTreeSet};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::slice::Iter;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};

use egui::{Context, Key, TextBuffer, Ui};
use tracing::{error, info};

use crate::app::interview::InterviewSwiper;
use crate::app::number_selector::number_changer;
use crate::app::section::{primary_section, secondary_section};

mod export;
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
    codes: Vec<Code>,
    /// a code the user has not added yet,
    code_builder: Code,
    /// receive files asynchronously
    #[serde(skip)]
    interview_channel: (Sender<Vec<u8>>, Receiver<Vec<u8>>),
    #[serde(skip)]
    codes_channel: (Sender<Vec<u8>>, Receiver<Vec<u8>>),
    settings_open: bool,
    export_codes_open: bool,
    export_interview_open: bool,
    speaker_builder: String,
}

impl QualityQualitativeCoding {
    pub(crate) fn handle_keyboard_shortcuts(&mut self, ctx: &Context) {
        let shortcut_map = &self.settings.shortcut_map;
        Self::handle_next(&mut self.interview, ctx, shortcut_map);
        Self::handle_prev(&mut self.interview, ctx, shortcut_map);
    }

    fn handle_prev(
        interview: &mut Option<InterviewSwiper>,
        ctx: &Context,
        shortcut_map: &BTreeMap<Action, Key>,
    ) {
        if ctx.input().key_pressed(
            shortcut_map
                .get(&Action::Prev)
                .copied()
                .unwrap_or_else(|| Action::Prev.default_key()),
        ) {
            if let Some(interview) = interview {
                interview.try_prev();
            }
        }
    }

    fn handle_next(
        interview: &mut Option<InterviewSwiper>,
        ctx: &Context,
        shortcut_map: &BTreeMap<Action, Key>,
    ) {
        if ctx.input().key_pressed(
            shortcut_map
                .get(&Action::Next)
                .copied()
                .unwrap_or_else(|| Action::Next.default_key()),
        ) {
            if let Some(interview) = interview {
                interview.try_next();
            }
        }
    }

    fn hash(target: &impl Hash) -> u64 {
        let mut hasher = DefaultHasher::new();
        target.hash(&mut hasher);
        hasher.finish()
    }

    fn set_key_for_action(settings: &mut Settings, ui: &mut Ui, action: &Action) {
        ui.group(|ui| {
            ui.label(format!(
                "{:?}: {:?}",
                action,
                settings
                    .shortcut_map
                    .get(action)
                    .copied()
                    .unwrap_or_else(|| action.default_key())
            ));
            if ui.button("change").clicked() {
                settings.setting_key = Some(*action)
            }
        });
    }
}

#[derive(
    serde::Deserialize, serde::Serialize, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone,
)]
enum Action {
    Next,
    Prev,
    SwapSpeaker,
}

impl Action {
    fn default_key(&self) -> Key {
        match self {
            Action::Next => Key::ArrowDown,
            Action::Prev => Key::ArrowUp,
            Action::SwapSpeaker => Key::ArrowRight,
        }
    }

    fn iter() -> Iter<'static, Action> {
        [Action::Next, Action::Prev, Action::SwapSpeaker].iter()
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct Settings {
    code_columns: usize,
    shortcut_map: BTreeMap<Action, Key>,
    setting_key: Option<Action>,
    context_before: usize,
    context_after: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            code_columns: 5,
            shortcut_map: BTreeMap::default(),
            setting_key: None,
            context_before: 1,
            context_after: 1,
        }
    }
}

impl Default for QualityQualitativeCoding {
    fn default() -> Self {
        Self {
            settings: Default::default(),
            interview: None,
            codes: Vec::default(),
            code_builder: Code {
                name: "".to_string(),
                description: "".to_string(),
            },
            interview_channel: channel(),
            codes_channel: channel(),
            settings_open: false,
            export_codes_open: false,
            export_interview_open: false,
            speaker_builder: "".to_string(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Code {
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
    codes: BTreeSet<usize>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct CsvSerializableSection {
    speaker: String,
    text: String,
    code0: Option<String>,
    code1: Option<String>,
    code2: Option<String>,
    code3: Option<String>,
    code4: Option<String>,
    code5: Option<String>,
    code6: Option<String>,
    code7: Option<String>,
    code8: Option<String>,
}

impl CsvSerializableSection {
    fn from_section(
        speakers: &BTreeMap<u64, String>,
        codes: &[Code],
        Section {
            speaker_id,
            codes: section_codes,
            text,
        }: &Section,
    ) -> CsvSerializableSection {
        let mut codes = section_codes
            .iter()
            .map(|id| codes.get(*id).unwrap())
            .map(|Code { name, .. }| name);
        CsvSerializableSection {
            speaker: speakers[speaker_id].clone(),
            text: text.clone(),
            code0: codes.next().cloned(),
            code1: codes.next().cloned(),
            code2: codes.next().cloned(),
            code3: codes.next().cloned(),
            code4: codes.next().cloned(),
            code5: codes.next().cloned(),
            code6: codes.next().cloned(),
            code7: codes.next().cloned(),
            code8: codes.next().cloned(),
        }
    }
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

    fn try_update_codes(codes: &mut Vec<Code>, codes_recv: &mut Receiver<Vec<u8>>) {
        match codes_recv.try_recv() {
            Ok(bytes) => {
                codes.clear();
                let mut reader = csv::Reader::from_reader(&bytes[..]);
                for record in reader.deserialize::<Code>() {
                    match record {
                        Ok(record) => codes.push(record),
                        Err(err) => {
                            error!(error = ?err, "failed to parse csv")
                        }
                    }
                }
            }
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
                match std::str::from_utf8(bytes.deref()) {
                    Ok(utf8str) => match parse_interview::parse(utf8str) {
                        Ok(parsed_interview) => {
                            tracing::trace!(?parsed_interview);
                            *interview = Some(InterviewSwiper::new(parsed_interview))
                        }
                        Err(err) => {
                            tracing::trace!(error = ?err, "failed to parse json");
                        }
                    },
                    Err(err) => {
                        tracing::warn!(error = ?err, "failed to parse utf8");
                    }
                };
            }
            Err(TryRecvError::Empty) => { /* no file has been uploaded yet - no problem! */ }
            Err(TryRecvError::Disconnected) => {
                panic!("impossible to upload files. sender has been dropped.")
            }
        }
    }

    fn add_new_code(codes: &mut Vec<Code>, code_builder: &mut Code) {
        let name = std::mem::take(&mut code_builder.name);
        let description = std::mem::take(&mut code_builder.description);
        codes.push(Code { name, description });
    }

    fn open_csv_upload_dialog(codes_tx: &mut Sender<Vec<u8>>) {
        file_upload::open_upload_dialog(codes_tx.clone(), ("csv", &["csv"]))
    }

    fn open_interview_upload_dialog(interview_tx: &mut Sender<Vec<u8>>) {
        file_upload::open_upload_dialog(
            interview_tx.clone(),
            ("interview", parse_interview::file_extensions()),
        )
    }

    fn get_next_speaker_id(speakers: &BTreeMap<u64, String>, current: u64) -> u64 {
        let keys = speakers.keys().collect::<Vec<_>>();
        let split = keys.split(|k| **k == current).collect::<Vec<_>>();
        debug_assert!(split.len() == 2, "current was not in speakers");
        let new_speaker_id = **split[1]
            .first()
            .or_else(|| split[0].first())
            .unwrap_or(&&current);
        info!(current, new_speaker_id, "changed speaker_id");
        new_speaker_id
    }
}

impl eframe::App for QualityQualitativeCoding {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        self.handle_keyboard_shortcuts(ctx);

        let Self {
            interview,
            codes,
            code_builder,
            interview_channel: (interview_tx, interview_rx),
            codes_channel: (codes_tx, codes_rx),
            settings,
            settings_open,
            export_codes_open,
            export_interview_open,
            speaker_builder,
        } = self;

        Self::try_update_interview(interview, interview_rx);
        Self::try_update_codes(codes, codes_rx);

        egui::Window::new("export codes")
            .open(export_codes_open)
            .show(ctx, |ui| export::export_codes(codes, ui));

        egui::Window::new("export interview")
            .open(export_interview_open)
            .show(ctx, |ui| match interview {
                None => ui.label("nothing to export"),
                Some(InterviewSwiper { interview, .. }) => {
                    export::export_interview(codes, ui, interview)
                }
            });

        egui::Window::new("settings")
            .open(settings_open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("aesthetics");
                    ui.group(|ui| {
                        ui.label("Codes per row");
                        ui.add(number_changer(&mut settings.code_columns))
                    });
                    ui.group(|ui| {
                        ui.label("number of segments before");
                        ui.add(number_changer(&mut settings.context_before))
                    });
                    ui.group(|ui| {
                        ui.label("number of segments after");
                        ui.add(number_changer(&mut settings.context_after))
                    });
                });
                ui.add_space(20.0);
                ui.horizontal(|ui| {
                    ui.heading("shortcuts");
                    ui.group(|ui| {
                        if let Some(action) = settings.setting_key {
                            ui.label(format!("setting {:?}", action));
                            if ui.button("cancel").clicked() {
                                settings.setting_key = None;
                            }
                            if let Some(key) = ctx.input().keys_down.iter().next() {
                                settings.shortcut_map.insert(action, *key);
                                settings.setting_key = None;
                            }
                        }
                        for action in Action::iter() {
                            Self::set_key_for_action(settings, ui, action);
                        }
                        if ui
                            .button("reset")
                            .on_hover_text("reset the shortcuts to their defaults")
                            .clicked()
                        {
                            for action in Action::iter() {
                                settings.shortcut_map.insert(*action, action.default_key());
                            }
                        }
                    });
                });
                ui.add_space(20.0);
                ui.horizontal(|ui| {
                    ui.heading("the danger zone");

                    if ui
                        .button("reset")
                        .on_hover_text("delete this invterview and start coding a new one")
                        .clicked()
                    {
                        *interview = None;
                    }
                })
            });

        egui::TopBottomPanel::top("top bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let export_menu_button = ui.menu_button("export", |ui| {
                    if !codes.is_empty() && ui.button("codes").clicked() {
                        *export_codes_open = true;
                    }
                    if interview.is_some() && ui.button("interview").clicked() {
                        *export_interview_open = true;
                    }
                });
                if codes.is_empty() && interview.is_none() {
                    export_menu_button
                        .response
                        .on_hover_text("nothing to export");
                }
                ui.menu_button("import", |ui| {
                    if ui.button("codes").clicked() {
                        Self::open_csv_upload_dialog(codes_tx);
                    }
                });
                if ui.button("settings").clicked() {
                    *settings_open = true;
                }
            });
        });

        egui::SidePanel::right("codes edit and create").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Codes");
                if codes.is_empty() {
                    ui.label("no codes at the moment, try adding one or importing");
                }
                let mut codes_to_be_removed = Vec::new();
                for (idx, Code { name, description }) in codes.iter_mut().enumerate() {
                    ui.group(|ui| {
                        ui.text_edit_singleline(name);
                        ui.text_edit_singleline(description);
                        if ui.button("remove").clicked() {
                            codes_to_be_removed.push(idx)
                        }
                    });
                }
                codes_to_be_removed.reverse();
                for idx in codes_to_be_removed {
                    codes.remove(idx);
                    if let Some(interview) = interview {
                        for section in &mut interview.interview.sections {
                            section.codes.remove(&idx);
                        }
                    }
                }
                ui.heading("New Code");
                ui.label("name");
                ui.text_edit_singleline(&mut code_builder.name);
                ui.label("description");
                ui.text_edit_singleline(&mut code_builder.description);
                if ui.button("add new code").clicked() {
                    info!(code = ?code_builder, "adding new code");
                    Self::add_new_code(codes, code_builder);
                }
            });
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
                });
                ui.group(|ui| {
                    ui.label("speakers");
                    for name in interview.interview.speakers.values_mut() {
                        ui.text_edit_singleline(name);
                    }
                    ui.label("new speaker");
                    ui.text_edit_singleline(speaker_builder);
                    if ui.button("add").clicked() {
                        interview
                            .interview
                            .speakers
                            .insert(Self::hash(speaker_builder), speaker_builder.take());
                    }
                });
            });

            egui::TopBottomPanel::bottom("codes select").show(ctx, |ui| {
                let current = interview.current_mut();
                egui::Grid::new("code grid").show(ui, |ui| {
                    for (idx, Code { name, description }) in codes.iter().enumerate() {
                        if idx != 0 && idx % settings.code_columns == 0 {
                            ui.end_row()
                        }
                        let checked = &mut current.codes.contains(&idx);
                        let checkbox = ui.checkbox(checked, name).on_hover_text(description);
                        if checkbox.changed() {
                            if *checked {
                                current.codes.insert(idx);
                            } else {
                                current.codes.remove(&idx);
                            }
                        }
                    }
                });
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| match interview {
            None => {
                ui.heading("quality qualitative coding");
                ui.label("welcome to quality qualitative coding. To start, upload an interview. If you run into any issues or have any suggestion please let Marcus know on the issue tracker.");
                ui.hyperlink_to("github", "https://github.com/marcusdunn/qqc");
                if ui.button("Upload interview").clicked() {
                    Self::open_interview_upload_dialog(interview_tx);
                }
            }
            Some(interview) => {
                ui.horizontal(|ui| {
                    ui.heading("Coding!");
                    ui.weak(format!("{} of {}", interview.index, interview.interview.sections.len()));
                });
                let (before, curr, after) = InterviewSwiper::window_mut(
                    &mut interview.interview.sections,
                    interview.index,
                    settings.context_before,
                    settings.context_after,
                );
                for section in before {
                    let section_response = ui.add(secondary_section(
                        section,
                        &interview.interview.speakers[&section.speaker_id],
                    ));
                    if section_response.clicked() {}
                }

                let primary_section = ui.add(primary_section(
                    curr,
                    &interview.interview.speakers[&curr.speaker_id],
                ));
                if ctx.input().key_pressed(
                    settings
                        .shortcut_map
                        .get(&Action::SwapSpeaker)
                        .copied()
                        .unwrap_or(Key::ArrowRight),
                ) {
                    curr.speaker_id =
                        Self::get_next_speaker_id(&interview.interview.speakers, curr.speaker_id)
                }
                if primary_section.clicked() {
                    curr.speaker_id =
                        Self::get_next_speaker_id(&interview.interview.speakers, curr.speaker_id)
                }
                for section in after {
                    ui.add(secondary_section(
                        section,
                        &interview.interview.speakers[&section.speaker_id],
                    ));
                }
            }
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}

mod number_selector;
mod section;
