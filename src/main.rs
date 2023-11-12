use chrono::{Local, TimeZone};
use clap::{CommandFactory, Parser};
use color_eyre::{eyre::Result, Report};
use prmait::input::{Args, Configs};
use prmait::tasks::handlers::{mark_task_as, tasks_by_state, todays_task};
use prmait::tasks::task::{Task, TaskState};
use prmait::tasks::tasklist::TaskList;
use prmait::time::TimeRange;
use prmait::{git, journal, tasks};
use std::{ffi::OsString, path::PathBuf, sync::Arc};

const DEFAULT_CONFIG_PATH: &str = "/home/a/.config/prmait/config.json";

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let config = Configs::try_from(&args.config.unwrap_or(PathBuf::from(DEFAULT_CONFIG_PATH)))?;

    match args.command {
        Some(general_command) => match general_command {
            prmait::input::Commands::Journal { journal_command } => match journal_command {
                prmait::input::JournalCommands::New {
                    entry,
                    tag,
                    mood,
                    people,
                } => journal::handlers::new_entry(
                    journal::Entry {
                        at: Local::now(),
                        body: Arc::new(entry),
                        tag,
                        mood,
                        people,
                    },
                    &config.journal_path()?,
                    Local::now(),
                )?,
                prmait::input::JournalCommands::List => {
                    journal::handlers::list_entries(&config.journal_path()?)?
                }
                prmait::input::JournalCommands::Edit(edit_type) => match edit_type {
                    prmait::input::JournalEditCommands::Last => journal::handlers::edit_last_entry(
                        &config.journal_path()?,
                        editor(std::env::var_os("EDITOR"))?,
                    )?,
                    prmait::input::JournalEditCommands::All => journal::handlers::edit_all_entries(
                        &config.journal_path()?,
                        editor(std::env::var_os("EDITOR"))?,
                    )?,
                    prmait::input::JournalEditCommands::Specific { item } => {
                        journal::handlers::edit_specific_entry(
                            &config.journal_path()?,
                            item,
                            editor(std::env::var_os("EDITOR"))?,
                        )?
                    }
                },
                prmait::input::JournalCommands::Delete => {
                    journal::handlers::delete_interactive(&config.journal_path()?, 20)?
                }
            },

            prmait::input::Commands::Task { task_command } => match task_command {
                prmait::input::TaskCommands::New {
                    title,
                    description,
                    area,
                    people,
                    deadline,
                    best_starting_time,
                    projects,
                } => {
                    let deadline = match deadline {
                        Some(s) => {
                            let nt = chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d")?
                                .and_hms_opt(23, 59, 59)
                                .ok_or(Report::msg("could not form date time for deadline time"))?;
                            let lo = Local
                                .from_local_datetime(&nt)
                                .single()
                                .ok_or(Report::msg("invalid deadline format"))?;
                            Some(lo)
                        }
                        None => None,
                    };
                    let best_starting_time = match best_starting_time {
                        Some(s) => {
                            let nt = chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d")?
                                .and_hms_opt(0, 0, 0)
                                .ok_or(Report::msg(
                                    "could not form date time for best starting time",
                                ))?;
                            let lo = Local
                                .from_local_datetime(&nt)
                                .single()
                                .ok_or(Report::msg("invalid best-starting-time format"))?;
                            Some(lo)
                        }
                        None => None,
                    };
                    let start_to_end = TimeRange::build(best_starting_time, deadline)?;
                    let mut projects = projects.unwrap_or(vec![]);
                    if let Ok(Ok(p)) =
                        git::repo_root(std::env::current_dir()?).map(git::repo_directory_name)
                    {
                        projects.push(p);
                    }
                    let now = Local::now();
                    let t = Task {
                        id: now.timestamp(),
                        time_created: now,
                        state_log: vec![TaskState::ToDo(now)],
                        title,
                        description,
                        area,
                        people: people.unwrap_or(vec![]),
                        projects,
                        start_to_end,
                    };
                    tasks::handlers::new_task(&config.task_path()?, t)?;
                }
                prmait::input::TaskCommands::List(task_list_command) => match task_list_command {
                    prmait::input::TaskListCommand::Today => {
                        let current = chrono::Local::now();
                        let time_range = TimeRange::day_range_of_time(&current)?;
                        let project = git::repo_root(std::env::current_dir()?)
                            .map(git::repo_directory_name)
                            .ok()
                            .and_then(Result::ok);
                        let tasklist = TaskList::try_from(&config.task_path()?)?;

                        todays_task(tasklist, time_range, project, current)?;
                    }
                    prmait::input::TaskListCommand::Todo => {
                        let current = chrono::Local::now();
                        let project = git::repo_root(std::env::current_dir()?)
                            .map(git::repo_directory_name)
                            .ok()
                            .and_then(Result::ok);
                        let tasklist = TaskList::try_from(&config.task_path()?)?;
                        tasks_by_state(
                            tasklist,
                            |x| matches!(x, &TaskState::ToDo(_)),
                            project,
                            current,
                        )?;
                    }
                    prmait::input::TaskListCommand::Done => {
                        let current = chrono::Local::now();
                        let project = git::repo_root(std::env::current_dir()?)
                            .map(git::repo_directory_name)
                            .ok()
                            .and_then(Result::ok);
                        let tasklist = TaskList::try_from(&config.task_path()?)?;
                        tasks_by_state(
                            tasklist,
                            |x| matches!(x, &TaskState::Done(_)),
                            project,
                            current,
                        )?;
                    }
                    prmait::input::TaskListCommand::Abandoned => {
                        let current = chrono::Local::now();
                        let project = git::repo_root(std::env::current_dir()?)
                            .map(git::repo_directory_name)
                            .ok()
                            .and_then(Result::ok);
                        let tasklist = TaskList::try_from(&config.task_path()?)?;
                        tasks_by_state(
                            tasklist,
                            |x| matches!(x, TaskState::Abandoned(_, _)),
                            project,
                            current,
                        )?;
                    }
                    prmait::input::TaskListCommand::Backlogged => {
                        let current = chrono::Local::now();
                        let project = git::repo_root(std::env::current_dir()?)
                            .map(git::repo_directory_name)
                            .ok()
                            .and_then(Result::ok);
                        let tasklist = TaskList::try_from(&config.task_path()?)?;
                        tasks_by_state(
                            tasklist,
                            |x| matches!(x, TaskState::Backlog(_)),
                            project,
                            current,
                        )?;
                    }
                },
                prmait::input::TaskCommands::Done { id } => {
                    let current = chrono::Local::now();
                    mark_task_as(&config.task_path()?, TaskState::Done(current), id)?;
                }
                prmait::input::TaskCommands::Backlog { id } => {
                    let current = chrono::Local::now();
                    mark_task_as(&config.task_path()?, TaskState::Backlog(current), id)?;
                }
                prmait::input::TaskCommands::Abandon { id, content } => {
                    let current = chrono::Local::now();
                    mark_task_as(
                        &config.task_path()?,
                        TaskState::Abandoned(current, content),
                        id,
                    )?;
                }
                prmait::input::TaskCommands::Todo { id } => {
                    let current = chrono::Local::now();
                    mark_task_as(&config.task_path()?, TaskState::ToDo(current), id)?;
                }
            },
            prmait::input::Commands::Completions { shell } => {
                shell.generate(&mut Args::command(), &mut std::io::stdout());
            }
        },
        None => unreachable!("because of clap, it should not be possible to reach this point"),
    }

    Ok(())
}

fn editor(extractor: Option<OsString>) -> Result<OsString> {
    let editor = extractor.ok_or(Report::msg("editor variable is not specified"))?;
    if editor.is_empty() {
        return Err(Report::msg("editor variable is not specified"));
    };
    Ok(editor)
}
