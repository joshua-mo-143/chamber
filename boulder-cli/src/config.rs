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

    pub fn get() -> Self {
        let cfg_dir = home::home_dir().unwrap().join(".config/boulder");
        let cfg_file = home::home_dir()
            .unwrap()
            .join(".config/boulder/config.toml");
        if !cfg_dir.as_path().exists() {
            fs::create_dir_all(cfg_dir).unwrap();
        }

        if !cfg_file.as_path().exists() {
            fs::File::create(cfg_file.clone()).unwrap();
        }

        let file = std::fs::read_to_string(cfg_file).unwrap();

        let cfg: Self = toml::from_str(&file).unwrap();

        cfg
    }

    pub fn set_website(mut self, website: &str) {
        let cfg_file = home::home_dir()
            .unwrap()
            .join(".config/boulder/config.toml");

        let website = if website.starts_with("localhost") {
            format!("http://{website}")
        } else {
            website.to_string()
        };

        if website.starts_with("http://") & !website.contains("localhost") {
            panic!("Stop using HTTP! HTTPS is free.");
        }

        self.website = Some(website);

        let meme = toml::to_string_pretty(&self).unwrap();

        fs::write(cfg_file, meme).unwrap();
    }

    pub fn set_token(mut self, token: &str) {
        let cfg_file = home::home_dir()
            .unwrap()
            .join(".config/boulder/config.toml");

        self.jwt_key = Some(token.to_owned());

        let meme = toml::to_string_pretty(&self).unwrap();

        fs::write(cfg_file, meme).unwrap();
    }
}
