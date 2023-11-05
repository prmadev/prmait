use std::{path::PathBuf, rc::Rc};

use chrono::{DateTime, Local};
use color_eyre::owo_colors::OwoColorize;

use crate::files::ToFileName;
use crate::time::TimeRange;

use super::{
    task::{Task, TaskState},
    tasklist::TaskList,
    Error,
};

pub fn new_task(task_dir: &PathBuf, t: Task) -> Result<(), Error> {
    _ = fs_extra::dir::create(task_dir, false).map_err(Error::DirCouldNotBeCreated);

    let file_path = task_dir.join(t.to_file_name());

    if file_path.exists() {
        return Err(Error::TaskFileAlreadyExists);
    }

    fs_extra::file::write_all(
        file_path,
        &serde_json::to_string_pretty(&t).map_err(Error::FileCouldNotSerializeEntryIntoJson)?,
    )
    .map_err(Error::FileCouldNotBeWrittenTo)?;

    Ok(())
}

pub fn mark_task_as(
    task_dir: &PathBuf,
    state: TaskState,
    task_identifier: i64,
) -> Result<(), Error> {
    let task_list = TaskList::try_from(task_dir)?;
    let tasks: Box<[Task]> = task_list
        .0
        .into_iter()
        .filter(|x| x.id.to_string().contains(&task_identifier.to_string()))
        .collect();
    if tasks.len() > 1 {
        return Err(Error::MoreThanOneTaskWasFound(tasks));
    }
    let mut the_task: Task = tasks.get(0).ok_or(Error::NoTasksFound)?.to_owned();
    let mut new_state_log = the_task.state_log.clone();
    new_state_log.push(state);
    the_task.state_log = new_state_log;

    let file_path = task_dir.join(the_task.to_file_name());

    fs_extra::file::write_all(
        file_path,
        &serde_json::to_string_pretty(&the_task)
            .map_err(Error::FileCouldNotSerializeEntryIntoJson)?,
    )
    .map_err(Error::FileCouldNotBeWrittenTo)?;

    Ok(())
}

pub fn todays_task(
    all_tasks: TaskList,
    time_range: TimeRange,
    of_project: Option<String>,
    current_time: DateTime<Local>,
) -> Result<(), Error> {
    let today = chrono::Local::now();
    let todays_tasks_starting: Vec<&Task> = all_tasks
        .0
        .iter()
        .filter(|t| {
            t.state_log
                .last()
                .is_some_and(|s| matches!(s, TaskState::ToDo(_)))
        })
        .filter(|t| {
            if let Some(bst) = t.start_to_end.from {
                time_range.intersects_with(bst)
            } else {
                false
            }
        })
        .filter(|t| {
            if let Some(proj) = &of_project {
                t.projects.contains(proj)
            } else {
                true
            }
        })
        .collect();

    let todays_tasks_deadline: Vec<&Task> = all_tasks
        .0
        .iter()
        .filter(|t| {
            if let Some(deadlined) = t.start_to_end.to {
                time_range.intersects_with(deadlined)
            } else {
                false
            }
        })
        .filter(|t| {
            if let Some(proj) = &of_project {
                t.projects.contains(proj)
            } else {
                true
            }
        })
        .collect();

    println!();
    println!(
        "{}",
        format!("Date Today: {}", today.format("%Y-%m-%d"))
            .bold()
            .black()
            .on_cyan()
    );

    if !todays_tasks_starting.is_empty() {
        println!();
        println!(
            "{:61}",
            "Starting from today:".bold().black().on_bright_blue()
        );
        todays_tasks_starting
            .iter()
            .for_each(|x| println!("\n{}", x.print_colorful_with_current_duration(current_time)));
    }
    if !todays_tasks_deadline.is_empty() {
        println!();
        println!("{:61}", "Deadline at today:".bold().black().on_red());
        todays_tasks_deadline
            .iter()
            .for_each(|x| println!("\n{}", x.print_colorful_with_current_duration(current_time)));
    }

    Ok(())
}
pub fn tasks_by_state(
    all_tasks: TaskList,
    task_state_finder: Rc<impl Fn(&TaskState) -> bool>,
    of_project: Option<String>,
    current_time: DateTime<Local>,
) -> Result<(), Error> {
    let today = chrono::Local::now();
    let chosen_tasks: Vec<Task> = all_tasks
        .0
        .into_iter()
        .filter(|task| {
            task.state_log
                .last()
                .is_some_and(|task_state| task_state_finder(task_state))
        })
        .filter(|task| {
            if let Some(proj) = &of_project {
                task.projects.contains(proj)
            } else {
                true
            }
        })
        .collect();

    if !chosen_tasks.is_empty() {
        println!();
        println!("{}", today_formatted(today));
        println!();

        println!(
            "{:61}",
            "tasks with that criteria:".bold().black().on_bright_blue()
        );

        chosen_tasks
            .iter()
            .for_each(|x| println!("\n{}", x.print_colorful_with_current_duration(current_time)));
    }

    Ok(())
}
fn today_formatted(today: chrono::DateTime<Local>) -> String {
    format!("Date Today: {}", today.format("%Y-%m-%d"))
        .bold()
        .black()
        .on_cyan()
        .to_string()
}
