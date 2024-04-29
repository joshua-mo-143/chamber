use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Encryption error")]
    EncryptionError,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("SQL error: {0}")]
    SQLError(#[from] sqlx::Error),
    #[error("bincode error: {0}")]
    Bincode(bincode::ErrorKind)
}
