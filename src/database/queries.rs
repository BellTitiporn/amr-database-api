use sqlx::SqlitePool;
use crate::models::{Pose, AmrState};

pub struct DatabaseRepository {
    pub pool: SqlitePool,
}

impl DatabaseRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 1. ดึงพิกัด (Pose) ล่าสุดจากชื่อ AMR
    pub async fn get_pose_by_name(&self, amr_name: &str) -> Result<Pose, sqlx::Error> {
        let row: Option<(Option<f64>, Option<f64>, Option<f64>)> = sqlx::query_as(
            r#"
            SELECT pose_x, pose_y, pose_theta
            FROM amrs
            WHERE name = ?1
            "#,
        )
        .bind(amr_name)
        .fetch_optional(&self.pool)
        .await?;

        if let Some((Some(x), Some(y), Some(theta))) = row {
            return Ok(Pose { x, y, theta });
        }

        Ok(Pose { x: 0.0, y: 0.0, theta: 0.0 })
    }

    /// 2. อัปเดตพิกัด (Pose) ของ AMR
    pub async fn update_pose(&self, amr_name: &str, pose: &Pose) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE amrs
            SET pose_x = ?1, pose_y = ?2, pose_theta = ?3
            WHERE name = ?4
            "#,
        )
        .bind(pose.x)
        .bind(pose.y)
        .bind(pose.theta)
        .bind(amr_name)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 3. อัปเดตสถานะ (State) ของ AMR ตาม ID
    pub async fn update_amr_state(&self, amr_id: u64, state: AmrState) -> Result<(), sqlx::Error> {
        let state_str = format!("{:?}", state);
        sqlx::query(
            r#"
            UPDATE amrs
            SET state_type = ?1
            WHERE id = ?2
            "#,
        )
        .bind(state_str)
        .bind(amr_id as i64)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}