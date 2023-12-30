#[derive(Debug)]
pub enum DatabaseError {
    KeyNotFound,
    UserNotFound,
    UserAlreadyExists,
    RoleNotFound,
    RoleAlreadyExists,
    Forbidden,
    Utf8Error,
    EncryptionError,
    IoError(std::io::Error),
    SQLError(sqlx::Error),
    Argon2Error(argon2::password_hash::Error)
}

impl From<std::str::Utf8Error> for DatabaseError {
    fn from(_error: std::str::Utf8Error) -> Self {
        Self::Utf8Error
    }
}

impl From<sqlx::Error> for DatabaseError {
    fn from(err: sqlx::Error) -> Self {
        Self::SQLError(err)
    }
}

impl From<argon2::password_hash::Error> for DatabaseError {
    fn from(err: argon2::password_hash::Error) -> Self {
        Self::Argon2Error(err)
    }
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::KeyNotFound => write!(f, "Key wasn't found"),
            Self::UserNotFound => write!(f, "User wasn't found"),
            Self::UserAlreadyExists => write!(f, "User already exists"),
            Self::RoleNotFound => write!(
                f,
                "Attempted to delete a role that wasn't found for a given user"
            ),
            Self::RoleAlreadyExists => {
                write!(f, "Attempted to add role that already existed for user")
            }
            Self::Forbidden => write!(
                f,
                "User attempted to access a key that they don't have access to"
            ),
            Self::Utf8Error => write!(f, "Error while trying to convert bytes to UTF8 string"),
            Self::EncryptionError => write!(f, "Error while trying to encrypt or decrypt a value"),
            Self::SQLError(e) => write!(f, "SQL error: {e}"),
            Self::IoError(e) => write!(f, "IO error: {e}"),
            Self::Argon2Error(e) => write!(f, "Hashing error: {e}"),
        }
    }
}

impl std::error::Error for DatabaseError {}
