use std::path::PathBuf;

pub fn get_git_root(p: PathBuf) -> Option<PathBuf> {
    if p.join(".git").is_dir() {
        Some(p)
    } else {
        let mut p_par = p;
        if !p_par.pop() {
            None
        } else {
            get_git_root(p_par)
        }
    }
}
pub fn directory_name_from_path(p: PathBuf) -> Result<String, Error> {
    let git_root = get_git_root(p).ok_or(Error::CouldNotGetGitRoot)?;
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
    CouldNotGetGitRoot,
    #[error("could not get the directory name of the git project for some reason")]
    CouldNotGetGitRootName,
    #[error("directory name is not a valid utf-8")]
    DirectoryNameIsNotValidUTF8,
}
