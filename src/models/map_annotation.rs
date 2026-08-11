use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct MapAnnotation {
    pub id: Option<i64>,
    pub map_id: i64,
    pub name: String,
    pub line_type: String,
    pub start_x: f64,
    pub start_y: f64,
    pub end_x: f64,
    pub end_y: f64,
}