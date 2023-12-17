use clap::{Parser};
use inquire::Text;
use boulder_server::auth::AuthBody;
use reqwest::StatusCode; 

pub mod structs;
use structs::{Cli, Commands, WebsiteCommands, UserCommands, SecretsCommands};

pub mod config;
use config::AppConfig;

fn main() {
    let cfg = AppConfig::get();
    let cli = Cli::parse();

    match cli.command {
        Commands::Secrets { cmd }  => {
            match cmd {
                SecretsCommands::Get { key } => {
                let jwt = match cfg.clone().jwt_key() {
                        Some(res) => res,
                        None => panic!("You need to log in before you can do that!")
                   };

                let website = match cfg.website() {
                    Some(res) => format!("{res}/secrets/get"),
                    None => panic!("You didn't set a URL for a Boulder instance to log into!")
                };

                let ctx = reqwest::blocking::Client::new();
    
                let res = ctx.post(website)
                    .header("Content-Type", "application/json")
                    .header("Authorization", jwt)
                    .json(&serde_json::json!({"key":key}))
                    .send()
                    .unwrap();
                
                let body = res.text().unwrap();

                println!("{body}");  
                }

                SecretsCommands::Set { key, value } => {
                let jwt = match cfg.clone().jwt_key() {
                        Some(res) => res,
                        None => panic!("You need to log in before you can do that!")
                   };

                let website = match cfg.website() {
                    Some(res) => format!("{res}/secrets/set"),
                    None => panic!("You didn't set a URL for a Boulder instance to log into!")
                };

                let ctx = reqwest::blocking::Client::new();
    
                let res = ctx.post(website)
                    .header("Content-Type", "application/json")
                    .header("Authorization", jwt)
                    .json(&serde_json::json!({"key":key,"value":value}))
                    .send()
                    .unwrap();
                
            match res.status() {
                StatusCode::CREATED => println!("Key successfully set."),
                _ => {println!("Bad credentials: {}", res.status())}
            }
                }
            }             
        }
        Commands::Users { cmd } => {
            match cmd {
                UserCommands::Create => {

                let website = match cfg.website() {
                    Some(res) => format!("{res}/users/create"),
                    None => panic!("You didn't set a URL for a Boulder instance to log into!")
                };

                let name = Text::new("Please enter your root key").prompt();  
                let res = match name {
                    Ok(res) => res,
                    Err(e) => panic!("An error occurred while trying to store root key: {e}")
                };

                let ctx = reqwest::blocking::Client::new();
    
                let res = ctx.post(website)
                    .header("Content-Type", "application/json")
                    .header("x-boulder-key", res)
                    .json(&serde_json::json!({"name":"josh"}))
                    .send()
                    .unwrap();
                
                let body = res.text().unwrap();

                println!("{body}"); 

            }
        }
        }
        Commands::Website { cmd }  => {
            match cmd {
                WebsiteCommands::Get => {
                    match cfg.website() {
                        Some(res) => println!("{res}"),
                        None => println!("No website has been set!"),
                    }
                },
                WebsiteCommands::Set{ value } => {
                    cfg.set_website(&value);
                }
            }
        },

        Commands::Login { api_key } => {
            let ctx = reqwest::blocking::Client::new();

            let website = match cfg.to_owned().website() {
                Some(res) => format!("{res}/login"),
                None => panic!("You didn't set a URL for a Boulder instance to log into!")
            };

            let res = ctx.post(website) 
                    .header("Content-Type", "application/json")
                    .json(&serde_json::json!({"password": api_key }))
                    .send()
                    .unwrap();

            let res = res.json::<AuthBody>().unwrap();

            let meme = format!("{} {}", res.token_type, res.access_token);
            cfg.set_token(&meme);

            println!("You've logged in successfully!");
        }

        Commands::Unseal { boulder_key } => {
            let ctx = reqwest::blocking::Client::new();

            let website = match cfg.to_owned().website() {
                Some(res) => format!("{res}/unseal"),
                None => panic!("You didn't set a URL for a Boulder instance to log into!")
            };

            let res = ctx.post(website) 
                    .header("Content-Type", "application/json")
                    .header("x-boulder-key", boulder_key)
                    .send()
                    .unwrap();

            match res.status() {
                StatusCode::OK => println!("The database has been unsealed and is ready to use!"),
                _ => {println!("Bad credentials.")}
            }
        }
    }
}
