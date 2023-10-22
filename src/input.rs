use std::path::PathBuf;

use clap::{command, Parser, Subcommand};
use figment::{
    providers::{Env, Format, Json},
    Figment,
};

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
    EditLast,
    EditAll,
    DeleteI,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Configs {
    pub journal_configs: Option<JournalConfigs>,
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
    pub journal_path: Option<PathBuf>,
}
