use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
    process,
};
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

pub fn add(repo: &OsStr, files: &[OsString]) -> Result<()> {
    let git_command = OsStr::new("git");
    let repo_arg = OsStr::new("-C");
    let command = OsStr::new("add");
    let args = [
        [
            repo_arg.to_os_string(),
            repo.to_owned(),
            command.to_os_string(),
        ]
        .to_vec(),
        files.to_owned(),
    ]
    .concat();

    process_command(process::Command::new(git_command).args(args))
}

pub fn commit(repo: &OsStr, commit_message: &str) -> Result<()> {
    let git_command: &OsStr = OsStr::new("git");
    let repo_arg: &OsStr = OsStr::new("-C");
    let command: &OsStr = OsStr::new("commit");
    let command_flag: &OsStr = OsStr::new("-m");
    let args = [
        repo_arg.to_os_string(),
        repo.to_owned(),
        command.to_os_string(),
        command_flag.to_os_string(),
        OsStr::new(commit_message).to_os_string(),
    ];

    process_command(process::Command::new(git_command).args(args))
}

pub fn push(repo: &OsStr) -> Result<()> {
    let git_command: &OsStr = OsStr::new("git");
    let repo_arg: &OsStr = OsStr::new("-C");
    let command: &OsStr = OsStr::new("push");
    let args = [repo_arg, repo, command];

    process_command(process::Command::new(git_command).args(args))
}
pub fn pull(repo: &OsStr) -> Result<()> {
    let git_command: &OsStr = OsStr::new("git");
    let repo_arg: &OsStr = OsStr::new("-C");
    let command: &OsStr = OsStr::new("pull");
    let args = [repo_arg, repo, command];

    process_command(process::Command::new(git_command).args(args))
}

fn process_command(cmd: &mut process::Command) -> Result<()> {
    if let Some(status) = cmd.status().map_err(Error::CommandCouldNotBeRan)?.code() {
        if status != 0_i32 {
            return Err(Error::CommandReturnedNon0StatusCode(status));
        }
    };
    Ok(())
}
pub fn full_hook(repo_root: &OsStr, files: &[OsString], commit_message: &str) -> Result<()> {
    add(repo_root, files)?;
    commit(repo_root, commit_message)?;
    pull(repo_root)?;
    push(repo_root)?;
    Ok(())
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
