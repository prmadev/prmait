use figment::providers::{Env, Format, Json};
use figment::Figment;
use std::path::PathBuf;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Configs {
    pub journal: Option<JournalConfigs>,
    pub task: Option<TaskConfigs>,
}

impl TryFrom<PathBuf> for Configs {
    type Error = ConfigErr;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
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
pub struct JournalConfigs {
    pub path: Option<PathBuf>,
}
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TaskConfigs {
    pub path: Option<PathBuf>,
}

impl Configs {
    pub fn journal_path(&self) -> Result<PathBuf, Error> {
        self.journal
            .clone()
            .ok_or(Error::DirDoesNotExist)?
            .path
            .ok_or(Error::DirDoesNotExist)
    }
    pub fn task_path(&self) -> Result<PathBuf, Error> {
        self.task
            .clone()
            .ok_or(Error::DirDoesNotExist)?
            .path
            .ok_or(Error::DirDoesNotExist)
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, thiserror::Error)]
pub enum Error {
    #[error("The path to the directory is not given")]
    DirDoesNotExist,
}
