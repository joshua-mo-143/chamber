use crate::core::{Database, LockedStatus};
use crate::errors::DatabaseError;
use chamber_crypto::secrets::KeyFile;
use crate::Postgres;
use sqlx::PgPool;

use crate::consts::KEYFILE_PATH;
use shuttle_persist::PersistInstance;

#[async_trait::async_trait]
pub trait AppState: std::fmt::Debug + Clone + Send + Sync + 'static {
    type D: Database;

    fn db(&self) -> &Self::D;
    fn locked_status(&self) -> LockedStatus;
    fn get_keyfile(&self) -> Result<KeyFile, DatabaseError>;
    async fn unlock(&self, key: String) -> Result<bool, DatabaseError> {
        let keyfile = self.get_keyfile();

        if key != keyfile?.unseal_key() {
            return Err(DatabaseError::Forbidden);
        }

        self.locked_status().unlock().await.unwrap();

        Ok(true)
    }

    #[tracing::instrument]
    fn check_keyfile_exists(&self) {
        if std::fs::read(KEYFILE_PATH).is_err() {
            println!("No chamber.bin file attached, generating one now...");
            let key = KeyFile::new();
            tracing::warn!("Your root key is: {}", key.unseal_key());

            let encoded = bincode::serialize(&key).unwrap();

            std::fs::create_dir("data").unwrap();

            std::fs::write(KEYFILE_PATH, encoded).unwrap();
            println!("Successfully saved. Don't forget that you can generate a new chamber file from the CLI and upload it!");
        }
    }

    fn save_keyfile(&self, keyfile: KeyFile) -> Result<(), DatabaseError>;
}

#[derive(Clone, Debug)]
pub struct ShuttleAppState {
    pub db: Postgres,
    pub lock: LockedStatus,
    pub persist: PersistInstance,
}

impl ShuttleAppState {
    pub fn new(db: PgPool, persist: PersistInstance) -> Self {
        Self {
            db: Postgres::from_pool(db),
            lock: LockedStatus::default(),
            persist,
        }
    }
}

impl AppState for ShuttleAppState {
    type D = Postgres;

    fn db(&self) -> &Self::D {
        &self.db
    }
    fn locked_status(&self) -> LockedStatus {
        self.lock.to_owned()
    }

    fn get_keyfile(&self) -> Result<KeyFile, DatabaseError> {
        let mut res = match self.persist.load::<KeyFile>("KEYFILE") {
            Ok(res) => res,
            Err(_) => {
                self.check_keyfile_exists();
                let res = match std::fs::read(KEYFILE_PATH) {
                    Ok(res) => res,
                    Err(e) => return Err(DatabaseError::IoError(e)),
                };

                let decoded: KeyFile = bincode::deserialize(&res)?;

                self.persist.save::<KeyFile>("KEYFILE", decoded)?;
                self.persist.load::<KeyFile>("KEYFILE").unwrap()
            }
        };

        res.nonce_number += 1;

        Ok(res)
    }

    fn save_keyfile(&self, keyfile: KeyFile) -> Result<(), DatabaseError> {
        self.persist.save::<KeyFile>("KEYFILE", keyfile)?;

        Ok(())
    }
}
