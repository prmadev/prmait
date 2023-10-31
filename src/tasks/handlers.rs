use std::path::PathBuf;

use chrono::{DateTime, Local};
use color_eyre::owo_colors::OwoColorize;

use crate::files::ToFileName;

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

#[derive(Debug, Clone)]
pub struct TimeRange {
    pub from: Option<DateTime<Local>>,
    pub to: Option<DateTime<Local>>,
}
pub fn todays_task(
    task_dir: &PathBuf,
    time_range: TimeRange,
    of_project: Option<String>,
) -> Result<(), Error> {
    let today = chrono::Local::now();
    let all_tasks = TaskList::try_from(task_dir)?;
    let todays_tasks_starting: Vec<&Task> = all_tasks
        .0
        .iter()
        .filter(|t| {
            t.state_log
                .last()
                .is_some_and(|s| matches!(s, TaskState::ToDo(_)))
        })
        .filter(|t| {
            if let Some(bst) = t.best_starting_time {
                if time_range.from.is_some_and(|fr| fr > bst) {
                    return false;
                }
                if time_range.to.is_some_and(|to| to < bst) {
                    return false;
                }
                true
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
            if let Some(deadlined) = t.deadline {
                if time_range.from.is_some_and(|fr| fr > deadlined) {
                    return false;
                }
                if time_range.to.is_some_and(|to| to < deadlined) {
                    return false;
                }
                true
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
    println!();

    println!(
        "{:61}",
        "Starting from today:".bold().black().on_bright_blue()
    );
    todays_tasks_starting.iter().for_each(|x| println!("\n{x}"));
    println!();
    println!("{:61}", "Deadline at today:".bold().black().on_red());
    todays_tasks_deadline.iter().for_each(|x| println!("\n{x}"));

    Ok(())
}
