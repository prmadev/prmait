use super::Result;
use crate::effects::{CreateDirOpts, EffectKind, EffectMachine, FileWriterOpts, OpenInEditorOpts};
use crate::files::ToFileName;
use crate::git;
use crate::journal::entry::Entry;
use crate::journal::{Book, Error};

use std::borrow::Cow;
use std::path::{Path, PathBuf};
use time::formatting::Formattable;
use time::OffsetDateTime;

pub fn new_entry(
    entry: &Entry,
    journal_path: &PathBuf,
    repo_root: &str,
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

    let fp = file_path.to_string_lossy().into_owned();

    effects.add(git::add(repo_root, &[fp]), false);
    effects.add(
        git::commit(
            repo_root,
            &format!("feat(journal): add new journal entry {file_name}"),
        ),
        false,
    );
    effects.add(git::pull(repo_root), false);
    effects.add(git::push(repo_root), false);

    Ok(effects)
}

pub fn list_entries(
    book: &Book,
    time_format_descriptor: &(impl Formattable + ?Sized),
) -> Result<EffectMachine> {
    let mut efs = EffectMachine::default();

    efs.add(
        EffectKind::PrintToStdOut(book.table_list(time_format_descriptor)?.clone()),
        false,
    );

    Ok(efs)
}

pub fn edit_last_entry(
    journal_path: &Path,
    book: &Book,
    repo_root: &str,
    editor: String,
) -> Result<EffectMachine> {
    let mut effects = EffectMachine::default();

    let last_entry = &book.entries.last().ok_or(Error::NoEntries)?;
    let file_name = &last_entry.file_name;
    let ent_path = journal_path.join(file_name);

    effects.add(
        EffectKind::OpenInEditor(OpenInEditorOpts {
            editor,
            files_to_edit: vec![ent_path.clone()],
        }),
        true,
    );

    let fp = ent_path.to_string_lossy().into_owned();

    effects.add(git::add(repo_root, &[fp]), false);
    effects.add(
        git::commit(
            repo_root,
            &format!("feat(journal): edit the entry {file_name}"),
        ),
        false,
    );
    effects.add(git::pull(repo_root), false);
    effects.add(git::push(repo_root), false);

    Ok(effects)
}

pub fn edit_specific_entry(
    journal_path: &Path,
    specifier: &str,
    book: &Book,
    repo_root: &str,
    editor: String,
) -> Result<EffectMachine> {
    let mut effects = EffectMachine::default();

    let ent_path: Vec<PathBuf> = book
        .entries
        .iter()
        .filter(|x| x.file_name.contains(specifier))
        .map(|ent| journal_path.join(&ent.file_name))
        .collect();

    effects.add(
        EffectKind::OpenInEditor(OpenInEditorOpts {
            editor,
            files_to_edit: ent_path.clone(),
        }),
        true,
    );

    let fp: Vec<String> = ent_path
        .iter()
        .map(|x| Path::to_string_lossy(x))
        .map(Cow::into_owned)
        .collect();

    effects.add(git::add(repo_root, &fp), false);
    effects.add(
        git::commit(repo_root, "feat(journal): edit the few entries"),
        false,
    );
    effects.add(git::pull(repo_root), false);
    effects.add(git::push(repo_root), false);

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
pub fn edit_all_entries(editor: String, book: &Book, repo_root: &str) -> Result<EffectMachine> {
    let mut effects = EffectMachine::default();

    effects.add(
        EffectKind::OpenInEditor(OpenInEditorOpts {
            editor,
            files_to_edit: book.files(),
        }),
        true,
    );

    effects.add(git::add(repo_root, &[repo_root.to_owned()]), false);
    effects.add(
        git::commit(repo_root, "feat(journal): edit the few entries"),
        false,
    );
    effects.add(git::pull(repo_root), false);
    effects.add(git::push(repo_root), false);

    Ok(effects)
}

#[cfg(test)]
mod testing {
    use std::sync::Arc;

    use crate::journal::Mood;

    #[allow(clippy::wildcard_imports)]
    use super::*;
    use rstest::*;
    use time::format_description::well_known;

    #[fixture]
    fn entry() -> Entry {
        let now = time::OffsetDateTime::now_utc();
        let body = Arc::new(String::from("body"));
        let mood = Mood::Good;
        let tag = vec!["tag1".to_owned(), "tag2".to_owned()];
        let people = vec!["hoverbear".to_owned(), "mr_leafslug".to_owned()];

        Entry {
            at: now,
            body,
            tag,
            mood,
            people,
        }
    }
    #[fixture]
    fn journal_path() -> PathBuf {
        PathBuf::new()
    }

    #[rstest]
    #[case("somerepo", time::OffsetDateTime::now_utc())]
    fn test_new(
        entry: Entry,
        journal_path: PathBuf,
        #[case] repo_root: &str,
        #[case] at: OffsetDateTime,
    ) {
        let em = new_entry(&entry, &journal_path, &repo_root, at, &well_known::Rfc3339).unwrap();
        assert_eq!(em.0.len(), 6);
    }
}
