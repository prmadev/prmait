use clap::{CommandFactory, Parser, Subcommand};
use color_eyre::eyre::Result;
use color_eyre::Report;
use figment::providers::{Env, Format, Json};
use figment::Figment;
use prmait::effects::{EffectKind, EffectMachine};
use prmait::journal::Mood;
use prmait::{git, journal};
use std::env;
use std::{ffi::OsString, path::PathBuf, sync::Arc};
use time::{format_description, OffsetDateTime};

const DEFAULT_CONFIG_PATH: &str = "/home/a/.config/prmait/jnl.json";

fn main() -> Result<()> {
    // error message management
    color_eyre::install()?;

    // tracing
    tracing_subscriber::fmt::init();

    // getting arugments
    let args = Args::parse();

    // forming config out of arguments
    let config = Configs::try_from(&args.config.unwrap_or(PathBuf::from(DEFAULT_CONFIG_PATH)))?;

    // getting current time offset
    let time_offset = config
        .time_offset
        .ok_or(Report::msg("time offset for is not set"))
        .map(|(h, m, s)| time::UtcOffset::from_hms(h, m, s))??;

    // getting current time
    let now = OffsetDateTime::now_utc().to_offset(time_offset);

    let Some(command) = args.command else {
        return Ok(());
    };

    let efs = to_effect_machine(command, now, &config)?;
    efs.run()?;

    Ok(())
}

fn to_effect_machine(
    journal_command: Commands,
    now: OffsetDateTime,
    config: &Configs,
) -> Result<EffectMachine, Report> {
    Ok(match journal_command {
        Commands::Completions { shell } => {
            let mut ef = EffectMachine::default();
            ef.add(
                EffectKind::GenerateShellCompletion(shell, Args::command()),
                false,
            );
            ef
        },
                Commands::New {
                    entry,
                    tag,
                    mood,
                    people,
                } => {
                    let repo_root =  git::repo_root(&config.journal_path()?)?.to_string_lossy().into_owned();
                    journal::effectors::new_entry(
                        &journal::Entry {
                            at: now,
                            body: Arc::new(entry),
                            tag,
                            mood,
                            people,
                        },
                        &config.journal_path()?,
                        &repo_root,
                        now,
                        &config.journal_file_formatting()?,
                    )?
                }
                Commands::List => {
                    let format = time::format_description::parse_borrowed::<2>("[year]-[month]-[day] [hour]:[minute]")?;
                    journal::effectors::list_entries(
                                    &journal::Book::try_from(&config.journal_path()?)?,
                                    &format,
                                )?
                },
                Commands::Edit(edit_type) => {
                    let repo_root = git::repo_root(&config.journal_path()?)?.to_string_lossy().into_owned();
                    match edit_type {
                        JournalEditCommands::Last => journal::effectors::edit_last_entry(
                            &config.journal_path()?,
                            &journal::Book::try_from(&config.journal_path()?)?,
                            &repo_root,
                            editor(env::var_os("EDITOR"))?,
                        )?,
                        JournalEditCommands::All => journal::effectors::edit_all_entries(
                            editor(env::var_os("EDITOR"))?,
                            &journal::Book::try_from(&config.journal_path()?)?,
                            &repo_root,
                        )?,
                        JournalEditCommands::Specific { item } => {
                            journal::effectors::edit_specific_entry(
                                &config.journal_path()?,
                                &item,
                                &journal::Book::try_from(&config.journal_path()?)?,
                                &repo_root,
                                editor(env::var_os("EDITOR"))?,
                            )?
                        }
                    }
                }
                // JournalCommands::Delete => journal::handlers::delete_interactive(
                //     &config.journal_path()?,
                //     20,
                //     journal::Book::try_from(&config.journal_path()?)?,
                //     &well_known::Rfc3339,
                // )?,
            })
}

fn editor(extractor: Option<OsString>) -> Result<String> {
    let editor = extractor.ok_or(Report::msg("editor variable is not specified"))?;
    if editor.is_empty() {
        return Err(Report::msg("editor variable is not specified"));
    };
    match editor.into_string() {
        Ok(s) => Ok(s),
        Err(e) => Err(Report::msg(format!(
            "could not convert file name to string: {e:?}"
        ))),
    }
}

#[derive(Clone, Debug, Parser)]
#[command(version,about="Personal journaling, like never before", long_about = None, arg_required_else_help = true)]
pub struct Args {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Commands {
    Completions {
        /// The shell to generate the completions for
        #[arg(value_enum)]
        shell: clap_complete_command::Shell,
    },
    /// New Entry
    New {
        entry: String,
        /// Tags that apply to this item [OPTIONAL]
        #[arg(short = 't', long)]
        tag: Vec<String>,
        /// Mood associated with this entry [Required]
        #[arg(short = 'm', long)]
        mood: Mood,
        /// People related to this entry  [OPTIONAL]
        #[arg(short = 'p', long)]
        people: Vec<String>,
    },
    /// List of entries
    List,
    /// edit commands
    #[command(subcommand)]
    Edit(JournalEditCommands),
    // interactively delete an entry
    // Delete,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Subcommand)]
pub enum JournalEditCommands {
    /// Only edit the last entry
    Last,
    /// Open every entry in the editor
    All,
    /// Open only strings matching the given entry
    Specific { item: String },
}
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Configs {
    pub time_offset: Option<(i8, i8, i8)>,
    pub path: Option<PathBuf>,
    pub file_name_format: Option<String>,
}

impl TryFrom<PathBuf> for Configs {
    type Error = Error;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        Ok(Figment::new()
            .merge(Json::file(value))
            .merge(Env::prefixed("JNL_"))
            .extract()?)
    }
}
impl TryFrom<&PathBuf> for Configs {
    type Error = Error;

    fn try_from(value: &PathBuf) -> Result<Self, Self::Error> {
        Ok(Figment::new()
            .merge(Json::file(value))
            .merge(Env::prefixed("JNL_"))
            .extract()?)
    }
}
#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("could not extract configuration: {0}")]
    ExtractionFailed(#[from] figment::Error),
    #[error("The path to the directory is not given.")]
    DirDoesNotExist,
    #[error("{0} was not set in the the configuration.")]
    UnsetConfiguration(String),
    #[error("File format descriptor for journal is not valid: {0}.")]
    TheFormatIsNotValid(#[from] time::error::InvalidFormatDescription),
}

impl Configs {
    pub fn journal_path(&self) -> Result<PathBuf, Error> {
        self.clone().path.ok_or(Error::DirDoesNotExist)
    }
    pub fn journal_file_formatting(&self) -> Result<format_description::OwnedFormatItem, Error> {
        Ok(format_description::parse_owned::<2>(
            &self
                .clone()
                .file_name_format
                .ok_or(Error::UnsetConfiguration("file_name_format".to_owned()))?,
        )?)
    }
}
