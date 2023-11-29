use super::Result;
use crate::files::{edit_with_editor, ToFileName};
use crate::git;
use crate::journal::entry::Entry;
use crate::journal::{Book, Error};
use color_eyre::owo_colors::OwoColorize;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, FuzzySelect};
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use time::formatting::Formattable;
use time::OffsetDateTime;

pub fn new_entry(
    entry: Entry,
    journal_path: &PathBuf,
    at: OffsetDateTime,
    time_format_descriptor_for_file_name: &Vec<time::format_description::FormatItem>,
) -> Result<()> {
    _ = fs_extra::dir::create(journal_path, false).map_err(Error::JournalDirCouldNotBeCreated);

    let file_name = at.to_file_name(time_format_descriptor_for_file_name)?;
    let file_path = journal_path.join(&file_name);

    if file_path.exists() {
        return Err(Error::JournalEntryFileAlreadyExists);
    }

    fs_extra::file::write_all(
        &file_path,
        &serde_json::to_string_pretty(&entry).map_err(Error::FileCouldNotSerializeEntryIntoJson)?,
    )
    .map_err(Error::FileCouldNotBeWrittenTo)?;

    let repo_root = git::repo_root(journal_path.clone()).map_err(Error::GitError)?;
    git::git_hook(
        repo_root.as_os_str().to_os_string(),
        vec![file_path.as_os_str().to_os_string()],
        &format!("feat(journal): add new journal entry {file_name}"),
    )
    .map_err(Error::GitError)?;

    Ok(())
}

pub fn list_entries(
    journal_path: &PathBuf,
    time_format_descriptor: &(impl Formattable + ?Sized),
) -> Result<()> {
    let book = Book::try_from(journal_path)?;

    println!("{}", book.table_list(time_format_descriptor)?);

    Ok(())
}

pub fn edit_last_entry(journal_path: &PathBuf, editor: OsString) -> Result<()> {
    let book = Book::try_from(journal_path)?;
    let file_name = &book.entries.last().ok_or(Error::NoEntries)?.file_name;

    let ent_path = journal_path.join(file_name).into_os_string();

    edit_with_editor(editor, vec![ent_path.clone()]).map_err(Error::EditorFailed)?;

    let repo_root = git::repo_root(journal_path.clone()).map_err(Error::GitError)?;
    git::git_hook(
        repo_root.as_os_str().to_os_string(),
        vec![ent_path.as_os_str().to_os_string()],
        &format!("feat(journal): edit the entry {file_name}"),
    )
    .map_err(Error::GitError)?;

    Ok(())
}

pub fn edit_specific_entry(
    journal_path: &PathBuf,
    specifier: String,
    editor: OsString,
) -> Result<()> {
    let book = Book::try_from(journal_path)?;

    let ent_path: Vec<PathBuf> = book
        .entries
        .iter()
        .filter(|x| x.file_name.contains(&specifier))
        .map(|ent| journal_path.join(&ent.file_name))
        .collect();

    edit_with_editor(
        editor,
        ent_path
            .iter()
            .map(|ent| ent.clone().into_os_string())
            .collect::<Vec<OsString>>(),
    )
    .map_err(Error::EditorFailed)?;

    let repo_root = git::repo_root(journal_path.clone()).map_err(Error::GitError)?;
    git::git_hook(
        repo_root.as_os_str().to_os_string(),
        vec![
            OsStr::new(&format!("{}/.", journal_path.as_os_str().to_string_lossy())).to_os_string(),
        ],
        "feat(journal): edit the few entries",
    )
    .map_err(Error::GitError)?;

    Ok(())
}
pub fn delete_interactive(
    journal_path: &PathBuf,
    truncation_amount: usize,
    time_format_descriptor_for_displaying: &(impl Formattable + ?Sized),
) -> Result<()> {
    let book = Book::try_from(journal_path)?;

    let options: Vec<String> = book
        .entries
        .iter()
        .map(|ent| {
            let mut truncated_body = ent.entry.body.to_string();
            truncated_body.truncate(truncation_amount);
            format!("{} -> {} ... ", ent.file_name, truncated_body)
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

    println!(
        "{}\n",
        selected
            .entry
            .pretty_formated(time_format_descriptor_for_displaying)?
    );

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
        let file_path = journal_path.join(&selected.file_name);
        fs_extra::remove_items(&[file_path]).map_err(Error::FileCouldNotBeDeleted)?;
    };

    Ok(())
}
pub fn edit_all_entries(journal_path: &PathBuf, editor: OsString) -> Result<()> {
    let book = Book::try_from(journal_path)?;

    let es = book
        .entries
        .iter()
        .map(|entry| journal_path.clone().join(&entry.file_name).into_os_string())
        .collect();

    edit_with_editor(editor, es).map_err(Error::EditorFailed)?;

    let repo_root = git::repo_root(journal_path.clone()).map_err(Error::GitError)?;
    git::git_hook(
        repo_root.as_os_str().to_os_string(),
        vec![
            OsStr::new(&format!("{}/.", journal_path.as_os_str().to_string_lossy())).to_os_string(),
        ],
        "feat(journal): edit bunch of entries",
    )
    .map_err(Error::GitError)?;
    Ok(())
}
