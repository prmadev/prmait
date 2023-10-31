use std::path::PathBuf;

use crate::{files::is_json, fold_or_err};

use super::{task::Task, Error};

pub struct TaskList(pub Vec<Task>);

impl TryFrom<&PathBuf> for TaskList {
    type Error = Error;

    fn try_from(value: &PathBuf) -> Result<Self, Self::Error> {
        let mut task_list = fs_extra::dir::get_dir_content(value)
            .map_err(Error::DirCouldNotBeRead)?
            .files
            .into_iter()
            .map(PathBuf::from)
            .filter(is_json)
            .map(Task::try_from)
            .try_fold(vec![], fold_or_err)?;
        task_list.sort();
        Ok(TaskList(task_list))
    }
}
#[cfg(test)]
mod testing {
    use super::*;

    fn is_normal<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    fn normal_types() {
        is_normal::<TaskList>();
    }
}
