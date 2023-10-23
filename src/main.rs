use clap::Parser;
use color_eyre::eyre::Result;
use prmait::{
    input::{Args, Configs},
    journal::{
        delete_interactive_handler, edit_all_entries_handler, edit_last_entry_handler,
        edit_specific_entry_handler, entry::Entry, list_entries_handler, new_journal_entry_handler,
    },
};
use std::{path::PathBuf, sync::Arc};

const DEFAULT_CONFIG_PATH: &str = "/home/a/.config/prmait/config.json";

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let config = {
        let config_file_path = match &args.config {
            Some(p) => p.clone(),
            None => PathBuf::from(DEFAULT_CONFIG_PATH),
        };
        Configs::try_from(config_file_path)?
    };

    match args.command {
        Some(general_command) => match general_command {
            prmait::input::Commands::Journal { journal_command } => match journal_command {
                prmait::input::JournalCommands::New { entry, tag } => new_journal_entry_handler(
                    Entry {
                        at: chrono::Local::now(),
                        body: Arc::new(entry),
                        tag,
                    },
                    &config.journal_path()?,
                    chrono::Local::now(),
                )?,
                prmait::input::JournalCommands::List => {
                    list_entries_handler(&config.journal_path()?)?
                }
                prmait::input::JournalCommands::Edit(edit_type) => match edit_type {
                    prmait::input::JournalEditCommands::Last => {
                        edit_last_entry_handler(&config.journal_path()?)?
                    }
                    prmait::input::JournalEditCommands::All => {
                        edit_all_entries_handler(&config.journal_path()?)?
                    }
                    prmait::input::JournalEditCommands::One { item } => {
                        edit_specific_entry_handler(&config.journal_path()?, item)?
                    }
                },
                prmait::input::JournalCommands::DeleteI => {
                    delete_interactive_handler(&config.journal_path()?)?
                }
            },
        },
        None => unreachable!("because of clap, it should not be possible to reach this point"),
    }

    Ok(())
}
