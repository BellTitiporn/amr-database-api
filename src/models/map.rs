// Map data struct

use serde::{Deserialize, Serialize};
use super::pose::Pose;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapData {
    pub id: String,
    pub name: String,
    pub resolution: f64,          // เมตร/พิกเซล
    pub origin: Pose,            // พิกัดเริ่มต้นของแผนที่
    pub filter_map_data: Vec<u8>, // occupancy grid หรือ binary filter data
}