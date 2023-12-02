use super::Result;
use crate::effects::{
    CreateDirOpts, EffectKind, EffectMachine, FileWriterOpts, GitHookOpts, OpenInEditorOpts,
};
use crate::files::ToFileName;
use crate::journal::entry::Entry;
use crate::journal::{Book, Error};

use std::path::PathBuf;
use time::formatting::Formattable;
use time::OffsetDateTime;

pub fn new_entry(
    entry: Entry,
    journal_path: &PathBuf,
    at: OffsetDateTime,
    time_format_descriptor_for_file_name: &(impl Formattable + ?Sized),
) -> Result<EffectMachine> {
    let mut effects = EffectMachine::default();

    let file_name = at.to_file_name(time_format_descriptor_for_file_name)?;
    let file_path = journal_path.join(&file_name);

    effects.add(
        EffectKind::CreateDir(CreateDirOpts {
            folder_path: journal_path.to_owned(),
            ok_if_exists: true,
        }),
        false,
    );

    effects.add(
        EffectKind::WriteToFile(FileWriterOpts {
            content: serde_json::to_string_pretty(&entry)
                .map_err(|e| Error::FileCouldNotSerializeEntryIntoJson(e, file_name.clone()))?
                .as_bytes()
                .to_vec(),
            file_path: file_path.clone(),
            can_create: true,
            can_overwrite: false,
        }),
        false,
    );

    effects.add(
        EffectKind::GitHook(GitHookOpts {
            start_path: journal_path.to_owned(),
            files_to_add: vec![file_path],
            message: format!("feat(journal): add new journal entry {file_name}"),
        }),
        true,
    );

    Ok(effects)
}

pub fn list_entries(
    journal_path: &PathBuf,
    time_format_descriptor: &(impl Formattable + ?Sized),
) -> Result<()> {
    let book = Book::try_from(journal_path)?;

    println!("{}", book.table_list(time_format_descriptor)?);

    Ok(())
}

pub fn edit_last_entry(
    journal_path: &PathBuf,
    book: Book,
    editor: String,
) -> Result<EffectMachine> {
    let mut effects = EffectMachine::default();

    let file_name = &book.entries.last().ok_or(Error::NoEntries)?.file_name;
    let ent_path = journal_path.join(file_name);

    effects.add(
        EffectKind::OpenInEditor(OpenInEditorOpts {
            editor,
            files_to_edit: vec![ent_path.clone()],
        }),
        true,
    );

    effects.add(
        EffectKind::GitHook(GitHookOpts {
            start_path: journal_path.to_owned(),
            files_to_add: vec![journal_path.to_owned()],
            message: format!("feat(journal): edit the entry {file_name}"),
        }),
        true,
    );

    Ok(effects)
}

pub fn edit_specific_entry(
    journal_path: &PathBuf,
    specifier: String,
    book: Book,
    editor: String,
) -> Result<EffectMachine> {
    let mut effects = EffectMachine::default();

    let ent_path: Vec<PathBuf> = book
        .entries
        .iter()
        .filter(|x| x.file_name.contains(&specifier))
        .map(|ent| journal_path.join(&ent.file_name))
        .collect();

    effects.add(
        EffectKind::OpenInEditor(OpenInEditorOpts {
            editor,
            files_to_edit: ent_path,
        }),
        true,
    );

    effects.add(
        EffectKind::GitHook(GitHookOpts {
            start_path: journal_path.to_owned(),
            files_to_add: vec![journal_path.to_owned()],
            message: format!("feat(journal): edit the few entries"),
        }),
        true,
    );

    Ok(effects)
}

// pub fn delete_interactive(
//     journal_path: &PathBuf,
//     truncation_amount: usize,
//     book: Book,
//     time_format_descriptor_for_displaying: &(impl Formattable + ?Sized),
// ) -> Result<()> {
//     let options: Vec<String> = book.truncated_form(truncation_amount);

//     let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
//         .with_prompt("which file")
//         .items(&options)
//         .interact()
//         .map_err(Error::DialoguerError)?;

//     println!("\n{}", "This is the selected item:".bold().red());

//     let selected = book
//         .entries
//         .get(selection)
//         .ok_or(Error::EntryCouldNotBeFound)?;

//     println!(
//         "{}\n",
//         selected
//             .entry
//             .pretty_formated(time_format_descriptor_for_displaying)?
//     );

//     if Confirm::with_theme(&ColorfulTheme::default())
//         .with_prompt(format!(
//             "Are you {} you want to {} the above entry?",
//             "absolutely sure".bold().red(),
//             "delete".bold().red()
//         ))
//         .default(false)
//         .interact()
//         .map_err(Error::DialoguerError)?
//     {
//         let file_path = journal_path.join(&selected.file_name);
//         fs_extra::remove_items(&[file_path]).map_err(Error::FileCouldNotBeDeleted)?;
//     };

//     Ok(())
// }
pub fn edit_all_entries(
    journal_path: &PathBuf,
    editor: String,
    book: Book,
) -> Result<EffectMachine> {
    let mut effects = EffectMachine::default();

    effects.add(
        EffectKind::OpenInEditor(OpenInEditorOpts {
            editor,
            files_to_edit: book.files(),
        }),
        true,
    );

    effects.add(
        EffectKind::GitHook(GitHookOpts {
            start_path: journal_path.to_owned(),
            files_to_add: vec![journal_path.to_owned()],
            message: format!("feat(journal): edit bunch of entries"),
        }),
        true,
    );

    Ok(effects)
}
