use crate::files::is_json;
use crate::fold_or_err;
use crate::journal::entry::{Entry, ToFileName};
use crate::journal::Error;
use comfy_table::{Cell, ContentArrangement};
use std::path::PathBuf;
use std::sync::Arc;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Book {
    pub entries: Arc<[Entry]>,
}
impl Book {
    pub fn table_list(&self) -> String {
        let mut table = comfy_table::Table::new();
        table.load_preset(comfy_table::presets::NOTHING);

        table.set_content_arrangement(ContentArrangement::Dynamic);
        self.entries.iter().for_each(|entry| {
            table.add_row(vec![
                Cell::new(format!(
                    "{}",
                    entry.at.format(crate::time::DATE_DISPLAY_FORMATTING)
                ))
                .bg(comfy_table::Color::White)
                .fg(comfy_table::Color::Black),
                Cell::new(format!("{}", entry.body)),
                Cell::new(entry.to_file_name()).fg(comfy_table::Color::Blue),
                if entry.tag.is_empty() {
                    Cell::new("")
                } else {
                    Cell::new(
                        entry
                            .tag
                            .iter()
                            .fold("".to_string(), |accu, item| format!("{accu}, {item}")),
                    )
                    .fg(comfy_table::Color::Blue)
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
    type Error = super::Error;

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
#[cfg(test)]
mod testing {
    use super::*;

    fn is_normal<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    fn normal_types() {
        is_normal::<Book>();
    }
}
