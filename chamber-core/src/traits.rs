use crate::core::{LockedStatus, Database};
use crate::errors::DatabaseError;
use sqlx::PgPool;
use crate::Postgres;
use crate::secrets::KeyFile;
use serde::Serialize;
use shuttle_persist::PersistInstance;

#[async_trait::async_trait]
pub trait AppState: Clone + Send + Sync + 'static {
    type D: Database;

    fn db(&self) -> &Self::D;
    fn locked_status(&self) -> LockedStatus; 
    fn get_keyfile(&self) -> KeyFile;
    async fn unlock(&self, key: String) -> Result<bool, DatabaseError> {
        let keyfile = self.get_keyfile();

        if key != keyfile.unseal_key() {
            return Err(DatabaseError::Forbidden);
        }

        self.locked_status().unlock().await.unwrap();

        Ok(true)
    }
}

#[derive(Clone)]
pub struct ShuttleAppState {
    pub db: Postgres,
    pub lock: LockedStatus,
    pub persist: PersistInstance
}

impl ShuttleAppState {
    fn new(db: PgPool, persist: PersistInstance) -> Self {
        Self {
            db: Postgres::from_pool(db),
            lock: LockedStatus::default(),
            persist
        }
    }
}

#[derive(Clone)]
pub struct RegularAppState {
    pub db: Postgres,
    pub lock: LockedStatus,
}

impl RegularAppState {
    fn new(db: PgPool) -> Self {
        Self {
            db: Postgres::from_pool(db),
            lock: LockedStatus::default()
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

    fn get_keyfile(&self) -> KeyFile {
         self.persist.load::<KeyFile>("KEYFILE").unwrap() 

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
    fn get_keyfile(&self) -> KeyFile {
        let res = std::fs::read("boulder.bin").unwrap();

        let decoded: KeyFile = bincode::deserialize(&res).unwrap();

        decoded
    }
}
