use axum::{
    extract::Multipart,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
    Json,
};
use axum_extra::TypedHeader;
use chamber_core::consts::KEYFILE_PATH;
use chamber_core::secrets::EncryptedSecret;
use ring::aead::{BoundKey, OpeningKey, SealingKey};

use chamber_core::secrets::{KeyFile, NonceCounter};
use serde::Deserialize;
use std::sync::Arc;

use crate::auth::Claims;
use crate::errors::ApiError;

use chamber_core::core::Database;
use chamber_core::secrets::EncryptedSecretBuilder;
use chamber_core::signing::check_signing_key_exists;
use chamber_core::traits::AppState;

use crate::header::ChamberHeader;
use chamber_core::core::CreateSecretParams;

#[tracing::instrument]
pub async fn create_secret<S: AppState>(
    State(state): State<Arc<S>>,
    _claim: Claims,
    Json(secret): Json<CreateSecretParams>,
) -> Result<impl IntoResponse, ApiError> {
    let mut keyfile = state.get_keyfile()?;

    check_signing_key_exists().unwrap();

    let new_secret = EncryptedSecretBuilder::new(secret.key, secret.value)
        .with_access_level(secret.access_level)
        .with_tags(secret.tags)
        .with_whitelist(secret.role_whitelist)
        .build(keyfile.get_crypto_seal_key(), keyfile.nonce_number);

    state.db().create_secret(new_secret).await.unwrap();

    Ok(StatusCode::CREATED)
}

#[tracing::instrument]
pub async fn delete_secret<S: AppState>(
    State(state): State<Arc<S>>,
    _claim: Claims,
    Json(SecretKey { key }): Json<SecretKey>,
) -> Result<impl IntoResponse, ApiError> {
    state.db().delete_secret(key).await?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize, Debug)]
pub struct SecretKey {
    key: String,
}

#[derive(Deserialize, Debug)]
pub struct ListSecretsArgs {
    pub tag_filter: Option<String>,
}

#[tracing::instrument(fields(secret_key = secret.key))]
pub async fn view_secret<S: AppState>(
    State(state): State<Arc<S>>,
    claim: Claims,
    Json(secret): Json<SecretKey>,
) -> Result<impl IntoResponse, ApiError> {
    let user = state.db().get_user_from_name(claim.sub).await?;
    let secret = state.db().view_secret_decrypted(user, secret.key).await?;

    let unsealer = state.get_keyfile()?.get_crypto_open_key(secret.nonce.0);

    let decrypted_secret = secret.decrypt(unsealer);

    Ok(decrypted_secret)
}

#[tracing::instrument]
pub async fn view_all_secrets<S: AppState>(
    State(state): State<Arc<S>>,
    claim: Claims,
    Json(secret): Json<ListSecretsArgs>,
) -> Result<impl IntoResponse, ApiError> {
    let user = state.db().get_user_from_name(claim.sub).await?;

    let secrets_info = state.db().view_all_secrets(user, secret.tag_filter).await?;

    Ok(Json(secrets_info))
}

#[derive(Deserialize, Clone, Debug)]
pub struct UpdateSecret {
    key: String,
    update_data: Vec<String>,
}

#[tracing::instrument]
pub async fn update_secret<S: AppState>(
    State(state): State<Arc<S>>,
    claim: Claims,
    Json(secret): Json<UpdateSecret>,
) -> Result<impl IntoResponse, ApiError> {
    let user = state.db().get_user_from_name(claim.sub).await?;
    let mut secret_key = state.db().view_secret(user, secret.clone().key).await?;

    secret_key.replace_tags(secret.update_data);

    state.db().update_secret(secret.key, secret_key).await?;

    Ok(StatusCode::OK)
}

pub async fn check_locked<S: AppState>(
    State(state): State<Arc<S>>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<impl IntoResponse, ApiError> {
    match state.locked_status().is_locked().await {
        false => Ok(next.run(req).await),
        true => Err(ApiError::Locked),
    }
}

#[tracing::instrument]
pub async fn upload_binfile<S: AppState>(
    State(state): State<Arc<S>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
    let mut data: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await.unwrap() {
        data = Some(field.bytes().await.unwrap().to_vec());
    }

    let data: Vec<u8> = data.unwrap();

    let decoded: KeyFile = bincode::deserialize(&data.clone()).unwrap();

    let secrets = state.db().view_all_secrets_admin().await?;

    let secrets: Vec<EncryptedSecret> = secrets
        .into_iter()
        .map(|mut secret| {
            let unbound_key_old = state.get_keyfile().unwrap().crypto_key().make_key();
            let unbound_key_new = decoded.crypto_key().make_key();

            let nonce_sequence_open = NonceCounter::from_num(secret.nonce.inner());
            let nonce_sequence_seal = NonceCounter::from_num(secret.nonce.inner());
            let opening_key = OpeningKey::new(unbound_key_old, nonce_sequence_open);
            let sealing_key = SealingKey::new(unbound_key_new, nonce_sequence_seal);

            secret.reencrypt(opening_key, sealing_key);

            secret
        })
        .collect();

    state.db().rekey_all_secrets(secrets).await?;

    std::fs::write(KEYFILE_PATH, data).unwrap();

    state.save_keyfile(decoded).unwrap();

    tracing::warn!("New chamberfile uploaded");

    Ok(StatusCode::OK)
}

#[tracing::instrument(skip(state))]
pub async fn unlock<S: AppState>(
    State(state): State<Arc<S>>,
    TypedHeader(auth): TypedHeader<ChamberHeader>,
) -> Result<impl IntoResponse, ApiError> {
    if auth.key() != state.get_keyfile()?.unseal_key() {
        tracing::warn!("Unseal didn't match {}", state.get_keyfile()?.unseal_key());
        return Err(ApiError::Forbidden);
    }

    match state.locked_status().unlock().await {
        Ok(true) => {
            tracing::info!("Vault has been unlocked!");
            Ok(StatusCode::OK)
        }
        Ok(false) => Err(ApiError::Forbidden),
        Err(_e) => Err(ApiError::Forbidden),
    }
}
