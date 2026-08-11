// DB connection and pool management

use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::str::FromStr;
use crate::utils::AppError;

pub struct DbPool {
    pub pool: SqlitePool,
}

impl DbPool {
    /// สร้าง Connection Pool และสร้างไฟล์ DB ให้อัตโนมัติหากยังไม่มี
    pub async fn new(db_path: &str) -> Result<Self, AppError> {
        // กำหนด Options: ถ้าไม่มีไฟล์ตาม path ให้สร้างขึ้นมาใหม่ทันที
        let options = SqliteConnectOptions::from_str(db_path)
            .map_err(|e| AppError::DatabaseError(e.to_string()))?
            .create_if_missing(true);

        // สร้าง Connection Pool
        let pool = SqlitePool::connect_with(options)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // ตัวอย่าง: สร้างตารางอัตโนมัติหากยังไม่มี (Auto-Migration)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS amrs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL,
                pose_x REAL,
                pose_y REAL,
                pose_theta REAL,
                state_type TEXT NOT NULL DEFAULT 'Idle',
                state_target_location TEXT,
                state_error_code INTEGER,
                state_error_message TEXT,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            "#,
        )
        .execute(&pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(Self { pool })
    }
}