use std::fmt;

#[derive(Debug)]
pub enum CliError {
    ConfigError(ConfigError),
    IoError(std::io::Error),
    RequestError(reqwest::Error),
    PromptError(inquire::error::InquireError),
    AtLeastOneArgError
}

impl std::error::Error for CliError {}
impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ConfigError(err) => write!(f, "Config error - {err}"),
            Self::RequestError(err) => write!(f, "Error while using HTTP request: {err}"),
            Self::PromptError(err) => write!(f, "Error while attempting to use prompt: {err}"),
            Self::IoError(err) => write!(f, "Error during file I/O: {err}"),
            Self::AtLeastOneArgError => write!(f, "You need at least one option filled."),
        }
    }
}

impl From<ConfigError> for CliError {
    fn from(err: ConfigError) -> Self {
        Self::ConfigError(err)
    }
}

impl From<reqwest::Error> for CliError {
    fn from(err: reqwest::Error) -> Self {
        Self::RequestError(err)
    }
}

impl From<inquire::error::InquireError> for CliError {
    fn from(err: inquire::error::InquireError) -> Self {
        Self::PromptError(err)
    }
}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Deserialization(toml::de::Error),
    Serialization(toml::ser::Error),
    IoError(std::io::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Deserialization(err) => write!(f, "Config error during deserialization: {err}"),
            Self::Serialization(err) => write!(f, "Config error during serialization: {err}"),
            Self::IoError(err) => write!(f, "Config error during file I/O: {err}"),
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> Self {
        Self::Deserialization(err)
    }
}

impl From<toml::ser::Error> for ConfigError {
    fn from(err: toml::ser::Error) -> Self {
        Self::Serialization(err)
    }
}
