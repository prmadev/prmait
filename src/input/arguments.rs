use crate::tasks::task::Area;
use clap::{arg, command, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Clone, Debug, Parser)]
#[command(version,about, long_about = None, arg_required_else_help = true)]
pub struct Args {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Commands {
    River,
    Completions {
        /// The shell to generate the completions for
        #[arg(value_enum)]
        shell: clap_complete_command::Shell,
    },
}

// #[derive(Clone, Debug, Subcommand)]
// pub enum JnlCommands {
//     Completions {
//         /// The shell to generate the completions for
//         #[arg(value_enum)]
//         shell: clap_complete_command::Shell,
//     },
//     /// New Entry
//     New {
//         entry: String,
//         /// Tags that apply to this item [OPTIONAL]
//         #[arg(short = 't', long)]
//         tag: Vec<String>,
//         /// Mood associated with this entry [Required]
//         #[arg(short = 'm', long)]
//         mood: Mood,
//         /// People related to this entry  [OPTIONAL]
//         #[arg(short = 'p', long)]
//         people: Vec<String>,
//     },
//     /// List of entries
//     List,
//     /// edit commands
//     #[command(subcommand)]
//     Edit(JournalEditCommands),
//     // interactively delete an entry
//     // Delete,
// }

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
    Done { id: Vec<i64> },
    /// Set the task as backlogged
    Backlog { id: Vec<i64> },
    /// Set the task as abandoned
    Abandon {
        id: Vec<i64>,
        #[arg(last = true)]
        content: Option<String>,
    },
    /// Set the task as todo
    Todo { id: Vec<i64> },
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Subcommand)]
pub enum TaskListCommand {
    /// Only show tasks that start today, or have a deadline for today
    Today,
    Todo,
    Done,
    Abandoned,
    Backlogged,
    // All, // Specific {
    //     #[arg(short = 's', long)]
    //     status_is:

    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn is_normal<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn normal_types() {
        is_normal::<Commands>();
    }
}
