use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

#[cfg(not(target_arch = "wasm32"))]
pub fn open_upload_dialog(
    sender: Sender<Vec<u8>>,
    (name, ext): (&'static str, &'static [&'static str]),
) {
    let file = rfd::FileDialog::new().add_filter(name, ext).pick_file();
    read_and_send(sender, file)
}

#[cfg(target_arch = "wasm32")]
pub fn open_upload_dialog(
    sender: Sender<Vec<u8>>,
    (name, ext): (&'static str, &'static [&'static str]),
) {
    wasm_bindgen_futures::spawn_local(async move {
        let file = rfd::AsyncFileDialog::new()
            .add_filter(name, ext)
            .pick_file()
            .await;
        if let Some(file) = file {
            sender.send(file.read().await).expect("failed to send file")
        } else {
            tracing::warn!("no file picked!")
        }
    })
}

fn read_and_send(sender: Sender<Vec<u8>>, file: Option<PathBuf>) {
    if let Some(file) = file {
        tracing::trace!(?file, "received file");
        let mut file = File::open(file).expect("could not open passed file");
        tracing::trace!(?file, "successfully opened file");
        let mut buf = Vec::new();
        let bytes_read = file.read_to_end(&mut buf).expect("failed to read file");
        tracing::trace!(bytes_read, "successfully read the file");
        sender.send(buf).expect("failed to send file")
    } else {
        tracing::warn!("no file picked!")
    }
}
