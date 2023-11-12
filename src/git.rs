use std::path::PathBuf;
pub fn git_root(p: PathBuf) -> Result<PathBuf, Error> {
    let (repo_path, _): (_, _) = gix_discover::upwards(&p).map_err(Error::CouldNotGetGitRoot)?;
    repo_path
        .into_repository_and_work_tree_directories()
        .0
        .parent()
        .ok_or(Error::DirectoryParentIsNotFound)
        .map(PathBuf::from)
}
pub fn git_directory_name(p: PathBuf) -> Result<String, Error> {
    let git_root = git_root(p)?;
    let project_name = git_root
        .file_name()
        .ok_or(Error::CouldNotGetGitRootName)?
        .to_str()
        .ok_or(Error::DirectoryNameIsNotValidUTF8)?;
    Ok(project_name.to_owned())
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
}
