use chrono::{Datelike, Days, Local, TimeZone};
use clap::Parser;
use color_eyre::{eyre::Result, Report};
use prmait::input::{Args, Configs};
use prmait::tasks::handlers::todays_task;
use prmait::tasks::task::{Task, TaskState};
use prmait::time::TimeRange;
use prmait::{git, journal, tasks};
use std::result;
use std::{ffi::OsString, path::PathBuf, sync::Arc};

const DEFAULT_CONFIG_PATH: &str = "/home/a/.config/prmait/config.json";

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let config = {
        let config_file_path = match &args.config {
            Some(p) => p.clone(),
            None => PathBuf::from(DEFAULT_CONFIG_PATH),
        };
        Configs::try_from(config_file_path)?
    };

    match args.command {
        Some(general_command) => match general_command {
            prmait::input::Commands::Journal { journal_command } => match journal_command {
                prmait::input::JournalCommands::New { entry, tag } => journal::handlers::new_entry(
                    journal::Entry {
                        at: Local::now(),
                        body: Arc::new(entry),
                        tag,
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
                    let now = Local::now();
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

                    let cwd = std::env::current_dir()?;
                    let project_name = git::git_root(cwd).map(git::git_directory_name);
                    let mut projects = projects.unwrap_or(vec![]);
                    if let Ok(Ok(p)) = project_name {
                        projects.push(p);
                    }
                    let t = Task {
                        id: now.timestamp(),
                        time_created: now,
                        state_log: Arc::new([TaskState::ToDo(now)]),
                        title,
                        description,
                        area,
                        people: people.unwrap_or(vec![]),
                        projects,
                        deadline,
                        best_starting_time,
                    };
                    tasks::handlers::new_task(&config.task_path()?, t)?;
                }
                prmait::input::TaskCommands::List(t) => match t {
                    prmait::input::TaskListCommand::Today => {
                        let current = chrono::Local::now();
                        let current_day_start = chrono::Local
                            .with_ymd_and_hms(
                                current.year(),
                                current.month(),
                                current.day(),
                                0,
                                0,
                                0,
                            )
                            .single()
                            .ok_or(Report::msg("could not form current day start"))?;
                        let current_day_end = current_day_start
                            .checked_add_days(Days::new(1))
                            .ok_or(Report::msg("could not get current day's end"))?;
                        let time_range = TimeRange {
                            from: Some(current_day_start),
                            to: Some(current_day_end),
                        };
                        let cwd = std::env::current_dir()?;
                        let project_name = git::git_root(cwd).map(git::git_directory_name);
                        let project = project_name.ok().and_then(result::Result::ok);

                        todays_task(&config.task_path()?, time_range, project)?;
                    }
                },
            },
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
