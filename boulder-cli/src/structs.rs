use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Secrets {
        #[command(subcommand)]
        cmd: SecretsCommands,
    },
    Users {
        #[command(subcommand)]
        cmd: UserCommands,
    },
    Website {
        #[command(subcommand)]
        cmd: WebsiteCommands,
    },
    Login {
        api_key: Option<String>,
    },

    Unseal {
        boulder_key: String,
    },
}

#[derive(Subcommand)]
pub enum UserCommands {
    Create,
}

#[derive(Subcommand)]
pub enum SecretsCommands {
    Get { key: String },
    Set { key: String, value: String },
    List
}

#[derive(Subcommand)]
pub enum WebsiteCommands {
    Get,
    Set { value: String },
}
