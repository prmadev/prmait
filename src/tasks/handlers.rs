use std::ffi::OsStr;
use std::path::PathBuf;

use chrono::{DateTime, Local};
use color_eyre::owo_colors::OwoColorize;

use crate::time::TimeRange;
use crate::{files::ToFileName, git};

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
        &file_path,
        &serde_json::to_string_pretty(&t).map_err(Error::FileCouldNotSerializeEntryIntoJson)?,
    )
    .map_err(Error::FileCouldNotBeWrittenTo)?;
    let repo_root = git::repo_root(task_dir.to_path_buf())
        .map_err(Error::GitError)?
        .into_os_string();
    git::add(&repo_root, &[&file_path.into_os_string()]).map_err(Error::GitError)?;
    git::commit(
        &repo_root,
        OsStr::new(&format!("feat: new task created {}", t.id)),
    )
    .map_err(Error::GitError)?;
    git::push(&repo_root).map_err(Error::GitError)?;

    Ok(())
}

pub fn mark_task_as(
    task_dir: &PathBuf,
    state: TaskState,
    task_identifier: i64,
) -> Result<(), Error> {
    let mut tasks = TaskList::try_from(task_dir)?.0;
    tasks.retain(|x| x.id.to_string().contains(&task_identifier.to_string()));

    if tasks.len() > 1 {
        return Err(Error::MoreThanOneTaskWasFound(Box::new(tasks)));
    }

    let mut the_task: Task = tasks.get(0).ok_or(Error::NoTasksFound)?.to_owned();
    the_task.state_log.push(state);

    fs_extra::file::write_all(
        task_dir.join(the_task.to_file_name()),
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
    let mut todays_tasks_starting = all_tasks.0.clone();
    todays_tasks_starting.retain(|t| {
        let Some(last) = t.state_log.last() else {
            return false;
        };
        if !matches!(last, TaskState::ToDo(_)) {
            return false;
        };
        let Some(bst) = t.start_to_end.from else {
            return false;
        };
        time_range.intersects_with(bst)
    });

    let mut todays_tasks_deadline: Vec<Task> = all_tasks.0.clone();
    todays_tasks_deadline.retain(|t| {
        let Some(deadlined) = t.start_to_end.to else {
            return false;
        };
        if !time_range.intersects_with(deadlined) {
            return false;
        }
        let Some(proj) = &of_project else { return true };
        t.projects.contains(proj)
    });

    let mut todays_tasks_overdue: Vec<Task> = all_tasks.0;
    todays_tasks_overdue.retain(|t| {
        let Some(deadlined) = t.start_to_end.to else {
            return false;
        };
        if !time_range.is_after(deadlined) {
            return false;
        };
        let Some(proj) = &of_project else { return true };
        t.projects.contains(proj)
    });

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
    if !todays_tasks_overdue.is_empty() {
        println!();
        println!("{:61}", "overdue at today:".bold().black().on_red());
        todays_tasks_deadline
            .iter()
            .for_each(|x| println!("\n{}", x.print_colorful_with_current_duration(current_time)));
    }

    Ok(())
}
pub fn tasks_by_state<F>(
    all_tasks: TaskList,
    task_state_finder: F,
    of_project: Option<String>,
    current_time: DateTime<Local>,
) -> Result<(), Error>
where
    F: Fn(&TaskState) -> bool,
{
    let today = chrono::Local::now();
    let mut chosen_tasks: Vec<Task> = all_tasks.0;
    chosen_tasks.retain(|task| {
        let Some(last) = task.state_log.last() else {
            return false;
        };
        if !task_state_finder(last) {
            return false;
        };
        let Some(proj) = &of_project else {
            return true;
        };
        task.projects.contains(proj)
    });

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
