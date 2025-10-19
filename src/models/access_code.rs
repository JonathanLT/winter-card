use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AccessCode {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub active: bool,
}