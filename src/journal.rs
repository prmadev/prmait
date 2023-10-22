use std::{path::PathBuf, process::Command, sync::Arc};

use color_eyre::owo_colors::OwoColorize;
use comfy_table::{Cell, ContentArrangement};
use dialoguer::{theme::ColorfulTheme, Confirm, FuzzySelect};

use crate::input::Configs;

use self::entry::{Entry, ToFileName};

pub mod entry;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Book {
    pub enteries: Arc<[entry::Entry]>,
}

impl Book {
    pub fn table_list(&self) -> String {
        let mut table = comfy_table::Table::new();
        table.load_preset(comfy_table::presets::NOTHING);

        table.set_content_arrangement(ContentArrangement::Dynamic);
        self.enteries.iter().for_each(|entry| {
            table.add_row(vec![
                Cell::new(format!("{}", entry.at.format("%Y-%m-%d %H:%M:%S")))
                    .bg(comfy_table::Color::White)
                    .fg(comfy_table::Color::Black),
                Cell::new(format!("{}", entry.body)),
                Cell::new(entry.to_file_name()).fg(comfy_table::Color::Blue),
            ]);
        });

        table.to_string()
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

    let file_path = journal_path.join(at.to_file_name());

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

    println!("{}", Book::from(entries).table_list());

    Ok(())
}
pub fn edit_last_entry_handler(config: &Configs) -> Result<(), JournalEntryError> {
    let journal_path = config
        .journal_configs
        .clone()
        .ok_or(JournalEntryError::JournalDirDoesNotExist)?
        .journal_path
        .ok_or(JournalEntryError::JournalDirDoesNotExist)?;

    let s = fs_extra::dir::get_dir_content(&journal_path)
        .map_err(JournalEntryError::DirCouldNotBeRead)?;

    let mut entries = s
        .files
        .into_iter()
        .map(PathBuf::from)
        .filter(isjson)
        .map(Entry::try_from)
        .try_fold(vec![], fold_or_err)?;

    entries.sort();

    let ent_path = journal_path.join(
        entries
            .last()
            .ok_or(JournalEntryError::NoEntries)?
            .to_file_name(),
    );

    let editor = std::env::var_os("EDITOR").ok_or(JournalEntryError::EditorEnvNotSet)?;
    if editor.is_empty() {
        return Err(JournalEntryError::EditorEnvNotSet);
    }

    Command::new(editor)
        .arg(ent_path.clone().into_os_string())
        .status()
        .map_err(JournalEntryError::EditorError)?;

    Ok(())
}

pub fn edit_all_entries_handler(config: &Configs) -> Result<(), JournalEntryError> {
    let journal_path = config
        .journal_configs
        .clone()
        .ok_or(JournalEntryError::JournalDirDoesNotExist)?
        .journal_path
        .ok_or(JournalEntryError::JournalDirDoesNotExist)?;

    let s = fs_extra::dir::get_dir_content(&journal_path)
        .map_err(JournalEntryError::DirCouldNotBeRead)?;

    let mut entries = s
        .files
        .into_iter()
        .map(PathBuf::from)
        .filter(isjson)
        .map(Entry::try_from)
        .try_fold(vec![], fold_or_err)?;

    entries.sort();
    let es = entries
        .iter()
        .map(ToFileName::to_file_name)
        .map(|file_name| journal_path.clone().join(file_name).into_os_string());

    let editor = std::env::var_os("EDITOR").ok_or(JournalEntryError::EditorEnvNotSet)?;

    if editor.is_empty() {
        return Err(JournalEntryError::EditorEnvNotSet);
    }

    Command::new(editor)
        .args(es)
        .status()
        .map_err(JournalEntryError::EditorError)?;

    Ok(())
}
pub fn delete_interactive_handler(config: &Configs) -> Result<(), JournalEntryError> {
    let journal_path = config
        .journal_configs
        .clone()
        .ok_or(JournalEntryError::JournalDirDoesNotExist)?
        .journal_path
        .ok_or(JournalEntryError::JournalDirDoesNotExist)?;

    let s = fs_extra::dir::get_dir_content(&journal_path)
        .map_err(JournalEntryError::DirCouldNotBeRead)?;

    let entries = s
        .files
        .into_iter()
        .map(PathBuf::from)
        .filter(isjson)
        .map(Entry::try_from)
        .try_fold(vec![], fold_or_err)?;

    _ = entries;
    let options: Vec<String> = entries
        .iter()
        .map(|ent| {
            let mut truncated_body = ent.body.to_string();
            truncated_body.truncate(20);
            format!(
                "{} -> {} ... ",
                ToFileName::to_file_name(ent),
                truncated_body
            )
        })
        .collect();

    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("which file")
        .items(&options)
        .clear(true)
        .interact()
        .map_err(JournalEntryError::DialoguerError)?;

    println!();
    println!("{}", "This is the selected item:".bold().red());
    let selected = entries.get(selection).unwrap();
    println!(
        "{}",
        selected.at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed()
    );
    println!("{}", selected.body.bold());
    let tgs = match selected.tag.clone() {
        Some(t) => t.into_iter().fold("".to_owned(), |accu, item| {
            format!("{}#{} ", accu, item.italic())
        }),
        None => "".to_owned(),
    };
    println!("{tgs}");

    println!();
    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Are you {} you want to {} the above entry?",
            "absolutely sure".bold().red(),
            "delete".bold().red()
        ))
        .default(false)
        .interact()
        .map_err(JournalEntryError::DialoguerError)?
    {
        let file_path = journal_path.join(selected.to_file_name());
        fs_extra::remove_items(&[file_path]).map_err(JournalEntryError::FileCouldNotBeDeleted)?;
    };

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
    #[error("deleting file, failed: {0}")]
    FileCouldNotBeDeleted(fs_extra::error::Error),
    #[error("there is entry to be found")]
    NoEntries,
    #[error("editor returned error: {0}")]
    EditorError(std::io::Error),
    #[error("interactive tools failed: {0}")]
    DialoguerError(dialoguer::Error),
    #[error("editor env is not set")]
    EditorEnvNotSet,
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
