use std::{path::PathBuf, sync::Arc};

use chrono::{DateTime, Local};

use super::JournalEntryError;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Entry {
    pub at: DateTime<Local>,
    pub body: Arc<String>,
    pub tag: Option<Vec<String>>,
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.at.partial_cmp(&other.at)
    }
}
impl Ord for Entry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.at.cmp(&other.at)
    }
}

#[cfg(test)]
mod testing {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    fn is_normal<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    fn normal_types() {
        is_normal::<Entry>();
    }
}

impl TryFrom<PathBuf> for Entry {
    type Error = JournalEntryError;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let content =
            fs_extra::file::read_to_string(value).map_err(JournalEntryError::FileCouldNotBeRead)?;

        let entry: Entry = serde_json::from_str(&content)
            .map_err(JournalEntryError::FileCouldNotDeserializeEntryFromJson)?;
        Ok(entry)
    }
}
