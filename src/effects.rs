use std::{path::PathBuf, sync::Arc};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Effect {
    WriteToFile(FileWriterOpts),
}

impl Effect {
    pub fn apply(self) -> Result<()> {
        match self {
            Effect::WriteToFile(opts) => file_writer(opts),
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FileWriterOpts {
    content: Arc<[u8]>,
    file_path: PathBuf,
    can_create: bool,
    can_overwrite: bool,
}

fn file_writer(opts: FileWriterOpts) -> Result<()> {
    let exists = opts
        .file_path
        .try_exists()
        .map_err(Error::CheckFileExistenceFailed)?;
    if !opts.can_create && !exists {
        return Err(Error::FileDoesNotExists);
    };
    if !opts.can_overwrite && exists {
        return Err(Error::FileAlreadyExists);
    };

    std::fs::write(opts.file_path, opts.content).map_err(Error::CouldNotWriteToFile)
}

#[cfg(test)]
mod testing {
    #[allow(clippy::wildcard_imports)]
    use super::*;
    fn is_normal<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    fn normal_types() {
        is_normal::<Effect>();
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("checking for the existence of the file failed: {0}")]
    CheckFileExistenceFailed(std::io::Error),
    #[error("this file already exists")]
    FileAlreadyExists,
    #[error("this file does not exist")]
    FileDoesNotExists,
    #[error("I could not write to file: {0}")]
    CouldNotWriteToFile(std::io::Error),
}

type Result<T> = std::result::Result<T, Error>;
