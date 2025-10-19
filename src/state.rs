use std::sync::Mutex;
use crate::db::SqlitePool;

pub struct AppState {
    pub is_authenticated: Mutex<bool>,
    pub db_pool: SqlitePool,
}

impl AppState {
    pub fn new(db_pool: SqlitePool) -> Self {
        Self {
            is_authenticated: Mutex::new(false),
            db_pool,
        }
    }
}