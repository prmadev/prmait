use crate::files::edit_with_editor;
use crate::journal::entry::{Entry, ToFileName};
use crate::journal::{Book, Error};
use color_eyre::owo_colors::OwoColorize;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, FuzzySelect};
use std::ffi::OsString;
use std::path::PathBuf;

pub fn new_entry(
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
pub fn list_entries(journal_path: &PathBuf) -> Result<(), Error> {
    let book = Book::try_from(journal_path)?;

    println!("{}", book.table_list());

    Ok(())
}
pub fn edit_last_entry(journal_path: &PathBuf, editor: OsString) -> Result<(), Error> {
    let book = Book::try_from(journal_path)?;

    let ent_path = journal_path
        .join(book.entries.last().ok_or(Error::NoEntries)?.to_file_name())
        .into_os_string();

    edit_with_editor(editor, vec![ent_path]).map_err(Error::EditorFailed)?;

    Ok(())
}

pub fn edit_specific_entry(
    journal_path: &PathBuf,
    specifier: String,
    editor: OsString,
) -> Result<(), Error> {
    let book = Book::try_from(journal_path)?;

    let ent_path: Vec<PathBuf> = book
        .entries
        .iter()
        .filter(|x| x.to_file_name().contains(&specifier))
        .map(|ent| journal_path.join(ent.to_file_name()))
        .collect();

    edit_with_editor(
        editor,
        ent_path
            .iter()
            .map(|ent| ent.clone().into_os_string())
            .collect::<Vec<OsString>>(),
    )
    .map_err(Error::EditorFailed)?;

    Ok(())
}
pub fn delete_interactive(journal_path: &PathBuf, truncation_amount: usize) -> Result<(), Error> {
    let book = Book::try_from(journal_path)?;

    let options: Vec<String> = book
        .entries
        .iter()
        .map(|ent| {
            let mut truncated_body = ent.body.to_string();
            truncated_body.truncate(truncation_amount);
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
        .interact()
        .map_err(Error::DialoguerError)?;

    println!("\n{}", "This is the selected item:".bold().red());

    let selected = book
        .entries
        .get(selection)
        .ok_or(Error::EntryCouldNotBeFound)?;

    println!("{selected}\n");

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
pub fn edit_all_entries(journal_path: &PathBuf, editor: OsString) -> Result<(), Error> {
    let book = Book::try_from(journal_path)?;

    let es = book
        .entries
        .iter()
        .map(ToFileName::to_file_name)
        .map(|file_name| journal_path.clone().join(file_name).into_os_string())
        .collect();

    edit_with_editor(editor, es).map_err(Error::EditorFailed)?;

    Ok(())
}
