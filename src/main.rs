use clap::{CommandFactory, Parser};
use color_eyre::{eyre::Result, Report};
use prmait::input::{Args, Configs};
use prmait::tasks::handlers::{mark_task_as, tasks_by_state, todays_task};
use prmait::tasks::task::{Task, TaskState};
use prmait::tasks::tasklist::TaskList;
use prmait::{git, journal, river, tasks, timeutils};
use std::{ffi::OsString, path::PathBuf, sync::Arc};
use time::format_description::well_known;
use time::OffsetDateTime;

const DEFAULT_CONFIG_PATH: &str = "/home/a/.config/prmait/config.json";
#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let config = Configs::try_from(&args.config.unwrap_or(PathBuf::from(DEFAULT_CONFIG_PATH)))?;
    let time_offset = config
        .time_offset
        .ok_or(color_eyre::Report::msg(
            "time offset for journal is not set",
        ))
        .map(|(h, m, s)| time::UtcOffset::from_hms(h, m, s))??;

    let now = OffsetDateTime::now_utc().to_offset(time_offset);
    let journal_file_formatting = &config
        .journal
        .clone()
        .ok_or(Report::msg("journal configuration must be set"))?
        .file_name_format
        .ok_or(Report::msg(
            "file name format decriptor for journal must be set",
        ))?;
    let file_format_for_journal =
        { time::format_description::parse_borrowed::<2>(&journal_file_formatting)? };
    let task_file_formatting = &config
        .task
        .clone()
        .ok_or(Report::msg("task configuration must be set"))?
        .file_name_format
        .ok_or(Report::msg(
            "file name format decriptor for task must be set",
        ))?;
    let file_format_for_tasks =
        time::format_description::parse_borrowed::<2>(task_file_formatting)?;

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
                        at: now,
                        body: Arc::new(entry),
                        tag,
                        mood,
                        people,
                    },
                    &config.journal_path()?,
                    now,
                    &file_format_for_journal,
                )?,
                prmait::input::JournalCommands::List => {
                    journal::handlers::list_entries(&config.journal_path()?, &well_known::Rfc3339)?
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
                prmait::input::JournalCommands::Delete => journal::handlers::delete_interactive(
                    &config.journal_path()?,
                    20,
                    &well_known::Rfc3339,
                )?,
            },

            prmait::input::Commands::Task { task_command } => {
                let task_dir = config.task_path()?;
                let project = git::repo_root(std::env::current_dir()?)
                    .map(git::repo_directory_name)
                    .ok()
                    .and_then(Result::ok);
                match task_command {
                    prmait::input::TaskCommands::New {
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
                        let mut projects = projects.unwrap_or(vec![]);
                        if let Ok(Ok(p)) =
                            git::repo_root(std::env::current_dir()?).map(git::repo_directory_name)
                        {
                            projects.push(p);
                        }
                        let t = Task {
                            id: now.unix_timestamp(),
                            time_created: now,
                            state_log: vec![TaskState::ToDo(now)],
                            title,
                            description,
                            area,
                            people: people.unwrap_or(vec![]),
                            projects,
                            start,
                            end,
                        };
                        let efs = tasks::handlers::new_task(
                            config.task_path()?,
                            t,
                            &file_format_for_tasks,
                        )?;
                        efs.run()?;
                    }
                    prmait::input::TaskCommands::List(task_list_command) => {
                        let tasklist = TaskList::try_from(&task_dir)?;
                        match task_list_command {
                            prmait::input::TaskListCommand::Today => {
                                todays_task(
                                    tasklist,
                                    now.date(),
                                    project,
                                    now,
                                    &well_known::Rfc3339,
                                )?;
                            }
                            prmait::input::TaskListCommand::Todo => {
                                tasks_by_state(
                                    tasklist,
                                    |x| matches!(x, &TaskState::ToDo(_)),
                                    project,
                                    now,
                                    &well_known::Rfc3339,
                                )?;
                            }
                            prmait::input::TaskListCommand::Done => {
                                tasks_by_state(
                                    tasklist,
                                    |x| matches!(x, &TaskState::Done(_)),
                                    project,
                                    now,
                                    &well_known::Rfc3339,
                                )?;
                            }
                            prmait::input::TaskListCommand::Abandoned => {
                                tasks_by_state(
                                    tasklist,
                                    |x| matches!(x, TaskState::Abandoned(_, _)),
                                    project,
                                    now,
                                    &well_known::Rfc3339,
                                )?;
                            }
                            prmait::input::TaskListCommand::Backlogged => {
                                tasks_by_state(
                                    tasklist,
                                    |x| matches!(x, TaskState::Backlog(_)),
                                    project,
                                    now,
                                    &well_known::Rfc3339,
                                )?;
                            }
                        }
                    }
                    prmait::input::TaskCommands::Done { id } => {
                        let task_list = TaskList::try_from(&task_dir)?;
                        let effects = mark_task_as(task_dir, task_list, TaskState::Done(now), id)?;
                        effects.run()?;
                    }
                    prmait::input::TaskCommands::Backlog { id } => {
                        let task_list = TaskList::try_from(&task_dir)?;
                        let effects =
                            mark_task_as(task_dir, task_list, TaskState::Backlog(now), id)?;
                        effects.run()?;
                    }
                    prmait::input::TaskCommands::Abandon { id, content } => {
                        let task_list = TaskList::try_from(&task_dir)?;
                        let effects = mark_task_as(
                            task_dir,
                            task_list,
                            TaskState::Abandoned(now, content),
                            id,
                        )?;
                        effects.run()?;
                    }
                    prmait::input::TaskCommands::Todo { id } => {
                        let task_list = TaskList::try_from(&task_dir)?;
                        let effects = mark_task_as(task_dir, task_list, TaskState::ToDo(now), id)?;
                        effects.run()?;
                    }
                }
            }
            prmait::input::Commands::Completions { shell } => {
                shell.generate(&mut Args::command(), &mut std::io::stdout());
            }
            prmait::input::Commands::River => {
                let river_config = &config
                    .river
                    .ok_or(color_eyre::Report::msg("river settings not found"))?;
                river::run(
                    river_config.border_width,
                    &river_config.colors,
                    &river_config.hardware,
                    &river_config.startups,
                    &river_config.apps,
                )
                .await?
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
