use std::{env::current_dir, str::FromStr, sync::Arc};

use chrono::{DateTime, Local};

use crate::git::get_git_root;

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
        let now = chrono::Local::now();
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
        Self::ToDo(chrono::Local::now())
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

fn new_task_handler(t: Task) -> Result<(), Error> {
    todo!()
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, thiserror::Error)]
pub enum Error {}
