use std::path::PathBuf;
use std::{str::FromStr, sync::Arc};

use chrono::{DateTime, Local};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Task {
    pub id: i64,
    pub time_created: DateTime<Local>,
    pub state_log: Arc<[TaskState]>,
    pub title: String,
    pub description: Option<String>,
    pub area: Option<Area>,
    pub people: Vec<String>,
    pub projects: Vec<String>,
    pub deadline: Option<DateTime<Local>>,
    pub best_starting_time: Option<DateTime<Local>>,
}

impl Task {
    pub fn current_state(&self) -> Option<&TaskState> {
        self.state_log.last()
    }
}

const FILE_NAME_FORMAT: &str = "%Y-%m-%d-%H-%M-%S.json";

pub trait ToFileName {
    fn to_file_name(&self) -> String;
}

impl ToFileName for Task {
    fn to_file_name(&self) -> String {
        self.time_created.format(FILE_NAME_FORMAT).to_string()
    }
}

impl Default for Task {
    fn default() -> Self {
        let now = Local::now();
        Self {
            id: now.timestamp(),
            state_log: Arc::new([TaskState::default()]),
            title: "".to_owned(),
            description: None,
            area: None,
            people: vec![],
            projects: vec![],
            time_created: now,
            deadline: None,
            best_starting_time: None,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TaskState {
    Backlog(DateTime<Local>),
    Abandoned(DateTime<Local>, Option<String>),
    Done(DateTime<Local>),
    ToDo(DateTime<Local>),
}

impl Default for TaskState {
    fn default() -> Self {
        Self::ToDo(Local::now())
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Area {
    Work,
    Home,
    Personal,
}

impl FromStr for Area {
    type Err = AreaParsingError;

    //noinspection SpellCheckingInspection
    //noinspection SpellCheckingInspection
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "work" | "w" | "wo" | "wor" => Ok(Self::Work),
            "home" | "h" | "ho" | "hom" => Ok(Self::Home),
            "personal" | "p" | "pe" | "per" | "pers" | "perso" | "person" | "persona" => {
                Ok(Self::Personal)
            }
            _ => Err(AreaParsingError::AreaIsNotAMatch),
        }
    }
}
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, thiserror::Error)]
pub enum AreaParsingError {
    #[error("area given is not matching any particular ones")]
    AreaIsNotAMatch,
}

pub fn new_task_handler(task_dir: &PathBuf, t: Task) -> Result<(), Error> {
    _ = fs_extra::dir::create(task_dir, false).map_err(Error::DirCouldNotBeCreated);

    let file_path = task_dir.join(t.to_file_name());

    if file_path.exists() {
        return Err(Error::TaskFileAlreadyExists);
    }

    fs_extra::file::write_all(
        file_path,
        &serde_json::to_string_pretty(&t).map_err(Error::FileCouldNotSerializeEntryIntoJson)?,
    )
    .map_err(Error::FileCouldNotBeWrittenTo)?;

    Ok(())
}

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
}
