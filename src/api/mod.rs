pub mod routes;

use axum::{
    extract::DefaultBodyLimit,
    response::Html,
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::services::ServeDir;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::database::DatabaseRepository;
use routes::{
    delete_map_file, get_amr_pose, get_maps, set_amr_goal, update_amr_pose, update_amr_state,
    upload_map, ws_pose_handler, ApiDoc, AppState, PoseResponse,
};

async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../../static/index.html"))
}

pub fn create_router(repo: Arc<DatabaseRepository>) -> Router {
    let max_upload_size = 50 * 1024 * 1024;

    // สร้าง Broadcast Channel ขนาด buffer 100 รายการ
    let (tx, _) = broadcast::channel::<PoseResponse>(100);

    let app_state = Arc::new(AppState { repo, tx });

    Router::new()
        .route("/", get(index_handler))
        .route("/ws", get(ws_pose_handler))
        .nest_service("/maps", ServeDir::new("./uploads/maps"))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/api/amr/:name/pose", get(get_amr_pose).post(update_amr_pose))
        .route("/api/amr/:name/goal", post(set_amr_goal))
        .route("/api/amr/:id/state", post(update_amr_state))
        .route(
            "/api/map/upload",
            post(upload_map).layer(DefaultBodyLimit::max(max_upload_size)),
        )
        .route("/api/maps", get(get_maps))
        .route("/api/map/:filename", delete(delete_map_file))
        .with_state(app_state)
}