use std::{path::PathBuf, sync::Arc};

use crate::input::Configs;

use self::entry::Entry;

pub mod entry;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Book {
    pub enteries: Arc<[entry::Entry]>,
}

impl Book {
    pub fn simple_list(&self) -> Box<[String]> {
        let s: Box<[String]> = self
            .enteries
            .iter()
            .map(|entry| format!("{}\t{}", entry.at.format("%Y-%m-%d  %H:%M:%S"), entry.body))
            .collect();
        s
    }
}
impl From<Vec<Entry>> for Book {
    fn from(entries: Vec<Entry>) -> Self {
        Book {
            enteries: entries.into(),
        }
    }
}

pub fn new_journal_entry_handler(
    entry: Entry,
    config: &Configs,
    at: chrono::DateTime<chrono::Local>,
) -> Result<(), JournalEntryError> {
    let journal_path = config
        .journal_configs
        .clone()
        .ok_or(JournalEntryError::JournalDirDoesNotExist)?
        .journal_path
        .ok_or(JournalEntryError::JournalDirDoesNotExist)?;

    _ = fs_extra::dir::create(&journal_path, false)
        .map_err(JournalEntryError::JournalDirCouldNotBeCreated);

    let file_path = journal_path.join(at.format("%Y-%m-%d-%H-%M-%S.json").to_string());

    if file_path.exists() {
        return Err(JournalEntryError::JournalEntryFileAlreadyExists);
    }

    fs_extra::file::write_all(
        file_path,
        &serde_json::to_string_pretty(&entry)
            .map_err(JournalEntryError::FileCouldNotSerializeEntryIntoJson)?,
    )
    .map_err(JournalEntryError::FileCouldNotBeWrittenTo)?;

    Ok(())
}
pub fn list_entries_handler(config: &Configs) -> Result<(), JournalEntryError> {
    let journal_path = config
        .journal_configs
        .clone()
        .ok_or(JournalEntryError::JournalDirDoesNotExist)?
        .journal_path
        .ok_or(JournalEntryError::JournalDirDoesNotExist)?;

    let s = fs_extra::dir::get_dir_content(journal_path)
        .map_err(JournalEntryError::DirCouldNotBeRead)?;

    let mut entries = s
        .files
        .into_iter()
        .map(PathBuf::from)
        .filter(isjson)
        .map(Entry::try_from)
        .try_fold(vec![], fold_or_err)?;
    entries.sort();

    Book::from(entries)
        .simple_list()
        .iter()
        .for_each(|i| println!("{i}"));

    Ok(())
}

#[allow(clippy::ptr_arg)] // the whole function is just to here for making it easier to read
fn isjson(p: &PathBuf) -> bool {
    match p.extension() {
        Some(x) => x == "json",
        None => false,
    }
}

fn fold_or_err<T, E>(mut accu: Vec<T>, item: Result<T, E>) -> Result<Vec<T>, E> {
    accu.push(item?);
    Ok(accu)
}

#[derive(Debug, thiserror::Error)]
pub enum JournalEntryError {
    #[error("The path to the journal directory is not given")]
    JournalDirDoesNotExist,
    #[error("could not create journal directory")]
    JournalDirCouldNotBeCreated(fs_extra::error::Error),
    #[error("could not create journal directory")]
    JournalEntryFileAlreadyExists,
    #[error("could serialize entry to json: {0}")]
    FileCouldNotSerializeEntryIntoJson(serde_json::Error),
    #[error("could deserialize entry from json: {0}")]
    FileCouldNotDeserializeEntryFromJson(serde_json::Error),
    #[error("could not write entry to file: {0}")]
    FileCouldNotBeWrittenTo(fs_extra::error::Error),
    #[error("could not read directory: {0}")]
    DirCouldNotBeRead(fs_extra::error::Error),
    #[error("file content could not be read: {0}")]
    FileCouldNotBeRead(fs_extra::error::Error),
}

#[cfg(test)]
pub mod testing {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    fn is_normal<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    fn normal_types() {
        is_normal::<Book>();
    }
}
