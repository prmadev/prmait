use std::{fmt::Display, path::PathBuf, sync::Arc};

use chrono::{DateTime, Local};
use clap::ValueEnum;
use color_eyre::owo_colors::OwoColorize;

use super::Error;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Entry {
    pub at: DateTime<Local>,
    pub body: Arc<String>,
    pub tag: Vec<String>,
    pub mood: Mood,
    pub people: Vec<String>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default, ValueEnum)]
pub enum Mood {
    Good,
    Bad,
    #[default]
    Neutral,
}

impl Display for Mood {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Good => "Good",
                Self::Bad => "Bad",
                Self::Neutral => "Neutral",
            }
        )
    }
}

const FILE_NAME_FORMAT: &str = "%Y-%m-%d-%H-%M-%S.json";

impl ToFileName for Entry {
    fn to_file_name(&self) -> String {
        self.at.format(FILE_NAME_FORMAT).to_string()
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Entry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.at.cmp(&other.at)
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
impl Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let date = format!(
            "{}",
            self.at
                .format(crate::time::DATE_DISPLAY_FORMATTING)
                .to_string()
                .dimmed()
        );

        let body = format!("{}", self.body.bold());

        let tags = if self.tag.is_empty() {
            "".to_owned()
        } else {
            self.tag.iter().fold("".to_owned(), |accu, item| {
                format!("{}#{} ", accu, item.italic())
            })
        };

        write!(f, "{date}\n{body}\n{tags}")
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
