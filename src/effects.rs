use std::{ffi::OsString, path::PathBuf};

use tracing::{debug, error, info, trace};

use crate::git;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EffectKind {
    WriteToFile(FileWriterOpts),
    CreateDir(CreateDirOpts),
    GitHook(GitHookOpts),
}

impl EffectKind {
    pub fn apply(self) -> Result<()> {
        match self {
            EffectKind::WriteToFile(opts) => file_writer(opts),
            EffectKind::CreateDir(opts) => dir_creator(opts),
            EffectKind::GitHook(opts) => git_hooker(opts),
        }
    }
}
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Effect {
    pub effect_kind: EffectKind,
    pub forgiving: bool,
}

pub struct EffectMachine(Vec<Effect>);
impl EffectMachine {
    pub fn run(self) -> Result<()> {
        for ef in self.0.into_iter() {
            if let Err(error) = ef.effect_kind.apply() {
                error!("something went wrong during applying that effect");

                if !ef.forgiving {
                    error!(" effect is not forgiving, not continuing");
                    return Err(error);
                }
                info!(" effect is forgiving, continuing");
            }
            trace!("done with the effect");
        }
        trace!("done with all the effects");
        Ok(())
    }
    pub fn new() -> Self {
        EffectMachine(vec![])
    }
    pub fn add(&mut self, effect: EffectKind, forgiving: bool) {
        self.0.push(Effect {
            effect_kind: effect,
            forgiving,
        })
    }
}

impl Default for EffectMachine {
    fn default() -> Self {
        Self::new()
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
    #[error("this dir already exists")]
    DirAlreadyExists,
    #[error("file with this dir name exists")]
    FileWithDirNameExists,
    #[error("failed in creating dir: {0}")]
    CouldNotCreateDir(fs_extra::error::Error),
    #[error("got error from running git command: {0}")]
    GitError(git::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FileWriterOpts {
    pub content: Vec<u8>,
    pub file_path: PathBuf,
    pub can_create: bool,
    pub can_overwrite: bool,
}

#[tracing::instrument]
fn file_writer(opts: FileWriterOpts) -> Result<()> {
    trace!(stage = "starting to write to file");

    let exists = opts
        .file_path
        .try_exists()
        .map_err(Error::CheckFileExistenceFailed)?;
    trace!(stage = "checked for existence of file");
    if !opts.can_create && !exists {
        error!(
            stage = "checked for precondition in case opts can't create",
            result = "precondition is not fullfilled"
        );
        return Err(Error::FileDoesNotExists);
    };
    if !opts.can_overwrite && exists {
        error!(
            stage = "checked for precondition in case opts can't overwrite",
            result = "precondition is not fullfilled"
        );
        return Err(Error::FileAlreadyExists);
    };

    trace!(stage = "preconditions are fullfilled");
    trace!(stage = "writing to file");
    std::fs::write(opts.file_path, opts.content).map_err(Error::CouldNotWriteToFile)
}

#[tracing::instrument]
fn dir_creator(opts: CreateDirOpts) -> Result<()> {
    trace!(stage = "dir_creator is starting");

    if opts.folder_path.exists() {
        debug!(stage = "something with that name exists");
        if !opts.folder_path.is_dir() {
            error!(stage = "file with that name exists");
            return Err(Error::FileWithDirNameExists);
        }
        if opts.ok_if_exists {
            debug!(stage = "not creating the directory, already exist");
            return Ok(());
        }
        error!(stage = "directory with that path already exists");
        return Err(Error::DirAlreadyExists);
    }
    info!(stage = "creating directory");
    fs_extra::dir::create_all(opts.folder_path, false).map_err(Error::CouldNotCreateDir)
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CreateDirOpts {
    pub folder_path: PathBuf,
    pub ok_if_exists: bool,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GitHookOpts {
    pub start_path: PathBuf,
    pub files_to_add: Vec<PathBuf>,
    pub message: String,
}

#[tracing::instrument]
fn git_hooker(opts: GitHookOpts) -> Result<()> {
    trace!("starting with git_hooker");
    let repo_root = git::repo_root(opts.start_path.clone()).map_err(Error::GitError)?;
    trace!("found the repo_root {:#?}", repo_root);

    let files: Vec<OsString> = opts
        .files_to_add
        .into_iter()
        .map(PathBuf::into_os_string)
        .collect();

    trace!("doing the git hooks");
    git::git_hook(repo_root.as_os_str().to_os_string(), files, &opts.message)
        .map_err(Error::GitError)?;
    trace!("done with the hooks");
    Ok(())
}

#[cfg(test)]
mod testing {
    #[allow(clippy::wildcard_imports)]
    use super::*;
    fn is_normal<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    fn normal_types() {
        is_normal::<EffectKind>();
    }
}
