use clap::{CommandFactory, Parser};
use color_eyre::eyre::Result;
use color_eyre::Report;
use prmait::effects::{EffectKind, EffectMachine};
use prmait::input::{
    Args, Commands, Configs, JournalCommands, JournalEditCommands, TaskCommands, TaskListCommand,
};
use prmait::tasks::handlers::{mark_task_as, tasks_by_state, todays_task};
use prmait::tasks::task::{State, Task};
use prmait::tasks::tasklist::TaskList;
use prmait::{git, journal, river, tasks, timeutils};
use std::env;
use std::{ffi::OsString, path::PathBuf, sync::Arc};
use time::format_description::well_known;
use time::OffsetDateTime;

const DEFAULT_CONFIG_PATH: &str = "/home/a/.config/prmait/config.json";

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

    let Some(general_command) = args.command else {
        return Ok(());
    };

    let efs = match general_command {
        Commands::J { journal_command } | Commands::Journal { journal_command } => {
            match journal_command {
                JournalCommands::New {
                    entry,
                    tag,
                    mood,
                    people,
                } => journal::handlers::new_entry(
                    &journal::Entry {
                        at: now,
                        body: Arc::new(entry),
                        tag,
                        mood,
                        people,
                    },
                    &config.journal_path()?,
                    now,
                    &config.journal_file_formatting()?,
                )?,
                JournalCommands::List => journal::handlers::list_entries(
                    &journal::Book::try_from(&config.journal_path()?)?,
                    &well_known::Rfc3339,
                )?,
                JournalCommands::Edit(edit_type) => match edit_type {
                    JournalEditCommands::Last => journal::handlers::edit_last_entry(
                        &config.journal_path()?,
                        &journal::Book::try_from(&config.journal_path()?)?,
                        editor(env::var_os("EDITOR"))?,
                    )?,
                    JournalEditCommands::All => journal::handlers::edit_all_entries(
                        &config.journal_path()?,
                        editor(env::var_os("EDITOR"))?,
                        &journal::Book::try_from(&config.journal_path()?)?,
                    )?,
                    JournalEditCommands::Specific { item } => {
                        journal::handlers::edit_specific_entry(
                            &config.journal_path()?,
                            &item,
                            &journal::Book::try_from(&config.journal_path()?)?,
                            editor(env::var_os("EDITOR"))?,
                        )?
                    }
                },
                // JournalCommands::Delete => journal::handlers::delete_interactive(
                //     &config.journal_path()?,
                //     20,
                //     journal::Book::try_from(&config.journal_path()?)?,
                //     &well_known::Rfc3339,
                // )?,
            }
        }

        Commands::T { task_command } | Commands::Task { task_command } => match task_command {
            TaskCommands::New {
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

                tasks::handlers::new_task(
                    &config.task_path()?,
                    &t,
                    &config.task_file_formatting()?,
                )?
            }
            TaskCommands::List(task_list_command) => {
                let tasklist = TaskList::try_from(&task_dir)?;
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
            TaskCommands::Done { id } => {
                let task_list = TaskList::try_from(&task_dir)?;
                mark_task_as(&task_dir, &task_list, &State::Done(now), &id)?
            }
            TaskCommands::Backlog { id } => {
                let task_list = TaskList::try_from(&task_dir)?;
                mark_task_as(&task_dir, &task_list, &State::Backlog(now), &id)?
            }
            TaskCommands::Abandon { id, content } => {
                let task_list = TaskList::try_from(&task_dir)?;
                mark_task_as(&task_dir, &task_list, &State::Abandoned(now, content), &id)?
            }
            TaskCommands::Todo { id } => {
                let task_list = TaskList::try_from(&task_dir)?;
                mark_task_as(&task_dir, &task_list, &State::ToDo(now), &id)?
            }
        },
        Commands::Completions { shell } => {
            let mut ef = EffectMachine::default();
            ef.add(
                EffectKind::GenerateShellCompletion(shell, Args::command()),
                false,
            );
            ef
        }
        Commands::River => {
            let river_config = &config
                .river
                .ok_or(Report::msg("river settings not found"))?;
            
            river::run(
                river_config.border_width,
                &river_config.colors,
                &river_config.hardware,
                &river_config.startups,
                &river_config.apps,
            )?
        }
        Commands::Tasks => {
            let task_dir = config.task_path()?;
            let tasklist = TaskList::try_from(&task_dir)?;
            let project = git::repo_root(&env::current_dir()?)
                .map(|x| git::repo_directory_name(&x))
                .ok()
                .and_then(Result::ok);
            tasks_by_state(
                tasklist,
                |x| matches!(x, &State::ToDo(_)),
                &project,
                now,
                &well_known::Rfc3339,
            )?
        }
    };
    efs.run()?;

    Ok(())
}

fn editor(extractor: Option<OsString>) -> Result<String> {
    let editor = extractor.ok_or(Report::msg("editor variable is not specified"))?;
    if editor.is_empty() {
        return Err(Report::msg("editor variable is not specified"));
    };
    match editor.into_string() {
        Ok(s) => Ok(s),
        Err(e) => Err(Report::msg(format!(
            "could not convert file name to string: {e:?}"
        ))),
    }
}
