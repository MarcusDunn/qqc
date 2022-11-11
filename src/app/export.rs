use crate::app::{Code, CsvSerializableSection, Interview};
use csv::Writer;
use egui::{Response, Ui};
use serde::Serialize;
use std::error::Error;
use std::fs::File;
use std::io;
use tracing::warn;

#[cfg(target_arch = "wasm32")]
fn to_data_url_csv<T: Serialize>(iter: impl Iterator<Item = T>) -> Result<String, Box<dyn Error>> {
    let writer = to_csv(Vec::new(), iter);
    Ok(String::from("data:text/csv,")
        + &urlencoding::encode(String::from_utf8(writer?.into_inner()?)?.as_str()))
}

fn to_csv<W: io::Write, I: Serialize>(
    write: W,
    mut iterator: impl Iterator<Item = I>,
) -> Result<Writer<W>, csv::Error> {
    let mut writer = Writer::from_writer(write);
    iterator.try_for_each(|record| writer.serialize(record))?;
    Ok(writer)
}
#[cfg(target_arch = "wasm32")]
fn export_web(
    codes: &[Code],
    ui: &mut Ui,
    Interview { speakers, sections }: &Interview,
) -> Response {
    match to_data_url_csv(
        sections
            .iter()
            .map(|section| CsvSerializableSection::from_section(speakers, codes, section)),
    ) {
        Ok(data_url) => ui.hyperlink_to("download csv", data_url),
        Err(err) => {
            warn!(err, "failed to turn interview to data url");
            ui.label("failed")
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn export_native(codes: &[Code], ui: &mut Ui, interview: &Interview) -> Response {
    match write_to_file(codes, interview) {
        Ok(()) => ui.label("wrote to file"),
        Err(err) => {
            warn!(err, "failed to write to file");
            ui.label("failed to write to file")
        }
    }
}

fn write_to_file(
    codes: &[Code],
    Interview { speakers, sections }: &Interview,
) -> Result<(), Box<dyn Error>> {
    let file = File::options()
        .create(true)
        .write(true)
        .open("export_interview.csv")?;
    Ok(to_csv(
        file,
        sections
            .iter()
            .map(|section| CsvSerializableSection::from_section(speakers, codes, section)),
    )?
    .flush()?)
}

pub fn export_interview(codes: &[Code], ui: &mut Ui, interview: &Interview) -> Response {
    #[cfg(target_arch = "wasm32")]
    return export_web(codes, ui, interview);
    #[cfg(not(target_arch = "wasm32"))]
    return export_native(codes, ui, interview);
}

pub fn export_codes(codes: &[Code], ui: &mut Ui) -> Response {
    #[cfg(target_arch = "wasm32")]
    return export_codes_web(codes, ui);
    #[cfg(not(target_arch = "wasm32"))]
    return export_codes_native(codes, ui);
}

fn write_codes_to_file(codes: &[Code]) -> Result<(), Box<dyn Error>> {
    let file = File::options()
        .create(true)
        .write(true)
        .open("export_codes.csv")?;
    Ok(to_csv(file, codes.iter())?.flush()?)
}

fn export_codes_native(codes: &[Code], ui: &mut Ui) -> Response {
    match write_codes_to_file(codes) {
        Ok(()) => ui.label("created file export_codes.csv"),
        Err(err) => {
            warn!(?err);
            ui.label("failed to write codes to file")
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn export_codes_web(codes: &[Code], ui: &mut Ui) -> Response {
    match to_data_url_csv(codes.iter()) {
        Ok(data_url) => ui.hyperlink_to("download csv", data_url),
        Err(err) => {
            warn!(err, "failed to turn interview to data url");
            ui.label("failed")
        }
    }
}
