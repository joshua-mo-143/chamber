use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Key wasn't found")]
    KeyNotFound,
    #[error("User wasn't found")]
    UserNotFound,
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("Role doesn't exist")]
    RoleNotFound,
    #[error("Role already exists")]
    RoleAlreadyExists,
    #[error("Forbidden")]
    Forbidden,
    #[error("UTF8 error")]
    Utf8Error,
    #[error("Encryption error")]
    EncryptionError,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("SQL error: {0}")]
    SQLError(#[from] sqlx::Error),
    #[error("Argon2id error: {0}")]
    Argon2Error(argon2::password_hash::Error),
    #[error("shuttle-persist error: {0}")]
    ShuttlePersist(#[from] shuttle_persist::PersistError),
    #[error("bincode error: {0}")]
    Bincode(bincode::ErrorKind)
}

impl From<argon2::password_hash::Error> for DatabaseError {
    fn from(e: argon2::password_hash::Error) -> Self {
        Self::Argon2Error(e)
    }
}

impl From<Box<bincode::ErrorKind>> for DatabaseError {
    fn from(e: Box<bincode::ErrorKind>) -> Self {
        Self::Bincode(*e)
    }

}
