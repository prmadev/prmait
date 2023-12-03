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
    pub location: PathBuf,
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
        Ok(Self {
            entry: Entry::try_from(value).map_err(|e| {
                Error::CouldNotDeserializeEntryFromJson(Box::new(e), file_name.clone())
            })?,
            file_name,
        })
    }
}
impl Book {
    #[must_use]
    pub fn files(&self) -> Vec<PathBuf> {
        self.entries
            .iter()
            .map(|entry| self.location.join(&entry.file_name))
            .collect()
    }
    #[must_use]
    pub fn truncated_form(&self, truncate_at: usize) -> Vec<String> {
        self.entries
            .iter()
            .map(|ent| {
                let mut truncated_body = ent.entry.body.to_string();
                truncated_body.truncate(truncate_at);
                format!("{} -> {} ... ", ent.file_name, truncated_body)
            })
            .collect()
    }
    pub fn table_list(
        &self,
        time_format_descriptor: &(impl Formattable + ?Sized),
    ) -> Result<String, Error> {
        let mut table = comfy_table::Table::new();
        table.load_preset(comfy_table::presets::NOTHING);

        table.set_content_arrangement(ContentArrangement::Dynamic);
        self.entries
            .iter()
            .try_fold((), |(), entry_desc| -> Result<(), Error> {
                let bg_color = match entry_desc.entry.mood {
                    super::Mood::Good => comfy_table::Color::Green,
                    super::Mood::Bad => comfy_table::Color::Red,
                    super::Mood::Neutral => comfy_table::Color::White,
                };
                table.add_row(vec![
                    Cell::new((entry_desc.entry.at.format(time_format_descriptor)?).clone())
                        .bg(bg_color)
                        .fg(comfy_table::Color::Black),
                    Cell::new(format!("{}", &entry_desc.entry.body)).fg(bg_color),
                    // Cell::new(&entry.file_name).fg(comfy_table::Color::Blue),
                    // if entry.entry.tag.is_empty() {
                    //     Cell::new("")
                    // } else {
                    //     Cell::new(
                    //         entry
                    //             .entry
                    //             .tag
                    //             .iter()
                    //             .fold(String::new(), |accu, item| format!("{accu}#{item} "))
                    //             .italic(),
                    //     )
                    //     .fg(comfy_table::Color::Blue)
                    // },
                ]);
                Ok(())
            })?;

        Ok(table.to_string())
    }
}

impl From<(Vec<EntryDescription>, PathBuf)> for Book {
    fn from((entries, location): (Vec<EntryDescription>, PathBuf)) -> Self {
        Self {
            entries: entries.into(),
            location,
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
        Ok(Self::from((entries, value.clone())))
    }
}
#[cfg(test)]
mod testing {
    use super::*;

    const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn normal_types() {
        is_normal::<Book>();
    }
}
