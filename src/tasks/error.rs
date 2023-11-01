use super::task::Task;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("directory could not be created")]
    DirCouldNotBeCreated(fs_extra::error::Error),
    #[error("task with that name already exist")]
    TaskFileAlreadyExists,
    #[error("file could not be serialized")]
    FileCouldNotSerializeEntryIntoJson(serde_json::Error),
    #[error("file could not written to")]
    FileCouldNotBeWrittenTo(fs_extra::error::Error),
    #[error("directory could not be read")]
    DirCouldNotBeRead(fs_extra::error::Error),
    #[error("file cannot deserialize entry from json string: {0}")]
    FileCouldNotDeserializeEntryFromJson(serde_json::Error),
    #[error("file cannot be read: {0}")]
    FileCouldNotBeRead(fs_extra::error::Error),
    #[error("more than one task with that ID was found: {0:?}")]
    MoreThanOneTaskWasFound(Box<[Task]>),
    #[error("no tasks with that identifier was found")]
    NoTasksFound,
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
