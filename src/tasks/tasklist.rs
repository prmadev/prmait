use std::path::PathBuf;

use crate::{files::is_json, fold_or_err};

use super::{task::Task, Error};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TaskList(pub Vec<TaskDescription>);

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TaskDescription {
    pub task: Task,
    pub file_name: String,
}

impl TryFrom<&PathBuf> for TaskDescription {
    type Error = Error;

    fn try_from(value: &PathBuf) -> Result<Self, Self::Error> {
        let task = Task::try_from(value)?;
        let file_name = value
            .file_name()
            .ok_or(Error::IsNotAFile)?
            .to_str()
            .ok_or(Error::FileNameHasInvalidCharacters)?
            .to_owned();
        Ok(Self { task, file_name })
    }
}
impl TryFrom<PathBuf> for TaskDescription {
    type Error = Error;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let task = Task::try_from(&value)?;
        let file_name = value
            .file_name()
            .ok_or(Error::IsNotAFile)?
            .to_str()
            .ok_or(Error::FileNameHasInvalidCharacters)?
            .to_owned();
        Ok(Self { task, file_name })
    }
}

impl TryFrom<&PathBuf> for TaskList {
    type Error = Error;

    fn try_from(value: &PathBuf) -> Result<Self, Self::Error> {
        let mut task_list = fs_extra::dir::get_dir_content(value)
            .map_err(Error::DirCouldNotBeRead)?
            .files
            .into_iter()
            .map(PathBuf::from)
            .filter(is_json)
            .map(TaskDescription::try_from)
            .try_fold(vec![], fold_or_err)?;
        task_list.sort();
        Ok(Self(task_list))
    }
}
#[cfg(test)]
mod testing {
    use super::*;

    const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn normal_types() {
        is_normal::<TaskList>();
    }
}
