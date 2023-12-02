use crate::git;

pub(super) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not format to file name:{0}")]
    CouldNotFormatToFileName(#[from] time::error::Format),
    #[error("invalid time format for file name:{0}")]
    CouldNotParseTimeFormatDescription(#[from] time::error::InvalidFormatDescription),
    #[error("could not create journal directory")]
    JournalDirCouldNotBeCreated(fs_extra::error::Error),
    #[error("could not create journal directory")]
    JournalEntryFileAlreadyExists,
    #[error("could serialize entry {1} to json: {0}")]
    FileCouldNotSerializeEntryIntoJson(serde_json::Error, String),
    #[error("could deserialize entry {1} from json: {0}")]
    FileCouldNotDeserializeEntryFromJson(serde_json::Error, String),
    #[error("could deserialize entry {1} from json: {0}")]
    CouldNotDeserializeEntryFromJson(Box<Error>, String),
    #[error("could not write entry to file: {0}")]
    FileCouldNotBeWrittenTo(fs_extra::error::Error),
    #[error("could not read directory: {0}")]
    DirCouldNotBeRead(fs_extra::error::Error),
    #[error("file content could not be read: {0}")]
    FileCouldNotBeRead(fs_extra::error::Error),
    #[error("deleting file, failed: {0}")]
    FileCouldNotBeDeleted(fs_extra::error::Error),
    #[error("there is entry to be found")]
    NoEntries,
    // #[error("could not work with the editor: {0}")]
    // EditorFailed(files::Error),
    #[error("interactive tools failed: {0}")]
    DialoguerError(dialoguer::Error),
    #[error("entry could not be found")]
    EntryCouldNotBeFound,
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
