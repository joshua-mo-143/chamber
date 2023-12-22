use clap::Parser;
use boulder_cli::args::Cli;
use boulder_cli::commands::parse_cli;
use boulder_cli::config::AppConfig;
use boulder_cli::errors::CliError;

fn main() -> Result<(), CliError> {
    let cfg = AppConfig::get()?;
    let cli = Cli::parse();

    if let Err(e) = parse_cli(cli, cfg) {
        println!("There was an error: {e}");
        return Err(e);
    }
    Ok(())
}
