use std::{fmt::Display, path::PathBuf, sync::Arc};

use clap::ValueEnum;
use color_eyre::owo_colors::OwoColorize;
use time::{formatting::Formattable, OffsetDateTime};

use crate::files::ToFileName;

use super::Error;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Entry {
    #[serde(with = "time::serde::rfc3339")]
    pub at: OffsetDateTime,
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

// const FILE_NAME_FORMAT: &str = "[year]-[month]-[day]-[hour]-[minute]-[second].json";

impl ToFileName for Entry {
    type Error = Error;
    fn to_file_name(
        &self,
        time_format_descriptor_for_file_name: &(impl Formattable + ?Sized),
    ) -> Result<String, Self::Error> {
        Ok(self.at.format(time_format_descriptor_for_file_name)?)
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

        let entry: Entry = serde_json::from_str(&content)
            .map_err(|e| Error::FileCouldNotDeserializeEntryFromJson(e, content))?;
        Ok(entry)
    }
}

impl ToFileName for OffsetDateTime {
    type Error = Error;
    fn to_file_name(
        &self,
        time_format_descriptor: &(impl Formattable + ?Sized),
    ) -> Result<String, Self::Error> {
        Ok(self.format(time_format_descriptor)?)
    }
}
impl Entry {
    pub fn pretty_formated(
        &self,
        time_format_descriptor_for_display: &(impl Formattable + ?Sized),
    ) -> Result<String, Error> {
        let date = format!(
            "{}",
            self.at
                .format(&time_format_descriptor_for_display)?
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

        Ok(format!("{date}\n{body}\n{tags}"))
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
