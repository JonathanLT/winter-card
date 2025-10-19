use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Draw {
    pub id: i64,
    pub giver_id: i64,
    pub receiver_id: i64,
    pub year: i32,
    pub created_at: String,
}