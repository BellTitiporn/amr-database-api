use std::sync::Arc;
use tokio::net::TcpListener;

use amr_database::{
    api::create_router,
    database::{DatabaseRepository, DbPool},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting AMR Database Service...");

    // 1. กำหนด Path สำหรับไฟล์ SQLite Database
    let db_path = "sqlite://amr.db";

    // 2. เริ่มต้น Connection Pool และสร้างตารางใน DB หากยังไม่มี
    let db_pool = DbPool::new(db_path).await?;
    println!("✅ Connected to SQLite database: {}", db_path);

    // 3. ใส่ข้อมูลเริ่มต้น (Seed Data)
    sqlx::query(
        r#"
        INSERT INTO amrs (id, name, pose_x, pose_y, pose_theta, state_type)
        VALUES (1, 'AMR-01', 0.0, 0.0, 0.0, 'Idle')
        ON CONFLICT(name) DO NOTHING;
        "#
    )
    .execute(&db_pool.pool)
    .await?;
    println!("🌱 Seed data ready: AMR-01 (ID: 1)");

    // 4. สร้าง Repository
    let repo = Arc::new(DatabaseRepository::new(db_pool.pool.clone()));

    // 5. สร้าง Axum Router โดยส่ง repo เข้าไป
    let app = create_router(repo);

    // 6. ผูกกับ TCP Socket และเปิด Web Server ที่ port 3000
    let addr = "0.0.0.0:3000";
    let listener = TcpListener::bind(addr).await?;
    println!("🌐 Server running on http://localhost:3000");

    axum::serve(listener, app).await?;

    Ok(())
}