use std::path::PathBuf;

use color_eyre::owo_colors::OwoColorize;
use time::formatting::Formattable;
use time::{Date, OffsetDateTime};

use crate::effects::{CreateDirOpts, EffectKind, EffectMachine, FileWriterOpts};
use crate::files::ToFileName;

use super::Result;
use super::{
    task::{Task, TaskState},
    tasklist::TaskList,
    Error,
};

pub fn new_task(
    task_dir: PathBuf,
    t: Task,
    time_format_descriptor: &(impl Formattable + ?Sized),
) -> Result<EffectMachine> {
    let file_name = t.to_file_name(time_format_descriptor)?;
    let file_path = task_dir.join(&file_name);
    let mut effects = EffectMachine::default();

    effects.add(
        EffectKind::CreateDir(CreateDirOpts {
            folder_path: task_dir.clone(),
            ok_if_exists: true,
        }),
        false,
    );
    effects.add(
        EffectKind::WriteToFile(FileWriterOpts {
            content: serde_json::to_string_pretty(&t)
                .map_err(|e| Error::FileCouldNotSerializeEntryIntoJson(e, file_name))?
                .into_bytes(),

            file_path: file_path.clone(),
            can_create: true,
            can_overwrite: false,
        }),
        false,
    );
    effects.add(
        EffectKind::GitHook(crate::effects::GitHookOpts {
            start_path: task_dir,
            files_to_add: vec![file_path],
            message: format!("feat(tasks): add new task file  {}", t.id),
        }),
        true,
    );
    Ok(effects)
}

pub fn mark_task_as(
    task_dir: PathBuf,
    tasks_list: TaskList,
    state: TaskState,
    task_identifier: i64,
) -> Result<EffectMachine> {
    let mut effects = EffectMachine::default();
    let mut tasks = tasks_list.0;
    tasks.retain(|x| x.task.id.to_string().contains(&task_identifier.to_string()));
    tasks.retain(|x| {
        let Some(last_state) = x.task.state_log.last() else {
            return false;
        };
        last_state.ne(&state)
    });

    if tasks.len() > 1 {
        return Err(Error::MoreThanOneTaskWasFound(Box::new(tasks)));
    }

    let mut the_task_description = tasks.get(0).ok_or(Error::NoTasksFound)?.to_owned();
    the_task_description.task.state_log.push(state.clone());

    let file_path = task_dir.join(&the_task_description.file_name);
    let new_file_content = serde_json::to_string_pretty(&the_task_description.task)
        .map_err(|e| Error::FileCouldNotSerializeEntryIntoJson(e, the_task_description.file_name))?
        .into_bytes();
    effects.add(
        EffectKind::WriteToFile(FileWriterOpts {
            content: new_file_content,
            file_path: file_path.clone(),
            can_create: false,
            can_overwrite: true,
        }),
        false,
    );
    effects.add(
        EffectKind::GitHook(crate::effects::GitHookOpts {
            start_path: task_dir,
            files_to_add: vec![file_path],
            message: format!(
                "feat: updated task {} to the new state {}",
                the_task_description.task.id, state,
            ),
        }),
        true,
    );

    Ok(effects)
}

pub fn todays_task(
    all_tasks: TaskList,
    current_date: Date,
    of_project: Option<String>,
    current_time: OffsetDateTime,
    time_format_descriptor: &(impl Formattable + ?Sized),
) -> Result<()> {
    let mut todays_tasks_starting = all_tasks.0.clone();
    todays_tasks_starting.retain(|t| {
        let Some(last) = t.task.state_log.last() else {
            return false;
        };
        if !matches!(last, TaskState::ToDo(_)) {
            return false;
        };
        let Some(bst) = t.task.start else {
            return false;
        };
        current_date.eq(&bst)
    });

    let mut todays_tasks_deadline = all_tasks.0.clone();
    todays_tasks_deadline.retain(|t| {
        let Some(last) = t.task.state_log.last() else {
            return false;
        };
        if !matches!(last, TaskState::ToDo(_)) {
            return false;
        };
        let Some(deadlined) = t.task.end else {
            return false;
        };
        if !current_date.eq(&deadlined) {
            return false;
        }
        let Some(proj) = &of_project else { return true };
        t.task.projects.contains(proj)
    });

    let mut todays_tasks_overdue = all_tasks.0;
    todays_tasks_overdue.retain(|t| {
        let Some(deadlined) = t.task.end else {
            return false;
        };
        if !current_date.gt(&deadlined) {
            return false;
        };
        let Some(proj) = &of_project else { return true };
        t.task.projects.contains(proj)
    });

    if !todays_tasks_starting.is_empty() {
        println!();
        println!(
            "{:61}",
            "Starting from today:".bold().black().on_bright_blue()
        );
        todays_tasks_starting
            .iter()
            .map(|x| {
                try_print_colorful_with_current_duration(
                    &x.task,
                    current_time,
                    time_format_descriptor,
                )
            })
            .for_each(println_ok_or_eprintln);
    }
    if !todays_tasks_deadline.is_empty() {
        println!();
        println!("{:61}", "Deadline at today:".bold().black().on_red());
        todays_tasks_deadline
            .iter()
            .map(|x| {
                try_print_colorful_with_current_duration(
                    &x.task,
                    current_time,
                    time_format_descriptor,
                )
            })
            .for_each(println_ok_or_eprintln);
    }
    if !todays_tasks_overdue.is_empty() {
        println!();
        println!("{:61}", "overdue at today:".bold().black().on_red());
        todays_tasks_deadline
            .iter()
            .map(|x| {
                try_print_colorful_with_current_duration(
                    &x.task,
                    current_time,
                    time_format_descriptor,
                )
            })
            .for_each(println_ok_or_eprintln);
    }

    Ok(())
}

fn println_ok_or_eprintln(x: Result<String>) {
    match x {
        Ok(f) => println!("{f}"),
        Err(e) => eprintln!("{e}"),
    }
}
pub fn tasks_by_state<F>(
    all_tasks: TaskList,
    task_state_finder: F,
    of_project: Option<String>,
    current_time: OffsetDateTime,
    time_format_descriptor: &(impl Formattable + ?Sized),
) -> Result<()>
where
    F: Fn(&TaskState) -> bool,
{
    let mut chosen_tasks = all_tasks.0;
    chosen_tasks.retain(|task_description| {
        let Some(last) = task_description.task.state_log.last() else {
            return false;
        };
        if !task_state_finder(last) {
            return false;
        };
        let Some(proj) = &of_project else {
            return true;
        };
        task_description.task.projects.contains(proj)
    });

    if !chosen_tasks.is_empty() {
        println!(
            "{:61}",
            "tasks with that criteria:".bold().black().on_bright_blue()
        );

        chosen_tasks
            .iter()
            .map(|x| {
                try_print_colorful_with_current_duration(
                    &x.task,
                    current_time,
                    time_format_descriptor,
                )
            })
            .for_each(println_ok_or_eprintln);
    }

    Ok(())
}

fn try_print_colorful_with_current_duration(
    x: &Task,
    current_time: OffsetDateTime,
    time_format_descriptor: &(impl Formattable + ?Sized),
) -> std::result::Result<String, Error> {
    Ok(format!(
        "\n{}",
        x.print_colorful_with_current_duration(current_time, time_format_descriptor)?
    ))
}
