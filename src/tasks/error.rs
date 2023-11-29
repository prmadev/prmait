use crate::git;

use super::tasklist::TaskDescription;

pub(super) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("every task should have at least one state")]
    EveryTaskShouldHaveAtLeastOneState,
    #[error("could not format to file name:{0}")]
    CouldNotFormatToFileName(#[from] time::error::Format),
    #[error("invalid time format for file name:{0}")]
    CouldNotParseTimeFormatDescription(#[from] time::error::InvalidFormatDescription),
    #[error("directory could not be created")]
    DirCouldNotBeCreated(fs_extra::error::Error),
    #[error("task with that name already exist")]
    TaskFileAlreadyExists,
    #[error("file could not be serialized")]
    FileCouldNotSerializeEntryIntoJson(serde_json::Error, String),
    #[error("file could not written to")]
    FileCouldNotBeWrittenTo(fs_extra::error::Error),
    #[error("directory could not be read")]
    DirCouldNotBeRead(fs_extra::error::Error),
    #[error("file cannot deserialize entry {1} from json string: {0}")]
    FileCouldNotDeserializeEntryFromJson(serde_json::Error, String),
    #[error("file cannot be read: {0}")]
    FileCouldNotBeRead(fs_extra::error::Error),
    #[error("more than one task with that ID was found: {0:?}")]
    MoreThanOneTaskWasFound(Box<Vec<TaskDescription>>),
    #[error("no tasks with that identifier was found")]
    NoTasksFound,
    #[error("got error from running git command: {0}")]
    GitError(git::Error),
    #[error("file name has invalid characters")]
    FileNameHasInvalidCharacters,
    #[error("the path is not a file")]
    IsNotAFile,
}
#[cfg(test)]
mod testing {
    use super::*;

    fn is_normal<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    fn normal_types() {
        is_normal::<Error>();
    }
}
