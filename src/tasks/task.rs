use std::{fmt::Display, ops::Sub, path::PathBuf, str::FromStr};

use color_eyre::owo_colors::OwoColorize;
use time::{formatting::Formattable, Date, OffsetDateTime};

use crate::files::ToFileName;

use super::Error;

// const DATE_DISPLAY_FORMATTING: &str = "%Y-%m-%d %H:%M";

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Task {
    pub id: i64,
    #[serde(with = "time::serde::rfc3339")]
    pub time_created: OffsetDateTime,
    pub state_log: Vec<State>,
    pub title: String,
    pub description: Option<String>,
    pub area: Option<Area>,
    pub people: Vec<String>,
    pub projects: Vec<String>,
    pub start: Option<Date>,
    pub end: Option<Date>,
}

impl Task {
    pub fn print_colorful_with_current_duration(
        &self,
        current_time: OffsetDateTime,
        time_format_description: &(impl Formattable + ?Sized),
    ) -> Result<String, Error> {
        let mut all_buf = String::new();
        all_buf.push_str(&format!(
            "{}\t{} {}",
            self.current_state()
                .ok_or(Error::EveryTaskShouldHaveAtLeastOneState)?
                .black()
                .on_magenta(),
            "⍙".bright_black(),
            self.id.bright_black().bold(),
        ));
        all_buf.push('\n');

        all_buf.push_str(&format!("⍜ {}", self.title.bold()));
        all_buf.push('\n');

        if let Some(description) = &self.description {
            all_buf.push_str(&format!(
                "{} {}",
                "⍘".bright_black(),
                &description.bright_black()
            ));
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
            if let Some(at) = self.start {
                buf.push_str(&format!(
                    "{}",
                    at.format(time_format_description)?.italic().bright_black()
                ));
            };
            buf.push_str(&format!("{}", " ⇰ ".bright_black().bold()));
            if let Some(at) = self.end {
                let until = at.sub(current_time.date());
                buf.push_str(&format!(
                    "{}",
                    at.format(time_format_description)?.italic().bright_black()
                ));
                buf.push_str(&format!(
                    "{}",
                    format!(" ◕ {}d{}h", until.whole_days(), until.whole_hours())
                        .bright_black()
                        .bold()
                ));
            };

            buf
        });

        Ok(all_buf)
    }
    #[must_use]
    pub fn current_state(&self) -> Option<&State> {
        self.state_log.last()
    }
}

impl TryFrom<&PathBuf> for Task {
    type Error = Error;

    fn try_from(value: &PathBuf) -> Result<Self, Self::Error> {
        let content = fs_extra::file::read_to_string(value).map_err(Error::FileCouldNotBeRead)?;

        let task: Self = serde_json::from_str(&content)
            .map_err(|e| Error::FileCouldNotDeserializeEntryFromJson(e, content))?;
        Ok(task)
    }
}

impl TryFrom<PathBuf> for Task {
    type Error = Error;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let content = fs_extra::file::read_to_string(value).map_err(Error::FileCouldNotBeRead)?;

        let task: Self = serde_json::from_str(&content)
            .map_err(|e| Error::FileCouldNotDeserializeEntryFromJson(e, content))?;
        Ok(task)
    }
}
impl ToFileName for Task {
    type Error = Error;

    fn to_file_name(
        &self,
        time_format_descriptor: &(impl Formattable + ?Sized),
    ) -> Result<String, Self::Error> {
        Ok(self.time_created.format(time_format_descriptor)?)
    }
}
// impl Default for Task {
//     fn default() -> Self {
//         let now = Local::now();
//         Self {
//             id: now.timestamp(),
//             state_log: vec![TaskState::default()],
//             title: "".to_owned(),
//             description: None,
//             area: None,
//             people: vec![],
//             projects: vec![],
//             time_created: now,
//             start_to_end: TimeRange::build(None, None).unwrap(), // It should never return errors
//                                                                  // with these values.
//         }
//     }
// }
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum State {
    Backlog(#[serde(with = "time::serde::rfc3339")] OffsetDateTime),
    Abandoned(
        #[serde(with = "time::serde::rfc3339")] OffsetDateTime,
        Option<String>,
    ),
    Done(#[serde(with = "time::serde::rfc3339")] OffsetDateTime),
    ToDo(#[serde(with = "time::serde::rfc3339")] OffsetDateTime),
}

// impl Default for TaskState {
//     fn default() -> Self {
//         Self::ToDo(Local::now())
//     }
// }
impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Backlog(_) => "⚏ BKLG",
                Self::Abandoned(_, _) => "☓ ABND",
                Self::Done(_) => "☑ DONE",
                Self::ToDo(_) => "☐ TODO",
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
                Self::Work => "work",
                Self::Home => "home",
                Self::Personal => "personal",
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

    const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn normal_types() {
        is_normal::<Task>();
    }
}
