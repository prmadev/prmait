use clap::{arg, CommandFactory, Parser, Subcommand};
use color_eyre::eyre::Result;
use color_eyre::Report;
use figment::providers::{Env, Format, Json};
use figment::Figment;
use prmait::effects::{EffectKind, EffectMachine};
use prmait::tasks::effectors::{mark_task_as, tasks_by_state, todays_task};
use prmait::tasks::task::{Area, State, Task};
use prmait::tasks::tasklist::TaskList;
use prmait::{git, tasks, timeutils};
use std::env;
use std::path::PathBuf;
use time::format_description::{self, well_known};
use time::OffsetDateTime;

const DEFAULT_CONFIG_PATH: &str = "/home/a/.config/prmait/tsk.json";

fn main() -> Result<()> {
    // error message management
    color_eyre::install()?;

    // tracing
    tracing_subscriber::fmt::init();

    // getting arugments
    let args = Args::parse();

    // forming config out of arguments
    let config = Configs::try_from(&args.config.unwrap_or(PathBuf::from(DEFAULT_CONFIG_PATH)))?;

    // getting current time offset
    let time_offset = config
        .time_offset
        .ok_or(Report::msg("time offset for is not set"))
        .map(|(h, m, s)| time::UtcOffset::from_hms(h, m, s))??;

    // getting current time
    let now = OffsetDateTime::now_utc().to_offset(time_offset);

    // getting task directory
    let task_dir = config.task_path()?;

    // getting current directory's project
    let project = git::repo_root(&env::current_dir()?)
        .map(|x| git::repo_directory_name(&x))
        .ok()
        .and_then(Result::ok);

    let Some(command) = args.command else {
        return Ok(());
    };

    let efs = to_effect_machine(command, now, config, time_offset, project, &task_dir)?;
    efs.run()?;

    Ok(())
}

fn to_effect_machine(
    command: Commands,
    now: OffsetDateTime,
    config: Configs,
    time_offset: time::UtcOffset,
    project: Option<String>,
    task_dir: &PathBuf,
) -> Result<EffectMachine, Report> {
    Ok(match command {
        Commands::New {
            title,
            description,
            area,
            people,
            deadline,
            best_starting_time,
            projects,
        } => {
            let end = deadline
                .map(|s| timeutils::parse_date(&s, time_offset))
                .transpose()?;

            let start = best_starting_time
                .map(|s| timeutils::parse_date(&s, time_offset))
                .transpose()?;

            let projects = {
                let mut buf = projects.unwrap_or(vec![]);
                if let Some(current_folder_project) = project {
                    buf.push(current_folder_project);
                };
                buf
            };

            let t = Task {
                id: now.unix_timestamp(),
                time_created: now,
                state_log: vec![State::ToDo(now)],
                title,
                description,
                area,
                people: people.unwrap_or(vec![]),
                projects,
                start,
                end,
            };
            let repo_root = git::repo_root(&config.task_path()?)?
                .to_string_lossy()
                .into_owned();

            tasks::effectors::new_task(
                &config.task_path()?,
                &t,
                &repo_root,
                &config.task_file_formatting()?,
            )?
        }
        Commands::List(task_list_command) => {
            let tasklist = TaskList::try_from(task_dir)?;
            match task_list_command {
                TaskListCommand::Today => {
                    todays_task(tasklist, now.date(), &project, now, &well_known::Rfc3339)?
                }
                TaskListCommand::Todo => tasks_by_state(
                    tasklist,
                    |x| matches!(x, &State::ToDo(_)),
                    &project,
                    now,
                    &well_known::Rfc3339,
                )?,
                TaskListCommand::Done => tasks_by_state(
                    tasklist,
                    |x| matches!(x, &State::Done(_)),
                    &project,
                    now,
                    &well_known::Rfc3339,
                )?,
                TaskListCommand::Abandoned => tasks_by_state(
                    tasklist,
                    |x| matches!(x, State::Abandoned(_, _)),
                    &project,
                    now,
                    &well_known::Rfc3339,
                )?,
                TaskListCommand::Backlogged => tasks_by_state(
                    tasklist,
                    |x| matches!(x, State::Backlog(_)),
                    &project,
                    now,
                    &well_known::Rfc3339,
                )?,
            }
        }
        Commands::Done { id } => {
            let task_list = TaskList::try_from(task_dir)?;
            let repo_root = git::repo_root(&config.task_path()?)?
                .to_string_lossy()
                .into_owned();
            mark_task_as(task_dir, &task_list, &State::Done(now), &repo_root, &id)?
        }
        Commands::Backlog { id } => {
            let task_list = TaskList::try_from(task_dir)?;
            let repo_root = git::repo_root(&config.task_path()?)?
                .to_string_lossy()
                .into_owned();
            mark_task_as(task_dir, &task_list, &State::Backlog(now), &repo_root, &id)?
        }
        Commands::Abandon { id, content } => {
            let task_list = TaskList::try_from(task_dir)?;
            let repo_root = git::repo_root(&config.task_path()?)?
                .to_string_lossy()
                .into_owned();
            mark_task_as(
                task_dir,
                &task_list,
                &State::Abandoned(now, content),
                &repo_root,
                &id,
            )?
        }
        Commands::Todo { id } => {
            let task_list = TaskList::try_from(task_dir)?;
            let repo_root = git::repo_root(&config.task_path()?)?
                .to_string_lossy()
                .into_owned();
            mark_task_as(task_dir, &task_list, &State::ToDo(now), &repo_root, &id)?
        }
        Commands::Completions { shell } => {
            let mut ef = EffectMachine::default();
            ef.add(
                EffectKind::GenerateShellCompletion(shell, Args::command()),
                false,
            );
            ef
        }
    })
}

#[derive(Clone, Debug, Parser)]
#[command(version,about="The tusky task manager", long_about = None, arg_required_else_help = true)]
pub struct Args {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Commands {
    Completions {
        /// The shell to generate the completions for
        #[arg(value_enum)]
        shell: clap_complete_command::Shell,
    },
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

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Configs {
    pub time_offset: Option<(i8, i8, i8)>,
    pub path: Option<PathBuf>,
    pub file_name_format: Option<String>,
}

impl Configs {
    pub fn task_path(&self) -> Result<PathBuf, Error> {
        self.clone().path.ok_or(Error::DirDoesNotExist)
    }
    pub fn task_file_formatting(&self) -> Result<format_description::OwnedFormatItem, Error> {
        Ok(format_description::parse_owned::<2>(
            &self
                .clone()
                .file_name_format
                .ok_or(Error::UnsetConfiguration(
                    "task.file_name_format".to_owned(),
                ))?,
        )?)
    }
}
impl TryFrom<PathBuf> for Configs {
    type Error = Error;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        Ok(Figment::new()
            .merge(Json::file(value))
            .merge(Env::prefixed("TSK_"))
            .extract()?)
    }
}
impl TryFrom<&PathBuf> for Configs {
    type Error = Error;

    fn try_from(value: &PathBuf) -> Result<Self, Self::Error> {
        Ok(Figment::new()
            .merge(Json::file(value))
            .merge(Env::prefixed("TSK_"))
            .extract()?)
    }
}
#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("could not extract configuration: {0}")]
    ExtractionFailed(#[from] figment::Error),
    #[error("The path to the directory is not given.")]
    DirDoesNotExist,
    #[error("{0} was not set in the the configuration.")]
    UnsetConfiguration(String),
    #[error("File format descriptor for journal is not valid: {0}.")]
    TheFormatIsNotValid(#[from] time::error::InvalidFormatDescription),
}
