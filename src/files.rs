use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;

#[allow(clippy::ptr_arg)] // the whole function is just to here for making it easier to read
pub fn is_json(p: &PathBuf) -> bool {
    match p.extension() {
        Some(x) => x == "json",
        None => false,
    }
}

pub fn edit_with_editor(
    editor: OsString,
    files_complete_paths: Vec<OsString>,
) -> Result<(), Error> {
    Command::new(editor)
        .args(files_complete_paths)
        .status()
        .map_err(Error::EditorError)?;
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("editor returned error: {0}")]
    EditorError(std::io::Error),
}

pub trait ToFileName {
    fn to_file_name(&self) -> String;
}
