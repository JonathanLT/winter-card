use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessCode {
    pub id: Option<i64>,
    pub code: String,
    pub active: bool,
}