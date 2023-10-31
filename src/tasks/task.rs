use std::{fmt::Display, path::PathBuf, str::FromStr, sync::Arc};

use chrono::{DateTime, Local};
use color_eyre::owo_colors::OwoColorize;

use crate::files::ToFileName;

use super::{Error, FILE_NAME_FORMAT};

const DATE_DISPLAY_FORMATTING: &str = "%Y-%m-%d %H:%M:%S";

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
impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &{
            let mut all_buf = String::new();
            all_buf.push_str(&format!(
                "{}\t{}\t{}",
                self.current_state()
                    .expect("every task should have a current_state")
                    .black()
                    .on_magenta(),
                self.id.black().on_white().bold(),
                self.title.bold().on_green().black(),
            ));
            all_buf.push('\n');

            if let Some(description) = &self.description {
                all_buf.push_str(&description.to_string());
                all_buf.push('\n');
            }

            all_buf.push_str(&{
                let mut buf = String::new();
                if let Some(content) = &self.area {
                    buf.push_str(&format!("{}{} ", "#".green(), content.green()));
                };
                self.projects.iter().for_each(|project| {
                    buf.push_str(&format!("{}{} ", "?".yellow(), project.yellow()));
                });
                self.people.iter().for_each(|person| {
                    buf.push_str(&format!("{}{} ", "@".blue().bold(), person.blue()));
                });

                buf
            });

            all_buf.push('\n');

            all_buf.push_str(&{
                let mut buf = String::new();
                if let Some(at) = self.best_starting_time {
                    buf.push_str(&format!(
                        "start: {}\t",
                        at.format(DATE_DISPLAY_FORMATTING).italic()
                    ));
                };
                if let Some(at) = self.deadline {
                    buf.push_str(&format!(
                        "deadline: {}\t",
                        at.format(DATE_DISPLAY_FORMATTING).italic()
                    ));
                };

                buf
            });

            all_buf
        })
    }
}

impl Task {
    pub fn current_state(&self) -> Option<&TaskState> {
        self.state_log.last()
    }
}

impl TryFrom<PathBuf> for Task {
    type Error = Error;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let content = fs_extra::file::read_to_string(value).map_err(Error::FileCouldNotBeRead)?;

        let task: Task =
            serde_json::from_str(&content).map_err(Error::FileCouldNotDeserializeEntryFromJson)?;
        Ok(task)
    }
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
impl Display for TaskState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TaskState::Backlog(_) => "BKLG",
                TaskState::Abandoned(_, _) => "ABND",
                TaskState::Done(_) => "DONE",
                TaskState::ToDo(_) => "TODO",
            }
        )
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Area {
    Work,
    Home,
    Personal,
}

impl Display for Area {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Area::Work => "work",
                Area::Home => "home",
                Area::Personal => "personal",
            }
        )
    }
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
#[cfg(test)]
mod testing {
    use super::*;

    fn is_normal<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    fn normal_types() {
        is_normal::<Task>();
    }
}
