use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Multipart, Path, State,
    },
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::path::Path as FilePath;
use std::sync::Arc;
use tokio::fs::{create_dir_all, read_dir, remove_file, File};
use tokio::io::AsyncWriteExt;
use tokio::sync::broadcast;
use utoipa::{OpenApi, ToSchema};

use crate::{
    database::DatabaseRepository,
    models::{AmrState, Pose},
    utils::AppError,
};

// ==========================================
// DTOs (Data Transfer Objects)
// ==========================================

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePoseRequest {
    pub x: f64,
    pub y: f64,
    pub theta: f64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetGoalRequest {
    pub x: f64,
    pub y: f64,
    pub theta: f64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateStateRequest {
    pub state: AmrState,
}

#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct PoseResponse {
    pub amr_name: String,
    pub pose: Pose,
}

// Global AppState รวม Database Repo และ Broadcast Sender สำหรับ WebSocket
pub struct AppState {
    pub repo: Arc<DatabaseRepository>,
    pub tx: broadcast::Sender<PoseResponse>,
}

// ==========================================
// Swagger OpenAPI Definition
// ==========================================

#[derive(OpenApi)]
#[openapi(
    paths(
        get_amr_pose,
        update_amr_pose,
        set_amr_goal,
        update_amr_state,
        upload_map,
        get_maps,
        delete_map_file
    ),
    components(
        schemas(UpdatePoseRequest, SetGoalRequest, UpdateStateRequest, PoseResponse, Pose, AmrState)
    ),
    tags(
        (name = "AMR Database API", description = "Endpoints สำหรับจัดการข้อมูลหุ่นยนต์ AMR และ Map Files")
    )
)]
pub struct ApiDoc;

// ==========================================
// Handlers
// ==========================================

/// 🌐 WebSocket Handler: บรอดแคสต์ข้อมูล Pose แบบ Real-time
pub async fn ws_pose_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.tx.subscribe();

    while let Ok(pose_msg) = rx.recv().await {
        if let Ok(json_str) = serde_json::to_string(&pose_msg) {
            if socket.send(Message::Text(json_str)).await.is_err() {
                // Client ปิดการเชื่อมต่อ
                break;
            }
        }
    }
}

/// GET /api/amr/:name/pose -> ดึงค่า Pose ล่าสุดจากชื่อ AMR
#[utoipa::path(
    get,
    path = "/api/amr/{name}/pose",
    responses(
        (status = 200, description = "ดึงข้อมูลตำแหน่งสำเร็จ", body = PoseResponse),
        (status = 404, description = "ไม่พบ AMR ในระบบ")
    ),
    params(("name" = String, Path, description = "ชื่อของ AMR เช่น AMR-01"))
)]
pub async fn get_amr_pose(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<PoseResponse>, AppError> {
    let pose = state.repo.get_pose_by_name(&name).await?;
    Ok(Json(PoseResponse {
        amr_name: name,
        pose,
    }))
}

/// POST /api/amr/:name/pose -> อัปเดตตำแหน่ง Pose ของ AMR และส่งผ่าน WebSocket
#[utoipa::path(
    post,
    path = "/api/amr/{name}/pose",
    request_body = UpdatePoseRequest,
    responses((status = 200, description = "อัปเดตตำแหน่งสำเร็จ")),
    params(("name" = String, Path, description = "ชื่อของ AMR"))
)]
pub async fn update_amr_pose(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(payload): Json<UpdatePoseRequest>,
) -> Result<StatusCode, AppError> {
    let pose = Pose {
        x: payload.x,
        y: payload.y,
        theta: payload.theta,
    };

    state.repo.update_pose(&name, &pose).await?;

    // 📡 ยิงข้อมูลใหม่ไปยัง WebSocket Clients ทุกคนทันที
    let _ = state.tx.send(PoseResponse {
        amr_name: name,
        pose,
    });

    Ok(StatusCode::OK)
}

/// POST /api/amr/:name/goal -> กำหนดตำแหน่งเป้าหมาย (Goal Pose)
#[utoipa::path(
    post,
    path = "/api/amr/{name}/goal",
    request_body = SetGoalRequest,
    responses((status = 200, description = "ตั้งค่าพิกัดเป้าหมายสำเร็จ")),
    params(("name" = String, Path, description = "ชื่อของ AMR"))
)]
pub async fn set_amr_goal(
    State(_state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(payload): Json<SetGoalRequest>,
) -> Result<StatusCode, AppError> {
    println!(
        "🎯 [GOAL RECEIVED] AMR: {}, Target X: {:.2}, Y: {:.2}",
        name, payload.x, payload.y
    );
    Ok(StatusCode::OK)
}

/// POST /api/amr/:id/state -> อัปเดต State Machine ของ AMR
#[utoipa::path(
    post,
    path = "/api/amr/{id}/state",
    request_body = UpdateStateRequest,
    responses((status = 200, description = "อัปเดตสถานะสำเร็จ")),
    params(("id" = u64, Path, description = "ID ของ AMR"))
)]
pub async fn update_amr_state(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u64>,
    Json(payload): Json<UpdateStateRequest>,
) -> Result<StatusCode, AppError> {
    state.repo.update_amr_state(id, payload.state).await?;
    Ok(StatusCode::OK)
}

/// POST /api/map/upload -> อัปโหลดไฟล์แผนที่
#[utoipa::path(
    post,
    path = "/api/map/upload",
    responses((status = 200, description = "อัปโหลดไฟล์แผนที่สำเร็จ"))
)]
pub async fn upload_map(mut multipart: Multipart) -> Result<StatusCode, AppError> {
    create_dir_all("./uploads/maps")
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        if let Some(file_name) = field.file_name() {
            let file_name = file_name.to_string();
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(e.to_string()))?;

            let filepath = format!("./uploads/maps/{}", file_name);
            let mut file = File::create(filepath)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;

            file.write_all(&data)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
        }
    }
    Ok(StatusCode::OK)
}

/// GET /api/maps -> ดึงรายชื่อไฟล์แผนที่ทั้งหมด
#[utoipa::path(
    get,
    path = "/api/maps",
    responses((status = 200, description = "ดึงรายชื่อไฟล์แผนที่สำเร็จ", body = Vec<String>))
)]
pub async fn get_maps() -> Result<Json<Vec<String>>, AppError> {
    let mut file_list = Vec::new();
    if let Ok(mut entries) = read_dir("./uploads/maps").await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(file_name) = entry.file_name().into_string() {
                file_list.push(file_name);
            }
        }
    }
    Ok(Json(file_list))
}

/// DELETE /api/map/:filename -> ลบไฟล์แผนที่
#[utoipa::path(
    delete,
    path = "/api/map/{filename}",
    params(("filename" = String, Path, description = "ชื่อไฟล์แผนที่ที่ต้องการลบ")),
    responses(
        (status = 200, description = "ลบไฟล์แผนที่สำเร็จ"),
        (status = 404, description = "ไม่พบไฟล์ที่ต้องการลบ")
    )
)]
pub async fn delete_map_file(Path(filename): Path<String>) -> impl IntoResponse {
    let sanitized_filename = FilePath::new(&filename)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    if sanitized_filename.is_empty() {
        return (StatusCode::BAD_REQUEST, "ชื่อไฟล์ไม่ถูกต้อง".to_string());
    }

    let file_path = format!("./uploads/maps/{}", sanitized_filename);

    if FilePath::new(&file_path).exists() {
        match remove_file(&file_path).await {
            Ok(_) => (StatusCode::OK, format!("ลบไฟล์ {} สำเร็จ", sanitized_filename)),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("ไม่สามารถลบไฟล์ได้: {}", err),
            ),
        }
    } else {
        (StatusCode::NOT_FOUND, format!("ไม่พบไฟล์ {}", sanitized_filename))
    }
}