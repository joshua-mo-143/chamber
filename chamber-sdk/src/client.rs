use crate::consts::{GET_SECRETS_URL, LOGIN_URL, GET_SECRETS_BY_TAG_URL};
use chamber_shared::SecretPublic;
use reqwest::Client as ReqClient;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use url::Url;

pub struct Client {
    ctx: ReqClient,
    url: Url,
    credentials: Credentials,
}

impl Client {
    fn builder() -> ClientBuilder {
        ClientBuilder {
            ctx: ReqClient::new(),
            url: None,
            credentials: None,
        }
    }
}

impl Client {
    pub async fn login(self, username: String, password: String) -> Result<Self, ClientError> {
        let json = json!({
            "username": username,
            "password": password
        });

        let response = self
            .ctx
            .post(format!("{}{}", self.url, LOGIN_URL))
            .json(&json)
            .send()
            .await?;

        let token = match response.status() {
            StatusCode::OK => response.json::<AuthBody>().await?,
            _ => return Err(ClientError::RequestError(response.text().await?)),
        };

        self.credentials
            .to_owned()
            .set_jwt(format!("{} {}", token.token_type, token.access_token));

        Ok(self)
    }

    pub async fn get_secret(&self, key: &str) -> Result<String, ClientError> {
        let jwt = match &self.credentials.jwt {
            Some(res) => res,
            None => todo!("Implement error here"),
        };

        let json = json!({
            "key": key
        });

        let response = self
            .ctx
            .post(format!("{}{}", self.url, GET_SECRETS_URL))
            .header("Authorization", jwt)
            .json(&json)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(response.text().await?),
            _ => Err(ClientError::RequestError(response.text().await?)),
        }
    }

    pub async fn get_secrets_by_tag(&self, tag: &str) -> Result<Vec<SecretPublic>, ClientError> {
        let jwt = match &self.credentials.jwt {
            Some(res) => res,
            None => todo!("Implement error here"),
        };

        let json = json!({
            "key": tag
        });

        let response = self
            .ctx
            .post(format!("{}{}", self.url, GET_SECRETS_BY_TAG_URL))
            .header("Authorization", jwt)
            .json(&json)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(response.json::<Vec<SecretPublic>>().await?),
            _ => Err(ClientError::RequestError(response.text().await?)),
        }
    }

    pub async fn get_secret_info_with_tag(&self, tag: &str) -> Result<Vec<SecretInfo>, ClientError> {
        let jwt = match &self.credentials.jwt {
            Some(res) => res,
            None => todo!("Implement error here"),
        };

        let json = json!({
            "tag_filter": tag
        });

        let response = self
            .ctx
            .post(format!("{}{}", self.url, GET_SECRETS_URL))
            .header("Authorization", jwt)
            .json(&json)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(response.json::<Vec<SecretInfo>>().await?),
            _ => Err(ClientError::RequestError(response.text().await?)),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SecretInfo {
    pub key: String,
    pub tags: Vec<String>,
    pub access_level: i32,
    pub role_whitelist: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthBody {
    pub access_token: String,
    pub token_type: String,
}

pub struct ClientBuilder {
    ctx: ReqClient,
    url: Option<Url>,
    credentials: Option<Credentials>,
}

impl ClientBuilder {
    fn url(mut self, url: &str) -> Self {
        let url = Url::parse(url).unwrap();

        self.url = Some(url);
        self
    }

    fn credentials(mut self, api_key: &str) -> Self {
        let creds = Credentials::new(api_key);
        self.credentials = Some(creds);

        self
    }

    fn build(self) -> Client {
        if self.url.is_none() | self.credentials.is_none() {
            panic!("The URL or API key is unset!");
        }

        Client {
            ctx: self.ctx,
            url: self.url.unwrap(),
            credentials: self.credentials.unwrap(),
        }
    }
}

#[derive(Clone)]
pub struct Credentials {
    api_key: String,
    jwt: Option<String>,
}

impl Credentials {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_owned(),
            jwt: None,
        }
    }

    fn set_jwt(mut self, jwt: String) {
        self.jwt = Some(jwt);
    }
}

#[derive(thiserror::Error, Debug)]
enum ClientError {
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Error during HTTP request: {0}")]
    RequestError(String),
}
