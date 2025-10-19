use std::sync::Mutex;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use crate::models::access_code::AccessCode;

pub struct AppState {
    pub db_pool: Pool<SqliteConnectionManager>,
    pub is_authenticated: Mutex<bool>,
    pub current_user: Mutex<Option<i64>>,
    pub current_access_code: Mutex<Option<AccessCode>>, // Nouveau champ
}

impl AppState {
    pub fn new(db_pool: Pool<SqliteConnectionManager>) -> Self {
        Self {
            db_pool,
            is_authenticated: Mutex::new(false),
            current_user: Mutex::new(None),
            current_access_code: Mutex::new(None),
        }
    }
}