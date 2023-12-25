use chamber_cli::args::Cli;
use chamber_cli::commands::parse_cli;
use chamber_cli::config::AppConfig;
use chamber_cli::errors::CliError;
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
