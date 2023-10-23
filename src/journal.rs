use std::{ffi::OsString, path::PathBuf, process::Command, sync::Arc};

use color_eyre::owo_colors::OwoColorize;
use comfy_table::{Cell, ContentArrangement};
use dialoguer::{theme::ColorfulTheme, Confirm, FuzzySelect};

use self::entry::{Entry, ToFileName};

pub mod entry;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Book {
    pub entries: Arc<[entry::Entry]>,
}
const DATE_DISPLAY_FORMATTING: &str = "%Y-%m-%d %H:%M:%S";

impl Book {
    pub fn table_list(&self) -> String {
        let mut table = comfy_table::Table::new();
        table.load_preset(comfy_table::presets::NOTHING);

        table.set_content_arrangement(ContentArrangement::Dynamic);
        self.entries.iter().for_each(|entry| {
            table.add_row(vec![
                Cell::new(format!("{}", entry.at.format(DATE_DISPLAY_FORMATTING)))
                    .bg(comfy_table::Color::White)
                    .fg(comfy_table::Color::Black),
                Cell::new(format!("{}", entry.body)),
                Cell::new(entry.to_file_name()).fg(comfy_table::Color::Blue),
                match &entry.tag {
                    Some(tags) => Cell::new(
                        tags.iter()
                            .fold("".to_string(), |accu, item| format!("{accu}, {item}")),
                    )
                    .fg(comfy_table::Color::Blue),
                    None => Cell::new(""),
                },
            ]);
        });

        table.to_string()
    }
}
impl From<Vec<Entry>> for Book {
    fn from(entries: Vec<Entry>) -> Self {
        Book {
            entries: entries.into(),
        }
    }
}
impl TryFrom<&PathBuf> for Book {
    type Error = Error;

    fn try_from(value: &PathBuf) -> Result<Self, Self::Error> {
        let mut entries = fs_extra::dir::get_dir_content(value)
            .map_err(Error::DirCouldNotBeRead)?
            .files
            .into_iter()
            .map(PathBuf::from)
            .filter(is_json)
            .map(Entry::try_from)
            .try_fold(vec![], fold_or_err)?;
        entries.sort();
        Ok(Self::from(entries))
    }
}

pub fn new_journal_entry_handler(
    entry: Entry,
    journal_path: &PathBuf,
    at: chrono::DateTime<chrono::Local>,
) -> Result<(), Error> {
    _ = fs_extra::dir::create(journal_path, false).map_err(Error::JournalDirCouldNotBeCreated);

    let file_path = journal_path.join(at.to_file_name());

    if file_path.exists() {
        return Err(Error::JournalEntryFileAlreadyExists);
    }

    fs_extra::file::write_all(
        file_path,
        &serde_json::to_string_pretty(&entry).map_err(Error::FileCouldNotSerializeEntryIntoJson)?,
    )
    .map_err(Error::FileCouldNotBeWrittenTo)?;

    Ok(())
}
pub fn list_entries_handler(journal_path: &PathBuf) -> Result<(), Error> {
    let book = Book::try_from(journal_path)?;

    println!("{}", book.table_list());

    Ok(())
}
pub fn edit_last_entry_handler(journal_path: &PathBuf) -> Result<(), Error> {
    let book = Book::try_from(journal_path)?;

    let ent_path = journal_path.join(book.entries.last().ok_or(Error::NoEntries)?.to_file_name());

    let editor = std::env::var_os("EDITOR").ok_or(Error::EditorEnvNotSet)?;
    if editor.is_empty() {
        return Err(Error::EditorEnvNotSet);
    }

    Command::new(editor)
        .arg(ent_path.clone().into_os_string())
        .status()
        .map_err(Error::EditorError)?;

    Ok(())
}

pub fn edit_specific_entry_handler(journal_path: &PathBuf, specifier: String) -> Result<(), Error> {
    let book = Book::try_from(journal_path)?;

    let ent_path: Vec<PathBuf> = book
        .entries
        .iter()
        .filter(|x| x.to_file_name().contains(&specifier))
        .map(|ent| journal_path.join(ent.to_file_name()))
        .collect();

    let editor = std::env::var_os("EDITOR").ok_or(Error::EditorEnvNotSet)?;
    if editor.is_empty() {
        return Err(Error::EditorEnvNotSet);
    }

    Command::new(editor)
        .args(
            ent_path
                .iter()
                .map(|ent| ent.clone().into_os_string())
                .collect::<Vec<OsString>>(),
        )
        .status()
        .map_err(Error::EditorError)?;

    Ok(())
}

pub fn edit_all_entries_handler(journal_path: &PathBuf) -> Result<(), Error> {
    let book = Book::try_from(journal_path)?;

    let es = book
        .entries
        .iter()
        .map(ToFileName::to_file_name)
        .map(|file_name| journal_path.clone().join(file_name).into_os_string());

    let editor = std::env::var_os("EDITOR").ok_or(Error::EditorEnvNotSet)?;

    if editor.is_empty() {
        return Err(Error::EditorEnvNotSet);
    }

    Command::new(editor)
        .args(es)
        .status()
        .map_err(Error::EditorError)?;

    Ok(())
}

pub fn delete_interactive_handler(journal_path: &PathBuf) -> Result<(), Error> {
    let book = Book::try_from(journal_path)?;

    let options: Vec<String> = book
        .entries
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
        .map_err(Error::DialoguerError)?;

    println!();
    println!("{}", "This is the selected item:".bold().red());
    let selected = book.entries.get(selection).unwrap();
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
        .map_err(Error::DialoguerError)?
    {
        let file_path = journal_path.join(selected.to_file_name());
        fs_extra::remove_items(&[file_path]).map_err(Error::FileCouldNotBeDeleted)?;
    };

    Ok(())
}

#[allow(clippy::ptr_arg)] // the whole function is just to here for making it easier to read
fn is_json(p: &PathBuf) -> bool {
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
pub enum Error {
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
