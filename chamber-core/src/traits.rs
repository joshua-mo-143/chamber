use crate::core::{Database, LockedStatus};
use crate::errors::DatabaseError;
use crate::secrets::KeyFile;
use crate::Postgres;
use sqlx::PgPool;

use shuttle_persist::PersistInstance;
use crate::consts::KEYFILE_PATH;

#[async_trait::async_trait]
pub trait AppState: Clone + Send + Sync + 'static {
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
    fn check_keyfile_exists(&self) {
        if std::fs::read(KEYFILE_PATH).is_err() {
            println!("No chamber.bin file attached, generating one now...");
            let key = KeyFile::new();
            println!("Your root key is: {}", key.unseal_key());

            let encoded = bincode::serialize(&key).unwrap();

            std::fs::create_dir("data").unwrap();

            std::fs::write(KEYFILE_PATH, encoded).unwrap();
            println!("Successfully saved. Don't forget that you can generate a new chamber file from the CLI and upload it!");
        }
    }
}

#[derive(Clone)]
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

#[derive(Clone)]
pub struct RegularAppState {
    pub db: Postgres,
    pub lock: LockedStatus,
}

impl RegularAppState {
    pub fn new(db: PgPool) -> Self {
        Self {
            db: Postgres::from_pool(db),
            lock: LockedStatus::default(),
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
        let res = match self.persist.load::<KeyFile>("KEYFILE") {
            Ok(res) => res,
            Err(_) => {
        self.check_keyfile_exists();
        let res = match std::fs::read(KEYFILE_PATH) {
            Ok(res) => res,
            Err(e) => return Err(DatabaseError::IoError(e)),
        };

        let decoded: KeyFile = bincode::deserialize(&res).unwrap();

        self.persist.save::<KeyFile>("KEYFILE", decoded).unwrap();
        self.persist.load::<KeyFile>("KEYFILE").unwrap() 


        }
        };

        Ok(res)
    }
}

impl AppState for RegularAppState {
    type D = Postgres;

    fn db(&self) -> &Self::D {
        &self.db
    }
    fn locked_status(&self) -> LockedStatus {
        self.lock.to_owned()
    }
    fn get_keyfile(&self) -> Result<KeyFile, DatabaseError> {
        self.check_keyfile_exists();
        let res = match std::fs::read(KEYFILE_PATH) {
            Ok(res) => res,
            Err(e) => return Err(DatabaseError::IoError(e)),
        };

        let decoded: KeyFile = bincode::deserialize(&res).unwrap();

        Ok(decoded)
    }
}
