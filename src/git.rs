use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::effects::{EffectKind, EffectMachine};
type Result<T> = std::result::Result<T, Error>;
pub fn repo_root(p: &Path) -> Result<PathBuf> {
    let (repo_path, _): (_, _) = gix_discover::upwards(p).map_err(Error::CouldNotGetGitRoot)?;
    repo_path
        .into_repository_and_work_tree_directories()
        .0
        .parent()
        .ok_or(Error::DirectoryParentIsNotFound)
        .map(PathBuf::from)
}

pub fn repo_directory_name(p: &Path) -> Result<String> {
    let git_root = repo_root(p)?;
    let project_name = git_root
        .file_name()
        .ok_or(Error::CouldNotGetGitRootName)?
        .to_str()
        .ok_or(Error::DirectoryNameIsNotValidUTF8)?;
    Ok(project_name.to_owned())
}

pub fn add(repo: &str, files: &[String]) -> EffectKind {
    let args = [
        ["-C", repo, "add"]
            .iter()
            .copied()
            .map(std::borrow::ToOwned::to_owned)
            .collect(),
        files.to_owned(),
    ]
    .concat();

    EffectKind::RunExternalCommand("git".to_owned(), args, HashMap::default())
}

pub fn commit(repo: &str, commit_message: &str) -> EffectKind {
    EffectKind::RunExternalCommand(
        "git".to_owned(),
        ["-C", repo, "commit", "-m", commit_message]
            .into_iter()
            .map(std::borrow::ToOwned::to_owned)
            .collect(),
        HashMap::default(),
    )
}

pub fn push(repo: &str) -> EffectKind {
    EffectKind::RunExternalCommand(
        "git".to_owned(),
        ["-C", repo, "push"]
            .into_iter()
            .map(std::borrow::ToOwned::to_owned)
            .collect(),
        HashMap::default(),
    )
}

pub fn pull(repo: &str) -> EffectKind {
    EffectKind::RunExternalCommand(
        "git".to_owned(),
        ["-C", repo, "pull"]
            .into_iter()
            .map(std::borrow::ToOwned::to_owned)
            .collect(),
        HashMap::default(),
    )
}

pub fn full_hook(repo_root: &str, files: &[String], commit_message: &str) -> EffectMachine {
    let mut efm = EffectMachine::default();
    efm.add(add(repo_root, files), false);
    efm.add(commit(repo_root, commit_message), false);
    efm.add(pull(repo_root), false);
    efm.add(push(repo_root), false);
    efm
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not get current working directory {0}")]
    CouldNotGetCWD(std::io::Error),
    #[error("could not find a git root")]
    CouldNotGetGitRoot(gix_discover::upwards::Error),
    #[error("could not get the directory name of the git project for some reason")]
    CouldNotGetGitRootName,
    #[error("directory name is not a valid utf-8")]
    DirectoryNameIsNotValidUTF8,
    #[error("directory parent is not found")]
    DirectoryParentIsNotFound,
    #[error("error running the command: {0}")]
    CommandCouldNotBeRan(std::io::Error),
    #[error("command returned non-zero status: {0}")]
    CommandReturnedNon0StatusCode(i32),
}
