use crate::files::is_json;
use crate::fold_or_err;
use crate::journal::entry::Entry;
use crate::journal::Error;
use comfy_table::{Cell, ContentArrangement};
use std::path::PathBuf;
use std::sync::Arc;
use time::formatting::Formattable;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Book {
    pub entries: Arc<[EntryDescription]>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EntryDescription {
    pub entry: Entry,
    pub file_name: String,
}

impl TryFrom<PathBuf> for EntryDescription {
    type Error = Error;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let file_name = value
            .file_name()
            .ok_or(Error::IsNotAFile)?
            .to_str()
            .ok_or(Error::FileNameHasInvalidCharacters)?
            .to_owned();
        Ok(EntryDescription {
            entry: Entry::try_from(value)?,
            file_name,
        })
    }
}
impl Book {
    pub fn table_list(
        &self,
        time_format_descriptor: &(impl Formattable + ?Sized),
    ) -> Result<String, Error> {
        let mut table = comfy_table::Table::new();
        table.load_preset(comfy_table::presets::NOTHING);

        table.set_content_arrangement(ContentArrangement::Dynamic);
        self.entries
            .iter()
            .fold(Ok(()), |accu, entry| -> Result<(), Error> {
                let Ok(()) = accu else { return accu };
                table.add_row(vec![
                    Cell::new((entry.entry.at.format(time_format_descriptor)?).to_string())
                    .bg(comfy_table::Color::White)
                    .fg(comfy_table::Color::Black),
                    Cell::new(format!("{}", &entry.entry.body)),
                    Cell::new(&entry.file_name).fg(comfy_table::Color::Blue),
                    if entry.entry.tag.is_empty() {
                        Cell::new("")
                    } else {
                        Cell::new(
                            entry
                                .entry
                                .tag
                                .iter()
                                .fold("".to_string(), |accu, item| format!("{accu}, {item}")),
                        )
                        .fg(comfy_table::Color::Blue)
                    },
                ]);
                Ok(())
            })?;

        Ok(table.to_string())
    }
}

impl From<Vec<EntryDescription>> for Book {
    fn from(entries: Vec<EntryDescription>) -> Self {
        Book {
            entries: entries.into(),
        }
    }
}
impl TryFrom<&PathBuf> for Book {
    type Error = super::Error;

    fn try_from(value: &PathBuf) -> Result<Self, Self::Error> {
        let mut entries = fs_extra::dir::get_dir_content(value)
            .map_err(Error::DirCouldNotBeRead)?
            .files
            .into_iter()
            .map(PathBuf::from)
            .filter(is_json)
            .map(EntryDescription::try_from)
            .try_fold(vec![], fold_or_err)?;
        entries.sort();
        Ok(Self::from(entries))
    }
}
#[cfg(test)]
mod testing {
    use super::*;

    fn is_normal<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    fn normal_types() {
        is_normal::<Book>();
    }
}
