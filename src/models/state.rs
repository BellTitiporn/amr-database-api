// state machine struct

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum AmrState {
    Idle,
    Navigating { target_location: String },
    Charging,
    Error { code: u32, message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StateLog {
    pub amr_id: u64,
    pub state: AmrState,
    pub timestamp: i64,
}