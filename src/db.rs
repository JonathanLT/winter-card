use r2d2::{Pool};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;

pub type SqlitePool = Pool<SqliteConnectionManager>;

pub fn init_pool(path: &str) -> SqlitePool {
    let manager = SqliteConnectionManager::file(path);
    Pool::new(manager).expect("Failed to create SQLite pool")
}

pub fn init_db(pool: &SqlitePool) {
    let conn = pool.get().expect("Get connection from pool");
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS access_codes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            code TEXT NOT NULL UNIQUE,
            active BOOLEAN NOT NULL DEFAULT 1
        );
        ",
    ).expect("Failed to create tables");

    // ensure an admin access code exists
    conn.execute(
        "INSERT OR IGNORE INTO access_codes (name, code, active) VALUES (?1, ?2, ?3)",
        params!["Admin", "Winter2025", 1],
    ).expect("Failed to insert default admin");

    // create draws table
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS draws (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            giver_id INTEGER NOT NULL,
            receiver_id INTEGER NOT NULL,
            year INTEGER NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (giver_id) REFERENCES access_codes(id),
            FOREIGN KEY (receiver_id) REFERENCES access_codes(id),
            UNIQUE(giver_id, year),
            UNIQUE(receiver_id, year)
        );
        ",
    ).expect("Failed to create tables");
}