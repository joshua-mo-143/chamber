use boulder_db::{kv::InMemoryDatabase, core::Database};

#[derive(Clone)]
pub struct AppState {
    pub db: InMemoryDatabase,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            db: InMemoryDatabase::new(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
