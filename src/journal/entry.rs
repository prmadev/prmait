use std::{path::PathBuf, sync::Arc};

use chrono::{DateTime, Local};

use super::Error;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Entry {
    pub at: DateTime<Local>,
    pub body: Arc<String>,
    pub tag: Option<Vec<String>>,
}

const FILE_NAME_FORMAT: &str = "%Y-%m-%d-%H-%M-%S.json";

impl ToFileName for Entry {
    fn to_file_name(&self) -> String {
        self.at.format(FILE_NAME_FORMAT).to_string()
    }
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
    type Error = Error;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let content = fs_extra::file::read_to_string(value).map_err(Error::FileCouldNotBeRead)?;

        let entry: Entry =
            serde_json::from_str(&content).map_err(Error::FileCouldNotDeserializeEntryFromJson)?;
        Ok(entry)
    }
}

pub trait ToFileName {
    fn to_file_name(&self) -> String;
}

impl ToFileName for DateTime<Local> {
    fn to_file_name(&self) -> String {
        self.format(FILE_NAME_FORMAT).to_string()
    }
}
