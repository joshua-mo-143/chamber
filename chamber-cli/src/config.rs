use crate::errors::ConfigError;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Deserialize, Serialize, Clone)]
pub struct AppConfig {
    website: Option<String>,
    jwt_key: Option<String>,
}

impl AppConfig {
    pub fn website(self) -> Option<String> {
        self.website.clone()
    }

    pub fn jwt_key(self) -> Option<String> {
        self.jwt_key.clone()
    }

    pub fn get() -> Result<Self, ConfigError> {
        let cfg_dir = home::home_dir().unwrap().join(".config/chamber");
        let cfg_file = home::home_dir()
            .unwrap()
            .join(".config/chamber/config.toml");
        if !cfg_dir.as_path().exists() {
            fs::create_dir_all(cfg_dir)?;
        }

        if !cfg_file.as_path().exists() {
            fs::File::create(cfg_file.clone())?;
        }

        let file = std::fs::read_to_string(cfg_file)?;

        let cfg: Self = toml::from_str(&file)?;

        Ok(cfg)
    }

    pub fn set_website(mut self, website: &str) -> Result<(), ConfigError> {
        let cfg_file = home::home_dir()
            .unwrap()
            .join(".config/chamber/config.toml");

        let website = if website.starts_with("localhost") {
            format!("http://{website}")
        } else {
            website.to_string()
        };

        if website.starts_with("http://") & !website.contains("localhost") {
            println!("Please note that HTTP is much less secure than using HTTPS.");
        }

        self.website = Some(website);

        let toml_string = toml::to_string_pretty(&self)?;

        fs::write(cfg_file, toml_string)?;

        Ok(())
    }

    pub fn set_token(mut self, token: &str) -> Result<(), ConfigError> {
        let cfg_file = home::home_dir()
            .unwrap()
            .join(".config/chamber/config.toml");

        self.jwt_key = Some(token.to_owned());

        let toml = toml::to_string_pretty(&self)?;

        fs::write(cfg_file, toml)?;

        Ok(())
    }
}
