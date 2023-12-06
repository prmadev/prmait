use clap::{CommandFactory, Parser};
use color_eyre::eyre::Result;
use color_eyre::Report;
use prmait::effects::{EffectKind, EffectMachine};
use prmait::input::{Args, Commands, Configs};
use prmait::river;
use std::path::PathBuf;

const DEFAULT_CONFIG_PATH: &str = "/home/a/.config/prmait/rvr.json";

fn main() -> Result<()> {
    // error message management
    color_eyre::install()?;

    // tracing
    tracing_subscriber::fmt::init();

    // getting arugments
    let args = Args::parse();

    // forming config out of arguments
    let config = Configs::try_from(&args.config.unwrap_or(PathBuf::from(DEFAULT_CONFIG_PATH)))?;

    // getting current time
    let Some(general_command) = args.command else {
        return Ok(());
    };

    let efs = to_effect_machine(general_command, config)?;
    efs.run()?;

    Ok(())
}

fn to_effect_machine(general_command: Commands, config: Configs) -> Result<EffectMachine, Report> {
    Ok(match general_command {
        Commands::Completions { shell } => {
            let mut ef = EffectMachine::default();
            ef.add(
                EffectKind::GenerateShellCompletion(shell, Args::command()),
                false,
            );
            ef
        }
        Commands::River => {
            let river_config = &config;

            river::run(
                river_config.border_width,
                &river_config.colors,
                &river_config.hardware,
                &river_config.startups,
                &river_config.apps,
            )?
        }
    })
}
