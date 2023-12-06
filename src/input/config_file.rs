use figment::providers::{Env, Format, Json};
use figment::Figment;
use std::path::PathBuf;

// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
// #[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
// pub struct Configs {
//     pub river: Option<Configs>,
// }

impl TryFrom<PathBuf> for Configs {
    type Error = ConfigErr;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        Ok(Figment::new()
            .merge(Json::file(value))
            .merge(Env::prefixed("PRMA_IT_"))
            .extract()?)
    }
}
impl TryFrom<&PathBuf> for Configs {
    type Error = ConfigErr;

    fn try_from(value: &PathBuf) -> Result<Self, Self::Error> {
        Ok(Figment::new()
            .merge(Json::file(value))
            .merge(Env::prefixed("PRMA_IT_"))
            .extract()?)
    }
}
#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum ConfigErr {
    #[error("could not extract configuration: {0}")]
    ExtractionFailed(#[from] figment::Error),
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Configs {
    pub colors: crate::river::Colors,
    pub hardware: crate::river::Hardware,
    pub apps: crate::river::Apps,
    pub startups: Vec<crate::river::CommandSet>,
    pub border_width: i8,
}

impl Configs {
    // pub fn journal_path(&self) -> Result<PathBuf, Error> {
    //     self.journal
    //         .clone()
    //         .ok_or(Error::DirDoesNotExist)?
    //         .path
    //         .ok_or(Error::DirDoesNotExist)
    // }
    // pub fn journal_file_formatting(&self) -> Result<format_description::OwnedFormatItem, Error> {
    //     Ok(format_description::parse_owned::<2>(
    //         &self
    //             .journal
    //             .clone()
    //             .ok_or(Error::UnsetConfiguration("journal".to_owned()))?
    //             .file_name_format
    //             .ok_or(Error::UnsetConfiguration(
    //                 "journal.file_name_format".to_owned(),
    //             ))?,
    //     )?)
    // }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("The path to the directory is not given.")]
    DirDoesNotExist,
    #[error("{0} was not set in the the configuration.")]
    UnsetConfiguration(String),
    #[error("File format descriptor for journal is not valid: {0}.")]
    TheFormatIsNotValid(#[from] time::error::InvalidFormatDescription),
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn is_normal<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn normal_types() {
        is_normal::<Configs>();
    }
}
