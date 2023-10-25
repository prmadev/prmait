use std::path::PathBuf;

use clap::{command, Parser, Subcommand};
use figment::{
    providers::{Env, Format, Json},
    Figment,
};

use crate::tasks::Area;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Parser)]
#[command(version,about, long_about = None, arg_required_else_help = true)]
pub struct Args {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Subcommand)]
pub enum Commands {
    /// Journal
    Journal {
        /// journal commands
        #[command(subcommand)]
        journal_command: JournalCommands,
    },
    Task {
        /// task commands
        #[command(subcommand)]
        task_command: TaskCommands,
    },
}
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Subcommand)]
pub enum JournalCommands {
    /// New Entry
    New {
        entry: String,
        tag: Option<Vec<String>>,
    },
    /// List of entries
    List,
    /// edit commands
    #[command(subcommand)]
    Edit(JournalEditCommands),
    Delete,
}
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Subcommand)]
pub enum TaskCommands {
    /// New Entry
    New {
        /// The main title of Task [REQUIRED]
        #[arg(short = 't', long)]
        title: String,
        /// The details of the Task [OPTIONAL]
        #[arg(short = 'd', long)]
        description: Option<String>,
        /// Area that the task falls onto [OPTIONAL]
        #[arg(short = 'a', long)]
        area: Option<Area>,
        /// Names of the people related to the task [OPTIONAL]
        #[arg(short = 'P', long)]
        people: Option<Vec<String>>,
        /// Projects this task belongs to [OPTIONAL]
        #[arg(short = 'p', long)]
        projects: Option<Vec<String>>,
        /// Deadline of the the task in this format "%Y-%m-%d" or "2023-10-24" [OPTIONAL]
        #[arg(short = 'D', long)]
        deadline: Option<String>,
        /// Ideal starting time for the task in this format "%Y-%m-%d" or "2023-10-24" [OPTIONAL]
        #[arg(short = 'S', long)]
        best_starting_time: Option<String>,
    },
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Subcommand)]
pub enum JournalEditCommands {
    Last,
    All,
    One { item: String },
}

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
