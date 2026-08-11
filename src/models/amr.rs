use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Amr {
    pub id: Option<i64>,
    pub name: String,
    pub ip_address: String,
    pub status: String,
    pub current_x: Option<f64>,
    pub current_y: Option<f64>,
    pub current_yaw: Option<f64>,
}