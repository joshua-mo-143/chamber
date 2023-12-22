use boulder_core::core::Database;
use std::sync::Arc;

pub type DynDatabase = Arc<dyn Database  + Send + Sync + 'static>;
