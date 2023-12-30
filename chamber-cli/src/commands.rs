use chamber_core::core::AuthBody;
use chamber_core::secrets::SecretInfo;
use comfy_table::Table;
use inquire::Text;
use reqwest::StatusCode;

use crate::errors::CliError;

use crate::args::{Cli, Commands, SecretsCommands, UserCommands, WebsiteCommands};

use crate::config::AppConfig;
use chamber_core::secrets::KeyFile;

pub fn parse_cli(cli: Cli, cfg: AppConfig) -> Result<(), CliError> {
    match cli.command {
        Commands::Secrets { cmd } => match cmd {
            SecretsCommands::Get(args) => {
                let Some(jwt) = cfg.clone().jwt_key() else {
                    panic!("You need to log in before you can do that!");
                };

                let website = match cfg.website() {
                    Some(res) => format!("{res}/secrets/get"),
                    None => panic!("You didn't set a URL for a Chamber instance to log into!"),
                };

                let key = match args.key {
                    Some(res) => res,
                    None => Text::new("Please enter the key you want to retrieve:").prompt()?,
                };

                let ctx = reqwest::blocking::Client::new();

                let res = ctx
                    .post(website)
                    .header("Content-Type", "application/json")
                    .header("Authorization", jwt)
                    .json(&serde_json::json!({"key":key}))
                    .send()?;

                let body = res.text()?;

                println!("{body}");
            }

            SecretsCommands::Set { key, value } => {
                let Some(jwt) = cfg.clone().jwt_key() else {
                    panic!("You need to log in before you can do that!");
                };

                let website = match cfg.website() {
                    Some(res) => format!("{res}/secrets/set"),
                    None => panic!("You didn't set a URL for a Chamber instance to log into!"),
                };

                let ctx = reqwest::blocking::Client::new();

                let res = ctx
                    .post(website)
                    .header("Content-Type", "application/json")
                    .header("Authorization", jwt)
                    .json(&serde_json::json!({"key":key,"value":value}))
                    .send()?;

                match res.status() {
                    StatusCode::CREATED => println!("Key successfully set."),
                    _ => {
                        println!("Bad credentials: {}", res.status())
                    }
                }
            }
            SecretsCommands::Update { key, tags } => {
                let Some(jwt) = cfg.clone().jwt_key() else {
                    panic!("You need to log in before you can do that!");
                };

                let website = match cfg.website() {
                    Some(res) => format!("{res}/secrets"),
                    None => panic!("You didn't set a URL for a Chamber instance to log into!"),
                };

                let ctx = reqwest::blocking::Client::new();

                let res = ctx
                    .put(website)
                    .header("Authorization", jwt)
                    .json(&serde_json::json!({
                        "key": key,
                        "update_data": tags
                    }))
                    .send()?;

                match res.status() {
                    StatusCode::OK => println!("Meme"),
                    _ => println!("Not OK!"),
                }
            }
            SecretsCommands::List(args) => {
                let Some(jwt) = cfg.clone().jwt_key() else {
                    panic!("You need to log in before you can do that!");
                };

                let website = match cfg.website() {
                    Some(res) => format!("{res}/secrets"),
                    None => panic!("You didn't set a URL for a Chamber instance to log into!"),
                };

                let ctx = reqwest::blocking::Client::new();

                let res = ctx
                    .post(website)
                    .header("Authorization", jwt)
                    .json(&serde_json::json!({
                        "tag_filter": args.tags
                    }))
                    .send()?;

                let json = res.json::<Vec<SecretInfo>>().unwrap();

                let table = secrets_table(json);

                println!("{table}");
            }
            SecretsCommands::Rm(args) => {
                let Some(jwt) = cfg.clone().jwt_key() else {
                    panic!("You need to log in before you can do that!");
                };

                let website = match cfg.website() {
                    Some(res) => format!("{res}/secrets"),
                    None => panic!("You didn't set a URL for a Chamber instance to log into!"),
                };

                let key = match args.key {
                    Some(res) => res,
                    None => Text::new("Please enter the key you want to retrieve:").prompt()?,
                };

                let ctx = reqwest::blocking::Client::new();

                let res = ctx
                    .delete(website)
                    .header("Authorization", jwt)
                    .json(&serde_json::json!({"key":key}))
                    .send()?;

                match res.status() {
                    StatusCode::OK => println!("Key successfully deleted."),
                    _ => println!("Error while deleting key: {}", res.text().unwrap()),
                }
            }
        },
        Commands::Keygen(args) => {
            let key = match args.key {
                Some(res) => KeyFile::from_key(&res),
                None => KeyFile::new(),
            };

            let encoded = bincode::serialize(&key).unwrap();

            std::fs::write("chamber.bin", encoded).unwrap();

            println!("Your root key: {}", key.unseal_key());
            println!(
                "Be sure to keep this file somewhere safe - you won't be able to get it back!"
            );
            println!("---");
        }

        Commands::Users { cmd } => match cmd {
            UserCommands::Create(args) => {
                let website = match cfg.website() {
                    Some(res) => format!("{res}/users/create"),
                    None => panic!("You didn't set a URL for a Chamber instance to log into!"),
                };

                let key = Text::new("Please enter your root key:").prompt()?;

            let username = match args.username {
                Some(res) => res,
                None => Text::new("Please enter your desired username:").prompt()?,
            };
            let password = match args.password {
                Some(res) => res,
                None => Text::new("Please enter your desired password:").prompt()?,
            };
                let ctx = reqwest::blocking::Client::new();

                let res = ctx
                    .post(website)
                    .header("Content-Type", "application/json")
                    .header("x-chamber-key", key)
                    .json(&serde_json::json!({"username": username, "password": password}))
                    .send()?;

                match res.status() {
                    StatusCode::CREATED => {
                        println!("User created! Make sure you keep the credentials somewhere safe.!");
                    }
                    _ => {
                        println!("Error: {}", res.text()?)
                    }
                }
            }
            UserCommands::Update(args) => {
                let website = match cfg.website() {
                    Some(res) => format!("{res}/users/create"),
                    None => panic!("You didn't set a URL for a Chamber instance to log into!"),
                };

                if args.access_level.is_none() & args.roles.is_none() {
                    return Err(CliError::AtLeastOneArgError);
                } 

                let key = Text::new("Please enter your root key:").prompt()?;

                let ctx = reqwest::blocking::Client::new();

                let res = ctx
                    .post(website)
                    .header("Content-Type", "application/json")
                    .header("x-chamber-key", key)
                    .json(&serde_json::json!({
                        "username": args.username, 
                        "access_level": args.access_level,
                        "roles": args.roles
                        }))
                    .send()?;

                match res.status() {
                    StatusCode::OK => {
                        println!("User has been updated.");
                    }
                    _ => {
                        println!("Error: {}", res.text()?)
                    }
                }
            }

            UserCommands::Delete(args) => {
                let website = match cfg.website() {
                    Some(res) => format!("{res}/users/delete"),
                    None => panic!("You didn't set a URL for a Chamber instance to log into!"),
                };

                let key = Text::new("Please enter your root key:").prompt()?;

            let username = match args.username {
                Some(res) => res,
                None => Text::new("Name of the user to be deleted:").prompt()?,
            };

                let ctx = reqwest::blocking::Client::new();

                let res = ctx
                    .post(website)
                    .header("Content-Type", "application/json")
                    .header("x-chamber-key", key)
                    .json(&serde_json::json!({
                        "username": username
                    }))
                    .send()?;

                match res.status() {
                    StatusCode::OK => {
                        println!("User has been deleted.");
                    }
                    _ => {
                        println!("Error: {}", res.text()?)
                    }
                }
            }
        },
        Commands::Website { cmd } => match cmd {
            WebsiteCommands::Get => match cfg.website() {
                Some(res) => println!("{res}"),
                None => println!("No website has been set!"),
            },
            WebsiteCommands::Set(args) => {
                let value = match args.value {
                Some(res) => res,
                None => Text::new("Enter the website URL:").prompt()?,
                };
                cfg.set_website(&value)?;
            }
        },

        Commands::Login(args) => {
            let username = match args.username {
                Some(res) => res,
                None => Text::new("Please enter your username:").prompt()?,
            };
            let password = match args.password {
                Some(res) => res,
                None => Text::new("Please enter your password:").prompt()?,
            };

            let ctx = reqwest::blocking::Client::new();

            let website = match cfg.to_owned().website() {
                Some(res) => format!("{res}/login"),
                None => panic!("You didn't set a URL for a Chamber instance to log into!"),
            };

            let res = ctx
                .post(website)
                .header("Content-Type", "application/json")
                .json(&serde_json::json!({
                    "username": username,
                    "password": password 
                }))
                .send()?;
        match res.status() {
            StatusCode::OK => {
            let res = res.json::<AuthBody>()?;

            let token = format!("{} {}", res.token_type, res.access_token);
            cfg.set_token(&token)?;

            println!("You've logged in successfully!");
            },
            _ => {
            println!("Something went wrong: {}", res.text()?);
            },

        }

        }

        Commands::Unseal { chamber_key } => {
            let ctx = reqwest::blocking::Client::new();

            let website = match cfg.to_owned().website() {
                Some(res) => format!("{res}/unseal"),
                None => panic!("You didn't set a URL for a Chamber instance to log into!"),
            };

            let res = ctx
                .post(website)
                .header("Content-Type", "application/json")
                .header("x-chamber-key", chamber_key)
                .send()?;

            match res.status() {
                StatusCode::OK => println!("The instance has been unsealed and is ready to use!"),
                _ => {
                    println!("{}", res.text()?);
                }
            }
        }
        Commands::Upload(args) => {
            let key = match args.key {
                Some(res) => res,
                None => Text::new("Please enter your root key:").prompt()?,
            };
            let ctx = reqwest::blocking::Client::new();

            let website = match cfg.to_owned().website() {
                Some(res) => format!("{res}/binfile"),
                None => panic!("You didn't set a URL for a Chamber instance to log into!"),
            };

            let file = std::fs::read("chamber.bin")?;

            let form = reqwest::blocking::multipart::Form::new();
            let file_as_bytes = reqwest::blocking::multipart::Part::bytes(file);

            let form = form.part("file", file_as_bytes);

            let res = ctx
                .post(website)
                .header("x-chamber-key", key)
                .multipart(form)
                .send()?;

            match res.status() {
                StatusCode::OK => {
                    println!("The new crypto key and root key have been uploaded!");
                    println!(
                        "Note that any previous secrets you stored will need to be re-uploaded."
                    );
                }
                _ => {
                    println!("{}", res.text()?);
                }
            }
        }
    }

    Ok(())
}

pub fn secrets_table(secrets: Vec<SecretInfo>) -> Table {
    let mut table = Table::new();
    table.set_header(vec!["Secret Key", "Tags"]);

    secrets.into_iter().for_each(|x| {
        table.add_row(vec![x.key, x.tags.join(", ")]);
    });

    table
}
