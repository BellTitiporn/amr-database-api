// pose struct

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Pose {
    pub x: f64,
    pub y: f64,
    pub theta: f64,
}