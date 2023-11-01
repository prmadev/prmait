use crate::tasks::task::Area;
use clap::{command, Parser, Subcommand};
use std::path::PathBuf;

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
    /// Journaling
    Journal {
        /// journal commands
        #[command(subcommand)]
        journal_command: JournalCommands,
    },
    /// Task Management
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
    /// interactively delete an entry
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
        /// Deadline of the the task in this format "%Y-%m-%d" or "2022-10-24" [OPTIONAL]
        #[arg(short = 'D', long)]
        deadline: Option<String>,
        /// Ideal starting time for the task in this format "%Y-%m-%d" or "2022-10-24" [OPTIONAL]
        #[arg(short = 'S', long)]
        best_starting_time: Option<String>,
    },
    /// List tasks commands
    #[command(subcommand)]
    List(TaskListCommand),
    /// Set the task as done
    Done { id: i64 },
    /// Set the task as backlogged
    Backlog { id: i64 },
    /// Set the task as abandoned
    Abandon { id: i64, content: Option<String> },
    /// Set the task as todo
    Todo { id: i64 },
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Subcommand)]
pub enum TaskListCommand {
    /// Only show tasks that start today, or have a deadline for today
    Today,
    // All, // Specific {
    //     #[arg(short = 's', long)]
    //     status_is:

    // }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Subcommand)]
pub enum JournalEditCommands {
    /// Only edit the last entry
    Last,
    /// Open every entry in the editor
    All,
    /// Open only strings matching the given entry
    Specific { item: String },
}
