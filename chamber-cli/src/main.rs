pub mod args;
pub mod commands;
pub mod config;
pub mod errors;

use crate::args::Cli;
use crate::commands::parse_cli;
use crate::config::AppConfig;
use crate::errors::CliError;
use clap::Parser;

fn main() -> Result<(), CliError> {
    let cfg = AppConfig::get()?;
    let cli = Cli::parse();

    if let Err(e) = parse_cli(cli, cfg) {
        println!("There was an error: {e}");
        return Err(e);
    }
    Ok(())
}
